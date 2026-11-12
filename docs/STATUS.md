# Project status — 2026-11-12

Everything the original request asked for exists in a first version. This page says how far
each piece goes, what is verified, and what is not. Earlier versions of this page (the
interrupted overnight build) are in the git history.

## Checks

Run on the whole workspace, locally, after the adversarial review fixes:

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | clean |
| `cargo test --workspace --locked` | **1 884 passed, 0 failed**, 19 ignored (opt-in live and reference tests); re-run 2026-11-12 after the ink-limit work |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | clean |
| `scripts/wasm-crates.sh build` (all pure libraries for `wasm32-unknown-unknown`) | clean |
| `cargo deny check` | advisories, bans, licences, sources: ok |
| `e2e/run.sh` (10 Playwright tests in Chromium) | 10 passed |
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
| Card designer web app | `web/` | done |
| Web service: redemption pages, creator and partner APIs, OPDS, pairing, send-to-device | `crates/ebc-server` | done |
| Fictional stores "Septante" and "Grande Enseigne", printer, e-readers | `demo/` | done |
| 21 Rust end-to-end journeys + `ebc-demo` binary | `demo/integration` | done |
| 9 browser end-to-end tests | `e2e/` | done |
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
- **Server storage** is one JSON snapshot rewritten per change; uploads are buffered in memory;
  rate-limit state is in memory.
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
- `docs/api.md` does not yet list `cover_rights`, the 422 rights codes and a few fields the
  server accepts (they are documented in `crates/ebc-server/README.md`).

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

## Owner actions

- **Secrets** (sparring review): `ANTHROPIC_API_KEY` or `CLAUDE_CODE_OAUTH_TOKEN`.
- **Settings**: allow auto-merge and squash merging; create labels `dependencies`, `rust`,
  `github-actions`, `javascript`; protect `main` requiring `ci-ok`.
- **Network** for live validation of the catalogue: allow the provider hosts, then
  `EBC_LIVE=1 cargo test -p ebc-catalog --features client -- --ignored`.
- **Reviews**: a lawyer on `docs/research/07`–`09`; native speakers on the UI strings; a real
  printer quote and a test print.

## How the work was done

The orchestration scripts used are in `tools/orchestration/`. Reviewer findings and reports
from the interrupted first night are kept in `docs/status/` for traceability.
