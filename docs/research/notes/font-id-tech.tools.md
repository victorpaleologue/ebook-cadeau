# Identifying cover fonts in the browser - existing tools, models, runtimes, and a non-ML baseline

Research notes, angle "existing tools and models". Date of research: 2026-11-12. Scope: font identification from a cover image,
text localisation, in-browser inference runtimes, and a render-and-compare baseline. Legal questions (font/cover rights) are out
of scope here except for licences of tools, models and datasets.

Confidence markers: **[V2]** verified in >= 2 independent sources this session, **[V1]** single first-party source this session
(package registry metadata or the README shipped inside the published package), **[X]** measured by me in this session
(method in sections 6 and 7; scripts were in an ephemeral scratchpad, not in the repo), **[M]** from memory, NOT verified this
session, **[est.]** my own extrapolation.

## 0. Method and limits (read first)

- **Zero web searches were executed.** The first batch of four `WebSearch` calls was refused: the session's shared web-search
  budget (200 calls, shared by all agents of this run) was already exhausted. WebFetch is blocked by the network allowlist and
  the GitHub connector is restricted to this repository. I did not work around the limit.
- Consequence: **everything about commercial services (features, APIs, prices, terms) and about research papers / GitHub repos is
  [M]** and must be re-checked before it carries a decision. No paper, vendor page or GitHub URL is linked, because none appeared
  in a search result; papers are given as bibliographic references only.
- What I could verify first-hand, because package registries are reachable from the sandbox: versions, licences and shipped
  binary sizes from **npm, crates.io and PyPI** (48 registry URLs, all returned HTTP 200 and are listed in Sources), plus the
  READMEs packaged inside those releases. A README is a first-party claim, not an independent test.
- To replace the missing literature numbers I ran **two small experiments** [X]: (a) wasm binary size and single-thread speed of
  three inference runtimes on three embedding-CNN architectures; (b) a render-and-compare font-matching benchmark on 529 Google
  Fonts faces. Both are synthetic and are upper bounds, not field results.

## 1. Bottom line

1. **No commercial identifier is usable as a pipeline component.** To my knowledge none of WhatTheFont, Adobe, Matcherator or
   Identifont offers a public identification API; WhatFontIs is the only one I remember advertising one [M]. All of them match
   mainly against commercial catalogues (the answer is a font we may not embed) and all require uploading the cover to a third
   party, which contradicts the "runs in the browser" requirement. Use them as a manual QA oracle only.
2. **The problem to solve is not "identify the font" but "find the nearest font we are allowed to typeset with".** Cover fonts
   are overwhelmingly commercial or hand-lettered [M]. The target gallery is therefore open fonts: the Google Fonts corpus is
   1,908 families (1,892 with a Latin subset), 98 % under SIL OFL 1.1 [V1] (section 4).
3. **Rust-only in-browser inference is practical today.** `rten` 0.28 compiles to a 3.3 MB wasm (1.0 MB gzip) with no JS glue
   and ran fp32 and int8 ONNX models; `tract-onnx` 0.23 also works but is 9.7-17.8 MB and needs a `getrandom` workaround;
   ONNX Runtime Web 1.31 is 14.85 MB (3.8 MB gzip) [X].
4. **The 2-10 MB model budget is realistic.** A MobileNetV1-1.0-class embedding net is 13.8 MB fp32 / 3.7 MB int8 and runs in
   17-27 ms single-threaded wasm on a 2.3 GHz Xeon vCPU; ResNet-18 is 45.2 / 11.4 MB and 28-77 ms [X]. Phone timings are
   extrapolated, not measured (section 6).
5. **Stay single-threaded.** Threaded wasm needs cross-origin isolation (COOP/COEP headers) [V2], which is hostile to a widget
   embedded in a reseller's checkout page. The measured single-thread latencies make threads unnecessary for this workload.
6. **The non-ML baseline works on easy covers and collapses on hard ones** [X]: with the true face in a 529-face gallery,
   top-1 / top-5 = 97.5 % / 100 % on clean renders, 76 % / 94 % on "typical web cover" degradations, 13 % / 20 % on hard ones
   (small text, textured background, wide tracking). Binarisation on textured backgrounds and unmodelled letter-spacing are the
   two dominant failure causes; fitting tracking by re-rendering recovers part of the latter (42 % -> 60 % top-1).
7. **Top-5 with a human choice beats any top-1 automation at this stage.** The product is interactive: show five rendered spine
   previews and let the user pick.
8. **Recommended staging:** (now) user-confirmed title box + render-and-compare over a curated gallery of a few hundred open
   faces; (later) a 1-4 MB int8 embedding CNN for coarse retrieval over the whole open corpus, with render-and-compare kept as
   the verification and parameter-fitting step. Generative re-lettering (diffusion) is not feasible on mid-range phones [M].

## 2. Commercial identifiers (all [M] - not re-verified, see section 0)

