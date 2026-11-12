# Font identification on covers — tools, models, browser runtimes, and a training plan

_Status: synthesised from researcher notes, not independently fact-checked — research date 2026-11-12_

**Scope.** How eBook Cadeau finds, in the browser, an openly licensed font that resembles the title lettering of a book cover, so that the 10 mm spine can be typeset in a matching face. Covered: commercial identifiers, open models and datasets, title localisation, in-browser inference runtimes, a non-ML baseline, generative re-lettering, and a training plan for a small font-embedding model. Not covered: the law of fonts, look-alikes and generated lettering (`08-legal-fonts-and-genai.md` governs), the spine layout itself (`11-spine-design.md`), and the measured accuracy of the implemented pipeline (`fontid-eval.md`, maintained separately).

**Inputs.** `notes/font-id-tech.tools.md` (identification tools, runtimes, two local experiments) and `notes/font-id-tech.generation.md` (generation side, backgrounds, legibility). The code description in the last section comes from the source and documentation of `crates/ebc-fontid`, read on 2026-11-12.

**Evidence and its limits — read first.**

- **No web search ran** for either note; the shared search budget was spent. The tools note relied on package registries (npm, crates.io, PyPI: 48 registry URLs, versions, licences, packaged READMEs); the generation note read about 70 primary files on `raw.githubusercontent.com`. Hugging Face was blocked, so no model card or weight file size from there was checked.
- Everything about commercial services and research papers that is not tied to a fetched file is **from memory** and marked [M].
- Two **local experiments** [X] replace missing literature numbers: wasm binary size and single-thread speed of three runtimes on three CNNs, and a render-and-compare benchmark on 529 Google Fonts faces. Both are synthetic upper bounds: no real cover was processed, no phone was measured.
- This synthesis did not add evidence and was not fact-checked. Markers are the researchers': [V2] two independent sources read in that session, [V1] one, [X] measured locally, [L] computed locally, [M] memory, [E] engineering estimate.

---

## 1. Executive summary

| Question | Answer | Confidence |
|---|---|---|
| Can a commercial identifier (WhatTheFont, Adobe, Matcherator, WhatFontIs) be a pipeline component? | No. They answer with commercial fonts we may not embed, need the cover to leave the browser, and (as far as recalled) offer no public API except WhatFontIs | [M] |
| What is the real task? | Not "which font is this" but "which **open** font that we may typeset with is nearest". The gallery is the open corpus: Google Fonts has 1,908 families (1,892 with Latin), 98 % SIL OFL 1.1 | [V1] |
| Is there an off-the-shelf open model for "cover crop → nearest open Latin font"? | No. Reusable parts: the DeepFont recipe (synthetic renders + domain adaptation), multi-task heads, perhaps one ONNX classifier to re-check | [M] |
| Does a non-ML baseline work? | On easy covers, yes: render the known title in each candidate and compare. Top-1 / top-5 = 97.5 / 100 % clean, 76 / 94 % on typical web-cover degradations, 13 / 20 % on hard ones (529 faces, true face present) | [X], synthetic upper bound |
| Can a Rust-only stack run a small CNN in the browser? | Yes. `rten` 0.28 builds to a 3.3 MB wasm (1.0 MB gzip) and ran fp32 and int8 ONNX; MobileNet-1.0 class takes 17–27 ms single-threaded on a server vCPU | [X] |
| Is a 2–10 MB model realistic? | Yes: MobileNetV1-1.0 class is 13.8 MB fp32 / 3.7 MB int8; ResNet-18 is 45.2 / 11.4 MB | [X] |
| Should generative models re-draw the lettering? | No. Few-shot font generators are blocked by licence, script or both; scene-text editors are research-grade and gigabyte-sized; none produces print-resolution vector text | [V1] per model |
| What should ship? | Stage 1: classical render-and-compare over a curated open gallery, five previews for the user to pick. Stage 2: a 1–4 MB int8 embedding CNN for coarse retrieval over the whole open corpus, with render-and-compare kept as the verifier | [E] |

Findings that shape the code:

1. **The spine repeats the cover's own title.** The string is known, so the problem is retrieval and verification, not recognition, and no unseen glyph has to be generated [L, reasoning].
2. **At spine size the fine style is invisible.** A 9–12 pt title has a cap height of 2.1–2.8 mm. What survives is weight, width, serif class, case, slant, spacing and colour, which a good open face reproduces [L/E].
3. **Failure causes of the baseline, in order:** ink extraction on textured backgrounds, letter-spacing, small text. Rotation up to 3°, JPEG and simple shadows cost little [X].
4. **Top-5 with a human pick beats top-1 automation:** top-5 is 18 points above top-1 on typical input [X].
5. **Stay single-threaded.** Threaded wasm needs cross-origin isolation (COOP/COEP), which a widget embedded in a reseller page cannot assume [V2]. Measured single-thread latencies make threads unnecessary.
6. **The true font is usually absent from any open gallery.** With the true family removed, the nearest face is in the right broad category 74 % of the time on typical input and at chance on hard input [X]. The product must say "nearest open face", never "your cover's font".

---

## 2. Commercial identifiers

All [M]: the tools note could not search, and nothing here was re-verified.

