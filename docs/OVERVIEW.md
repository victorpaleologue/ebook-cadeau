# eBook Cadeau — project overview

_Written 2026-11-12. This is the one document to read first. It explains what the project is,
how it works, what was decided and why, what is proven and what is not, and where everything
lives. Each section points to the detailed document behind it._

---

## 1. The idea in one page

**The problem.** You can give someone an eBook, but there is nothing to hand over: no object to
wrap, nothing to put under a tree or on a shelf. Retailers send a gift e-mail at best. Surveys
cited in the market research list "you can't wrap it" among the reasons people do not give
eBooks.

**The product.** eBook Cadeau generates a **print-ready PDF of a book-shaped card**:

- **Outside**, as one wraparound jacket: the back, a **10 mm spine** carrying the title and author
  (so the card can stand on a shelf next to real books), and the front with the book's cover.
- **Inside**: a personal message ("for Sacha, from Camille"), three short steps, a **QR code**,
  the service's address in clear, and a **12-letter code** (`XXXX-XXXX-XXXX`).

The recipient scans the QR code with a phone, or types the address and code on their e-reader's
browser, and the eBook arrives on their device. The card can be printed at home on A4/Letter or
by a professional printer, and folds at two creases around the spine.

**Two ways to use it.**

| Mode | Who | How the book is delivered |
|---|---|---|
| **Reseller-integrated** (the main business case) | An eBook store adds an "Offer this eBook" option at checkout | The store keeps the content, the DRM and the customer account. eBook Cadeau issues the code, renders the card and sends the recipient back to the store to claim the book — or, for a store that serves DRM-free files, redirects the recipient's device to a short-lived download link the store issues on each claim (`partner_file`) |
| **Standalone** | Authors, publishers, individuals | Only for files the person may distribute (own work, public domain, open licence, or as a licensed distributor), and only through a **shelf**: a separate service that hosts those files and is registered with eBook Cadeau exactly like a store (next step; see below). Covers need a declared rights basis |

**Two services (owner decision, contract v0.3).** The core eBook Cadeau service — codes,
cards, redemption pages — **never holds a book file**. A file is only ever served by a
registered partner: a store, or the shelf. Whoever stores users' files is a hosting
provider with DSA duties (notice and action, statements of reasons, a repeat-infringer
policy) and a fragile liability exemption when it knows which work each file is
(`docs/research/09` §2.6, §3); those duties sit with the shelf, while the core keeps gift
records only, with a contact point and short retention (names and message deleted 12
months after the first claim). Details: `docs/architecture.md` §1.1, `docs/api.md` §3 and
§4.5.

**Market:** France first (all defaults are French), then the EU. Seven interface languages:
French, English, German, Spanish, Italian, Dutch, Portuguese.

**Status:** a complete first version. The software runs end to end and is heavily tested, but
**nothing has met the real world yet**: no real printer, real e-reader, live ISBN service, real
store or lawyer has been involved. See §10.

---

## 2. A gift, step by step

1. **The buyer** buys an eBook in a partner store and ticks the gift option (fields: from, to,
   message; print at home for free, or printed and posted).
2. **The store** (or, in standalone mode, the shelf) calls eBook Cadeau's partner API (signed
   requests). The service creates a gift and returns its code and claim URL. The store may
   upload its cover image, declaring its rights over it.
3. **The card PDF** is composed (by the server for stores, or in the buyer's browser in the
   designer app). The buyer prints it, or the store sends it to a print provider.
4. **The recipient** receives the card and either:
   - scans the QR code with a phone → the gift page shows title, giver, message and a **device
     chooser** (Kobo, Kindle, PocketBook, tolino, phone, computer); or
   - opens the browser **on the e-reader itself**, types the short address and the code →
     downloads the EPUB directly (Kobo, PocketBook, tolino) — eBook Cadeau asks the store
     for a short-lived link and redirects the device there; the book never passes through
     eBook Cadeau; or
   - on a phone, asks for a **6-letter pair code** and types it on the e-reader at `/e`; or
   - for Kindle, uses **Send to Kindle** (by e-mail; Amazon requires approving the sender once).
   - For stores where books live in an account (the Fnac/Kobo model), the recipient is sent to
     the store, signs in, and the book **syncs** to their e-reader. Nothing is typed on e-ink.
5. **The store** is told when the gift is redeemed (signed webhooks) and can cancel or reissue a
   lost code.