| Tool | Owner | How it works (as I remember it) | Catalogue matched | Public API | Fit for us |
|---|---|---|---|---|---|
| WhatTheFont (web + mobile app) | MyFonts / Monotype | Deep-learning matcher since the 2017 relaunch; detects several text blocks in a photo | MyFonts commercial catalogue (>130k styles claimed in 2017) | None that I know of | Manual oracle only |
| Adobe Fonts visual search, Adobe Capture "Type", Photoshop "Match Font", Illustrator "Retype" | Adobe | DeepFont lineage (section 3); Match Font shipped in Photoshop CC 2015.5 | Adobe Fonts + locally installed fonts | None for identification | Manual oracle only; needs Creative Cloud |
| Fontspring Matcherator | Fontspring | Upload, glyph boxes, claims OpenType-feature awareness | Fontspring catalogue | None | Manual oracle only |
| Font Squirrel "Font Identifier" | Font Squirrel (sister company of Fontspring) | Same Matcherator engine | Font Squirrel free fonts + Fontspring | None | Interesting because results include free fonts, but Font Squirrel licences vary per font |
| WhatFontIs | WhatFontIs | User confirms per-glyph segmentation, then glyph matching; filter "free only" / "Google Fonts only" | Claims ~1M commercial + free fonts | **Yes, I recall a keyed HTTP API**; quota, price and terms not recalled reliably | Only candidate for a server-side fallback; uploads the cover |
| Identifont | Identifont | Questionnaire about glyph features, no image input | Own database | None | Not applicable |
| "Like That Garden" | - | As far as I know this is a plant-identification app, not a font tool. Possibly a confusion with LikeFont (image-based identifier with a large CJK index) | - | - | Not applicable |
| Browser extensions (WhatFont, Fonts Ninja) | various | Read CSS of live web text | - | - | Not applicable to raster covers |

Structural reasons the whole class is out, independent of the unverified details: (i) the answer is usually a commercial
typeface, which solves nothing for re-typesetting a spine; (ii) the cover image leaves the browser; (iii) terms of consumer web
tools generally forbid automated use [M].

## 3. Open-source and research models (all [M] unless marked)