| Tool | Owner | How it works (recalled) | Catalogue | Public API | Fit |
|---|---|---|---|---|---|
| WhatTheFont | MyFonts / Monotype | Deep-learning matcher since 2017; several text blocks per photo | MyFonts commercial | None known | Manual oracle only |
| Adobe Fonts visual search, Capture "Type", Photoshop "Match Font", Illustrator "Retype" | Adobe | DeepFont lineage | Adobe Fonts + installed fonts | None for identification | Manual oracle; needs Creative Cloud |
| Matcherator / Font Squirrel identifier | Fontspring | Upload, glyph boxes | Fontspring (+ Font Squirrel free fonts) | None | Manual oracle; free-font licences vary |
| WhatFontIs | WhatFontIs | User confirms per-glyph segmentation, then matching; "free only" and "Google Fonts only" filters | ~1M fonts claimed | A keyed HTTP API is recalled; quota, price, terms unknown | Only conceivable server-side fallback; uploads the cover |
| Identifont | Identifont | Questionnaire, no image | Own | None | Not applicable |
| "Like That Garden" | — | Recalled as a plant identifier, possibly confused with LikeFont | — | — | Not applicable |
| WhatFont, Fonts Ninja (extensions) | various | Read CSS of live web text | — | — | Not applicable to raster covers |

The class is out for three structural reasons independent of the unverified details: the answer is usually a commercial face, the cover leaves the browser, and consumer tools' terms generally forbid automated use [M].

---

## 3. Open models and datasets

### 3.1 Identification models (all [M] unless marked)

| Work | What it is | Data | Browser fit | Licence |
|---|---|---|---|---|
| DeepFont (Wang et al., ACM MM 2015) | CNN + stacked convolutional auto-encoder on unlabelled real + synthetic images to close the synthetic-to-real gap; 105×105 grey patches | AdobeVFR: 2,383 classes, ~1,000 synthetic images each, 4,384 labelled real test images, ~200k unlabelled real; top-5 above 80 % on real | The idea matters, not the weights | Research dataset; labels are commercial fonts |
| Chen et al., CVPR 2014 | Local features + nearest class mean | VFR-447 / 2420 | Pre-CNN | Paper |
| gaborcselle/font-identifier | ResNet-18 on synthetic snippets | 48 fonts | ~45 MB fp32 | Not recalled |
| Storia-AI/font-classify | Classifier, ONNX export published | Mostly Google Fonts | Closest existing asset; re-check architecture, size, licence | Not recalled |
| HENet (2021) | Plug-in block suppressing salient features, ResNet-18 | AdobeVFR, Explor_all | ResNet-18 class | Research code |
| YuzuMarker.FontDetection | CJK recogniser that also regresses size, stroke, colours, direction | ~6k CJK fonts | Multi-task head is a good idea | Not recalled |
| FontCLIP (Eurographics 2024) | CLIP fine-tuned on font attributes | ~200 Latin fonts, 37 attributes | ~350 MB: not phone material | Research code |
| Font embeddings (deep-fonts 2016, IDEO Font Map, fontjoy) | Embeddings of clean glyph bitmaps | 50k / ~750 fonts | Tiny, but trained on clean renders | Mixed |

**No off-the-shelf open model maps a cover crop to the nearest open Latin font.** Cover-specific prior art to read once search is available: Shinahara et al., ICDAR 2019; Iwana et al., 2016 [M].

### 3.2 Font corpus

From `google-font-metadata` 6.0.8 (MIT; snapshot with last font change 2026-01-07) [V1]:

| Fact | Value |
|---|---|
| Families | **1,908** |
| By category | sans-serif 705, display 464, handwriting 348, serif 343, monospace 48 |
| With a `latin` subset | 1,892 |
| Variable families | 525 |
| Licences (1,869 with a record) | OFL 1.1: 1,828; Apache-2.0: 36; Ubuntu Font Licence: 5 |

- A Latin `400-normal` WOFF2 subset weighs about **25 KB per face** (315 families measured, 8.1 MB) [X]. A 500-face gallery is about 12.5 MB if shipped whole, which drives the staging in §6.
- Synthetic-text tooling: `trdg` 1.8.0 (MIT), `fonttools` 4.68.0 (MIT) [V1]; SynthTIGER (MIT) and SynthText (Apache-2.0) generators [V1, generation note].
- Other libre sources: Velvetyne, Open Foundry, League of Moveable Type, Font Library [M].
- **No real-cover evaluation set labelled with "nearest open font" exists** to the researchers' knowledge. It has to be built (a few hundred covers rank methods).

### 3.3 Locating the title

The title string is known, so a detector only proposes boxes and the known string confirms them.

| Option | Facts | Fit |
|---|---|---|
| User draws or confirms the box | No model | Default for v1: the tool is interactive anyway |
| PaddleOCR in the browser | `@paddleocr/paddleocr-js` 0.4.2 (Apache-2.0) and `ppu-paddle-ocr` 6.7.0 (MIT) run PP-OCRv5/v6 on ONNX Runtime Web [V2]; v6 "tiny" detection + recognition ~6 MB, v5 mobile ~12 MB [V1] | Best automatic option; costs ORT Web (14.85 MB wasm) plus OpenCV.js |
| `ocrs` 0.14 + `rten` 0.28 | Pure Rust, MIT OR Apache-2.0 [V1]; models English-centric [M] | Best fit for the Rust/wasm stack; accent coverage untested |
| Tesseract.js 7.0.0 | Apache-2.0; wasm 2.9–3.5 MB; French data 7.0 MB; returns symbol boxes [V1]; weak on display lettering [M] | Symbol boxes attractive; quality on covers is the risk |
| CRAFT, EAST, DBNet, docTR | CRAFT ~80 MB [M]; a popular EAST implementation is GPL-3.0 [M]; docTR/OnnxTR Apache [V1] | Size or licence against them; DBNet via PP-OCR |
| Classical (MSER, SWT, Otsu, connected components) | `imageproc` 0.27 (MIT) has thresholding, components, contours, distance transform, template matching, HOG [V1] | Enough for flat covers with a confirmed box |

OpenCV.js (`@techstark/opencv-js` 5.0.0) is a single ~13 MB file [V1]: prefer `imageproc`.

---

## 4. In-browser inference runtimes

