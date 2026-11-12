# Project status — 2026-11-12

Everything the original request asked for exists in a first version. This page says how far
each piece goes, what is verified, and what is not. Earlier versions of this page (the
interrupted overnight build) are in the git history.

## Two-service model (contract v0.3, 2026-11-12)

Owner decision: **the core service stops holding book files; any file hosting acts exactly
like a reseller.** Done in this step:

- `hosted_file`, `POST /api/v1/uploads`, `POST /api/partner/v1/uploads` and the whole
  creator API (`/api/v1/gifts…`, manage tokens) are **removed** (`404`). Every gift belongs
  to a registered partner. Print orders moved to the partner API (`PUT …/card.pdf`,
  `POST …/print-orders`, `docs/api.md` §4.6).
- New delivery **`partner_file { format, url }`** (`docs/api.md` §4.5): on each claim the
  core sends the partner a signed request (`ebc_gift`, `ebc_order`, `ebc_ts`, `ebc_sig`) and
  redirects the device to the short-lived link it answers (JSON or `303`), checked against
  the partner's `redirect_hosts`. Typed code, pair code, OPDS, HEAD and the claim count work
  as before; Send to Kindle fetches the link into memory (≤ `EBC_SEND_MAX_MB`, size and
  SHA-256 checked).
- EPUB/PDF sniffing, DRM detection, the rights declaration and file-name sanitising moved
  out of the server into the library `crates/ebc-epub`, for the shelf.
- Names and message are deleted 12 months after the first claim
  (`EBC_RETENTION_AFTER_CLAIM_DAYS`); the core shows a contact point (`EBC_CONTACT_EMAIL`).
- The demo store Septante serves its own watermarked EPUB (stamped at claim time) through
  a signed download endpoint; Grande Enseigne is unchanged.

## The shelf (2026-11-12) — done

`crates/ebc-shelf` (binary `ebc-shelf`, `crates/ebc-shelf/README.md`): the second service,
registered with the core as partner `etagere` exactly like a store. It is the **host** in
law, so the hosting duties live there.