The redemption pages work **without JavaScript**, stay under 30 kB, and need no account,
e-mail address or cookie: holding the card is the credential. The design and its step counts
come from `docs/research/05-ereader-distribution-ux.md`.

---

## 3. What is in the repository

```
crates/      Rust libraries and programs (ebc-*)
web/         The card designer: a static web app running the Rust code as WebAssembly
demo/        Fictional stores, printer, e-readers, and end-to-end journeys
e2e/         Browser tests (Playwright/Chromium) and the print check
tools/       Business model (Python), orchestration scripts used to build the project
assets/      Bundled fonts (SIL OFL) and colour profiles
docs/        This overview, architecture, API, CI, research, business model, samples, status
.github/     CI, Dependabot, daily security audit, automated review workflow
.claude/     Review agents and the /spar-review command
```

### 3.1 The libraries (`crates/`)

All libraries are pure Rust, do no network or file I/O, and compile to WebAssembly, so the same
code runs on the server, in the command line and in the browser.

| Crate | What it does |
|---|---|
| `ebc-scene` | The shared vocabulary: millimetres, page boxes, paths, colours, images, glyph runs. Everything that composes a page produces this; the PDF writer consumes it |
| `ebc-isbn` | ISBN-10/13 parsing, conversion, hyphenation (range table), finding ISBNs in text |
| `ebc-model` | Book metadata, gifts, the 12-letter gift code with its check letter, the pair code, claim URLs |
| `ebc-i18n` | Every printed and displayed string in seven languages; spine reading direction per language |
| `ebc-color` | Palettes, contrast (WCAG), colour difference, and a **small ICC colour engine** that converts sRGB to the printer's CMYK profile |
| `ebc-qr` | QR codes for print, as vector shapes, with the claim URL encoded compactly |
| `ebc-text` | Fonts: loading, shaping (HarfRust), fitting, wrapping, outlines; protected against malicious fonts |
| `ebc-layout` | Card formats, bleed, safe areas, crop and fold marks, print profiles, home-print sheets |
| `ebc-pdf` | Writes PDF/X-4 files: page boxes, embedded font subsets, images, output intent, metadata |
| `ebc-preflight` | Reads any PDF back and checks it like a printer would (boxes, fonts, resolution, colour, ink, marks…) |
| `ebc-catalog` | 13 ISBN databases (BnF, DNB, Open Library, Google Books, Wikidata…) as request/parse pairs, plus merging |
| `ebc-covers` | Finding cover candidates, robots.txt and text-and-data-mining opt-outs, image sniffing, print quality, and the **rights basis** that decides whether a cover may be printed |
| `ebc-fontid` | Identifies the typeface of a cover's title from the image (classical image analysis, no AI model) and suggests the closest open font |
| `ebc-spine` | Designs the 10 mm spine: flat colour from the cover, title and author typeset along it |
| `ebc-card` | Composes the whole card (both pages) and enforces the rights and print rules |
| `ebc-epub` | Checks for a service that stores eBook files (the shelf, not the core): EPUB/PDF sniffing, DRM refusal, rights declaration, safe file names |

### 3.2 The programs

| Program | What it is |
|---|---|
| `ebc-server` | The core web service: redemption pages, partner API (stores and shelves), OPDS feeds, pairing, send-to-device, print orders, serving the web app at `/app/`. No file hosting, no creator API. One binary, configured by environment variables (`crates/ebc-server/README.md`) |
| `ebc` (`ebc-cli`) | Command line: `isbn`, `lookup`, `cover`, `fontid`, `card`, `preflight`, `qr` |
| `ebc-wasm` + `web/` | The **card designer** in the browser: type or look up a book, choose the cover and spine, write the message, preview both sides live, download the PDF — generated on the user's device, no cookies, no trackers. Its former "upload a file" path is hidden until the shelf exists |
| `ebc-demo` | Starts the whole demo world on local ports: the server, two stores, a printer, a Kindle mail sink |

### 3.3 The demo world (`demo/`)

Fictional, clearly labelled ("demo — not affiliated"):

- **Septante** — models an independent store selling DRM-free, watermarked eBooks (the 7switch
  kind). Gifts are `partner_file`: the store stamps a copy for the recipient when the gift
  is claimed and serves it through a signed, short-lived download link.
- **Grande Enseigne** — models a large retailer whose eBooks live in an e-reader account (the
  Fnac/Kobo kind). Gifts are claimed into the recipient's account and synced.