### 4.1 Registry facts (2026-11-12) [V1]

| Runtime | Version, licence | Notes | Shipped size |
|---|---|---|---|
| ONNX Runtime Web | 1.31.1, MIT | wasm (all browsers), WebGPU ("experimental"), WebGL (maintenance), WebNN (flag). Its README browser matrix appears to lag current browsers [M] | wasm 14.85 MB (3.80 MB gzip) [X] |
| Transformers.js | 4.3.2, Apache-2.0 | Wrapper over ORT; CDN by default (self-hosting configurable) | 2.3 MB + ORT |
| tract | `tract-onnx` 0.23.10, MIT OR Apache-2.0 | Pure Rust ONNX/NNEF/TFLite | §4.2 |
| rten | 0.28.0, MIT OR Apache-2.0 | Pure Rust, CPU, wasm SIMD; f32 and int8 | §4.2 |
| Burn | 0.22.1, MIT OR Apache-2.0 | `burn-onnx` generates Rust from ONNX; `burn-ndarray` and `burn-import` now marked deprecated | not measured |
| candle | 0.11.1 | wasm examples; partial ONNX [M] | not measured |
| ort (Rust) | 2.0.0-rc.15 with `ort-web` / `ort-tract` backends | One API over three engines | not measured |
| wonnx | 0.5.1, last release 2023-09 | Unmaintained | avoid |
| TensorFlow.js / LiteRT.js | 4.22.0 / 2.6.0, Apache-2.0 | — | no reason for a second format |

WebGPU support (MDN compat data [V1, generation note]): Chrome/Edge 113+ on Windows, macOS, ChromeOS; Linux only from 144 on Intel Gen12+; Chrome Android 121+; Safari 26+; Firefox 141+ on Windows and recent macOS on Apple silicon; no Firefox on Linux or Android. The identification net does not need WebGPU.

### 4.2 Measured binary size and single-thread speed [X]

Randomly initialised embedding CNNs, input 1×1×64×320, BatchNorm folded, opset 17; int8 = ORT static QDQ per-channel; `wasm32-unknown-unknown` + `simd128`, LTO, Node 22.23, one thread, 2.3 GHz Xeon vCPU, best of 30.

| Runtime (wasm raw / gzip) | MobileNetV1-0.5 (0.88M params) | MobileNetV1-1.0 (3.46M) | ResNet-18 (11.3M) |
|---|---|---|---|
| Model fp32 / int8 | 3.5 / 1.0 MB | 13.8 / 3.7 MB | 45.2 / 11.4 MB |
| ONNX Runtime Web 1.31.1 (14.85 / 3.80 MB) | 5.4 / 8.3 ms | 17.4 / 22.5 ms | 49–52 / 28–29 ms |
| tract-onnx, opt-level 3 (17.8 / 4.40 MB) | 4.9–5.1 / 5.7–6.2 ms | 16.9–17.7 / 19–20 ms | 59–63 / 64–67 ms |
| tract-onnx, opt-level s (9.7 / 2.54 MB) | 6.6 / 11.3 ms | 24.2 / 37.7 ms | 97 / 107 ms |
| **rten 0.28.0 (3.28 / 1.01 MB)** | 9.2 / 7.9–9.0 ms | 25.5 / 21–23 ms | 57–76 / 53–75 ms |
| Native ORT 1.29 (reference) | 0.85 / 0.65 ms | 3.1 / 1.5 ms | 9.0 / 3.5 ms |

- int8 buys a 4× smaller download, little or no wasm speed-up. Choose int8 for bandwidth.
- `tract-onnx` 0.23.10 does not build for `wasm32-unknown-unknown` out of the box (`getrandom` 0.4 needs a backend); a custom backend cfg worked.
- Peak wasm memory (tract, rten; model included): 9–13 MB, 26–47 MB, 72–141 MB for the three nets.
- **Phones were not measured.** At 2–4× slower than the vCPU [E]: 10–40 ms, 35–110 ms, 60–300 ms. Even a 10× miss stays under a second for a one-shot operation.

---

## 5. Non-ML baseline: render the known title and compare

### 5.1 Prior art [M]

Optical font recognition on scans (Zramdini and Ingold 1998, ~97 % over 280 models; Zhu, Tan and Wang 2001); Khoubyari and Hull (1996) matched word images against function words rendered in candidate fonts — the same idea with a known string; FyFont (2007, ~2,700 fonts). No published accuracy is known for known-string render-and-compare against an open gallery on real covers.

### 5.2 Experiment [X]

Gallery 529 faces from 315 Google Fonts families (stratified; ~50 cover-typical families hand-picked). Queries: 24 French titles × 10 random faces per condition = 240 (95 % interval about ±6 points); true face always present. Degradations: L0 clean 60–110 px; L1 "typical web cover" 28–60 px, gradient background, tracking −0.02..+0.06 em, rotation ±1°, blur, noise, JPEG q70–90, loose crop; L2 "hard" 18–36 px, textured low-contrast background, tracking 0..0.20 em, rotation ±3°, JPEG q40–70, shadow or outline. Matcher: per-channel Otsu with polarity from the border, deskew, normalise to 64×384; "line" score = z-scored sum of normalised cross-correlation, HOG cosine and truncated chamfer; "refit" = fit letter-spacing for the best 40–80 candidates, re-render, fuse with a glyph-level score.

