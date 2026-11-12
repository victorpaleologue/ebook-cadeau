# ebc-fontid — measured accuracy on synthetic covers

Measured on 2026-11-12 with `crates/ebc-fontid` (descriptor format version 1), release build,
native x86-64 (Intel Xeon @ 2.10 GHz vCPU, one thread), rustc 1.97.0. All numbers below were
printed by the `eval` example; nothing was edited by hand. **These are synthetic covers, not
real ones** — read "What the numbers mean" before quoting any of them.

Reproduce:

```sh
# Gallery A — the 8 bundled faces (offline, ~2 min):
cargo run -p ebc-fontid --release --example eval --features synth
# Gallery B — 238 openly licensed faces (needs raw.githubusercontent.com once, 44 MB, ~5 min):
python3 -I crates/ebc-fontid/scripts/fetch-eval-corpus.py          # -> target/fontid-corpus (git-ignored)
cargo run -p ebc-fontid --release --example eval --features synth -- \
    --corpus target/fontid-corpus --cases 128 [--verbose] [--dump FILE]
```

## Headline

| | Gallery A: 8 bundled faces, 4 families | Gallery B: 238 open faces, 139 families |
|---|---|---|
| Clean covers, title box given: exact face top-1 / top-3 | 100 % / 100 % | 98 % / 98 % |
| Clean: family top-1 | 100 % | 98 % |
| "Typical web cover" (degraded): face top-1 / top-3 | 100 % / 100 % | 90 % / 92 % |
| Typical: family top-1 | 100 % | 91 % |
| Hard (small, textured, blurred, JPEG q35–60): face top-1 / top-3 | 72 % / 98 % | 45 % / 52 % |
| Hard: family top-1 | 92 % | 55 % |
| Title found by the detector (first region), clean / typical / hard | 100 / 100 / 94 % | 96 / 96 / 95 % |
| End to end (detector's box), family top-1, clean / typical / hard | 100 / 100 / 89 % | 96 / 88 / 52 % |
| **True family removed** (real-cover proxy): nearest font has the same category, clean / typical | — | 83 % / 79 % (chance 30 %) |
| `identify()` time per cover (600 × 900, box given) | 5 ms | 7–8 ms |
| `identify()` time against 1,000 faces | 24 ms | 17 ms |
| `find_title_regions()` time per cover | 20–37 ms | 21–36 ms |
| `rerank()` time, 5 candidates | 24–51 ms | 25–44 ms |
| Index size | 71.5 KB (8 faces) | 2.12 MB (238 faces, ~8.9 KB per face) |

Gallery A is far too easy (four families that look nothing alike) and is reported because the
unit tests use it; **gallery B is the number to quote**, with the hold-out rows as the proxy for
real covers.

## Method

**Covers** (`ebc_fontid::synth::render_cover`, deterministic, seed 2026): 600 × 900 px; a title
from a list of 28 French and English titles (accents, apostrophes, digits, `?`) in the test
face, centred or left-aligned in the upper half; an author line (fictional names) in another
face of the gallery near the bottom; on half the covers a rule under the title and a disc
(ornaments the detector must ignore); light-on-dark on half the covers; 30 % of the titles shown
in capitals (the pipeline is always given the catalogue case and must infer the case). Then, per
condition: rotation, Gaussian blur, Gaussian noise, and a JPEG-like round trip (8 × 8 DCT with
the standard tables scaled as libjpeg does, 4:2:0 chroma), all implemented in the crate.

**Conditions** (ranges sampled uniformly per cover; cap heights for a 900 px cover):

| Condition | Cap height | Background | Blur σ | Noise σ | Rotation | JPEG q | Tracking (em) |
|---|---|---|---|---|---|---|---|
| clean | 30–70 px | flat | – | – | – | – | 0 |
| typical (degraded) | 26–60 | flat, gradient, textured | 0.4–0.9 | 2–5 | ±1° | 60–85 | −0.01..0.05 |
| hard (degraded) | 16–30 | gradient, textured | 0.8–1.4 | 6–12 | ±2° | 35–60 | 0..0.12 |
| clean + one factor | as clean, with one factor changed (see the row name) | | | | | | |

"Title unknown" rows run the same covers without the title (unlabelled fallback). The faces
are used in turn (cover *i* uses face *i* mod *n*): 64 covers per condition for gallery A
(each face 8 times), 128 for gallery B (128 of the 238 faces once per condition, a different
title, size and background each time).

**Measures.** *Box*: `identify_detailed` with the true title box (what a user who confirms the
box gives). *Rerank*: `rerank_detailed` on the 5 best descriptor matches with their font files.
*Detected*: the first region of `find_title_regions` (or `find_text_regions` without the
title) has IoU ≥ 0.5 with the true box, or lies inside it and spans ≥ 30 % of it (one line of
a multi-line title). *Auto*: `identify_detailed` on that first region. *Face* = family and style;
*family* = the family name of the font's name table. *Confident (right)*: share of covers flagged
`confident`, and in brackets the share of those whose first match is the right family.

**Against leakage.** The index stores one descriptor per character rendered *alone* at 96 px/em
with `ebc_text::try_rasterize`. The covers shape whole *words* (kerning, contextual forms) at
14–70 px cap height, rasterise them at twice the size with a random sub-pixel offset and average
2 × 2 blocks (other anti-aliasing), composite them over a background of another colour, then
degrade them; the region is cut, binarised and segmented from the cover like any other image,
and the case is inferred. What is *not* removed: covers and index use the same rasteriser and,
except in the hold-out rows, the true outlines are in the index. Clean rows are therefore an
upper bound — the research baseline in `notes/font-id-tech.tools.md` §7.2 reports the same
effect (97.5 % clean top-1 on 529 faces with the same kind of set-up).

## Gallery A — the 8 bundled faces

Atkinson Hyperlegible Regular/Bold/Italic, Spectral Regular/SemiBold/Italic, Bebas Neue,
Courier Prime Bold (all OFL-1.1, licence files in `assets/fonts`). 64 covers per condition.

| Condition | n | Labelled | Box: top-1 | Box: top-3 | Box: family | Rerank: top-1 | Rerank: top-3 | Rerank: family | Detected | Auto: top-1 | Auto: top-3 | Auto: family | Confident (right) | detect ms | identify ms | rerank ms |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| clean | 64 | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % (100 %) | 31.2 | 5.3 | 39.4 |
| typical (degraded) | 64 | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 97 % (100 %) | 29.5 | 5.7 | 31.0 |
| hard (degraded) | 64 | 73 % | 72 % | 98 % | 92 % | 84 % | 100 % | 97 % | 94 % | 67 % | 95 % | 89 % | 30 % (100 %) | 33.5 | 8.1 | 44.4 |
| clean + blur 1.0–1.4 | 64 | 98 % | 98 % | 100 % | 100 % | 98 % | 100 % | 100 % | 97 % | 98 % | 100 % | 100 % | 91 % (100 %) | 37.0 | 7.2 | 50.9 |
| clean + noise 8–12 | 64 | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % (100 %) | 31.4 | 5.1 | 35.6 |
| clean + rotation ±2° | 64 | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % (100 %) | 35.2 | 8.3 | 42.8 |
| clean + JPEG q30–45 | 64 | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % (100 %) | 29.9 | 5.4 | 38.9 |
| clean + small text 14–20 px | 64 | 98 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 95 % | 95 % | 95 % | 95 % | 97 % (100 %) | 27.9 | 5.2 | 37.1 |
| clean + textured background | 64 | 98 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 97 % | 100 % | 100 % | 100 % | 97 % (100 %) | 35.3 | 5.3 | 37.6 |
| clean + tracking 0.10–0.20 em | 64 | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 98 % | 98 % | 98 % | 98 % | 100 % (100 %) | 33.2 | 7.2 | 49.0 |
| clean + tight tracking −0.06..−0.03 | 64 | 84 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 94 % | 94 % | 94 % | 95 % | 83 % (100 %) | 30.7 | 4.8 | 32.5 |
| clean + multi-line | 64 | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % (100 %) | 35.4 | 6.3 | 35.1 |
| clean + ALL CAPS on cover | 64 | 98 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 100 % | 98 % (100 %) | 27.5 | 3.9 | 23.6 |
| clean, title unknown | 64 | 0 % | 100 % | 100 % | 100 % | – | – | – | 98 % | 100 % | 100 % | 100 % | 0 % (–) | 22.9 | 5.1 | – |
| typical, title unknown | 64 | 0 % | 100 % | 100 % | 100 % | – | – | – | 100 % | 100 % | 100 % | 100 % | 0 % (–) | 20.5 | 5.6 | – |

Hold-out (true face removed from the index; title box given):

| Condition | n | Sibling is top-1 | Sibling in top-3 | Chance (top-1) |
|---|---|---|---|---|
| clean | 48 | 83 % | 100 % | 29 % |
| typical (degraded) | 48 | 67 % | 100 % | 29 % |

## Gallery B — 238 openly licensed faces

The 8 bundled faces plus 233 font files of 138 families from the `ofl/` and `apache/`
directories of github.com/google/fonts (list: `crates/ebc-fontid/scripts/eval-corpus.txt`;
serif 55 families, sans serif 34, display 24, handwriting 20, monospace 5 by their
`METADATA.pb`). Every file entered the gallery only with its licence file next to it (233 ×
OFL-1.1, 5 × Apache-2.0 after deduplication) and through `FontIndex::build_checked`; 3 Zilla
Slab weights were **refused** because their `fsType` forbids bundling. The corpus is a
measuring tool kept under `target/`; it is not shipped and not committed. It was chosen by
hand for cover-like faces (book serifs, condensed display, scripts) plus common text faces —
it is not a random sample of Google Fonts. 128 covers per condition.

| Condition | n | Labelled | Box: top-1 | Box: top-3 | Box: family | Rerank: top-1 | Rerank: top-3 | Rerank: family | Detected | Auto: top-1 | Auto: top-3 | Auto: family | Confident (right) | detect ms | identify ms | rerank ms |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| clean | 128 | 94 % | 98 % | 98 % | 98 % | 98 % | 98 % | 98 % | 96 % | 96 % | 96 % | 96 % | 82 % (100 %) | 29.5 | 7.0 | 35.4 |
| typical (degraded) | 128 | 89 % | 90 % | 92 % | 91 % | 90 % | 92 % | 91 % | 96 % | 88 % | 90 % | 88 % | 81 % (99 %) | 31.5 | 8.4 | 36.6 |
| hard (degraded) | 128 | 69 % | 45 % | 52 % | 55 % | 48 % | 52 % | 58 % | 95 % | 43 % | 49 % | 52 % | 24 % (94 %) | 31.2 | 8.8 | 40.2 |
| clean + blur 1.0–1.4 | 128 | 85 % | 83 % | 90 % | 84 % | 90 % | 91 % | 91 % | 95 % | 81 % | 88 % | 83 % | 66 % (100 %) | 29.6 | 7.7 | 39.2 |
| clean + noise 8–12 | 128 | 92 % | 96 % | 97 % | 96 % | 96 % | 97 % | 97 % | 98 % | 95 % | 96 % | 95 % | 84 % (100 %) | 31.0 | 7.4 | 37.5 |
| clean + rotation ±2° | 128 | 90 % | 95 % | 95 % | 95 % | 95 % | 96 % | 97 % | 95 % | 92 % | 93 % | 92 % | 81 % (100 %) | 34.5 | 10.2 | 40.7 |
| clean + JPEG q30–45 | 128 | 92 % | 95 % | 96 % | 95 % | 98 % | 98 % | 98 % | 96 % | 94 % | 95 % | 94 % | 80 % (100 %) | 28.3 | 7.2 | 36.1 |
| clean + small text 14–20 px | 128 | 84 % | 84 % | 90 % | 85 % | 88 % | 91 % | 89 % | 92 % | 79 % | 85 % | 80 % | 73 % (100 %) | 28.3 | 7.1 | 35.0 |
| clean + textured background | 128 | 95 % | 98 % | 98 % | 98 % | 98 % | 99 % | 99 % | 98 % | 97 % | 98 % | 97 % | 86 % (100 %) | 36.3 | 8.1 | 38.8 |
| clean + tracking 0.10–0.20 em | 128 | 94 % | 96 % | 98 % | 96 % | 96 % | 98 % | 97 % | 95 % | 95 % | 95 % | 95 % | 83 % (100 %) | 28.2 | 7.9 | 44.2 |
| clean + tight tracking −0.06..−0.03 | 128 | 77 % | 94 % | 96 % | 94 % | 96 % | 97 % | 96 % | 91 % | 88 % | 89 % | 88 % | 66 % (100 %) | 28.2 | 6.2 | 33.6 |
| clean + multi-line | 128 | 93 % | 98 % | 98 % | 98 % | 98 % | 98 % | 98 % | 98 % | 96 % | 97 % | 96 % | 84 % (100 %) | 34.3 | 8.2 | 30.7 |
| clean + ALL CAPS on cover | 128 | 95 % | 98 % | 99 % | 98 % | 98 % | 99 % | 99 % | 98 % | 97 % | 98 % | 97 % | 73 % (100 %) | 29.9 | 6.1 | 25.5 |
| clean, title unknown | 128 | 0 % | 98 % | 98 % | 98 % | – | – | – | 98 % | 97 % | 97 % | 97 % | 0 % (–) | 21.4 | 6.0 | – |
| typical, title unknown | 128 | 0 % | 87 % | 91 % | 88 % | – | – | – | 100 % | 87 % | 91 % | 88 % | 0 % (–) | 20.7 | 6.9 | – |

Hold-out (true face removed from the index; title box given; only faces with a sibling style):

| Condition | n | Sibling is top-1 | Sibling in top-3 | Chance (top-1) |
|---|---|---|---|---|
| clean | 84 | 20 % | 27 % | 1 % |
| typical (degraded) | 84 | 20 % | 32 % | 1 % |

Family hold-out (every face of the true family removed; title box given; category from
google/fonts `METADATA.pb`, Bebas Neue has none and is skipped):

| Condition | n | Same category top-1 | Same category in top-3 | Chance (top-1) |
|---|---|---|---|---|
| clean | 127 | 83 % | 91 % | 30 % |
| typical (degraded) | 127 | 79 % | 86 % | 30 % |

Face top-1 by category (true box), from the per-cover dump of the same run:

| Condition | Serif | Sans serif | Display | Handwriting | Monospace |
|---|---|---|---|---|---|
| clean | 50/50 | 42/42 | 13/14 | 11/12 | 7/7 |
| typical | 44/50 | 41/42 | 10/14 | 10/12 | 7/7 |
| hard | 23/50 | 16/42 | 9/14 | 5/12 | 3/7 |

(3 covers drawn with Atkinson Hyperlegible/Bebas Neue faces have no category in this split.)

## Timings and scaling

Per cover, means over each condition (gallery B, 238 faces): `find_title_regions` 21–36 ms,
`identify` 6–10 ms, `rerank` of 5 candidates 25–47 ms (dominated by rasterising the title with
each candidate). Building the index: 12.6 ms per face (3.0 s for 238). `identify` on one typical
cover against an index of repeated entries:

| Faces in the index | detect ms | identify ms |
|---|---|---|
| 238 | 42.2 | 12.1 |
| 300 | 40.4 | 11.8 |
| 1000 | 38.0 | 17.0 |

The descriptor comparison is cheap (an index of 1,000 faces adds ~5 ms); segmentation and
labelling dominate. **Not measured:** the same code compiled to `wasm32-unknown-unknown` in a
browser. The crate builds for that target, uses no threads, no clock and no randomness; expect
it to be a few times slower than these native numbers (an estimate, not a measurement). The
index is ~8.9 KB per face uncompressed: 300 faces ≈ 2.7 MB, which argues for serving the index
compressed or in tiers.

## Failure cases (gallery B, from `--verbose`)

- **Connected scripts and stencils.** Great Vibes, Alex Brush, Allura, Cookie (joined letters)
  and Black Ops One (stencil: letters fall apart) give no labelled glyphs: the glyph count never
  fits the title, the unlabelled fallback is used and the true face can drop far down (Great
  Vibes at 15 px cap height: rank 147). The detector also misses them: it picks the author line,
  whose letters are separate. Handwriting faces are labelled on only half of the clean covers.
- **Hairline and high-contrast display faces** (Italiana, Elsie, Gilda Display, Cormorant SC):
  blur and JPEG break the hairlines, letters split into pieces, labelling fails and the fallback
  ranks the true face 19th–121st on typical covers.
- **Small and blurred text.** At 14–20 px cap height or blur σ ≥ 1, labelling drops to 84–85 %
  and face top-1 to 83–84 %; on hard covers to 45 %. Thin strokes merge or vanish in the
  binarisation; this is the first cause of error on the hard row, as in the research baseline.
- **Tight tracking** (−0.06..−0.03 em): letters touch, labelling falls to 77 % (the alignment of
  touching pairs recovers part of it); identification of what is labelled stays at 94 %.
- **Near-identical designs.** 5 of the 1,920 covers were flagged confident without the true face
  first. Two are a sibling style (Anonymous Pro Regular read as Anonymous Pro Bold, Hind Light
  as Hind Regular); three are another family by name with nearly the same letterforms (DM Serif
  Text read as DM Serif Display, Khand as Economica Bold, Khula as Hind Medium). For typesetting
  a spine these are harmless; for "naming the font" they are wrong.
- **Detection.** Without the prior, the largest regular text wins; with the title prior the
  first region is the title on 95–98 % of covers. Misses are scripts (above) and covers where the
  title is small and the author line is comparatively large.

## Confidence

`Identification::confident` requires ≥ 4 labelled glyphs, a best score ≥ 0.80 and a lead of
≥ 0.04 over the best font of another family. On gallery B it was set on 82 % of clean, 81 % of
typical and 24 % of hard covers; when set, the first match was of the right family 100 %, 99 %
and 94 % of the time. It is never set by the unlabelled fallback (title unknown, or glyph count
not matching the title). The thresholds were read from the `--dump` output of these synthetic
runs; on real covers they must be re-calibrated, and the product rule (`08-legal-fonts-and-genai.md`
§ Implications, item 6) of falling back to a house face below 0.80 remains the safe default.

## What the numbers mean (and do not)

- The clean and typical rows say the pipeline works when the cover's font is in the index:
  the descriptor stage alone ranks the right face first for 9 covers in 10 among 238 faces,
  the render-and-compare stage adds a few points on blur and JPEG, and the title box can be
  found automatically on 95 %+ of these covers.
- Real covers are mostly set in commercial or hand-drawn lettering that **no open index
  contains**. The family hold-out is the relevant proxy for them: with the true family removed,
  the nearest open face is of the same broad category 4 times in 5 (chance 3 in 10). Whether it
  *looks* close enough for a spine is a product judgement the numbers cannot make; offer the top
  few as rendered previews, as the research note recommends.
- Synthetic covers have one title font, no photographs behind the text, no outlines, shadows,
  arcs or textures inside letters, and the true glyph outlines. No real-cover test set labelled
  with "nearest open font" exists yet (`notes/font-id-tech.tools.md` §4); building one is the next
  step before trusting any threshold.