- **Imprimerie du Dos Carré** — a print-on-demand API that preflights every PDF.
- **Simulated e-readers** — Kobo, Kindle, PocketBook, tolino profiles with limited browsers
  (no JavaScript), an OPDS client, a Send-to-Kindle mail sink with sender approval, an
  account-sync device, a simplified LCP licence flow.

---

## 4. How it was built and how it is checked

**Architecture principles** (`docs/architecture.md`): heavy work runs in the browser; libraries
never do I/O; output is deterministic (same input, same bytes); print correctness is a tested
property; e-reader pages need no JavaScript; legally conservative defaults.

**Tests.** About 1 865 automated Rust tests, plus:

| Layer | What it proves |
|---|---|
| Unit and property tests | Each crate's logic, including hostile input (malformed PDFs, fonts, images, HTML) |
| PDF validation | Generated cards pass our preflight and independent tools: qpdf, pdffonts, Ghostscript, MuPDF; the QR code is decoded back from the rendered page |
| 21 integration journeys (`demo/integration`) | The real server, both stores, the printer and simulated devices talking over HTTP: checkout → card → redemption on each device type, account sync, printing, and the failure paths |
| 10 browser tests (`e2e/`) | The same journeys in Chromium, including e-reader browsers with JavaScript off and the designer app |
| Web package tests | The PDF made in the browser is **byte-identical** to the one made natively |
| Print check | Every format × print profile rendered and validated |

**CI** (`docs/ci.md`) runs all of this on every push to GitHub: format, lint, tests, docs,
WebAssembly build, licence and security audit (`cargo deny`), workflow lint, browser tests,
print check. The latest run (commit `ab5d778`) is fully green. **Dependabot** proposes weekly
dependency updates; minor and patch updates merge automatically once CI passes (after the owner
enables it). A **daily audit** opens an issue if a dependency gets a security advisory.

**Automated review.** Pull requests are reviewed by a "sparring" process: prosecutor agents look
for defects through different lenses, a defender tries to refute each finding, a judge keeps
what survives, and blocking findings fail the check (`.claude/agents/`, `claude-review.yml`;
needs an API secret). The same approach was used on the whole repository before the last
commit: three independent reviewers (security, print, cross-component correctness) found 23
problems, each proven by a failing test; all were fixed and those tests stay in the suite.

---

## 5. Key design decisions (and why)

| Decision | Why |
|---|---|
| **Code = 12 letters from 20 consonants**, `XXXX-XXXX-XXXX`, with a check letter | Digits need a keyboard layer switch on e-ink and phone keyboards and Shift on French AZERTY; letters avoid 0/O and 1/I confusion; no vowels means no accidental words. The check letter catches every single typo and swap, and a typo is never counted as a guessing attempt |
| **Typing the code on the e-reader is the default path** | E-ink readers have no camera; the QR code is for phones. 4 actions, about 18 characters |
| **Explicit device chooser** on every gift page | User-agent detection broke twice in 2024–2026 (Kobo and tolino now say "eReader") |
| **Typographic card by default** | A cover image is someone's copyrighted artwork. Without a declared rights basis, the card shows title and author as text in open fonts |
| **Rights basis for covers**: none / print licence / promotional use / rights-holder declared / private copy / public domain | Each basis allows different routes: home printing accepts all but "none"; professional printing only print licence, rights-holder declaration or public domain. Covers found online are previews only, never printed |
| **Covers never cropped or altered by default**; the spine is typeset, not cut from the artwork | French moral rights protect the integrity of the work; sampling or extending the art needs explicit authorisation |
| **Only openly licensed fonts** bundled; no generative AI lettering | Typeface and AI-reproduction risks (research 08) |
| **Conservative security**: signed partner requests, single-use signatures, codes stored only as hashes, rate limits, names and message deleted 12 months after the first claim and 30 days after a cancellation or expiry, no book files in the core | Codes are bearer secrets not tied to accounts; GDPR storage limitation |
| **Font stack migrated** to HarfRust/read-fonts | The original crates were declared unmaintained by RustSec |

A full list of decisions taken without the owner, open to veto, is in `docs/STATUS.md`.

---

## 6. Print: what a professional printer receives

From `docs/research/03-print-requirements.md` (fact-checked):