| Condition | Line top-1 | Line top-5 | Refit top-1 | Refit top-5 | Nearest face in same category, true family removed (chance ~26 %) |
|---|---|---|---|---|---|
| L0 clean | **97.5 %** | 100 % | 88.3 % | 94.6 % | 80 % |
| L1 typical | **75.8 %** | **94.2 %** | 69.2 % | 83.8 % | 74 % |
| L1 + tracking 0..0.20 em | 42.1 % | 64.6 % | **60.0 %** | **71.7 %** | 63 % |
| L1 + small text (18–36 px) | 62.1 % | 82.5 % | 54.2 % | 74.6 % | 74 % |
| L1 + textured, low contrast | 46.7 % | 61.7 % | 37.9 % | 54.2 % | 60 % |
| L1 + shadow / outline | 77.1 % | 93.3 % | 68.8 % | 82.1 % | 70 % |
| L1 + rotation ±3° | 78.7 % | 94.6 % | 69.2 % | 81.2 % | 71 % |
| L1 + JPEG q40–70, more blur | 73.3 % | 91.2 % | 63.7 % | 78.7 % | 74 % |
| L2 hard | 12.9 % | 19.6 % | 14.6 % | 22.5 % | 30 % |

Other measurements: aspect ratio alone gives 0–8 % top-1. Naive glyph segmentation agreed with the true face's segmentation in 84 % (L0), 60 % (L1), 26 % (L2) of queries, which is why glyph-level scoring alone was worse than line-level (53 % vs 76 % top-1 at L1) and why refit fusion hurt when tracking was small. 12×16 px glyph thumbnails scored the same as 40×64. Cost in unoptimised Python: ~8 ms to render and featurise a title per face, ~60 ms to score one query against 529 faces, ~370 ms for the tracking refit.

### 5.3 Reading

- **Upper bound**: same rasteriser for query and gallery, true font present, single-line titles, synthetic backgrounds. Real covers add multi-line lockups, mixed fonts, arcs, textured letters and absent fonts.
- Untested improvements that follow: perceptual colour clustering instead of single-channel Otsu; connected components with dynamic-programming alignment to the known characters; fitting tracking for every candidate before the first ranking; fitting the weight axis of variable fonts (525 families); upscaling small covers.

---

## 6. Generative re-lettering

| Family | Examples | Why not now | Evidence |
|---|---|---|---|
| Few-shot font generation (raster) | FontDiffuser, Diff-Font, DG-Font, CF-Font, MX/LF/DM-Font, FUNIT, VQ-Font, IF-Font, MC-GAN, STEFANN | Licences: FontDiffuser "non-commercial research purposes"; FUNIT, Attr2Font, AGIS-Net CC BY-NC; DG-Font, CF-Font, Diff-Font no licence file and trained on Founder fonts "for non-commercial users"; NAVER models MIT but weights trained on unreleased fonts. Almost all Chinese-centric; only MC-GAN, AGIS-Net and STEFANN verifiably Latin. Output 64–128 px | [V1] each |
| Vector generation | DeepVecFont (v1, v2), DualVector, VecFusion, DeepSVG | Expect vector references; rely on sampling many candidates or differentiable rasterisation — not browser material | [V1]/[E] |
| Scene-text editing | SRNet, MOSTEL, STEFANN; UDiffText, TextCtrl, DiffUTE (glyph encoders); AnyText, GlyphControl, TextDiffuser (CLIP or layout transformers); TextDiffuser-2, Glyph-ByT5 (language models) | GAN ones are small but GPL or unlicensed and trained on low-res synthetic scene text; diffusion ones are gigabyte downloads (AnyText ~7.5 GB GPU memory per 512² image); no ONNX/WebGPU port known | [V1]/[M] |
| Background inpainting | MI-GAN (27.0 MB, MIT code and weights, runs in browsers via ORT Web); LaMa (93–206 MB, Apache-2.0); Stable Diffusion (gigabytes, needs a text encoder) | MI-GAN is the only shippable one; see `08-legal-fonts-and-genai.md` §7.5 for the decision to allow it only in "own rights" mode | [V2]/[V1] |

NAVER's own README states the cause of the licence wall: "fonts are protected intellectual property and it is unable to release the collected font datasets unless license is cleaned-up" [V1]. **Verdict: re-typeset the known string in the nearest open font; take only colours from the cover.** Tracing the cover's own lettering (vtracer, MIT OR Apache-2.0, pure Rust, wasm build [V2]) is technically easy and legally the riskiest option (doc 08 rung 4–5).

---

## 7. Recommended staged architecture

**Stage 0 — no identification.** A publisher-supplied spine or full wrap is used when present (doc 11).

**Stage 1 — classical, shippable now (Rust + wasm, no model).**

1. Title box: a proposed box (colour clustering + connected components) confirmed by the user; optional OCR confirmation later via `ocrs`/`rten`.
2. Gallery: 300–600 curated open faces chosen for cover typography. Do not ship whole fonts eagerly (25 KB per face): subset gallery faces to the title's characters on the server at card creation (`subsetter`, `fontcull`, `allsorts` exist [V1, existence and licence only]) or lazy-load in tiers (60 most cover-typical faces first).
3. Render-and-compare with tracking fitted per candidate, rasterised with a pure-Rust stack (outline reading, HarfRust shaping, fill).
4. UI: five spine previews, user picks; a neutral serif and sans always offered when confidence is low.

**Stage 2 — small embedding CNN for coarse retrieval over the full open corpus** (training plan in §8). One 128-byte int8 vector per face (6,000 faces ≈ 0.8 MB); the browser downloads only the top 10–20 fonts and runs Stage 1 on them as verification and parameter fitting.

**Stage 3 — optional.** In-browser text detection (PP-OCRv6 tiny, ~6 MB) to remove the manual box. Revisit generative lettering only if phone-class WebGPU and sub-100 MB text-editing models appear.

---

## 8. Training plan for the font-embedding model (Stage 2)

This plan is written so that whoever runs the training can execute it without further research. Every number marked [E] is an engineering estimate, not a measurement. Legal constraints come from `08-legal-fonts-and-genai.md` (rules R1, R12, R13; implications 27–29): **open fonts only, no cover images in the training data, a published training manifest, no third-party checkpoints.**

