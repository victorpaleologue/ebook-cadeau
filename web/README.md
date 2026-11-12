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
5. **The eBook** — enter an existing code (from a store, or from a shelf service) and
   the address of the site where it is redeemed, or take a demonstration code. The core
   service never hosts book files (`docs/api.md` v0.3): a file is only ever served by a
   registered partner. When `config.json` names a **shelf** (`shelf_url`), a third choice
   "Host your own book on ‹shelf›" links to it: the shelf owns uploads, rights
   declarations and moderation (`crates/ebc-shelf`), creates the gift with the core and
   shows its code — there is **no cross-origin upload from the designer**. The shelf's
   gift list links back here with the code, title and author filled in (below). The
   note under the choices says to use the store's gift option for a bought eBook.
6. **The card** — size and printing (professional profiles are disabled, with the reason,
   for a private copy), warnings, "Download the PDF". "Order a printed card" needed a gift
   created on this server, which no longer happens: print orders go through the partner
   API (`docs/api.md` §4.6), from the store or the shelf that created the gift.

Seven languages (fr, en, de, es, it, nl, pt), French by default, `?lang=en` to choose; all
strings come from the package (`i18n_catalog`).

## Configuration

`config.json` (optional, same origin):

| Key | Meaning |
|---|---|
| `server` | base URL of the creator API; default: this origin when served under `/app/`, else none |
| `base_url` | default for "address of the site where the code is entered" |
| `print_orders` | offer "Order a printed card" (needs a gift created on this server) |
| `shelf_url` | a shelf service (`crates/ebc-shelf`, e.g. `https://etagere.example`): step 5 offers "Host your own book on ‹shelf›", a plain link to it. `null`: no such choice |
| `shelf_name` | the name shown for it (default: its host) |
| `catalog` | passed to `catalog_lookup_requests`: `base_urls` `{provider: url}`, `enable`, `disable`, `contact_email` |

### Prefilled from a link

`/app/#code=XXXX-XXXX-XXXX&title=…&author=…&lang=fr` opens the designer with the code
entered ("I already have a code"), the title and the author filled in, in that language
(values percent-encoded). This is what a shelf's gift list links to. The values are
read from the URL **fragment**, which browsers never send to a server, so the code (a
bearer secret) stays out of every access log; the same names in the query string are
accepted too. Once read, they are removed from the address bar. The claim-URL base is the
designer's own origin when it is served by the core under `/app/`.

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
  codes, `render_card` (`%PDF`, no warning, the rights gate), the preview, that the four
  golden cards (one a dark ground built under 240 % through FOGRA39, which exercises the
  ink-limiting transform) are byte-identical to the native rendering, that every string the page uses
  exists in all seven languages, and — when the tools are present — that the QR code read
  back from the PDF with `pdftoppm` + `zbarimg` is the upper-case claim URL and that
  `ebc preflight` passes.
* `lint.test.mjs`: no external URL, no inline script or event handler, no HTML sink.
* `browser-smoke.mjs` serves `web/` on a loopback port with correct MIME types and the
  server's CSP, aborts any request to another host, walks the flow (typed title/author,
  typographic card, demonstration code, PDF download, an image with statements and the
  adaptation tick, font detection, language switch) and writes screenshots to
  `tests/screenshots/` (see the README there).