- **Formats**: poche 110 × 178 mm (default), A6, 130 × 200, A5; spine 10 mm (5 mm minimum).
- **PDF/X-4** with a FOGRA39 output profile (basICColor "ISO Coated v2 300 %", zlib licence;
  ECI's own profiles may not be redistributed). 3 mm bleed, safe areas, crop and dashed fold
  marks outside the bleed for offset; a clean trim-plus-bleed page for upload printers.
- **Colour**: covers and grounds converted through the profile by an in-house ICC engine
  (within ΔE 1 of Little CMS); text, QR code and marks in black ink only.
- **QR**: 1 mm modules, 4-module quiet zone, about 33 mm, decodable at 150 dpi.
- **Spine text**: 9–13 pt, never below 8 pt; reads bottom-to-top for French and German books,
  top-to-bottom for English and Dutch.
- **Home printing**: the PDF page is the A4/Letter sheet, the card centred, everything inside a
  6.35 mm printable margin, with a duplex hint ("flip on the short edge").
- **Ink limits per printer**: 240 % total ink for upload/print-on-demand printers
  (`upload_eu`; IngramSpark's cap and its rich black C60 M40 Y40 K100), 300 % for offset on
  coated stock (`professional_eu`; FOGRA39 "ISO Coated v2 300 %", PSO Coated v3); home
  files are sRGB and grey, so nothing there is limited. The demo printer's digital
  products and the CLI's preflight profiles carry the same values.

**Dark covers** (implemented 2026-11-12). Dark covers keep their one-colour jacket. Every
colour the output profile would build above the printer's ink limit — the ground, the
pixels of the converted cover, image spines — is **rebuilt by grey-component
replacement**: the grey common to C, M and Y moves to black, C, M and Y are then scaled
down together if still needed, and the build closest to the original (ΔE2000 through the
profile's own tables) wins (`ebc_color::ink`). Measured on FOGRA39 at 240 %: navy ΔE 1.0,
black 0.1, deep red 1.7, forest green 1.7, brown 1.9, purple 1.1 (at 300 % they need no
change). The ground turns white only if its build under the limit is more than ΔE 3 from
the colour, or if text on it would not be legible. When the outside is dark or heavy on a
crease (above 180 % ink or darker than L\* 35), the card says so (`dark_across_creases` in
the CLI, the designer's warnings in seven languages and the partner API's
`X-EBC-Card-Warnings` header) and advises **matt or soft-touch lamination** and the
**paper grain parallel to the spine**; the preflight gives the same advice
(`INK_HEAVY_AT_CREASE`) and now checks the ink of lossless images too (CMYK JPEGs are not
decoded). Samples: `docs/samples/poche-professional_eu-cover` and
`poche-upload_eu-cover`.

---

## 7. Research (`docs/research/`)

Twelve studies, each with an "Implications for the implementation" section the code cites.

| # | Topic | Main conclusions |
|---|---|---|
| 01 | ISBN databases | National libraries (BnF, DNB, K10plus, Libris, Finna) and Open Library are open and usable; Google Books needs a key; French trade data (Dilicom, Electre) needs contracts |
| 02 | Cover sources | Locating a cover is fine, printing it is not: printable covers must come from the rights holder or the reseller |
| 03 | Print requirements | The print profile above |
| 04 | Printers | Prices, EU production and an environmental/social ranking. The cheapest posted route (a postal-letter API) takes one image per card, so the four-panel card goes by letter |
| 05 | E-reader UX | The redemption design of §2; Kindle cannot open EPUB in its browser; Kobo now supports Readium LCP on four models (not yet in France) |
| 06 | Terminology | "eBook cadeau", "liseuse", wording of the checkout option per language |
| 07 | Legal: covers and resellers | A reseller's licence covers promotion of the sale; printing a cover is a reproduction needing a basis; the typographic default |
| 08 | Legal: fonts and AI | Identifying a font is lawful; reuse needs the right licence; AI-reproduced lettering is risky (EU AI Act transparency applies from launch) |
| 09 | Legal: gifting and commerce | Gifting a purchased eBook file is not lawful without the rights holder (no digital exhaustion); France's fixed book price; VAT treatment of the card; consumer law; GDPR |
| 10 | Font identification | Pipeline, evaluation, and a **training plan** for a learned model (about 35–65 GPU-hours) |
| 11 | Spine design | Conventions by country and publisher, the spine algorithm |
| 12 | Market and business inputs | Market size, competitors, cost inputs |

Fact-check status: 01, 03, 05, 06 fact-checked; 02, 04, 07, 08, 09, 12 had fact-check
corrections applied; 10 and 11 are syntheses not independently checked. Web access was limited
at times, so many claims are single-source or marked as from memory. **The legal studies brief a
lawyer; they are not legal advice.**

---

## 8. Business (`docs/business/README.md`)

A reproducible model (`tools/bizmodel/`, 97 inputs with ranges, 2 000 Monte Carlo draws, 48 tests,
7 charts) gives:

- **Feasible as a self-funding side business, not as a funded company** on current evidence.
- **Sweet spot**: about €2 100–3 700 of cash and one to two days a week; pays back around month 25–28;
  €2 500–5 100 a year by year 3. The founder's time is not paid.
- **Who pays**: authors and publishers (yearly plans, packs for signings and fairs) first;
  **resellers paying per card with a monthly minimum** as the lever for scale. The free
  generator is the demo. A printed-and-posted card only makes sense at €7.90–8.90 or more.
- **Selling directly to consumers does not pay** at any investment level.
- **A salary** from resellers alone needs 8 large platforms (France has about six) at a 2 %
  attach rate.
- **Weakest assumptions**: whether a reseller will pay a monthly minimum, the attach rate, and the
  print cost — two reseller conversations and one printer quote would tell more than the model.

---

## 9. Running it

```sh
# All checks
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

# The demo world: server + two stores + printer (URLs printed at start)
EBC_WEB_DIR=web cargo run -p ebc-demo-integration --bin ebc-demo

# The card designer (build the WebAssembly first), served at http://127.0.0.1:8080/app/
scripts/build-wasm.sh && EBC_WEB_DIR=web cargo run -p ebc-server

# A card from the command line, then check it
cargo run -p ebc-cli -- card --title "Les Vagues du soir" --author "Jeanne Roy" \
    --random-code --base-url https://ebk.example --typographic -o card.pdf
cargo run -p ebc-cli -- preflight card.pdf

# Browser tests and print check
bash e2e/run.sh && bash e2e/print-check.sh
```

Sample cards (PDF and page images) are in `docs/samples/`.

---

## 10. What is not proven, and what to do next

**Not yet tested against reality**

- No card has been printed by a real printer or folded; no printer quote.
- No real e-reader has been used (devices are simulated).
- The 13 ISBN sources have never been queried live (the development sandbox blocked them).
- Font identification is measured on synthetic covers only.
- No lawyer has reviewed the legal studies; no native speaker the translations.
- No store has been approached; all business volumes are assumptions.

**Known technical gaps** (full list in `docs/STATUS.md`): per-printer profile records (only
the built-in profiles carry an ink limit) and print briefs, CMYK JPEG covers are neither
ink-limited nor checked, the dark-cover lamination advice is untested on a real fold,
server storage is a single JSON file (fine for a pilot, not for scale), no right-to-left or CJK spines, a partner cannot yet choose the spine
style or output profile, the learned font model is not built.

**Owner actions to switch everything on**: add the review secret, enable auto-merge and branch
protection on `main` (required check `ci-ok`), create the Dependabot labels, allow the ISBN
hosts for the live catalogue tests (`docs/STATUS.md`, "Owner actions").

**Suggested next steps, cheapest first**

1. Print and fold five cards at a local print shop and at one online printer; get a quote.
2. Test redemption on a real Kobo, Kindle and PocketBook.
3. Show the demo to two French eBook stores; ask whether they would pay per card and a minimum.
4. A one-hour lawyer review of the cover and gifting conclusions (studies 07 and 09).
5. Then decide: author/publisher plan first, or a reseller pilot.

---

## 11. Where to read more

| Document | Content |
|---|---|
| `README.md` | Short introduction and quick start |
| `docs/STATUS.md` | Exact state, check results, known gaps, decisions to veto, owner actions |
| `docs/architecture.md` | Design principles and the contract of every crate |
| `docs/api.md` | The HTTP contract (v0.3): redemption pages, partner API with `partner_file` delivery and print orders, printer interface, configuration |
| `docs/ci.md` | What CI runs, the review process, Dependabot, how to run everything locally |
| `docs/research/` | The twelve studies, their fact-check reports and the researchers' notes |
| `docs/business/` | Business model report, results and charts |
| `docs/samples/` | Sample cards |
| `crates/*/README.md`, `demo/README.md`, `web/README.md`, `e2e/README.md` | Per-component details |
| `docs/status/`, `tools/orchestration/` | Traceability: reports from the first build night and the scripts used to orchestrate the work |
