# Project status — 2026-11-12, 05:45 UTC

**The build was interrupted.** Around 01:30 UTC the account's usage limit was reached and all
17 background jobs (4 research workflows, 13 build units) stopped mid-flight. Nothing written
to disk was lost; this page says exactly how far each piece got. No new agents were launched
after the limit reset — see "How to resume" for the choice that is yours to make.

## What works today

- The workspace **compiles** (all 20 crates, all targets) and the pure libraries build for
  `wasm32-unknown-unknown`.
- `cargo fmt --all -- --check` is clean.
- `cargo test --workspace`: **945 pass, 128 fail**. The failures are listed per crate below;
  most are reviewer-written tests demonstrating defects whose fixes never ran.
- `cargo clippy --workspace --all-targets -- -D warnings` fails in `ebc-demo-stores`,
  `ebc-demo-ereaders`, and in test code of `ebc-model`, `ebc-pdf`, `ebc-qr`, `ebc-text`.
- `cargo deny check` fails on two "unmaintained" advisories (see Decisions).
- **No card PDF can be generated end to end yet**: the composer (`ebc-card`) is not written.

## Crates

Stages: **I** implemented · **R** reviewed by two sparring agents · **F** findings fixed ·
**V** independently verified. A dash means the stage did not run.

| Crate | Stage | Tests pass / fail | State |
|---|---|---|---|
| `ebc-scene` | I (by hand) | 18 / 0 | Stable contract |
| `ebc-isbn` | I – – – | 43 / 0 | ISBN-10/13, hyphenation table, extraction. Never reviewed |
| `ebc-model` | I – – – | 58 / 0 | Book, gift, HTTP types. **Gift code is still the v0.1 format** (see Open inconsistencies) |
| `ebc-i18n` | I – – – | 43 / 1 | 7 locales, spine direction. Never reviewed |
| `ebc-color` | I R – – | 96 / 6 | Palette, contrast, CIEDE2000. 4 audit findings unfixed |
| `ebc-qr` | I R – – | 54 / 3 | QR for print, vector output. Findings unfixed (URL upper-casing edge cases) |
| `ebc-text` | I – – – | 127 / 6 | Shaping, fitting, outlines, raster. Review died; failing tests are the breaker's |
| `ebc-layout` | I R – – | 88 / 12 | Formats, marks, profiles. **11 findings unfixed**, incl. "home-profile page is not the sheet" |
| `ebc-pdf` | I – – – | 118 / 0 | PDF/X writer; validated in its own tests with qpdf/poppler/gs. Never reviewed |
| `ebc-preflight` | partial | 98 / 79 | Implementer stopped mid-way; one shared fixture helper fails 39 tests |
| `ebc-catalog` | partial | 0 / 0 | ~2 700 lines, compiles, no tests, providers incomplete |
| `ebc-covers` | partial | 0 / 0 | ~2 600 lines (sniffing, placeholder, quality, upscale), no tests |
| `ebc-fontid` | partial | 0 / 0 | ~2 600 lines (components, descriptors, index), no tests, no evaluation |
| `ebc-spine` | partial | 0 / 0 | ~1 000 lines, compiles, no tests |
| `ebc-server` | partial | 0 / 0 | ~2 400 lines (config, codes, device detection, rate limiting, uploads); **no routes yet** |
| `ebc-card`, `ebc-cli`, `ebc-wasm` | not started | – | Skeletons only |
| `demo/stores` | I – – – | 102 / 6 | Two fictional stores ("Septante", "Grande Enseigne"). Built against API v0.1 |
| `demo/printer` | I – – – | 24 / 14 | Fictional printer; failures follow `ebc-preflight` |
| `demo/ereaders` | I – – – | 75 / 1 | Simulated e-readers, OPDS client, mail sink. Built against API v0.1 |
| `web/`, `e2e/`, `tools/` | not started | – | Empty |

Recovered for whoever continues: implementer reports in `docs/status/implement-reports/`,
reviewer findings (with evidence and suggested fixes) in `docs/status/review-findings/`.

## Research (`docs/research/`)

Intended pipeline per topic: independent researchers → synthesis → adversarial fact-check →
corrections applied. **Only one document completed it.**