### 8.1 Goal and acceptance criteria

- **Function.** Map a grey crop of one line of title lettering to a 128-dimensional, L2-normalised embedding such that crops of the same face are close, regardless of the text, colours, background and degradations. The embedding is **text-independent**; the known title is used afterwards by the render-and-compare verifier (`ebc-fontid::rerank`).
- **Retrieval.** Nearest neighbours of the query embedding in a gallery index (one vector per open face) give the top 20 candidates; `ebc-fontid` reranks them and shows five.
- **Acceptance gates** (on the synthetic benchmark of §8.8, conditions L0/L1/L2 as in §5.2, gallery = the full open corpus):
  1. Retrieval recall@20 (true face among the 20 candidates): ≥ 99 % at L0, ≥ 95 % at L1, ≥ 50 % at L2.
  2. End-to-end (embedding top-20 → rerank) top-5 at L1 no worse than the render-and-compare baseline on the 529-face gallery (94 %), while the gallery grows to the whole corpus.
  3. Held-out families (true face absent): nearest face in the same Google Fonts category ≥ 75 % at L1.
  4. int8 model ≤ 4 MB, recall@20 within 1 point of fp32; single-thread wasm inference ≤ 40 ms on the reference vCPU with `rten` (the runtime with the smallest binary, §4.2).
  5. On the real-cover evaluation set (§8.8, ≥ 200 covers), a top-5 candidate judged acceptable at spine size in ≥ 70 % of typeset (not lettered) titles [E target].

### 8.2 Data: fonts (open only)

1. **Source**: a pinned commit of the Google Fonts repository (`https://github.com/google/fonts`). Record the commit hash in the manifest.
2. **Licence filter**: keep only files whose directory licence is `OFL.txt` (OFL-1.1), `LICENSE.txt` (Apache-2.0) or `UFL.txt` (UFL-1.0) — the set allowed for training by doc 08 implication 27. Parse the family's `METADATA.pb` for the licence and category; reject a family whose metadata and licence file disagree. Expected: about 1,870 families [V1 counts, §3.2].
3. **Script filter**: keep families with a `latin` subset that cover at least `A–Z a–z 0–9` and the French letters `àâçéèêëîïôûùüÿœæ` and their capitals; record coverage per face (needed for accented titles).
4. **Faces as classes**: static files are one class each. Variable fonts are instanced at their **named instances** (fvar) with `fonttools varLib.instancer`; each instance is a class. Estimate: 5,000–7,000 faces [E; the notes estimate 6,000].
5. **Labels per face**: family id, face id, Google Fonts category (serif, sans-serif, display, handwriting, monospace), weight (usWeightClass), italic flag, width class (usWidthClass), and whether the family is a "cover-typical" pick (the ~50 hand-picked families of §5.2, for stratified evaluation).
6. **Splits by family, never by face** (sibling weights must not straddle splits): 85 % train, 5 % validation, 10 % held-out test families, stratified by category. Additionally keep a "seen-family, unseen-string" test from the training families.
7. **Manifest** (doc 08 implication 28): one row per font file — family, style, version (name ID 5), SPDX licence, copyright line, source path and commit, SHA-256. CI refuses training inputs not in the manifest.

Other libre foundries (Velvetyne, Open Foundry, League of Moveable Type) may be added later under the same per-file licence check; they are not needed for the first model.

### 8.3 Data: text and rendering

1. **Strings**: (a) titles of public-domain works in French and English (titles are facts; take them from a public-domain catalogue the project already uses for fixtures, doc 09 implication 3), (b) random word sequences from French and English word-frequency lists under an open licence, (c) a stress list: accented capitals (É, À, Ç), `œ`/`Œ`, apostrophes, digits, `&`, `!`, `?`, `:`. Length 1–8 words, 3–40 characters. Case variants: as given, ALL CAPS, Title Case, and small caps where the face has `smcp`.
2. **Renderer**: render with the **same shaping and rasterisation stack as production** (`ebc-text`: HarfRust shaping, outline fill) through a small command-line renderer, so that training and inference see the same glyph geometry. If that renderer is not ready, use FreeType through Pillow and record the renderer in the model card; expect a small domain gap.
3. **Clean renders**: render each string as an alpha mask at x-height 16–120 px, with a 0.2–0.6 em margin, single line (90 %) or two lines (10 %, leading 1.0–1.3). Store masks, not composited images, so that augmentation can recolour them.
4. **Volume**: about 400 masks per training face (≈ 2.4M masks for 6,000 faces) [E]; at roughly 8 ms per render this is about 5–6 CPU-hours [X-based estimate: §5.2 render cost]. Masks compressed as PNG ≈ 10–20 GB [E].

### 8.4 Augmentations (on the fly, deterministic per seed)

The list mirrors the measured failure causes (§5.2): backgrounds, spacing and size first.

| Group | Augmentation | Range | Probability |
|---|---|---|---|
| Spacing | letter-spacing applied at render time (re-shaping), or per-glyph horizontal offsets | −0.05 to +0.30 em | 0.7 |
| Size | downscale then upscale to the input height (simulates small text) | effective x-height 8–60 px | 0.5 |
| Colour | text and background colours drawn independently; polarity inverted (light on dark) | contrast ratio 1.8–21 | 1.0 / 0.4 |
| Background | flat; linear or radial gradient; procedural noise (Perlin/value noise, paper grain); random crops of CC0 or public-domain photographs, blurred or not | — | 0.2 / 0.2 / 0.2 / 0.4 |
| Effects | outline (0.5–3 px), drop shadow (offset 1–6 px, blur 0–4 px), inner texture fill of the letters | — | 0.15 each |
| Geometry | rotation ±5°; perspective (corner jitter ≤ 6 %); slight arc warp (≤ 4 % of width); shear ±8° (fake italics are a confounder: label unchanged) | — | 0.5 / 0.3 / 0.1 / 0.1 |
| Crop | loose or tight box, jitter ±15 % of height, partial clipping of the first or last glyph | — | 0.8 |
| Distortion | Gaussian blur σ 0–1.5 px; noise σ 0–8; JPEG quality 30–95; halftone dot pattern (simulates print-and-scan) | — | 0.5 / 0.5 / 0.7 / 0.05 |
| Occlusion | a random bar or blob over ≤ 10 % of the crop | — | 0.1 |

