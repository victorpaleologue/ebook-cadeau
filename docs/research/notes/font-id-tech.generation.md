# Regenerating cover lettering and the spine background in the browser — tools, models, feasibility

Angle: the generation side (lettering and background), image-only models, in-browser. Date: 2026-11-12.
Companion notes: `legal-fonts-genai.fonts.md` (legal), `print-requirements.prepress.md` (print minimums), the sibling font-identification note.

## 0. Status of this note — read first

**No web search could be run.** Every WebSearch call was refused because the workflow's shared budget (200 calls per turn) was already spent. I did not route around it through search engines, readers or archives. Instead I read **primary documents directly** on hosts the sandbox allows: README / LICENSE / source files of about 70 repositories on `raw.githubusercontent.com`, the crates.io and npm registry APIs, and the HTTP headers of two GitHub release assets. `huggingface.co` is blocked (proxy 403), so no model card or Hugging Face file size was checked.

Confidence tags:

- **[V2]** two independent primary documents read this session. **[V1]** one primary document read this session.
- **[L]** computed locally this session (script over the repo's own font files, or arithmetic shown).
- **[M]** from memory, unverified. **[M?]** memory, low confidence. **[E]** my engineering estimate, not measured.

URL policy: every link either returned HTTP 200 in this session or appears verbatim inside a document that did. A `github.com/<owner>/<repo>` link is the repository form of a `raw.githubusercontent.com/<owner>/<repo>/…` path that returned 200. "No licence file" means none was found at the repository root under the usual names. Nothing here is a survey of the 2025–2026 literature: recent work is only covered where a fetched document names it.

## 1. Key findings

1. **The spine repeats the cover's own title, so no unseen glyph is needed.** Few-shot font generation solves a problem this product mostly does not have. It only matters if text that is not on the cover must be set in the title's style. [L, reasoning]
2. **At spine size the fine style of display lettering is not visible.** A 9–12 pt title has a cap height of 2.1–2.8 mm, i.e. 50–67 px at 600 dpi [L]. What survives is weight, width, serif class, case, slant, spacing and colour. A good match in an open-font library reproduces those [E].
3. **Released few-shot font models are blocked by licence, script or both.** FontDiffuser is "non-commercial research purposes" only; FUNIT, Attr2Font and AGIS-Net are CC BY-NC; DG-Font, CF-Font and Diff-Font have no licence file and were trained on Founder fonts that are free "for non-commercial users"; the MIT-licensed NAVER models ship a weight trained on fonts they do not release. Almost all are Chinese-centric. [V1 each]
4. **I found no browser (ONNX/WebGPU) port of any few-shot font generator** [M; could not search]. FontDiffuser's public demo is a server-side Gradio app [V1].
5. **Scene-text editing models that avoid language models exist but are research-grade**: SRNet, MOSTEL, STEFANN (no text encoder at all), UDiffText and TextCtrl (character/glyph encoders trained from scratch). The diffusion ones sit on Stable Diffusion backbones; AnyText needs about 7.5 GB of GPU memory per 512×512 image [V1]. TextDiffuser-2 and Glyph-ByT5 use language models by design [V1].
6. **Background: classical methods should cover most covers and carry no generative risk** [E]. Edge-colour sampling, a per-row gradient and a palette fallback are deterministic, cost milliseconds and need no download.
7. **MI-GAN is the only learned inpainting model verified to run in a browser at a shippable size**: 27.0 MB, MIT code and MIT weights, used by inpaint-web through ONNX Runtime Web with WebGPU and a wasm fallback [V2]. LaMa is 93–206 MB depending on the export [V1]. Stable Diffusion builds exist for WebGPU but are gigabytes and need a text encoder [V1/M].
8. **WebGPU is not universal in 2026**: no Firefox on Linux or Android, Chrome on Linux only on Intel Gen12+ GPUs [V1]. Wasm multi-threading needs `crossOriginIsolated` [V1], which an embedded reseller widget cannot assume. The default path must therefore be classical and CPU-cheap.
9. **Nothing has to be trained for the recommended default.** Optional training (text segmentation, a Latin few-shot generator on OFL fonts, a font-identification embedding) is about 50–170 GPU-hours for one clean run of everything; budget two to three times that for iteration [E].
10. **Legibility on a 10 mm spine**: type lives in the central 6 mm [L, prepress note]; practical sizes are 9–12 pt mixed case or 12–16 pt all caps [L]; contrast target 7:1, floor 4.5:1, computed on the printed colours.

## 2. Few-shot font generation (raster)

| Method | Venue | Script of released code/weights | Resolution | Code licence | Data / weights caveat | Source |
|---|---|---|---|---|---|---|
| FontDiffuser | AAAI 2024 | Chinese; shows Chinese→Korean | 96×96 | none; README: "can only be used for non-commercial research purposes", commercial use by contacting the lab | weights on Google Drive / Baidu | [repo](https://github.com/yeungchenwa/FontDiffuser), [paper](https://arxiv.org/abs/2312.12142) [V1] |
| Diff-Font | arXiv 2022 | Chinese, 6,625 characters, stroke-conditioned | 80×80 | no licence file | Founder fonts, "free font download for non-commercial users" | [repo](https://github.com/Hxyz-123/Font-diff), [paper](https://arxiv.org/abs/2212.05895) [V1] |
| DG-Font | CVPR 2021 | Chinese | "only works with 80x80" | no licence file | Founder fonts | [repo](https://github.com/ecnuycxie/DG-Font) [V1] |
| CF-Font | CVPR 2023 | Chinese, GB/T 2312 (6,763 characters), 240 seen + 60 unseen fonts | as DG-Font | no licence file | Founder fonts | [repo](https://github.com/wangchi95/CF-Font) [V1] |
| MX-Font / LF-Font / DM-Font | ICCV 2021 / AAAI 2021 / ECCV 2020 | Chinese (MX, LF), Korean (DM); LF needs component labels | — | MIT, except FUNIT-derived modules | "we only provide the example font files; not the font files used for the training the provided weight" | [mxfont](https://github.com/clovaai/mxfont), [lffont](https://github.com/clovaai/lffont), [dmfont](https://github.com/clovaai/dmfont), [unified repo](https://github.com/clovaai/fewshot-font-generation) [V1] |
| FUNIT | ICCV 2019 | generic image translation, adapted to fonts by NAVER | — | CC BY-NC-SA 4.0, "academic research use only" | — | [repo](https://github.com/NVlabs/FUNIT) [V1] |
| VQ-Font | ICCV 2023 | Chinese | — | MIT | Founder fonts | [repo](https://github.com/awei669/VQ-Font) [V1] |
| FsFont | CVPR 2022 | Chinese | — | MIT-style (Baidu) | — | [repo](https://github.com/tlc121/FsFont) [V1] |
| IF-Font | NeurIPS 2024 | Chinese (ideographic description sequences, VQGAN tokens) | — | MIT | — | [repo](https://github.com/Stareven233/IF-Font) [V1] |
| MC-GAN | arXiv 2017 (CVPR 2018 [M]) | **Latin capitals A–Z** | 64×64 | BSD-style | 10K-font set plus "in-the-wild" fonts from flamingtext; the ornamentation stage is trained per target font (`train_StackGAN.sh ${DATA}`) | [repo](https://github.com/azadis/MC-GAN), [paper](https://arxiv.org/abs/1712.00516) [V1] |
| Attr2Font / AGIS-Net | SIGGRAPH 2020 / SIGGRAPH Asia 2019 [M] | Latin, attribute-driven [M] / English and Chinese artistic glyphs, English sets taken from MC-GAN | — | CC BY-NC 4.0 | — | [Attr2Font](https://github.com/hologerry/Attr2Font), [AGIS-Net](https://github.com/hologerry/AGIS-Net) [V1] |
| STEFANN (FANnet) | CVPR 2020 | **Latin, 62 alphanumerics** | 64×64 grey | Apache-2.0 | 1,015 train + 300 validation fonts, source not stated | [repo](https://github.com/prasunroy/stefann) [V1] |
| GAS-NeXt | 2022 [M] | Chinese↔English [M] | — | WTFPL | one-line README pointing to a branch | [repo](https://github.com/cmu-11785-F22-55/GAS-NeXt) [V1] |
| Calliffusion, DS-Font | 2023 [M?] | Chinese calligraphy / Chinese | — | no code located | Calliffusion is conditioned through a BERT-type text encoder [M?] | — |

Observations:

- **Licence wall.** NAVER's unified repository states the cause plainly: "fonts are protected intellectual property and it is unable to release the collected font datasets unless license is cleaned-up. Many font generation papers do not publicly release their own datasets due to this license issue." [V1, [README](https://github.com/clovaai/fewshot-font-generation)] No released weight in the table is safe for a commercial service. Any use means retraining on open fonts.
- **Script.** Only MC-GAN, AGIS-Net (through MC-GAN's English sets) and STEFANN verifiably target Latin [V1]; Attr2Font does too [M]. FontDiffuser claims cross-lingual transfer but shows Chinese→Korean only [V1].
- **Cost.** FontDiffuser samples with DPM-Solver++ at 20 steps and guidance 7.5 [V1, [script](https://raw.githubusercontent.com/yeungchenwa/FontDiffuser/main/scripts/sample_content_image.sh)], i.e. 40 U-Net evaluations per glyph with classifier-free guidance [L]. Its training recipe is 440,000 + 30,000 steps at batch 16 and 96 px [V1, [phase 1](https://raw.githubusercontent.com/yeungchenwa/FontDiffuser/main/scripts/train_phase_1.sh), [phase 2](https://raw.githubusercontent.com/yeungchenwa/FontDiffuser/main/scripts/train_phase_2.sh)]. Model file sizes were not verifiable (weights are on Google Drive).
- **Reference count is not the bottleneck.** "Le Comte de Monte-Cristo" already gives 13 distinct glyphs [L]; these methods take 1 to about 8 references [V1 for one-shot; M for 8]. The bottlenecks are clean segmented references from a textured cover, the 64–128 px output, and display faces: models trained on text fonts regress toward the mean on ornate lettering [M]. MC-GAN needing a per-font training stage for ornamentation illustrates this [V1].
- **Effects.** Texture and effect transfer is a separate literature ([TET-GAN](https://github.com/williamyang1991/TET-GAN), AAAI 2019, MIT; [Shape-Matching GAN](https://github.com/VITA-Group/ShapeMatchingGAN), ICCV 2019, MIT) [V1]. Not needed at spine size.

## 3. Vector approaches

| Method | Venue | Script | Input | Licence | Caveat | Source |
|---|---|---|---|---|---|---|
| DeepVecFont | SIGGRAPH Asia 2021 | Latin | "a few **vector** glyphs of a font as reference" | MIT | refinement through diffvg at test time, 20 candidates per glyph; dataset is a subset of SVG-VAE | [repo](https://github.com/yizhiwang96/deepvecfont) [V1] |
| DeepVecFont-v2 | CVPR 2023 | Latin and Chinese checkpoints | 4 references (Latin), 8 (Chinese) | MIT | 20–50 samples per glyph, selected by IoU; "all the Chinese fonts are collected from Founder, and the fonts CANNOT be used for any commercial uses" | [repo](https://github.com/yizhiwang96/deepvecfont-v2), [paper](https://arxiv.org/abs/2303.14585) [V1] |
| DualVector | CVPR 2023 | Latin | glyph images (unsupervised, dual-part representation) | MIT | same dataset as DeepVecFont; contour post-refinement step | [repo](https://github.com/thuliu-yt16/dualvector) [V1] |
| VecFusion | CVPR 2024 | Latin [M] | raster diffusion then vector diffusion [M] | no code located [M] | project page only | [paper](https://arxiv.org/abs/2312.10540), [page](https://vikastmz.github.io/VecFusion/) [V2 for existence and venue] |
| SVG-VAE, DeepSVG | ICCV 2019, NeurIPS 2020 [M] | Latin | SVG command sequences | DeepSVG MIT | font sets scraped from the web [M] | [deepsvg](https://github.com/alexandre01/deepsvg) [V1] |

DeepVecFont expects vector references (v1 says so [V1]; v2 is trained on the same vector data [M]), so a raster title crop would first need tracing, which adds a domain gap [M]. All of them depend on sampling many candidates or on differentiable-rasteriser refinement, which is not browser material [E]. For this project the vector route is better served by **tracing** a raster glyph: [vtracer](https://github.com/visioncortex/vtracer) is MIT OR Apache-2.0, pure Rust, has a binary mode and ships a WebAssembly build [V2: repo + crates.io]. Potrace is GPL and excluded by the repo rules [M].

## 4. Scene-text editing: which ones avoid language models

Four classes, by how the target text enters the model:

- **A. Rendered glyph image or one-hot character. No text encoder.**
- **B. Character or glyph encoder trained from scratch for glyph alignment.** Not a language model: it never models word sequences.
- **C. CLIP-family text encoder for a prompt or caption.** A text transformer, though not a generative LLM.
- **D. Pretrained language model** (T5/ByT5, LLM layout planner).

| Model | Venue | Class | Backbone / size | Licence | Source |
|---|---|---|---|---|---|
| SRNet | ACM MM 2019 [M] | A: target rendered in a standard font on grey (`i_t`) | GAN with a VGG19 perceptual loss; three sub-networks [M] | TensorFlow re-implementation is GPL-3.0; PyTorch ports have no licence file; no official code [M] | [youdao-ai](https://github.com/youdao-ai/SRNet), [Niwhskal](https://github.com/Niwhskal/SRNet) [V1] |
| STEFANN | CVPR 2020 | A: source character image + target character label [M] | two small CNNs at 64×64 | Apache-2.0 | [repo](https://github.com/prasunroy/stefann) [V1] |
| TextStyleBrush | Meta, 2021 [M] | A [M] | StyleGAN2-based [M] | no code or weights [M]; its Imgur5K dataset is CC BY-NC 4.0 [V1] | [dataset](https://github.com/facebookresearch/IMGUR5K-Handwriting-Dataset) |
| MOSTEL | AAAI (2023 [M]) | A: stroke-level, SRNet-style data | GAN + recogniser supervision | no licence file | [repo](https://github.com/qqqyd/MOSTEL), [paper](https://arxiv.org/abs/2212.01982) [V1] |
| RS-STE | CVPR 2025 | A/B: "GPT transformer" over image latents, trained on text images | 32×128 px images | no licence file | [repo](https://github.com/ZhengyaoFang/RS-STE) [V1] |
| UDiffText | ECCV 2024 | **B**: "Character-level encoder" (`LabelEncoder`) | Stable Diffusion 2 inpainting, 512 | MIT; data setup lists LAION-OCR, TextSeg, SynthText | [repo](https://github.com/ZYM-PKU/UDiffText), [paper](https://arxiv.org/abs/2312.04884) [V1] |
| TextCtrl | NeurIPS 2024 | **B**: "pretrained glyph encoder" + style encoder | Stable Diffusion v1-5 | no licence file | [repo](https://github.com/weichaozeng/TextCtrl) [V1] |
| DiffUTE | NeurIPS 2023 | B: glyph image encoder [M] | SD2 inpainting checkpoint | Apache-2.0; original data not released | [repo](https://github.com/chenhaoxing/DiffUTE) [V1] |
| DiffSTE | arXiv 2023 | B + C: "character encoder" and "instruction encoder", natural-language instructions | SD inpainting | MIT | [repo](https://github.com/UCSB-NLP-Chang/DiffSTE), [paper](https://arxiv.org/abs/2304.05568) [V1] |
| GlyphControl | NeurIPS 2023 | C: glyph image through a ControlNet-style branch [M], plus prompt | 1.3 B parameters | MIT | [repo](https://github.com/AIGText/GlyphControl-release) [V1] |
| TextDiffuser | NeurIPS 2023 | C: character segmentation masks from a layout transformer, plus prompt | checkpoint 3.2 GB | MIT repo, but "academic and research purposes ONLY" | [README](https://raw.githubusercontent.com/microsoft/unilm/master/textdiffuser/README.md) [V1] |
| AnyText / AnyText2 | ICLR 2024 / code released 2025-03 | C: OCR-model stroke embeddings that "blend with image caption embeddings"; loads `clip-vit-large-patch14` | SD 1.5; about 7.5 GB GPU memory at FP16 for one 512×512 image; AnyText2 adds font and colour attributes | Apache-2.0 | [AnyText](https://github.com/tyxsspa/AnyText), [AnyText2](https://github.com/tyxsspa/AnyText2), [paper](https://arxiv.org/abs/2311.03054) [V1] |
| TextDiffuser-2 | ECCV 2024 | **D**: "utilizes two language models for layout planning and layout encoding" | — | research only | [README](https://raw.githubusercontent.com/microsoft/unilm/master/textdiffuser-2/README.md) [V1] |
| Glyph-ByT5 / v2 | ECCV 2024 | **D**: fine-tuned "character-aware ByT5 encoder" | SDXL | Apache-2.0; 2024-06-28: "We have removed the weights and code that may have used potentially unauthorized datasets"; no restoration notice in the README today | [repo](https://github.com/AIGText/Glyph-ByT5), [paper](https://arxiv.org/abs/2403.09622) [V1] |
| TextFlux, Calligrapher | 2025 | glyph image or style reference into FLUX.1-Fill-dev | FLUX uses T5 and CLIP text encoders [M]; Calligrapher adds SigLIP | TextFlux code Apache-2.0; Calligrapher no licence file; FLUX.1-dev weights non-commercial [M] | [TextFlux](https://github.com/yyyyyxie/textflux), [Calligrapher](https://github.com/Calligrapher2025/Calligrapher) [V1] |

Reading against the "no language model" constraint: classes A and B comply. Class C complies in practice if the prompt is fixed, because the prompt embedding can be precomputed at build time and shipped as a constant, so no text encoder runs in the browser [E]. Class D does not comply.

Feasibility: none of these has a published ONNX/WebGPU build that I know of [M; could not search]. The Stable-Diffusion-based ones are gigabyte downloads with tens of denoising steps. The GAN ones (SRNet, MOSTEL) are small enough but were trained on synthetic scene text at low resolution, and their licences are GPL or absent [V1]. Training data is a further problem: UDiffText's data setup includes TextSeg, which is "academia-only and cannot be used on any commercial project and research" [V1, [TextSeg](https://github.com/SHI-Labs/Rethinking-Text-Segmentation)]. **Verdict: research-grade, not shippable in a browser today.** They also solve the harder problem of blending new text into a scene; our spine is a blank panel we design ourselves.

## 5. Spine background

### 5.1 Geometry

The spine is 10 mm × the card height. At 300 dpi and 180 mm that is 118 × 2,126 px, about 0.25 megapixel [L]. In the outside spread (back | spine | front) only the front is given; we design the back. So the fill is a **one-sided extrapolation** from the front cover's edge, unless the back uses the same colour. Using the same background for spine and back turns the problem into a simple colour choice. Because folds shift by 1–2 mm [L, prepress note], a spine colour sampled from the adjacent cover edge also hides fold misregistration [E].

### 5.2 Classical options (no model, deterministic)

| Technique | Works when | Fails when | Implementation |
|---|---|---|---|
| Flat colour: median of the cover strip next to the spine, in Oklab | edge strip is uniform (most typographic covers) [E] | photo or illustration reaching the edge | trivial; [`kmeans_colors`](https://github.com/okaneco/kmeans-colors) and [`okolors`](https://github.com/IanManske/Okolors) are MIT OR Apache-2.0 [V1] |
| Vertical gradient: per-row median of the strip, low-pass filtered | sky, vignette, smooth photo backgrounds | strong horizontal structures (streaks) | about 50 lines |
| Palette colour (dominant or darkest/lightest cluster of the whole cover) | edge strip is busy; always available | can look unrelated to the edge | k-means on a 128 px thumbnail |
| Mirrored edge with progressive blur | soft textures | faces, letters or objects near the edge get mirrored | cheap; needs an edge-density gate |
| Diffusion inpainting: Telea fast marching, Navier–Stokes | thin holes, scratches | wide one-sided regions: it smears the edge into streaks, i.e. no better than row replication [M] | `cv.inpaint` is in the OpenCV.js whitelist [V1, [config](https://raw.githubusercontent.com/opencv/opencv/master/platforms/js/opencv_js.config.py)]; the Rust [`inpaint`](https://codeberg.org/gillesvink/inpaint) crate (Telea port) is **EUPL-1.2**, outside the repo's allowed licences [V1] |
| Example-based texture synthesis: Efros–Leung, image quilting with minimum-cut seams, PatchMatch [M] | stochastic textures such as paper grain, canvas, noise | "Struggles with complex semantics", "Not great with regular textures (seams can become obvious)" [V1] | [`texture-synthesis`](https://github.com/EmbarkStudios/texture-synthesis): MIT OR Apache-2.0, has inpaint and tiling modes, deterministic only with one thread, last release 0.8.2 in 2022-02, maintainers say they are "not actively using or developing" it [V2: repo + crates.io]. PatchMatch may be covered by Adobe patents in the US [M?] |

### 5.3 Learned inpainting and outpainting

| Model | Size | Licence | Browser status | Notes |
|---|---|---|---|---|
| MI-GAN (ICCV 2023) | 27,048,675 bytes as TorchScript [V1, [asset](https://github.com/Sanster/models/releases/download/migan/migan_traced.pt)]; an ONNX "pipeline" with pre- and post-processing is published (size not verified, about 28–30 MB [M]) | MIT code and MIT weights ([LICENSE-WEIGHTS](https://raw.githubusercontent.com/Picsart-AI-Research/MI-GAN/main/LICENSE-WEIGHTS)) [V1] | **Runs in browsers**: [inpaint-web](https://github.com/lxfater/inpaint-web) loads `migan_pipeline_v2.onnx` with onnxruntime-web 1.16.3, WebGPU if available, else wasm [V2: MI-GAN README + inpaint-web [source](https://raw.githubusercontent.com/lxfater/inpaint-web/main/src/adapters/cache.ts)] | 512×512 Places2 model; "one order of magnitude smaller and faster than recent SOTA" [V1, [repo](https://github.com/Picsart-AI-Research/MI-GAN)]. inpaint-web itself is GPL-3.0: do not copy its code [V1] |
| LaMa (big-lama) | 205,669,692 bytes as TorchScript [V1, [asset](https://github.com/Sanster/models/releases/download/add_big_lama/big-lama.pt)]; OpenCV Zoo ONNX 92,591,623 bytes, its demo resizes to 512×512 [V1, [zoo](https://raw.githubusercontent.com/opencv/opencv_zoo/main/models/inpainting_lama/README.md)] | Apache-2.0 [V1] | no first-party browser build verified; community demos exist [M] | trained at 256×256, "generalizes surprisingly well to much higher resolutions" (about 2k), including "completion of periodic structures" [V1, [repo](https://github.com/advimman/lama)]. So the answer to "about 200 MB?" is 93–206 MB depending on the export |
| Stable Diffusion (SD-Turbo, 1.5, 2.1) | gigabytes [M] | model-specific | WebGPU demos exist: [ONNX Runtime Web SD-Turbo example](https://raw.githubusercontent.com/microsoft/onnxruntime-inference-examples/main/js/sd-turbo/README.md), [web-stable-diffusion](https://github.com/mlc-ai/web-stable-diffusion); [diffusers.js](https://github.com/dakenf/diffusers.js) needs a modified 64-bit ONNX Runtime build [V1] | needs a text encoder; can invent objects or pseudo-text; not recommended |

Browser runtime facts:

- **WebGPU support** [V1, [MDN compat data](https://raw.githubusercontent.com/mdn/browser-compat-data/main/api/GPU.json)]: Chrome/Edge 113+ on Windows, macOS, ChromeOS; Linux only from 144 and only on Intel Gen12+ GPUs; Chrome Android 121+; Safari 26+; Firefox 141+ on Windows, 145+ on macOS Tahoe (Apple silicon), 147+ on older macOS (Apple silicon); no Firefox on Linux, Intel Macs or Android.
- **ONNX Runtime Web limits** [V1, [large models](https://raw.githubusercontent.com/microsoft/onnxruntime/gh-pages/docs/tutorials/web/large-models.md), [flags](https://raw.githubusercontent.com/microsoft/onnxruntime/gh-pages/docs/tutorials/web/env-flags-and-session-options.md)]: `ArrayBuffer` about 2 GB in Chrome, protobuf 2 GB, WebAssembly 4 GB; wasm multi-threading only when `crossOriginIsolated` is enabled. A widget embedded in a reseller page cannot count on that, so the CPU fallback is single-threaded [E].
- **Runtimes**: `onnxruntime-web` 1.31.1, MIT [V1, [npm](https://registry.npmjs.org/onnxruntime-web)]. From Rust: [`ort`](https://github.com/pykeio/ort) with the `ort-web` or `ort-tract` backends, [`tract`](https://github.com/sonos/tract) ("in the browser via WebAssembly"), [`burn`](https://github.com/tracel-ai/burn) (WebGPU on wasm), all MIT OR Apache-2.0 [V2: repos + crates.io]. [`wonnx`](https://github.com/webonnx/wonnx) has had no release since 2023-09 [V1].

Latency was not measured. Expect tens of milliseconds for the classical ladder in wasm, well under a second for MI-GAN on WebGPU and a few seconds on single-thread wasm [E].

## 6. What would need training

| Component | Needed for | Data (open-licensed only) | Compute [E] | Evaluation |
|---|---|---|---|---|
| Nothing | default pipeline (§8) | — | 0 | — |
| Title text segmentation (small U-Net, 2–5 M parameters) | optional lettering reuse | synthetic: OFL fonts rendered with effects over CC0 / public-domain images; generators exist ([SynthTIGER](https://github.com/clovaai/synthtiger) MIT, [SynthText](https://github.com/ankush-me/SynthText) Apache-2.0, [TRDG](https://github.com/Belval/TextRecognitionDataGenerator) MIT) [V1]. Not TextSeg (academia-only) [V1] | 5–15 GPU-h | IoU on held-out synthetic data plus a hand-labelled set of about 200 covers |
| Latin few-shot glyph generator (GAN or distilled diffusion, 64–128 px, then traced) | optional: text not present on the cover | [Google Fonts](https://github.com/google/fonts): "Most of the fonts in the collection use the SIL Open Font License, v1.1. Some fonts use the Apache 2 license" [V1]; about 1,900 families [M]; A–Z, a–z, digits and French accented letters | diffusion at FontDiffuser's recipe: 30–80 GPU-h; GAN: 20–50; distillation to 1–4 steps: 20–40 | L1, SSIM, LPIPS, FID on unseen fonts as in the papers [M]; plus OCR accuracy on generated glyphs, font-retrieval consistency (does the generated glyph retrieve its source font), and a printed proof at 10 pt |
| Font-identification embedding | default lettering path | synthetic renders of the same OFL corpus | 10–30 GPU-h | see the identification note |
| Background model | not recommended | covers are copyrighted; training on them relies on text-and-data-mining exceptions | — | — |

Total for one clean run of all optional items: about 50–170 GPU-hours on one 24 GB GPU; budget two to three times that for iteration [E]. Whether a model trained on OFL fonts yields outputs bound by the OFL is unsettled [M]; see the legal note.

## 7. Legibility on a 10 mm spine

Metrics measured from the bundled fonts in `assets/fonts/` [L]:

| Font | Cap height | x-height | Mixed-case extent (d to p) | Max size in a 6 mm band: caps / mixed | Characters in 120 mm at 10 pt: mixed / caps |
|---|---|---|---|---|---|
| Spectral Regular | 0.660 em | 0.450 em | 0.980 em | 25.8 / 17.4 pt | 68 / 53 |
| Atkinson Hyperlegible Bold | 0.668 em | 0.496 em | 0.829 em | 25.5 / 20.5 pt | 65 / 56 |
| Bebas Neue (caps only) | 0.700 em | — | 0.700 em | 24.3 pt | 96 |

- **Band.** Type stays in the central 6 mm; the outer 2 mm on each side absorb cut and fold shift [L, prepress note]. The maxima above leave no breathing room; keep the extent at 4.5–5 mm or less.
- **Size ranges.** Mixed case 9–12 pt (cap height 2.1–2.8 mm, x-height 1.4–2.1 mm); all caps 12–16 pt (cap height about 2.8–4.0 mm) for short titles. Floors: 7 pt positive, one step above the printers' 6 pt minimum, and 8 pt reversed [L, prepress note].
- **Viewing distance.** Fluent reading needs an x-height of about 0.2° of visual angle or more (Legge and Bigelow, 2011) [M]. That is 1.4 mm at 40 cm, 2.1 mm at 60 cm, 3.5 mm at 1 m [L]. So 9–10 pt mixed case reads in the hand; reading on a shelf from 1 m needs all caps at about 14 pt.
- **Contrast.** WCAG sets 4.5:1 for normal text, 3:1 for large text (18 pt, or 14 pt bold) and 7:1 for the enhanced level [V1, [1.4.3](https://raw.githubusercontent.com/w3c/wcag/main/understanding/20/contrast-minimum.html), [1.4.6](https://raw.githubusercontent.com/w3c/wcag/main/understanding/20/contrast-enhanced.html)]. APCA asks for Lc 75 as the minimum where readability matters and Lc 60 for other content text [V1, [APCA](https://raw.githubusercontent.com/Myndex/SAPC-APCA/master/documentation/APCAeasyIntro.md)]. Both are screen standards; use them as proxies. **Target 7:1, floor 4.5:1**, since spine type is below 14 pt.
- **Compute contrast on printed colours.** Convert CMYK to Lab through the output profile first. On coated stock, paper white (L* about 95) against 100 % K (L* about 16) gives about 13:1; on uncoated (93 against 31) only about 7.5:1 [L arithmetic on M? L* values]. A ratio of 4.5:1 corresponds to a lightness difference of about 45–50 L*, 7:1 to about 60 [L].
- **Print rules.** Dark text as 100 % K; reversed text at 8 pt or more in a sturdy weight; thinnest stroke at least 0.25 pt positive, 0.5 pt reversed [L, prepress note]. Check this with a distance transform on the rasterised outlines: a hairline serif at 10 pt fails it [E]. If no text colour reaches the floor, shift the background lightness or put a solid band behind the title.

## 8. Recommended pipeline, with fallbacks

1. **Provided spine or full wrap**: crop and scale; re-typeset only if the width ratio is too far off.
2. **Background ladder** (all classical): analyse the cover strip next to the spine (row medians in Oklab, variance, edge density). Uniform → flat colour. Smooth → vertical gradient. Stochastic texture → gradient plus matched grain. Busy or containing objects or letters → palette colour. Use the same background on the back panel.
3. **Optional, opt-in**: MI-GAN outpainting for textured edges, downloaded on demand (27 MB, cached), WebGPU preferred. Accept the result only if its mean colour stays close to the edge strip and it adds no new high-contrast structure; otherwise keep step 2.
4. **Lettering, default**: nearest open font from the identification stage, set as outlines with `ebc-text`. Carry over case, weight, width, slant and colour measured from the cover crop.
5. **Lettering, experimental and off by default**: reuse the cover's own title pixels (segment, trace, re-flow on one line). This needs no generation, but tracing custom lettering is the legally riskiest option in the legal note, so gate it to reseller mode with rights cleared.
6. **Lettering, research**: few-shot generation of missing glyphs with a model trained in-house on OFL fonts. Not planned for launch.
7. **Fit and preflight**: size inside the 6 mm band; truncate long titles at the subtitle delimiter; enforce contrast, minimum stroke and ink limits; record which fallback was used.

| Step | Maturity today in a browser |
|---|---|
| Flat, gradient, palette backgrounds; outline typesetting; contrast and stroke checks | production-ready, pure Rust/wasm, no download |
| MI-GAN outpainting | engineering-ready: proven in a browser by a third party, permissive licence, but unmeasured on covers and WebGPU is not universal |
| Texture synthesis; lettering reuse by segmentation and tracing | prototype-grade; needs a cover test set |
| LaMa in the browser | possible, heavy (93–206 MB), little gain on a 10 mm strip |
| Few-shot font generation; scene-text editing; Stable Diffusion outpainting | research-grade; licence, size, script or language-model constraints block all released models |

First measurements to make: the share of real covers per background class on about 200 covers; MI-GAN latency on WebGPU and single-thread wasm; how often the retrieved open font is judged a match at spine size.

## Sources

All read in this session (see the URL policy in §0).

- Font generation: [FontDiffuser](https://github.com/yeungchenwa/FontDiffuser), [Diff-Font](https://github.com/Hxyz-123/Font-diff), [DG-Font](https://github.com/ecnuycxie/DG-Font), [CF-Font](https://github.com/wangchi95/CF-Font), [MX-Font](https://github.com/clovaai/mxfont), [LF-Font](https://github.com/clovaai/lffont), [DM-Font](https://github.com/clovaai/dmfont), [fewshot-font-generation](https://github.com/clovaai/fewshot-font-generation), [FUNIT](https://github.com/NVlabs/FUNIT), [VQ-Font](https://github.com/awei669/VQ-Font), [FsFont](https://github.com/tlc121/FsFont), [IF-Font](https://github.com/Stareven233/IF-Font), [MC-GAN](https://github.com/azadis/MC-GAN), [Attr2Font](https://github.com/hologerry/Attr2Font), [AGIS-Net](https://github.com/hologerry/AGIS-Net), [STEFANN](https://github.com/prasunroy/stefann), [GAS-NeXt](https://github.com/cmu-11785-F22-55/GAS-NeXt), [TET-GAN](https://github.com/williamyang1991/TET-GAN), [Shape-Matching GAN](https://github.com/VITA-Group/ShapeMatchingGAN).
- Vector: [DeepVecFont](https://github.com/yizhiwang96/deepvecfont), [DeepVecFont-v2](https://github.com/yizhiwang96/deepvecfont-v2), [DualVector](https://github.com/thuliu-yt16/dualvector), [DeepSVG](https://github.com/alexandre01/deepsvg), [VecFusion paper](https://arxiv.org/abs/2312.10540), [vtracer](https://github.com/visioncortex/vtracer).
- Curated lists used to confirm titles and venues: [Recommendations-Diffusion-Text-Image](https://github.com/yeungchenwa/Recommendations-Diffusion-Text-Image), [TypographyResearchCollection](https://github.com/IShengFang/TypographyResearchCollection).
- Scene-text editing: [SRNet (youdao-ai)](https://github.com/youdao-ai/SRNet), [SRNet (Niwhskal)](https://github.com/Niwhskal/SRNet), [MOSTEL](https://github.com/qqqyd/MOSTEL), [RS-STE](https://github.com/ZhengyaoFang/RS-STE), [UDiffText](https://github.com/ZYM-PKU/UDiffText), [TextCtrl](https://github.com/weichaozeng/TextCtrl), [DiffUTE](https://github.com/chenhaoxing/DiffUTE), [DiffSTE](https://github.com/UCSB-NLP-Chang/DiffSTE), [GlyphControl](https://github.com/AIGText/GlyphControl-release), [Glyph-ByT5](https://github.com/AIGText/Glyph-ByT5), [AnyText](https://github.com/tyxsspa/AnyText), [AnyText2](https://github.com/tyxsspa/AnyText2), [TextDiffuser](https://raw.githubusercontent.com/microsoft/unilm/master/textdiffuser/README.md), [TextDiffuser-2](https://raw.githubusercontent.com/microsoft/unilm/master/textdiffuser-2/README.md), [TextFlux](https://github.com/yyyyyxie/textflux), [Calligrapher](https://github.com/Calligrapher2025/Calligrapher), [Imgur5K](https://github.com/facebookresearch/IMGUR5K-Handwriting-Dataset), [TextSeg](https://github.com/SHI-Labs/Rethinking-Text-Segmentation), [Hi-SAM](https://github.com/ymy-k/Hi-SAM).
- Inpainting: [MI-GAN](https://github.com/Picsart-AI-Research/MI-GAN), [inpaint-web](https://github.com/lxfater/inpaint-web), [LaMa](https://github.com/advimman/lama), [IOPaint](https://github.com/Sanster/IOPaint), [OpenCV Zoo LaMa](https://raw.githubusercontent.com/opencv/opencv_zoo/main/models/inpainting_lama/README.md), [OpenCV photo.hpp](https://raw.githubusercontent.com/opencv/opencv/4.x/modules/photo/include/opencv2/photo.hpp), [texture-synthesis](https://github.com/EmbarkStudios/texture-synthesis), [inpaint crate](https://codeberg.org/gillesvink/inpaint).
- Runtimes: [ONNX Runtime](https://github.com/microsoft/onnxruntime), [WebGPU EP doc](https://raw.githubusercontent.com/microsoft/onnxruntime/gh-pages/docs/tutorials/web/ep-webgpu.md), [transformers.js](https://github.com/huggingface/transformers.js), [candle](https://github.com/huggingface/candle), [tract](https://github.com/sonos/tract), [ort](https://github.com/pykeio/ort), [burn](https://github.com/tracel-ai/burn), [wonnx](https://github.com/webonnx/wonnx), [web-stable-diffusion](https://github.com/mlc-ai/web-stable-diffusion), [diffusers.js](https://github.com/dakenf/diffusers.js), [MDN compat data](https://raw.githubusercontent.com/mdn/browser-compat-data/main/api/GPU.json).
- Registries: crates.io API (`https://crates.io/api/v1/crates/<name>` for texture-synthesis, vtracer, tract-onnx, ort, ort-web, ort-tract, burn, candle-core, wonnx, kmeans_colors, okolors, inpaint) and npm (`https://registry.npmjs.org/<name>` for onnxruntime-web, @huggingface/transformers, @techstark/opencv-js).
- Data and tools: [Google Fonts](https://github.com/google/fonts), [SynthTIGER](https://github.com/clovaai/synthtiger), [SynthText](https://github.com/ankush-me/SynthText), [TRDG](https://github.com/Belval/TextRecognitionDataGenerator).
- Legibility: [WCAG 1.4.3](https://raw.githubusercontent.com/w3c/wcag/main/understanding/20/contrast-minimum.html), [WCAG 1.4.6](https://raw.githubusercontent.com/w3c/wcag/main/understanding/20/contrast-enhanced.html), [APCA introduction](https://raw.githubusercontent.com/Myndex/SAPC-APCA/master/documentation/APCAeasyIntro.md); local files `assets/fonts/*.ttf` and `docs/research/notes/print-requirements.prepress.md`.

## Could not verify

- Anything requiring a web search: whether any few-shot font generator or scene-text editor has been ported to ONNX/WebGPU; the state of the 2025–2026 literature beyond titles listed in fetched documents; whether Glyph-ByT5 weights were restored elsewhere.
- Hugging Face facts (host blocked): size of `migan_pipeline_v2.onnx`, of `Carve/LaMa-ONNX`, of the SD-Turbo web build; model cards and their licences.
- Model file sizes and parameter counts of FontDiffuser, DG-Font, MX-Font, SRNet, MOSTEL (weights on Google Drive or Baidu).
- Papers without a located repository: TextStyleBrush internals, Calliffusion (including its text encoder), DS-Font, VecFusion code availability, GAS-NeXt details.
- All [M] items in the tables: venues not stated in READMEs, reference counts of 8, the text encoders of DiffSTE / GlyphControl / TextDiffuser, FLUX text encoders and licence, community LaMa browser demos.
- Latency and memory of MI-GAN, LaMa and the classical ladder in a browser: nothing was measured.
- Quality of any method on real book covers: no cover test set was run.
- The PatchMatch patent claim; the FOGRA L* values used in the contrast arithmetic; the Legge and Bigelow threshold; GPU-hour and family-count estimates.
- Whether MI-GAN's MIT weights licence settles the status of its Places2 training data, and whether models trained on OFL fonts carry OFL obligations.