| Document | Stage reached | Caveat |
|---|---|---|
| `01-isbn-databases` | synthesised | Not fact-checked |
| `02-cover-sources` | fact-checked, corrections **not applied** | Read with `factcheck/cover-sources.md` |
| `03-print-requirements` | synthesised | Not fact-checked |
| `04-printers` | fact-checked, corrections **not applied** | Read with `factcheck/printers.md`; prices are not quotes |
| `05-ereader-distribution-ux` | synthesised | Not fact-checked; device facts largely from memory |
| `06-terminology` | synthesised | Not fact-checked; needs native-speaker review |
| `07-legal-covers-and-reseller-rights` | complete | The fact-checker had no web access: nothing confirmed at source |
| `08-legal-fonts-and-genai` | fact-checked, corrections **not applied** | Read with `factcheck/legal-fonts-genai.md` |
| `09-legal-gifting-and-commerce` | corrections **partially** applied | Status line says "fact-checked" but the editor stopped mid-way |
| `10` font identification, `11` spine design | researcher notes only | See `notes/font-id-tech.*.md`, `notes/spine-design.*.md` |
| `12-market-and-business-inputs` | fact-checked, corrections **not applied** | Read with `factcheck/market-business.md` |

Two limits apply to all of it. Web search was rate-limited for part of the night, so several
researchers worked only from primary documents on `raw.githubusercontent.com` and tagged
claims as single-source `[V1]` or from memory `[M]`. And the legal documents are material to
brief a lawyer, not legal advice.

## Not done at all

Card composer and its samples · CLI · wasm bindings and web app · integration and
end-to-end tests · business model and ROI charts · font corpus and training kit ·
whole-repo adversarial review · any CI run on GitHub.

## Decisions taken without you (veto any)

1. **Gift code**: 12 letters from 20 consonants, `XXXX-XXXX-XXXX` (docs only so far).
   Rationale in `research/notes/ereader-ux.ux.md` §3: digits cost a keyboard layer switch on
   e-ink and phone keyboards and bring 0/O, 1/I confusion.
2. **Pairing code** 6 letters with a global failure budget; **explicit device chooser** on
   every gift page, because user-agent sniffing broke twice in 2024–2026.
3. **Demo store names** are fictional ("Septante", "Grande Enseigne") with a visible notice;
   the 7switch-like and Fnac/Kobo-like archetypes are named only in documentation.
4. **`jpeg-encoder` (IJG licence)** accepted as a test-only dependency in `deny.toml`.
5. **`ttf-parser` and `rustybuzz` are flagged unmaintained** (RUSTSEC-2026-0192, -0206).
   The plan was to migrate to `skrifa` + `harfrust`; the migration did not run, and the
   advisories are deliberately **not** suppressed, so the `deny` job stays red.

## Open inconsistencies

- `docs/api.md` and `docs/architecture.md` describe the **v0.2** code format; `ebc-model`
  still implements **v0.1** (10 Crockford symbols). `demo/stores` and `demo/ereaders` were
  built against v0.1; the partial server against v0.2.
- `ebc-layout` home profiles do not make the PDF page equal to the sheet (reviewer finding).
- `ebc-fontid/Cargo.toml` declared two examples that were never written; the declarations
  were removed so the workspace parses.

## How to resume

The orchestration is saved in `tools/orchestration/` (`ebc-research.workflow.js`,
`ebc-build.workflow.js`): each unit has its full brief, and the build script already
contains the briefs for the units that never started (card, font-stack migration, CLI,
wasm + web).

Cost basis: the session's usage counter read about **$579** when the limit hit, for roughly
75 agent runs, i.e. about $7–8 per run.

| Option | What it does | Rough size |
|---|---|---|
| **A. Full plan** | Finish the 5 partial crates, fix and verify every reviewed crate, review the unreviewed ones, build card / CLI / wasm + web, integration and e2e tests, business model, complete the research pipeline, whole-repo review | ~100 agent runs — about as much again as already spent |
| **B. Lean** | No sparring reviews, no research completion: fix failing tests, finish server + spine + card + CLI so a card PDF comes out end to end, minimal web page, business model | ~30 agent runs |
| **C. Stop** | Keep this state | — |

Suggested order inside A or B: `ebc-model` code v0.2 → `ebc-preflight` → `ebc-layout`
findings → `ebc-spine` → `ebc-card` (first end-to-end PDF) → server → the rest.