Background photographs: only images whose licence (CC0 or public domain) is recorded in the manifest. **No book cover, no scraped image, no image of unknown provenance** (doc 08 R12: TDM reservations and integrity rights).

Domain gap: DeepFont reduced it with unlabelled real images. Here **real covers are not used for training** (licence and integrity concerns, doc 08). The plan compensates with photographic CC0 backgrounds, print-scan simulation and strong colour augmentation; the real-cover evaluation set (§8.8) measures what remains. If the gap is large, the remedy is better synthesis, not scraped data.

### 8.5 Architecture

- **Input**: grey, height 64, width 320 (resize preserving aspect; pad right with the border's median grey; crop if wider after limiting the scale to fit 6–40 characters). Normalise to zero mean, unit variance per crop. Fixed shape `1×1×64×320` for export.
- **Backbone**: MobileNetV1-1.0-style depthwise-separable CNN (3.46M parameters, measured 3.7 MB int8 and 17–27 ms single-thread wasm, §4.2), first stride reduced so that the 64-px height keeps 4 rows at the last stage. Alternative for a size/accuracy trade-off: MobileNetV1-0.5 (0.88M, 1.0 MB int8). ResNet-18 only if both fail the gates (11.4 MB int8).
- **Operators**: restrict to Conv, depthwise Conv, BatchNorm (folded at export), ReLU or ReLU6, average pooling, global average pooling, Gemm, Add, Concat, Reshape, ReduceL2/Div for normalisation. Before training, export an untrained model and check that both `rten` and `tract` load and run it in wasm with these operators (the local experiment used the same operator set).
- **Head**: global average pooling over height, then generalised-mean (GeM) pooling over width; linear 512→128; L2 normalisation.
- **Auxiliary heads** (training only, dropped at export): category (5-way softmax), italic (sigmoid), weight (regression on usWeightClass/1000), width class (regression). Multi-task heads help the embedding encode the traits that survive at spine size (finding 2).

### 8.6 Loss and training schedule

- **Main loss**: sub-centre ArcFace over face classes (k = 3 sub-centres, to absorb alternate glyph forms and case variants), scale s = 30, margin m = 0.25. ArcFace-type losses scale to thousands of classes better than triplet mining [V1 for the library: `pytorch-metric-learning` 3.0.2, MIT; `timm` 1.0.31, Apache-2.0].
- **Family term**: a second ArcFace head over **families** with weight 0.3, so that sibling weights stay closer to each other than to other families (useful when the true face is absent).
- **Auxiliary losses**: 0.1 × (category CE + italic BCE + weight L1 + width L1).
- **Batch sampling**: P × K × M — 64 families × 2 faces × 4 crops = 512 crops per batch, so that every batch contains sibling hard negatives.
- **Optimiser**: AdamW, learning rate 2e-3 (backbone) and 1e-2 (ArcFace weights), weight decay 0.05, 2,000-step linear warm-up then cosine decay; bf16 mixed precision; gradient clipping at 5.
- **Length**: 120,000 steps (about 60M crops, ≈ 25 passes over the masks with fresh augmentation) [E]. Checkpoint every 5,000 steps; select on validation recall@20 at L1.
- **Curriculum**: first 20 % of steps with augmentation strength halved; full strength afterwards.
- **Pilot first**: 300 families, 20,000 steps, to validate the pipeline, the export and the wasm runtime before the full run.

### 8.7 Export and quantisation

1. PyTorch → ONNX opset 17, static shape `1×1×64×320`, BatchNorm folded, auxiliary heads removed.
2. Simplify with `onnxslim` [V1].
3. **Parity test**: on 1,000 validation crops, cosine similarity between PyTorch and ONNX (fp32) embeddings ≥ 0.9999; run the same test in `rten` and `tract` builds for `wasm32-unknown-unknown` (Node, single thread).
4. **int8**: ONNX Runtime static quantisation, QDQ format, per-channel weights, per-tensor activations, calibration on 2,000 augmented validation crops spanning L0–L2. Accept if recall@20 drops ≤ 1 point at L1 and L2; otherwise try (a) excluding the first conv and the final Gemm from quantisation, then (b) quantisation-aware training for 10,000 steps from the best checkpoint.
5. **Gallery index**: for each face, embed 32 renders of a fixed list of 32 strings (mixed case and caps, French and English) with light augmentation (L0–L1), average, L2-normalise, quantise to int8 (scale 127) → 128 bytes per face. Store face id, family, licence, manifest row and model hash. Version the index with the model: an index built by another model is refused.
6. **Deliverables**: `model.fp32.onnx`, `model.int8.onnx`, `gallery-index.bin`, the training manifest (fonts, background images, strings, renderer version, code commit, seeds), a model card (data, licences, metrics, limits, intended use: "nearest open face for spine typesetting, not identification of commercial fonts"), and the evaluation report of §8.8. Weights are reproducible from the manifest (doc 08 implication 28).

### 8.8 Evaluation

1. **Synthetic benchmark**, same conditions as §5.2 and as the `ebc-fontid` evaluation harness (`cargo run -p ebc-fontid --example eval`), so that numbers are comparable:
   - seen families, unseen strings: recall@1/5/10/20 at L0, L1, L2 and the single-factor conditions;
   - held-out families: category agreement of the nearest face; weight within ±100; italic agreement;
   - end-to-end: embedding top-20 → `ebc-fontid::rerank` → top-1/top-5.
2. **Real-cover set**: at least 200 covers the project may use for evaluation — covers supplied by consenting authors and publishers, public-domain editions, or reseller-provided covers under licence; never used for training. Two annotators mark, per cover, the title class (typeset / lettered / logo) and every gallery face they find acceptable at spine size (multi-label). Metric: share of typeset titles with an acceptable face in the top 5; inter-annotator agreement reported.
3. **Ablations** (on the pilot scale): without the family term; without photographic backgrounds; MobileNet-0.5 vs 1.0; input width 256 vs 320; with and without the arc/perspective augmentations.
4. **Runtime**: wasm size, int8 model size, single-thread latency and peak memory in `rten` and `tract` on the reference vCPU and, when available, on two mid-range phones (one Android, one iPhone).
5. **Bias check**: recall per category, per weight, per case (caps vs mixed) and for accented titles.

### 8.9 Compute estimate

| Item | Estimate [E] |
|---|---|
| Rendering masks (2.4M) | 5–6 CPU-hours (parallel, any machine) |
| Pilot (300 families, 20k steps) | 1–2 GPU-hours |
| Main run (120k steps, batch 512, 64×320 input, MobileNet-1.0) | 10–20 GPU-hours on one 24 GB GPU (A10, L4, RTX 4090 class); the arithmetic is about 0.7 GFLOP per crop forward + backward, so the run is bound by data loading and augmentation, not by the GPU |
| Hyper-parameter sweep (6–8 shortened runs) | 20–40 GPU-hours |
| Quantisation-aware fine-tune (if needed) | 1–2 GPU-hours |
| **Total for one model, with iteration** | **about 35–65 GPU-hours** plus ~10 CPU-hours; budget 100 GPU-hours to be safe |

This agrees with the generation note's 10–30 GPU-hours for one run of an identification embedding and its advice to budget two to three times that for iteration [E]. A larger backbone (ResNet-18) roughly doubles the GPU time.

### 8.10 Risks

- **Synthetic-to-real gap** without real training images: measured by §8.8 item 2; remedy is better synthesis.
- **Faces that differ only in weight** dominate errors; the render-and-compare verifier with weight fitting (variable fonts) resolves them better than the embedding.
- **Index growth** beyond Google Fonts needs the same licence check per file; the catalogue rule of doc 08 §5.5 (no close copies of recent commercial faces, no faces of living or recently deceased designers' clones) applies to the gallery, not to training.
- **Legal status of weights trained on OFL fonts** is unsettled (doc 08 question 15); the model never outputs glyphs or font files, only an embedding (doc 08 R13).

---

## 9. Open questions

1. Real-cover accuracy of every method: no real cover was processed. Build the evaluation set first (§8.8).
2. Phone performance of `rten`/`tract` in wasm; WebGPU availability in Safari, Firefox and Android WebViews.
3. Whether WhatFontIs still offers an API, its price and terms (only relevant as a manual QA aid).
4. Architecture, size and licence of Storia-AI/font-classify and of other public ONNX font classifiers.
5. `ocrs` language coverage (French accents) and model sizes; size of the PP-OCRv6 tiny detector alone.
6. OFL Reserved Font Name consequences of subsetting gallery fonts server-side (doc 08 §5.1 says subsets inside a PDF are free; shipped subset files are Modified Versions).
7. Prior published work on known-string render-and-compare on book covers.
8. Licence of the public-domain title list and word lists chosen for training strings.

---

## 10. Sources

Registry and packaged-README sources, primary repositories and local measurements are listed in full in `notes/font-id-tech.tools.md` (Sources) and `notes/font-id-tech.generation.md` (Sources). The most load-bearing:

- [google-font-metadata on npm](https://registry.npmjs.org/google-font-metadata) — family counts and licences.
- [onnxruntime-web](https://registry.npmjs.org/onnxruntime-web), [rten](https://crates.io/api/v1/crates/rten), [tract-onnx](https://crates.io/api/v1/crates/tract-onnx), [burn-onnx](https://crates.io/api/v1/crates/burn-onnx) — runtimes.
- [@paddleocr/paddleocr-js](https://registry.npmjs.org/@paddleocr/paddleocr-js), [ppu-paddle-ocr](https://registry.npmjs.org/ppu-paddle-ocr), [ocrs](https://crates.io/api/v1/crates/ocrs), [tesseract.js](https://registry.npmjs.org/tesseract.js) — text localisation.
- [timm](https://pypi.org/pypi/timm/json), [pytorch-metric-learning](https://pypi.org/pypi/pytorch-metric-learning/json), [onnxslim](https://pypi.org/pypi/onnxslim/json), [onnxruntime](https://pypi.org/pypi/onnxruntime/json), [fonttools](https://pypi.org/pypi/fonttools/json), [trdg](https://pypi.org/pypi/trdg/json) — training tools.
- [Google Fonts repository](https://github.com/google/fonts) — font source.
- [FontDiffuser](https://github.com/yeungchenwa/FontDiffuser), [fewshot-font-generation](https://github.com/clovaai/fewshot-font-generation), [MI-GAN](https://github.com/Picsart-AI-Research/MI-GAN), [LaMa](https://github.com/advimman/lama), [vtracer](https://github.com/visioncortex/vtracer) — generation side.
- [MDN WebGPU compat data](https://raw.githubusercontent.com/mdn/browser-compat-data/main/api/GPU.json).
- Papers cited from memory without a link: DeepFont (2015), Chen et al. (2014), FontCLIP (2024), HENet (2021), DBNet (2020), CRAFT (2019), EAST (2017), SWT (2010), Zramdini and Ingold (1998), Zhu et al. (2001), Khoubyari and Hull (1996), Solli and Lenz (2007), Shinahara et al. (2019), Iwana et al. (2016).

---

## Implications for the implementation

1. The identification answer is always "the nearest **open** face of our index", never a commercial font name; the UI shows the open face's own name only (doc 08 R7).
2. The index admits only fonts whose licence is recorded and allowed; each entry carries licence, family, version and SHA-256 from the font manifest.
3. Inference runs in the browser, single-threaded, with no cross-origin-isolation requirement; no cover image is sent to a third party.
4. The title box can come from the user; automatic detection is a proposal the user confirms.
5. Matching uses the known title string; the UI shows five rendered spine previews and lets the user pick, with a neutral serif and sans always available.
6. A confidence rule decides between the nearest face and a house face; titles classified as lettered or logo always get a house face (doc 08 item 6).
7. Gallery fonts are not shipped eagerly: subset to the title's characters or lazy-load in tiers.
8. Render-and-compare fits letter-spacing per candidate before ranking; it is kept as the verifier when an embedding model is added.
9. A learned model, if added, is ≤ 4 MB int8 ONNX, runs in a pure-Rust runtime behind a trait (rten by default), and its gallery index is versioned with the model hash.
10. Training uses open fonts, procedural and CC0 backgrounds only, with a published manifest; no cover image, no third-party checkpoint (§8, doc 08 R12).
11. Evaluation numbers are produced by the same harness for the classical and learned pipelines (L0/L1/L2), and a real-cover set with permitted covers is built before any learned model ships.
12. No generative re-lettering in the default product; colours, not pixels, are taken from the cover.

---

## What the code implements today

Read from `crates/ebc-fontid` (crate documentation and constants) on 2026-11-12. Accuracy figures for this code are in `fontid-eval.md` (maintained separately), not here.

- **Pipeline (classical, no learned weights, deterministic, `wasm32`-ready, `#![forbid(unsafe_code)]`)**:
  1. `find_text_regions` / `find_title_regions`: local-contrast binarisation in both polarities, connected components, letter filtering, grouping into lines and blocks; with the known title, blocks whose letter count fits win.
  2. Segmentation inside a region (accents and dots merged with their base); a dynamic programme assigns the known title's characters to the glyphs, tolerating touching or broken glyphs, and infers the case used on the cover (as given, UPPER, lower, Title).
  3. `identify`: each labelled glyph becomes a descriptor — a 4×4-cell, 8-bin gradient-orientation histogram on a 40-px box plus five scalars (aspect, density, stroke width, stroke contrast, slant), quantised to bytes — compared with the same character in every indexed font. The 75 % best-matching glyphs make up a font's distance.
  4. `rerank`: the known text is typeset with up to 64 candidates, fitted to the region (size and letter-spacing from −0.06 to +0.50 em), compared at 48 px height with a truncated chamfer distance, and fused with the descriptor score (render weight 0.5).
- **Index**: per font, the descriptors of `REFERENCE_CHARS` (A–Z, a–z, 0–9, `&`, `!`, `?`; accented letters matched through their base letter) rendered at 96 px per em, in a versioned binary format (version 1). `FontIndex::build_checked` refuses fonts whose licence is not in `OPEN_LICENCES` = {OFL-1.1, Apache-2.0}.
- **Bundled index** (feature `bundled-index`): the eight OFL fonts of `assets/fonts`, about 70 KB, regenerated by `cargo run -p ebc-fontid --example build-index`.
- **Confidence**: `Identification::confident` requires at least 4 labelled glyphs, a best score ≥ 0.8 and a lead ≥ 0.04 over the best font of another family.
- **Evaluation harness** (feature `synth`): procedural covers with seeded degradations; each condition is run with the true box, the detected box, and without the true font in the index (hold-out).

**Where the implemented default differs from the research:**

| Point | Research (this document) | Code today | Note |
|---|---|---|---|
| Gallery size | 300–600 curated open faces (Stage 1), then the whole open corpus (Stage 2) | 8 bundled fonts; any index can be built with `build-index` | The gallery is the main gap: with 8 fonts the answer is a house-face choice, not identification |
| Matching order | Line-level render-and-compare first; glyph-level scoring was worse alone (53 % vs 76 % top-1 at L1) and refit fusion hurt with small tracking | Glyph descriptors first, then line-level render-and-compare on up to 64 candidates, fused 50/50 | The code's labelling by dynamic programme is the improvement §5.3 proposed for naive segmentation; whether the fusion helps must be read in `fontid-eval.md` |
| Title box | User-confirmed box by default | Automatic detection with the known title; a caller can pass its own `TextRegion` | Both are possible; the UI should still offer confirmation |
| Allowed licences | Training and bundle: OFL-1.1, Apache-2.0, UFL-1.0 after per-file review (doc 08 R1, item 27) | Index: OFL-1.1 and Apache-2.0 only | Stricter than the research; UFL needs a manual review anyway |
| Letter-spacing fit | Fit per candidate before the first ranking; benchmark range up to +0.20 em, augmentation up to +0.30 em | Fit in the rerank stage only, −0.06 to +0.50 em | The descriptor stage is spacing-independent by construction |
| Learned model | Stage 2 embedding (§8) | None | As planned: Stage 1 first |
| OCR / text detector | Optional (PP-OCR, `ocrs`) | Classical detection only | As planned |
| Confidence threshold | 0.80 (doc 08 item 6), plus "five previews" | 0.8, plus glyph count and family margin | Consistent; the extra conditions are the code's own |