| Work | What it is | Classes / data | Size and browser fit | Licence (as remembered) |
|---|---|---|---|---|
| Chen et al., "Large-Scale Visual Font Recognition", CVPR 2014 | Local feature embedding + nearest class mean | VFR-447 / VFR-2420 | Pre-CNN, superseded | Paper only |
| **DeepFont** - Wang, Yang, Jin, Shechtman, Agarwala, Brandt, Huang, ACM MM 2015 (arXiv 1507.03196) | CNN with a stacked convolutional auto-encoder trained on unlabelled real + synthetic images to close the synthetic-to-real gap; 105x105 grey patches; low-rank compression | **AdobeVFR**: 2,383 font classes, ~1,000 synthetic images per class, 4,384 labelled real test images, ~200k unlabelled real images. Top-5 above 80 % on the real test set | Original net is large; idea (domain adaptation) matters more than weights | Dataset for research; classes are commercial Adobe fonts, so labels are useless for an open gallery |
| gaborcselle/font-identifier (2023) | ResNet-18 fine-tuned on synthetic snippets | 48 common fonts | ~45 MB fp32 (ResNet-18 measured at 45.2 MB [X]); too few classes | Not recalled |
| Storia-AI/font-classify | Classifier over a few thousand fonts, ONNX export published | Mostly Google Fonts, exact count not recalled | Closest existing asset to our need; must be re-checked (architecture, size, licence) | Not recalled |
| HENet (Chen et al., 2021) | "HE block" plug-in that suppresses the most salient features; ResNet-18 backbone | AdobeVFR, Explor_all | ResNet-18 class, fine for the browser | Research code |
| Swin-Transformer font recognisers | Several papers, mostly Chinese/Arabic | - | Swin-T ~28M params, ~110 MB fp32: over budget | I know of no canonical open Latin model |
| YuzuMarker.FontDetection (2023) | CJK font recognition (ResNet-18..101), also regresses size, stroke width, colours, direction | ~6k CJK fonts rendered over illustration backgrounds | Multi-task head is a good design idea; classes irrelevant for Latin | Not recalled |
| **FontCLIP** - Tatsukawa et al., Eurographics 2024 | CLIP fine-tuned on font-attribute prompts; attribute-driven and cross-lingual retrieval | ~200 Latin fonts with 37 attributes (O'Donovan et al. 2014) | CLIP ViT-B vision tower ~88M params (~350 MB fp32): **not** phone-browser material; semantic similarity, not identification | Research code |
| Font embeddings (Bernhardsson "deep-fonts" 2016; IDEO Font Map 2017; fontjoy) | Generative or CNN-feature embeddings of glyph bitmaps for similarity / pairing | 50k fonts (Bernhardsson), ~750 Google Fonts (IDEO) | Tiny models; trained on clean glyph renders, not on photos of covers | Mixed |
| TextStyleBrush (Krishnan et al., 2021) | One-shot text style transfer | Code and weights not released, only the Imgur5K dataset | Not available | - |

Cover-specific prior art worth reading when search is available again: Shinahara et al., "Serif or Sans: Visual Font Analytics on
Book Covers and Online Advertisements" (ICDAR 2019), and Iwana et al., "Judging a Book by its Cover" (2016, ~57k Amazon covers) [M].

Reading of the table: **there is no off-the-shelf open model that maps a cover crop to "nearest open Latin font"**. The reusable
parts are the recipe (synthetic renders + domain adaptation, DeepFont), multi-task heads (YuzuMarker) and possibly Storia's weights.

## 4. Font corpora and datasets

Measured from the `google-font-metadata` 6.0.8 npm package (MIT; data snapshot with last font modification 2026-01-07)
([registry](https://registry.npmjs.org/google-font-metadata)) [V1]:

| Fact | Value |
|---|---|
| Families in the Google Fonts API response | **1,908** (the task brief said ~1,800) |
| By category | sans-serif 705, display 464, handwriting 348, serif 343, monospace 48 |
| Families with a `latin` subset | 1,892 |
| Variable families | 525 |
| Licences (1,869 families with a licence record) | SIL OFL 1.1: 1,828; Apache-2.0: 36; Ubuntu Font Licence 1.0: 5 |

- Fontsource republishes each family as an npm package under its own licence (`@fontsource/playfair-display` 5.3.0 is tagged
  `OFL-1.1`) ([registry](https://registry.npmjs.org/@fontsource/playfair-display)) [V1]. I pulled 315 families this way.
- **Size of a Latin subset** [X]: the 315 `latin-400-normal` WOFF2 files total 8.1 MB, i.e. **~25 KB per face**. A 500-face
  gallery is ~12.5 MB if shipped whole; this drives the architecture in section 9.
- Synthetic-text tooling: `trdg` 1.8.0 (MIT, last release 2022) ([PyPI](https://pypi.org/pypi/trdg/json)), `fonttools` 4.68.0
  (MIT) ([PyPI](https://pypi.org/pypi/fonttools/json)) [V1].
- Other libre sources to consider for the gallery: Velvetyne, Open Foundry, The League of Moveable Type, Font Library [M].
  OFL "Reserved Font Name" rules may bite when we subset files ourselves; hand that to the legal track [M].
- AdobeVFR is the only public real-image font benchmark I know of, and its labels are commercial fonts [M]. **A real-cover
  evaluation set labelled with "nearest open font" does not exist to my knowledge; we would have to build one** (a few hundred
  covers is enough to rank methods).

## 5. Locating the title on the cover

We know the title string. That changes the problem: a detector only has to propose boxes, and a recogniser (or even a
render-and-compare score) confirms which box is the title by edit distance to the known string.

| Option | Facts | Fit |
|---|---|---|
| **User draws / confirms the box** | No model. | Default for v1: the tool is interactive anyway |
| PaddleOCR in the browser | Official SDK `@paddleocr/paddleocr-js` 0.4.2, Apache-2.0, runs PP-OCRv5 and PP-OCRv6 pipelines on ONNX Runtime Web with a worker mode ([registry](https://registry.npmjs.org/@paddleocr/paddleocr-js)); independent SDK `ppu-paddle-ocr` 6.7.0, MIT ([registry](https://registry.npmjs.org/ppu-paddle-ocr)). Both READMEs document PP-OCRv5 and v6 [V2]. `ppu` README: v6 "tiny" detection+recognition download ~6 MB, "small" ~30 MB, "medium" ~139 MB, v5 mobile ~12 MB; WebGPU "typically 2-5x faster", wasm fallback [V1]. `paddleocr` 3.7.0 on PyPI is Apache-2.0 ([PyPI](https://pypi.org/pypi/paddleocr/json)) [V1]; `rapidocr` 3.10.0 (Apache-2.0, [PyPI](https://pypi.org/pypi/rapidocr/json)) [V1] redistributes ONNX conversions of the PP-OCR models [M] | Best automatic option; costs ORT Web (14.85 MB wasm) plus OpenCV.js, which the official SDK bundles |
| `@gutenye/ocr-browser` 1.4.9 | MIT wrapper for PP-OCRv4 on ONNX Runtime ([registry](https://registry.npmjs.org/@gutenye/ocr-browser)) [V1] | Older model generation |
| **`ocrs` 0.14 + `rten` 0.28** | Pure-Rust OCR engine and runtime, MIT OR Apache-2.0, updated 2026-11-08 ([ocrs](https://crates.io/api/v1/crates/ocrs), [rten](https://crates.io/api/v1/crates/rten)) [V1]. Models trained on open data, English-centric [M] | Best match for the Rust/wasm stack; accent coverage must be tested |
| Tesseract.js 7.0.0 | Apache-2.0; `tesseract.js-core` 6.1.2 wasm binaries are 2.9-3.5 MB each; French data package 7.0 MB unpacked ([tesseract.js](https://registry.npmjs.org/tesseract.js), [core](https://registry.npmjs.org/tesseract.js-core), [fra data](https://registry.npmjs.org/@tesseract.js-data/fra)) [V1]. Returns word and symbol boxes [V1]; weak on display lettering [M] | Symbol boxes are attractive for glyph segmentation; recognition quality on covers is the risk |
| DBNet (Liao et al., AAAI 2020) | Basis of the PP-OCR detectors; ResNet-18 variant ~12M params [M] | Use through PP-OCR |
| CRAFT (Baek et al., CVPR 2019) | Character-region heat maps; VGG16-BN, ~20M params, ~80 MB weights; code MIT [M]. PyPI wrapper `craft-text-detector` 0.4.3 is MIT, last release 2022 ([PyPI](https://pypi.org/pypi/craft-text-detector/json)) [V1]; used by EasyOCR 1.7.2, Apache-2.0 ([PyPI](https://pypi.org/pypi/easyocr/json)) [V1] | Character-level output is ideal, size is not |
| EAST (Zhou et al., CVPR 2017) | Common ~95 MB frozen graph; a popular implementation is GPL-3.0 [M] | Avoid |
| docTR / OnnxTR | `python-doctr` 1.1.0 and `onnxtr` 0.9.0, Apache ([doctr](https://pypi.org/pypi/python-doctr/json), [onnxtr](https://pypi.org/pypi/onnxtr/json)) [V1]; MobileNetV3 DB detector ~4M params [M] | Alternative ONNX source |
| Classical: MSER, stroke width transform (Epshtein et al., CVPR 2010) | No model; brittle on photographic covers [M]. Building blocks exist in pure Rust: `imageproc` 0.27 (MIT) ships Otsu and adaptive thresholding, connected components, contours, distance transform, template matching and HOG ([registry](https://crates.io/api/v1/crates/imageproc)) [V1, module list read from the crate source] | Enough for flat-colour covers; pair with the user-confirmed box |

OpenCV.js is heavy: `@techstark/opencv-js` 5.0.0 (Apache-2.0) ships a single ~13 MB `opencv.js`
([registry](https://registry.npmjs.org/@techstark/opencv-js)) [V1]. Prefer `imageproc`.

## 6. In-browser inference runtimes

### 6.1 Registry facts (as of 2026-11-12) [V1]

| Runtime | Version, licence | What the package says | Shipped size |
|---|---|---|---|
| ONNX Runtime Web | 1.31.1, MIT ([registry](https://registry.npmjs.org/onnxruntime-web)) | Backends: WebAssembly (all browsers), WebGPU ("still an experimental feature"), WebGL (maintenance mode), WebNN (behind a flag). README matrix lists WebGPU for Chromium only, v113+ desktop and v121+ Android, and not for Safari or Firefox - **the matrix may lag**: Safari 26 and Firefox 141 on Windows shipped WebGPU in 2025 [M] | `ort-wasm-simd-threaded.wasm` 14.85 MB (3.80 MB gzip -9); WebGPU/WebNN "jsep" build 29.5 MB (6.88 MB gzip); JS loader 52 KB [X] |
| Transformers.js | `@huggingface/transformers` 4.3.2, Apache-2.0 ([registry](https://registry.npmjs.org/@huggingface/transformers)) | Wrapper over ONNX Runtime; CPU/wasm by default, `device: 'webgpu'` opt-in; `dtype` fp32 / fp16 / q8 (default on wasm) / q4; models and wasm fetched from a CDN by default (self-hosting is configurable) | 2.3 MB tarball + ORT Web |
| tract | `tract-onnx` 0.23.10, MIT OR Apache-2.0, updated 2026-11-05 ([registry](https://crates.io/api/v1/crates/tract-onnx)) | Pure-Rust ONNX/NNEF/TFLite inference | see 6.2 |
| rten | 0.28.0, MIT OR Apache-2.0 ([registry](https://crates.io/api/v1/crates/rten)) | Pure Rust, CPU only, WebAssembly an explicit goal; SIMD via AVX2/AVX-512/NEON/wasm SIMD; reads ONNX directly; f32 and int8, f16 up-converted; wasm SIMD needs Chrome 91 / Firefox 89 / Safari 16.4 | see 6.2 |
| Burn | 0.22.1, MIT OR Apache-2.0 ([burn](https://crates.io/api/v1/crates/burn)) | `burn-onnx` 0.22 turns an ONNX file into Rust source plus a weights file ("no ONNX runtime to ship") ([registry](https://crates.io/api/v1/crates/burn-onnx)); `burn-flex` is the CPU backend for std, no_std and wasm ([registry](https://crates.io/api/v1/crates/burn-flex)); `burn-wgpu` targets WebGPU ([registry](https://crates.io/api/v1/crates/burn-wgpu)). **`burn-ndarray` and `burn-import` are now marked deprecated/legacy on crates.io** ([ndarray](https://crates.io/api/v1/crates/burn-ndarray), [import](https://crates.io/api/v1/crates/burn-import)) - the brief's "ndarray backend" is outdated | not measured |
| candle | `candle-core` 0.11.1, MIT OR Apache-2.0 ([registry](https://crates.io/api/v1/crates/candle-core)) | wasm examples exist in the repo; CPU only in the browser; ONNX support is partial [M] | not measured |
| ort (Rust) | 2.0.0-rc.15 wrapping ONNX Runtime 1.31; alternative backends `ort-web` 0.5.0 and `ort-tract` 0.5.0 ([ort](https://crates.io/api/v1/crates/ort), [ort-web](https://crates.io/api/v1/crates/ort-web), [ort-tract](https://crates.io/api/v1/crates/ort-tract)) | One Rust API over native ORT, ORT Web or tract | not measured |
| wonnx | 0.5.1, last updated 2023-09-30 ([registry](https://crates.io/api/v1/crates/wonnx)) | WebGPU ONNX runtime in Rust | Looks unmaintained; avoid |
| TensorFlow.js / LiteRT.js | `@tensorflow/tfjs` 4.22.0, last published 2025-01; `@litertjs/core` 2.6.0, 2026-10; both Apache-2.0 ([tfjs](https://registry.npmjs.org/@tensorflow/tfjs), [litert](https://registry.npmjs.org/@litertjs/core)) | - | No reason to add a second model format |

Threaded wasm requires a cross-origin-isolated page (`Cross-Origin-Opener-Policy: same-origin` +
`Cross-Origin-Embedder-Policy: require-corp`); stated by both the official PaddleOCR.js README and the `ppu-paddle-ocr` README
[V2]. Without isolation ORT pins `numThreads` to 1 [V1].

### 6.2 Measured: binary size and single-thread wasm speed [X]

Setup: three randomly initialised embedding CNNs (BatchNorm folded, opset 17), input 1x1x64x320 (a grey text-line crop), output
128-d or 256-d. int8 = ONNX Runtime static QDQ quantisation, per-channel. Runtimes built for `wasm32-unknown-unknown` with
`+simd128`, LTO, no JS glue; run in Node 22.23, **one thread**, Intel Xeon 2.3 GHz vCPU; best of 30 runs, two repetitions.

| Runtime (wasm binary raw / gzip -9) | MobileNetV1-0.5, 0.88M params | MobileNetV1-1.0, 3.46M params | ResNet-18, 11.3M params |
|---|---|---|---|
| Model file fp32 / int8 | 3.5 MB / 1.0 MB | 13.8 MB / 3.7 MB | 45.2 MB / 11.4 MB |
| ONNX Runtime Web 1.31.1 (14.85 / 3.80 MB) | 5.4 ms / 8.3 ms | 17.4 / 22.5 ms | 49-52 / **28-29 ms** |
| tract-onnx 0.23.10, opt-level 3 (17.8 / 4.40 MB) | **4.9-5.1** / 5.7-6.2 ms | **16.9-17.7** / 19-20 ms | 59-63 / 64-67 ms |
| tract-onnx 0.23.10, opt-level s (9.7 / 2.54 MB) | 6.6 / 11.3 ms | 24.2 / 37.7 ms | 97 / 107 ms |
| rten 0.28.0, opt-level 3 (**3.28 / 1.01 MB**) | 9.2 / 7.9-9.0 ms | 25.5 / 21-23 ms | 57-76 / 53-75 ms |
| Native ONNX Runtime 1.29, 1 thread (reference) | 0.85 / 0.65 ms | 3.1 / 1.5 ms | 9.0 / 3.5 ms |

Findings:
- All three runtimes loaded both fp32 and QDQ-int8 files. **int8 buys a 4x smaller download but little or no wasm speed-up**
  except for ResNet-18 on ORT Web. Choose int8 for bandwidth, not for latency.
- `tract-onnx` 0.23.10 does **not build for `wasm32-unknown-unknown` out of the box**: its `tract-onnx-opl` dependency pulls
  `getrandom` 0.4, which refuses that target unless the `wasm_js` feature or a custom backend is configured
  ([getrandom](https://crates.io/api/v1/crates/getrandom)). I used `--cfg getrandom_backend="custom"` with a stub.
- Peak wasm linear memory with tract and rten (model bytes included; not measured for ORT Web): 9-13 MB (MobileNet-0.5),
  26-47 MB (MobileNet-1.0), 72-141 MB (ResNet-18); the low end of each range is the int8 file.
- **Phones were not measured.** If a mid-range phone core runs wasm 2-4x slower than this vCPU [est.], expect 10-40 ms for the
  small net, 35-110 ms for MobileNet-1.0 and 60-300 ms for ResNet-18. Even a 10x miss stays well below one second for a
  one-shot operation. WebGPU is unnecessary for the identification net; it only matters if a text detector is added.

## 7. Non-ML baseline: render the known title in each candidate font and compare

### 7.1 Prior art (all [M])

- Classic optical font recognition worked on scanned print: Zramdini and Ingold (IEEE TPAMI 1998, typographical features,
  ~97 % over 280 font models); Zhu, Tan and Wang (TPAMI 2001, Gabor texture). Khoubyari and Hull (CVIU 1996) matched word
  images against **function words rendered in candidate fonts** - the same idea as ours with a known string.
- Glyph-template search engines: Solli and Lenz, "FyFont" (SCIA 2007, eigen-images over ~2,700 fonts); the pre-2017 WhatTheFont
  and today's WhatFontIs, where the user labels segmented glyphs.
- Building blocks: chamfer matching (Borgefors 1988), shape contexts (Belongie et al. 2002), HOG (Dalal and Triggs 2005).
- I do not know of a published accuracy figure for known-string render-and-compare against an open-font gallery on real covers.

### 7.2 Experiment [X]

- **Gallery:** 529 faces from 315 Google Fonts families (315 regular, 137 bold, 77 italic; stratified: 90 sans, 85 serif,
  75 display, 50 handwriting, 15 monospace families, including about 50 hand-picked cover-typical families such as Playfair
  Display, Cormorant Garamond, Bebas Neue, Cinzel; the rest drawn at random). Fonts fetched as Fontsource Latin WOFF files.
- **Queries:** 24 French titles (20 mixed case, 4 all caps, with accents) x 10 random faces per condition = 240 per condition
  (binomial 95 % interval about +/-6 points). The true face is always in the gallery.
- **Degradations:** L0 = clean, 60-110 px, flat colours. L1 "typical web cover" = 28-60 px, gradient background, tracking
  -0.02..+0.06 em, rotation +/-1 deg, blur sigma 0.3-0.8, noise, JPEG q70-90, loose crop. L2 "hard" = 18-36 px, textured
  background with lower contrast, tracking 0..0.20 em, rotation +/-3 deg, blur 0.5-1.2, JPEG q40-70, shadow or outline on half.
- **Matcher:** per-channel Otsu ink extraction with polarity from the border, median filter, projection-profile deskew, tight
  crop, normalise to 64x384. "Line" score = z-scored sum of normalised cross-correlation, HOG cosine (8 px cells, 9 bins) and
  symmetric truncated chamfer distance on edges. "Refit" = for the best 40-80 candidates, solve the letter-spacing that
  reproduces the query's aspect ratio, re-render, re-score, and fuse with a glyph-level score.

| Condition | Line top-1 | Line top-5 | Line top-10 | Refit top-1 | Refit top-5 | Nearest face has same category when the true family is removed (chance ~26 %) |
|---|---|---|---|---|---|---|
| L0 clean | **97.5 %** | 100 % | 100 % | 88.3 % | 94.6 % | 80 % |
| L1 typical | **75.8 %** | **94.2 %** | 95.0 % | 69.2 % | 83.8 % | 74 % |
| L1 + tracking 0..0.20 em | 42.1 % | 64.6 % | 74.6 % | **60.0 %** | **71.7 %** | 63 % |
| L1 + small text (18-36 px) | 62.1 % | 82.5 % | 85.4 % | 54.2 % | 74.6 % | 74 % |
| L1 + textured, low-contrast background | 46.7 % | 61.7 % | 65.8 % | 37.9 % | 54.2 % | 60 % |
| L1 + shadow/outline on every query | 77.1 % | 93.3 % | 95.0 % | 68.8 % | 82.1 % | 70 % |
| L1 + rotation +/-3 deg | 78.7 % | 94.6 % | 95.8 % | 69.2 % | 81.2 % | 71 % |
| L1 + JPEG q40-70 and stronger blur | 73.3 % | 91.2 % | 95.4 % | 63.7 % | 78.7 % | 74 % |
| L2 hard (all of the above) | 12.9 % | 19.6 % | 25.0 % | 14.6 % | 22.5 % | 30 % |

Other measurements: aspect ratio alone gives 0-8 % top-1, so the scores are not a width shortcut. Naive column-gap glyph
segmentation agreed with the true face's own segmentation in only 84 % (L0), 60 % (L1) and 26 % (L2) of queries, which is why
glyph-level scoring alone was worse than line-level (53 % vs 76 % top-1 at L1) and why the refit fusion hurts when tracking is
small. Glyph thumbnails of 12x16 px scored the same as 40x64 px (52.9 % both at L1). Cost in unoptimised Python/numpy, one
thread: ~8 ms per face to render and featurise a title (4.3 s for 529 faces), ~60 ms to score a query against all 529 faces,
~370 ms for the tracking refit.

### 7.3 What the experiment does and does not show

- It is an **upper bound**: same rasteriser for query and candidates, true font always present, single-line titles, synthetic
  backgrounds. Real covers add multi-line lockups, mixed fonts, arcs, textures inside letters and, above all, fonts that are not
  in any open gallery. The hold-out column is the relevant proxy for that case: the nearest open face lands in the right broad
  category three times out of four on typical input and no better than chance on hard input.
- Ranking of failure causes: **(1) ink extraction on textured backgrounds, (2) letter-spacing, (3) small text.** Rotation up to
  3 degrees, JPEG, and simple shadows/outlines cost little.
- Untested improvements that follow directly: colour clustering in a perceptual space instead of single-channel Otsu;
  connected-component segmentation with dynamic-programming alignment to the known characters; fitting tracking for every
  candidate before the first ranking rather than after; fitting the weight axis of variable fonts (525 families); upscaling
  small covers before matching.
- Top-5 is 18 points above top-1 at L1. A five-option picker is the cheapest accuracy gain available.

## 8. Regenerating lettering with generative models (all [M])

Scene-text editing and few-shot font generation models (SRNet 2019, MOSTEL 2023, TextDiffuser 1/2, AnyText, TextCtrl,
FontDiffuser, MX-Font, DeepVecFont-v2) are either GAN-era research code producing ~64-128 px glyph strips or diffusion models
with roughly 1B parameters and multi-GB downloads. Browser demos of distilled Stable Diffusion exist on desktop WebGPU, not on
mid-range phones. None produces print-resolution vector text, and each can misspell the title. Recommendation: do not generate
glyphs; re-typeset the known string with the nearest open font, and take only **colours and background** from the cover
(median ink colour, dominant background colour or a blurred strip of the cover), which is deterministic image processing.

## 9. Recommended staged architecture

**Stage 0 - no identification needed.** Use a publisher-supplied spine or full jacket when one exists.

**Stage 1 - baseline, shippable now, Rust + wasm, no model.**
1. Title box: the user confirms a proposed box (Otsu/colour clustering + connected components from `imageproc`); optional OCR
   confirmation later via `ocrs`/`rten`.
2. Gallery: a curated set of 300-600 open faces chosen for cover typography. Do not ship whole fonts eagerly (25 KB per face).
   Either subset every gallery face to the title's characters on the server at card-creation time (the server already knows the
   title; pure-Rust subsetters exist: `subsetter`, `fontcull`, `allsorts`), or lazy-load in tiers (60 most cover-typical faces
   first) ([subsetter](https://crates.io/api/v1/crates/subsetter), [fontcull](https://crates.io/api/v1/crates/fontcull),
   [allsorts](https://crates.io/api/v1/crates/allsorts)) [V1 for existence and licence only].
3. Render-and-compare in wasm with tracking fitted per candidate; rasterise with a pure-Rust stack (`skrifa` for outlines,
   `harfrust` for shaping, `tiny-skia` for fill, `wuff` for WOFF2 decoding)
   ([skrifa](https://crates.io/api/v1/crates/skrifa), [harfrust](https://crates.io/api/v1/crates/harfrust),
   [tiny-skia](https://crates.io/api/v1/crates/tiny-skia), [wuff](https://crates.io/api/v1/crates/wuff)) [V1 existence only].
4. UI: five spine previews, user picks; always offer a neutral default (one serif, one sans) when confidence is low.

**Stage 2 - small embedding CNN for coarse retrieval over the full open corpus.**
- Data: synthetic renders of all open faces (1,908 Google Fonts families plus other libre foundries) with real title strings in
  French and English; augmentations that mirror section 7's failure causes (textured and photographic backgrounds, low
  contrast, tracking -5..+30 %, outlines, shadows, arcs, perspective, rotation +/-5 deg, blur, JPEG, 16-120 px text, all-caps
  and small-caps variants, multi-line crops).
- Model: MobileNetV1/V3-class or ResNet-18 backbone on a 64x320 crop, 128-d L2-normalised embedding. Loss: ArcFace/CosFace over
  face classes (scales to thousands of classes better than triplet mining) plus auxiliary heads for category, weight and italic;
  hard negatives are sibling weights of the same family. Libraries: `timm` 1.0.31 (Apache-2.0), `pytorch-metric-learning` 3.0.2
  (MIT) ([timm](https://pypi.org/pypi/timm/json), [pml](https://pypi.org/pypi/pytorch-metric-learning/json)) [V1].
- Domain gap: DeepFont's lesson is that synthetic-only training loses heavily on real images; add unlabelled real title crops
  (self-supervised or auto-encoder pre-training) and use Stage 1 as an automatic verifier to mine pseudo-labels [M + est.].
- Export: PyTorch -> ONNX opset 17 -> `onnxslim` -> ONNX Runtime static QDQ int8, per-channel
  ([onnxslim](https://pypi.org/pypi/onnxslim/json), [onnxruntime](https://pypi.org/pypi/onnxruntime/json)) [V1]. Measured
  sizes: 1.0 MB (0.88M params), 3.7 MB (3.46M), 11.4 MB (11.3M) [X]. The brief's 2-10 MB target corresponds to MobileNet-class
  fp32 or ResNet-18 int8.
- Runtime: `rten` if binary size matters most (1.0 MB gzip), `tract` for the fastest MobileNet inference, ONNX Runtime Web only
  if a PaddleOCR detector is added anyway. Keep the Rust API behind a trait so the runtime can be swapped.
- Index: one 128-byte int8 vector per face; 6,000 faces = 0.8 MB. The browser then downloads only the top 10-20 fonts and runs
  Stage 1 on them as verification and parameter fitting.
- Accuracy expectation: unknown. DeepFont reported top-5 above 80 % on real images for 2,383 classes [M]; our task is easier in
  one way (known string, top-5 with human pick) and harder in another (true font usually absent). Build the real-cover
  evaluation set before training anything.

**Stage 3 - optional.** In-browser text detection (PP-OCRv6 tiny, ~6 MB) to remove the manual box; revisit generative lettering
only if phone-class WebGPU and sub-100 MB text-editing models appear.

## Sources

Registry endpoints fetched this session (all HTTP 200 on 2026-11-12). These are the only URLs I can vouch for.

- npm: [onnxruntime-web](https://registry.npmjs.org/onnxruntime-web), [@huggingface/transformers](https://registry.npmjs.org/@huggingface/transformers),
  [@paddleocr/paddleocr-js](https://registry.npmjs.org/@paddleocr/paddleocr-js), [ppu-paddle-ocr](https://registry.npmjs.org/ppu-paddle-ocr),
  [@gutenye/ocr-browser](https://registry.npmjs.org/@gutenye/ocr-browser), [tesseract.js](https://registry.npmjs.org/tesseract.js),
  [tesseract.js-core](https://registry.npmjs.org/tesseract.js-core), [@tesseract.js-data/fra](https://registry.npmjs.org/@tesseract.js-data/fra),
  [@techstark/opencv-js](https://registry.npmjs.org/@techstark/opencv-js), [google-font-metadata](https://registry.npmjs.org/google-font-metadata),
  [@fontsource/playfair-display](https://registry.npmjs.org/@fontsource/playfair-display), [@tensorflow/tfjs](https://registry.npmjs.org/@tensorflow/tfjs),
  [@litertjs/core](https://registry.npmjs.org/@litertjs/core)
- crates.io: [tract-onnx](https://crates.io/api/v1/crates/tract-onnx), [rten](https://crates.io/api/v1/crates/rten),
  [ocrs](https://crates.io/api/v1/crates/ocrs), [burn](https://crates.io/api/v1/crates/burn), [burn-flex](https://crates.io/api/v1/crates/burn-flex),
  [burn-onnx](https://crates.io/api/v1/crates/burn-onnx), [burn-wgpu](https://crates.io/api/v1/crates/burn-wgpu),
  [burn-ndarray](https://crates.io/api/v1/crates/burn-ndarray), [burn-import](https://crates.io/api/v1/crates/burn-import),
  [candle-core](https://crates.io/api/v1/crates/candle-core), [ort](https://crates.io/api/v1/crates/ort), [ort-web](https://crates.io/api/v1/crates/ort-web),
  [ort-tract](https://crates.io/api/v1/crates/ort-tract), [wonnx](https://crates.io/api/v1/crates/wonnx), [imageproc](https://crates.io/api/v1/crates/imageproc),
  [getrandom](https://crates.io/api/v1/crates/getrandom), [skrifa](https://crates.io/api/v1/crates/skrifa), [harfrust](https://crates.io/api/v1/crates/harfrust),
  [wuff](https://crates.io/api/v1/crates/wuff), [subsetter](https://crates.io/api/v1/crates/subsetter), [fontcull](https://crates.io/api/v1/crates/fontcull),
  [allsorts](https://crates.io/api/v1/crates/allsorts), [tiny-skia](https://crates.io/api/v1/crates/tiny-skia)
- PyPI: [paddleocr](https://pypi.org/pypi/paddleocr/json), [rapidocr](https://pypi.org/pypi/rapidocr/json), [trdg](https://pypi.org/pypi/trdg/json),
  [timm](https://pypi.org/pypi/timm/json), [pytorch-metric-learning](https://pypi.org/pypi/pytorch-metric-learning/json),
  [onnxruntime](https://pypi.org/pypi/onnxruntime/json), [onnxslim](https://pypi.org/pypi/onnxslim/json), [easyocr](https://pypi.org/pypi/easyocr/json),
  [python-doctr](https://pypi.org/pypi/python-doctr/json), [onnxtr](https://pypi.org/pypi/onnxtr/json),
  [craft-text-detector](https://pypi.org/pypi/craft-text-detector/json), [fonttools](https://pypi.org/pypi/fonttools/json)
- READMEs read from inside the published packages: onnxruntime-web 1.31.1, @huggingface/transformers 4.3.2,
  @paddleocr/paddleocr-js 0.4.2, ppu-paddle-ocr 6.7.0, @gutenye/ocr-browser 1.4.9, tesseract.js 7.0.0, rten 0.28.0, ocrs 0.14.0,
  burn 0.22.1, burn-flex, burn-onnx; crate source of imageproc 0.27.0.
- Own measurements [X]: wasm builds of tract-onnx 0.23.10 and rten 0.28.0 (rustc 1.97, `wasm32-unknown-unknown`), Node 22.23,
  onnxruntime 1.29 (Python) for quantisation; render-and-compare benchmark in Python 3.13 with Pillow 12.3 (FreeType 2.14) and numpy.
- Papers cited from memory without a link: Wang et al. 2015 (DeepFont); Chen et al. 2014; Tatsukawa et al. 2024 (FontCLIP);
  Chen et al. 2021 (HENet); Krishnan et al. 2021 (TextStyleBrush); Liao et al. 2020 (DBNet); Baek et al. 2019 (CRAFT);
  Zhou et al. 2017 (EAST); Epshtein et al. 2010 (SWT); Zramdini and Ingold 1998; Zhu et al. 2001; Khoubyari and Hull 1996;
  Solli and Lenz 2007; Shinahara et al. 2019; Iwana et al. 2016; O'Donovan et al. 2014.

## Could not verify

Everything below is [M] or unknown because no web search could be run. Re-check before relying on it.

- Any commercial tool's current features, API availability, quota, price or terms of use (WhatTheFont, Adobe, Fontspring,
  Font Squirrel, WhatFontIs, Identifont). In particular: whether WhatFontIs still offers an API, its price, and whether its terms
  allow commercial server-side use. What "Like That Garden" refers to.
- DeepFont / AdobeVFR figures (2,383 classes, 4,384 real test images, top-5 above 80 %) and the dataset's licence terms.
- Architecture, class count, accuracy, model size and licence of gaborcselle/font-identifier, Storia-AI/font-classify,
  YuzuMarker.FontDetection, HENet and FontCLIP; whether any Swin-based Latin font recogniser has public weights.
- Existence and accuracy of prior published work on known-string render-and-compare font matching on book covers.
- Model sizes and licences for CRAFT, EAST, DBNet reference weights and docTR detectors; the exact size of the PP-OCRv6 tiny
  **detection** model alone (the ~6 MB figure is detection + recognition, from a third-party README that also lists a ~19.9M
  parameter recogniser, which I cannot reconcile with 6 MB without knowing the weight format).
- Training data, language coverage (French accents) and model sizes of `ocrs`.
- Current WebGPU availability in Safari, Firefox and Android WebViews; the ONNX Runtime Web README matrix looks older than
  the browsers.
- Real phone performance: all latencies were measured on an x86 server vCPU in Node, single-threaded; the 2-4x phone factor is
  an assumption. No device, no browser, no thermal throttling was tested.
- Field accuracy of the baseline: the benchmark is synthetic, uses one rasteriser, single-line titles, and always contains the
  true font. No real cover was processed.
- candle and Burn in the browser were not built or timed; `ort-web` was not tried.
- OFL Reserved-Font-Name consequences of self-subsetting gallery fonts.
