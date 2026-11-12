# eBook cadeau — the card creator (web app)

A static, dependency-free page that composes the gift card **in the browser**: the
WebAssembly build of `crates/ebc-wasm` runs in a Web Worker, draws a live SVG preview of
both sides and writes the print-ready PDF without any server.

No bundler, no CDN, no web font, no analytics, no cookie. The page itself contacts only
its own origin; the catalogue lookup (when the user presses the button), the preview image
of a looked-up cover and the creator API of the configured server are the only other
requests.

## Build and serve

```sh
scripts/build-wasm.sh          # cargo (profile wasm-release) + wasm-bindgen → web/pkg/
```

The script also copies the FOGRA39 output-intent profile to `web/pkg/fogra39.icc` (fetched
only for the professional PDF/X profiles), writes the native digests of the golden cards
to `web/pkg/golden-native.sha256`, prints the size of the `.wasm` and fails above the size
budget it documents. It needs a `wasm-bindgen` CLI of exactly the version pinned in
`crates/ebc-wasm/Cargo.toml` (0.2.130); `wasm-opt` is used when it is on `PATH`.

Serve the directory with any static server that sends `.wasm` as `application/wasm` and
`.js` as JavaScript — normally `ebc-server` with `EBC_WEB_DIR=web`, at `/app/`:

```sh
EBC_WEB_DIR=web cargo run -p ebc-server     # then open http://127.0.0.1:8080/app/
```

From `file://` the page does not work in most browsers (module workers need an origin).

## The flow

1. **The book** — an ISBN looked up through the catalogue providers that allow browser
   requests (Open Library by default), or title and author typed by hand.
2. **The cover** — the typographic card by default. To print an image the user picks a
   file of their own and says why they may print it (own work, print licence, publisher's
   authorisation, public domain, private copy for their own printer). A cover found by the
   lookup is only shown, as "is this the book?", and never reaches the card. Print quality
   (effective ppi) is shown for the chosen size and printing.
3. **The spine** — flat colour with the title typeset (default). The title face is one of
   the bundled open fonts; "suggest the closest open typeface" compares the lettering of the
   user's image with them (nothing is copied or generated). Styles that sample the image
   (colour from the image, edge continued onto the spine) and the image's colours as card
   colours appear only after ticking "I am authorised to adapt this cover", and only for a
   statement that allows it.
4. **The message** — giver, recipient, text, language of the card.
5. **The eBook** — upload a DRM-free file with its rights declaration (creates the gift on
   the server: `POST /api/v1/uploads` then `POST /api/v1/gifts`), or enter an existing
   code and the address of the site where it is redeemed, or take a demonstration code.
6. **The card** — size and printing (professional profiles are disabled, with the reason,
   for a private copy), warnings, "Download the PDF", and "Order a printed card" when the
   gift was created on this server and `print_orders` is on.

Seven languages (fr, en, de, es, it, nl, pt), French by default, `?lang=en` to choose; all
strings come from the package (`i18n_catalog`).

## Configuration

`config.json` (optional, same origin):

| Key | Meaning |
|---|---|
| `server` | base URL of the creator API; default: this origin when served under `/app/`, else none |
| `base_url` | default for "address of the site where the code is entered" |
| `print_orders` | offer "Order a printed card" (needs a gift created on this server) |
| `catalog` | passed to `catalog_lookup_requests`: `base_urls` `{provider: url}`, `enable`, `disable`, `contact_email` |

Catalogue base URLs are taken from `config.json` only: a relay of the deployment or a
fixture server in tests is configured there. (Query parameters are deliberately not
honoured: a crafted link could otherwise send the look-ups, and what they return onto the
card, to any host.)

## Files

| File | Role |
|---|---|
| `index.html`, `style.css`, `icon.svg`, `legal.html` | the page and the legal note (fr, en) |
| `app.js` | state, steps, preview scheduling, download |
| `worker.js` | the worker hosting the WebAssembly engine (protocol documented at the top) |
| `js/engine.js` | promise-based client of the worker; restarts it after a panic |
| `js/i18n.js`, `js/lookup.js`, `js/server.js` | strings, sans-IO catalogue lookup, creator API |
| `pkg/` | build output of `scripts/build-wasm.sh` (not committed) |

Security: everything from catalogues, servers or the user is inserted with `textContent`
or as an attribute, never as markup; previews are SVG shown through `<img>` (no script
runs); no inline script or handler, compatible with the server's Content-Security-Policy
(`index.html` also carries its own, stricter one).

## Tests

```sh
node --test web/tests/*.test.mjs   # package smoke test + lint (needs web/pkg)
node web/tests/browser-smoke.mjs   # real Chromium through Playwright (must be installed)
```

* `smoke.test.mjs` loads `web/pkg` in Node (`initSync`), checks `isbn_parse`, `qr_svg`,
  codes, `render_card` (`%PDF`, no warning, the rights gate), the preview, that the three
  golden cards are byte-identical to the native rendering, that every string the page uses
  exists in all seven languages, and — when the tools are present — that the QR code read
  back from the PDF with `pdftoppm` + `zbarimg` is the upper-case claim URL and that
  `ebc preflight` passes.
* `lint.test.mjs`: no external URL, no inline script or event handler, no HTML sink.
* `browser-smoke.mjs` serves `web/` on a loopback port with correct MIME types and the
  server's CSP, aborts any request to another host, walks the flow (typed title/author,
  typographic card, demonstration code, PDF download, an image with statements and the
  adaptation tick, font detection, language switch) and writes screenshots to
  `tests/screenshots/` (see the README there).