- **Modes**: `own` (default; only the operator's account — an author self-hosting) and
  `multi` (open sign-up; the operator moderates).
- **Accounts**: sign-in by e-mail link (256-bit, single use, 15 min, stored hashed; a
  confirmation button so mail scanners cannot spend it), verified address required,
  LCEN identification (person or organisation, name or company name, representative,
  postal address, optional phone and birth data) before the first upload; sessions in an
  `HttpOnly`, `SameSite=Strict`, `__Host-` cookie with form tokens and an `Origin` check.
- **Upload gate** through `ebc-epub` only (new: `check_upload` with a size limit,
  `package_metadata` for the ISBNs inside the file): sniffing, DRM refused; mandatory
  declaration from the closed list with details (death year for public domain — life + 70
  —, a closed CC list, licensor and contract for distributors), title, author, ISBN,
  recorded with time, account, name, e-mail, text version and hash; blocklist by SHA-256
  and ISBN (declared and embedded); quotas per account and global; files stored by hash.
- **Gifts**: 1–50 per form (giver optional, recipient, message, locale, reveal date),
  created with the core's signed API (idempotent, retried when the core is down), reissue
  and cancel through the core; the §4.5 endpoint verifies signature and timestamp and
  answers a five-minute 128-bit link on the shelf's host; the file is served byte-identical
  with `Content-Disposition`, `Range` and `HEAD`; deliveries counted per gift, downloads
  logged with a network pseudonym; no code, token, signed URL or `Referer` in logs. Card
  PDFs come from the core's partner `card.pdf`; the designer link puts code, title and
  author in the URL fragment (`/app/#code=…`), which the designer now reads.
- **Notice and action**: public form with the art. 16(2) elements and the (c) exception,
  acknowledgement, operator review; upholding deletes every copy of the file, cancels every
  gift at the core (`410`), blocklists hash and ISBN, mails a statement of reasons (art. 17)
  with redress; an internal complaint (kept although art. 20 binds platforms only);
  suspension after 2 upheld notices or 1 false declaration (configurable), ban on
  recurrence. Terms, legal notice and contact pages: **drafts, not reviewed by counsel**.
- **Retention** (configurable, to confirm with counsel): identity 5 years after closure,
  IP of content creation 1 year, declarations 5 years after the file is deleted, notices 6
  years, download pseudonyms 6 months, delivery records 5 years, files deleted on
  deletion, takedown, closure, ban, or 30 days after a book's last gift was cancelled; a
  record-of-processing template in the README.
- **Designer**: `config.json` `shelf_url` (and `shelf_name`) adds "Host your own book on
  ‹shelf›" as a link; the dead upload code is removed; strings in seven languages.
- **Demo**: the shelf is in `ebc-demo` and `World` (port 8104), its mail goes to demo
  mailboxes (`…@exemple.fr`, a new feature of the demo mail sink) read on a demo webmail
  (8105). Journeys in `demo/integration/tests/it/etagere.rs`, browser scenario
  `e2e/tests/shelf.test.mjs`, HTTP tests in `crates/ebc-shelf/tests/http.rs`.

**Needs counsel**: the identification field list (décret 2021-1362, recalled from memory),
every retention period, whether the shelf is a plain host or an online platform (arts 20,
23), the wording of the terms, legal notice and statements of reasons, and the
remaining upload safeguards of research 09 (retailer-watermark detection, ISBN-registrant
check with a review state for `own_work`, translator's death year, "mort pour la France").

Known caveat: e-reader browsers that ignore `Referrer-Policy` send the gift page URL (with
the code) as `Referer` to the partner's download host when following the redirect. That
host is the partner's own and the partner issued the code (documented in §4.5).

## Checks

Run on the whole workspace, locally, after the adversarial review fixes:

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | clean |
| `cargo test --workspace` | **1 933 passed, 0 failed**, 19 ignored (opt-in live and reference tests); re-run 2026-11-12 after the shelf (`ebc-shelf` unit and HTTP tests, 5 shelf journeys, mail-sink mailboxes, `ebc-epub` size limit and package metadata) |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | clean |
| `scripts/wasm-crates.sh build` (all pure libraries for `wasm32-unknown-unknown`; `ebc-shelf` is host-only) | clean |
| `cargo deny check` | advisories, bans, licences, sources: ok (the shelf adds no new third-party crate; `axum`'s `multipart` feature brings `multer`, MIT) |
| `e2e/run.sh` (13 Playwright tests in Chromium) | 13 passed (scenario 7: the shelf with JavaScript off, the designer's shelf link and prefill) |
| `e2e/print-check.sh` (every format × profile through preflight, qpdf, pdffonts, gs) | 21 PDFs, 0 failures |
| `python3 -m unittest` in `tools/bizmodel` | 48 passed |
| `CI=1 node --test web/tests/*.test.mjs` (incl. wasm output byte-identical to native for four golden cards, one through the ink-limited FOGRA39 transform) | 9 passed |

CI on GitHub runs the same checks (`docs/ci.md`). The run before the review (commit `52b5101`)
passed 10 of 11 jobs, including `e2e` and `print-check`; the one failure was a MuPDF version
difference in a test, since fixed. The sparring review workflow (`claude-review.yml`) needs a
secret and has never run.

## Adversarial review

Three independent reviewers (security and privacy; print domain; correctness where crates
meet) attacked the whole repository and backed findings with failing tests. All 23 findings
were fixed or decided; their tests now pass and stay in the suite (`spar_*`). Highlights:
partner-request replay closed, signatures checked before large bodies are read, atomic
send limits, connection timeouts, a 30-day retention of personal data, a real ICC colour
conversion for professional cards, upload preflight that refuses marks, an 8 pt floor for
spine text, a 5 mm minimum spine, structured print-order addresses, and CI now enforcing
the native-vs-wasm golden cards.

## What exists

| Area | Where | State |
|---|---|---|
| ISBN parsing, hyphenation, extraction | `crates/ebc-isbn` | done |
| Domain model, 12-letter gift code, pair code | `crates/ebc-model` | done |
| Strings in fr, en, de, es, it, nl, pt | `crates/ebc-i18n` | done; **no native-speaker review** |
| Colour, contrast, palettes | `crates/ebc-color` | done |
| QR for print (vector, alphanumeric claim URL) | `crates/ebc-qr` | done |
| Fonts: shaping, fitting, outlines (harfrust + read-fonts) | `crates/ebc-text` | done; hostile fonts bounded |
| Formats, bleed, marks, print profiles, A4/Letter sheets | `crates/ebc-layout` | done |
| PDF/X writer | `crates/ebc-pdf` | done |
| Preflight | `crates/ebc-preflight` | done |
| ISBN databases (13 sources, sans-IO) | `crates/ebc-catalog` | done; **never run against the live services** |
| Cover discovery, rights basis, robots/TDM, sniffing | `crates/ebc-covers` | done; **never run against the live services** |
| Font identification (classical, image-only) | `crates/ebc-fontid` | done; evaluated on synthetic covers only (`docs/research/fontid-eval.md`) |
| 10 mm spine (typeset by default) | `crates/ebc-spine` | done |
| Card composer (outside + inside, rights gate) | `crates/ebc-card` | done; samples in `docs/samples/` |
| Command line `ebc` | `crates/ebc-cli` | done |
| Browser bindings (3.8 MiB wasm, 1.5 MiB gzip) | `crates/ebc-wasm` | done |
| Card designer web app | `web/` | done; links to a configured shelf (`shelf_url`), prefilled from a shelf's gift link; print orders only through partners |
| Web service (the core): redemption pages, partner API with `partner_file`, OPDS, pairing, send-to-device, print orders — no file hosting | `crates/ebc-server` | done (v0.3) |
| eBook file checks for a hosting service (sniffing, size limit, DRM, rights declaration, package metadata, names) | `crates/ebc-epub` | done; used by the shelf |
| Shelf service (files of rights holders, notice and action, retention) | `crates/ebc-shelf` | done; legal wording and periods **not reviewed by counsel**; strings not reviewed by native speakers |
| Fictional stores "Septante" and "Grande Enseigne", printer, e-readers | `demo/` | done |
| 27 Rust end-to-end journeys + `ebc-demo` binary (with the shelf and a demo webmail) | `demo/integration` | done |
| 13 browser end-to-end tests | `e2e/` | done |
| Business model, charts | `tools/bizmodel/`, `docs/business/` | done; inputs are estimates |
| Research 01–12 | `docs/research/` | all written; fact-check status per document below |
| CI, Dependabot, daily audit, sparring review | `.github/`, `.claude/` | written; see "Owner actions" |

## Research status

| Document | Status |
|---|---|
| 01 ISBN databases, 03 print requirements, 05 e-reader UX, 06 terminology | fact-checked |
| 02 cover sources, 04 printers, 07 legal covers, 08 legal fonts and AI, 09 legal gifting, 12 market | fact-check corrections applied (the fact-check had limited web access) |
| 10 font identification (incl. training plan), 11 spine design | synthesised from researcher notes, **not fact-checked** |

The legal documents are material to brief a lawyer, not legal advice. Most legal authorities
in 07–09 and every vendor price except La Poste's remain unverified at source.

## Known gaps and approximations

- **Colour management is a small in-house CMM** (`ebc_color::icc`): sRGB → the output
  profile's CMYK through its `lut8`/`lut16` tables, relative colorimetric with black-point
  compensation, sampled on a 33³ grid; covers, spine images and grounds of PDF/X cards go
  through it (CMYK JPEG covers pass through unchanged). Checked against Little CMS 2.19
  with the bundled FOGRA39 profile: ΔE2000 ≤ 1.0 (mean 0.12) on a 9³ sRGB lattice. Not
  handled: ICC v4 `mAB`/`mBA` profiles (a professional cover is then refused unless the
  profile allows RGB), covers' own embedded profiles (treated as sRGB), and the ground
  of a CMYK JPEG cover is judged from a naive RGB decoding.
- **Ink limits** (2026-11-12): each print profile carries the printer's total ink limit —
  240 % upload/print-on-demand, 300 % offset — and the transform builds every process
  colour under it by grey-component replacement chosen by ΔE2000 through the profile
  (`ebc_color::ink`; FOGRA39 at 240 %: navy 1.0, black 0.1, dark red/green/brown 1.7–1.9,
  purple 1.1). So dark covers keep their jacket on professional cards; white remains only
  when the limited build is more than ΔE 3 away (a house threshold) or text would not be
  legible. Images get the limit through the 33³ grid (nodes rebuilt once, about 110 ms
  natively and 200–280 ms in WebAssembly; a full 1240 × 2008 cover then converts in about
  7 ms natively). Not handled: **CMYK JPEG covers** pass through unchanged, neither
  ink-limited nor decoded by the preflight; the method searches a two-parameter GCR family
  (grey removed, black added, CMY scaled together), not a full inversion of the profile; the
  appearance is the profile's characterisation, not a measured press; ICC profiles carry
  no maximum black, so black may reach 100 % (an `InkLimit` can cap it, no profile does).
  The preflight's image-ink check (lossless CMYK, 99.5th percentile, research 03 C4) is now
  an error in professional files.
- **Dark creases**: a dark or heavy outside (above 180 % ink or L\* 35 within 2 mm of a
  crease, house thresholds) gives `CardWarning::DarkAcrossCreases` — matt or soft-touch
  lamination, grain parallel to the spine — in the CLI, the designer (seven languages,
  not reviewed by native speakers) and the partner API (`X-EBC-Card-Warnings`), and the
  preflight warning `INK_HEAVY_AT_CREASE`. Advice only: no printer has confirmed that it
  prevents cracking on this card, and the preflight does not follow occlusion (a heavy
  fill under something lighter still counts).
- **Spine**: titles 9–13 pt, nothing below 8 pt (outlined/reversed floor). Direction falls
  back to the site locale where research 11 says top-to-bottom for unknown languages; no
  two-line spine; no right-to-left or CJK setting. Spines wider than 15 mm only get a
  warning (`SpineOutsideTextRange`).
- **Font identification** is the classical v0; the learned embedding (research 10, training
  plan) is not built. Connected scripts and stencils are not recognised.
- **The shelf**: one JSON snapshot like the core; uploads held in memory while checked
  (bounded by size and concurrency); gift codes kept in clear in its database (the
  rights holder prints cards from them); no print orders and no webhook receiver; not
  implemented from research 09: retailer-watermark detection, the ISBN-registrant check
  and review state for `own_work`, translator's death year and "mort pour la France".
- **Server storage** is one JSON snapshot rewritten per change; request bodies (covers, card
  PDFs) are buffered in memory; rate-limit state and partners' download links are in memory.
- **Mailed cards**: the cheapest postal API (research 04) takes one image per card, so the
  four-panel card would go by letter with the A4 PDF; no real printer has seen a card.
- **Devices**: e-reader behaviour is simulated (user agents, JavaScript off, mail sink, LCP
  shape only), not tested on hardware.
- **Partner card options**: a partner cannot yet choose output profile, spine style or ground.
- **Print-review gaps left open** (research 03 §10.1): (b) no per-printer profile records
  (bleed, crease encoding, label, `confirmed` status) — only the three built-in profiles,
  though any ink limit set on a `PrintProfile` (260 %, 270 %, …) is now honoured;
  (d) no generated plan PDF or French printer brief (implications 11–12). Also not checked
  by preflight: B8 colour *boundaries* near creases (heavy ink there is: a warning),
  D3/D5/D6 type-size and safety-zone geometry, C7 overprint; and the stated output
  condition is not compared with the profile's own description.
- `docs/api.md` does not yet list the 422 rights codes of `card.pdf` (they are documented in
  `crates/ebc-server/README.md`).

## Decisions taken without the owner (veto any)

1. Gift code: 12 letters from 20 consonants (`XXXX-XXXX-XXXX`) — no digits.
2. Pair code: 6 letters with a global failure budget; explicit device chooser on every gift page.
3. Conservative rights defaults: typographic card unless a rights basis is declared; covers
   never cropped by default; looked-up covers never printed; no generative lettering.
4. Fictional demo store names with a visible notice.
5. Bundled output profile: basICColor "ISO Coated v2 300 %" (FOGRA39L, zlib licence, same file
   as Scribus ships). ECI's own profiles may not be bundled.
6. `jpeg-encoder` (IJG licence) accepted as a test-only dependency.
7. Font stack migrated from the unmaintained `ttf-parser`/`rustybuzz` to `harfrust`/`read-fonts`.
8. The shelf: `own` is the default mode; its UI name is "Étagère"; a takedown removes every
   hosted copy of the same file (all uploaders are told; only the noticed uploader gets a
   strike); a suspended account's other gifts keep working; a further upheld notice while
   suspended bans; closed or banned addresses cannot sign up again until their identity is
   erased; the designer link carries the code in the URL fragment, not the query string.

## Owner actions

- **Secrets** (sparring review): `ANTHROPIC_API_KEY` or `CLAUDE_CODE_OAUTH_TOKEN`.
- **Settings**: allow auto-merge and squash merging; create labels `dependencies`, `rust`,
  `github-actions`, `javascript`; protect `main` requiring `ci-ok`.
- **Network** for live validation of the catalogue: allow the provider hosts, then
  `EBC_LIVE=1 cargo test -p ebc-catalog --features client -- --ignored`.
- **Shelf before going public**: set `EBC_SHELF_CONTACT_EMAIL` and the `EBC_SHELF_LEGAL_*`
  identities, a real SMTP relay, EU hosting; have counsel confirm the identification
  fields, retention periods, the host-vs-platform status and the legal wording
  (`crates/ebc-shelf/README.md`, "Legal duties").
- **Reviews**: a lawyer on `docs/research/07`–`09`; native speakers on the UI strings; a real
  printer quote and a test print.

## How the work was done

The orchestration scripts used are in `tools/orchestration/`. Reviewer findings and reports
from the interrupted first night are kept in `docs/status/` for traceability.
