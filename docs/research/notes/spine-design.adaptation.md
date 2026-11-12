# Spine design — adapting an existing spine to 10 mm, and synthesising one

Research notes, 2026-11-12. Angle: algorithms and parameter defaults for `ebc-spine`. Companion notes, not repeated here:
`print-requirements.folding.md` (spine rules of printers and POD platforms), `font-id-tech.generation.md` (background models,
legibility), `cover-sources.sources.md` (where spines can come from), the legal notes. Facts reused from them are tagged **[sib]**.

## 0. Method and limits — read first

- **No web search ran.** All five WebSearch calls were refused: the workflow's shared budget (200 calls) was already spent. I did
  not route around it through search engines, readers or archives. `arxiv.org` does not resolve from the sandbox.
- What I did instead: (a) read **primary documents** on the two hosts the sandbox allows (`raw.githubusercontent.com`, `crates.io`):
  31 files, all listed under Sources, each fetched successfully in this session; (b) read dependency sources already in the local
  cargo cache; (c) ran **four throwaway prototypes** (Python, PIL + numpy) on synthetic images and on the repo's bundled fonts.
- Consequence: the brief asked for 25+ searches and two independent sources per fact. That was not possible. Most sourced facts
  are single-source. Everything about commercial tools, academic papers and national conventions is from memory.
- Markers: **[V2]** two independent documents read this session · **[V1]** one · **[M]** memory, not verified · **[L]** measured or
  computed locally this session · **[calc]** arithmetic from stated inputs · **[sib]** sibling note · **[E]** my engineering
  default, not measured on real covers · no marker = design reasoning.
- No real cover was used. Every threshold marked [L] was checked on synthetic images only and needs a pass on about 200 real covers.

## 1. Key findings

1. **In a flat cover the spine is centred by construction** (front and back panels have the same width). Locating it is a
   one-dimensional problem: find its width. Matching the known front cover inside the wrap gave the width within 0.7 mm for
   spines of 3 to 30 mm on three synthetic styles [L].
2. **The front-cover match has one blind spot**: if the eBook cover is an *asymmetric* crop of the printed front, the error is
   twice the asymmetry (4.2 mm crop gave about 8 mm error) [L]. The result must be shown with two draggable guides.
3. **Geometry alone is not enough**: a 2.4 % difference between the eBook cover's aspect ratio and the print trim's moved the
   estimate by 6.7 mm [L].
4. **Adapt by scale, pad and crop; avoid warping.** A real spine fits unchanged only inside a narrow window (9.3–10.8 mm after
   scaling). Outside it, pad with the spine's own background (narrower) or crop background (wider). Seam carving adds little on a
   spine and has three costs: licence, no mask support in the MIT crate, and a deeper alteration of the artwork.
5. **"Not compatible" is common and cheap to detect**: scaled spine under 5 mm, content wider than 7 mm, text under 1.7 mm cap
   height, or under 150 ppi. Then re-typeset and keep only the colours.
6. **Background synthesis needs four classes and one pre-step.** Frame or padding detection must run before the edge strip is
   sampled; otherwise a white-framed cover is read as "busy" [L].
7. **Contrast limits are set by print, not by sRGB.** With printed black and paper white, 7:1 is reachable only on backgrounds
   of L\* 34 or less, or 73 or more, on coated stock; on uncoated stock dark text reaches 7:1 only on near-white (L\* 90+) [calc].
8. **Do not implement APCA.** The `apca-w3` licence limits use to web content and bans commercial use without a signed
   agreement [V1]. Use the WCAG 2 ratio plus a lightness-difference guard.
9. **Size text by the ink box of the actual string.** Accented capitals (É, Ç) take 1.0–1.1 em, not the 0.66–0.70 em cap
   height: the true all-caps maximum in a 5 mm band is 13–14 pt, not 24 pt [L].
10. **All four bundled fonts lack U+202F** (narrow no-break space, standard in French titles), and none covers Greek, Hebrew,
    Arabic or CJK; only Spectral covers Cyrillic [L]. Coverage must be checked before layout.
11. **Closest prior art: calibre 9.0's Bookshelf view** (2026-01-30), which synthesises spines from the cover's dominant colour,
    picks the text colour by contrast ratio and fits text between size multipliers [V1].

## 2. Where a real spine comes from, and how wide it is

Sources [sib]: ONIX List 158 has `29 FullCover` and `51 CoverFlap` (I re-read the code list: no entry mentions a spine)
[V1, [im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-158.yml)]; a self-publisher's
own wraparound PDF; a scan. No public source serves spines by ISBN [sib].

Width formulas:

| Rule | Formula | Conf. |
|---|---|---|
| Physics | sheet thickness (µm) = grammage (g/m²) × bulk (cm³/g; FR *main*, DE *Volumen*); spine = pages / 2 × thickness | [calc], unit identity |
| Pressbooks cover generator | spine (in) = pages / PPI; PPI = 2 / caliper (in); its example is 200 pages at 400 PPI = 0.5 in; hardcovers add the boards | [V1, [class-spine.php](https://raw.githubusercontent.com/pressbooks/pressbooks/dev/inc/covergenerator/class-spine.php)] |
| KDP | pages × 0.002252 in white, × 0.0025 cream, × 0.002347 premium colour | [sib, from [KDP help](https://kdp.amazon.com/en_US/help/topic/G201953020)] |
| Lulu | pages / 444 + 0.06 in | [sib, from [Lulu API help](https://help.api.lulu.com/en/support/solutions/articles/64000254616-how-is-spine-width-calculated-)] |
| IngramSpark, BoD | template generator only | [sib] |

1 / 0.002252 = 444 pages per inch: KDP white and Lulu use the same caliper; Lulu adds 1.5 mm for the cover [calc].

| Paper (bulk values from memory [M]) | Sheet | Page | Pages for 10 mm | Spine at 100 / 200 / 300 / 500 pages |
|---|---|---|---|---|
| Offset 80 g/m², bulk 1.25 | 100 µm | 0.050 mm | 200 | 5.0 / 10.0 / 15.0 / 25.0 mm |
| Offset 90 g/m², bulk 1.25 | 112 µm | 0.056 mm | 178 | 5.6 / 11.2 / 16.9 / 28.1 mm |
| KDP white (0.0572 mm/page [sib]) | 114 µm | 0.057 mm | 175 | 5.7 / 11.4 / 17.1 / 28.6 mm |
| Bouffant 80 g/m², *main* 1.8 | 144 µm | 0.072 mm | 139 | 7.2 / 14.4 / 21.6 / 36.0 mm |
| Bouffant 80 g/m², *main* 2.0 | 160 µm | 0.080 mm | 125 | 8.0 / 16.0 / 24.0 / 40.0 mm |
| Coated 115 g/m², bulk 0.85 | 98 µm | 0.049 mm | 205 | 4.9 / 9.8 / 14.7 / 24.4 mm |

All rows [calc]. Use the formula as a **prior window** (0.8 to 1.25 × the estimate) for the image-based detector, never as the
final value: paper is rarely known, and the same page count spans 10 to 16 mm between offset and bouffant.

## 3. Locating the spine in a flat wraparound image

### 3.1 Algorithm

1. **PDF input**: use TrimBox and BleedBox; with trim width `w` known, spine = trim width of the spread − 2w. No image analysis.
2. **Raster input, front cover known** (always true here, the eBook has one):
   - Downscale wrap and cover to a common height of 128 px; slide the cover over the right half of the wrap; scales 0.90–1.10 in
     1 % steps; score by normalised cross-correlation on grey. Refine at 512 px within ±8 coarse pixels and ±1 % scale.
   - The match gives `x_t`, `w_t` and the scale. Bleed = (wrap height − matched height) / 2. Right gap `g` = wrap width − bleed −
     (x_t + w_t). If g > 0 the eBook cover is a crop; assume it is symmetric: spine right edge = x_t − g.
   - Symmetry: spine width = 2 × (spine right edge) − wrap trim width.
3. **Cross-checks**, each inside the prior window of section 2:
   - *Band edges*: per-column median colour; the strongest pair of colour steps symmetric about the centre. Accept only if the
     weaker step exceeds 12 (summed 8-bit RGB difference across 2 px) **and** it agrees with step 2 within 1 mm.
   - *Rotated-text extent*: per column, sum |∂y| minus sum |∂x| over gradients above 24 grey levels, smoothed over 1 mm.
     Text rotated by 90° has more horizontal strokes, so the score is positive on spine text. The positive band is the text
     extent: a **lower bound** of the spine width and the content extent used in section 4.
4. **Confidence**: high if NCC ≥ 0.95, g < 0.5 mm and the cross-checks agree; otherwise show the guides and ask.
5. **Scans** (photo of an opened cover): two near-vertical lines (within ±3° of vertical, at least 70 % of the height,
   symmetric about the centre within 5 %, 3–60 mm apart). `imageproc` ships `hough.rs` and `gradients.rs` [L, crate source].
   Not prototyped. Scans need deskewing first; start with a manual crop.

### 3.2 Prototype result (synthetic wraps, 140 × 205 mm panels, 150 dpi, JPEG 85; eBook cover resampled to 800 px, JPEG 80)

| Estimator | Error over spines 3, 6, 10, 17, 30 mm × 3 styles | Reading |
|---|---|---|
| Geometry only, matching aspect | −0.11 mm | exact when the eBook cover has the trim's aspect |
| Geometry only, aspect off by 2.4 % | +6.66 mm | unusable alone |
| Template match, coarse-to-fine | −0.67 to +0.40 mm, NCC 0.986–0.999 | reliable; limited by the 512 px working height (0.4 mm per px) |
| Same, eBook cover cropped 1.5 % each side | identical | the right-gap correction works |
| Same, cropped 3 % on the left only | +7.3 to +8.4 mm | error = 2 × asymmetry; not detectable from the match |
| Band edges, spine a distinct colour band | 0.00 to ±0.17 mm | exact when a band exists |
| Band edges, continuous artwork | none, or false hits of −10.7 and −25.1 mm | must be gated by agreement |
| Rotated-text extent | −3.9 to −25.6 mm vs the spine | text extent only; meaningless at 3 mm (no text) |

Styles: distinct colour band; one photograph across the whole wrap; plain cream throughout. All [L]. Runtime in unoptimised
Python: 1.2 s per wrap; the search is a few million multiply-adds, so milliseconds in wasm [E].

Crates: `imageproc` 0.27.0 (MIT, 2026-06-02) has `match_template` with normalised methods, grey images only
[V1, [source](https://raw.githubusercontent.com/image-rs/imageproc/master/src/template_matching.rs),
[crates.io](https://crates.io/api/v1/crates/imageproc)]. Its default features pull `rayon`; disable them for wasm [L]. The 1-D
search above is about 40 lines and does not need the dependency.

## 4. Adapting a located spine to 10 mm

Let `k` = card height / book trim height, `w'` = k × real spine width, `c'` = k × content extent across the spine (1st to 99th
percentile of columns holding foreground, section 3.1), `t'` = k × cap height of the largest spine text.

| Order | Condition | Action | Why |
|---|---|---|---|
| 0 | effective resolution < 150 ppi at final size, or t' < 1.7 mm, or w' < 5 mm, or no text found | **incompatible**: synthesise, reuse the spine's colour | 1.7 mm is the cap height of 7 pt, the positive-text floor [sib]; below 5 mm spines carry no text [sib] |
| 1 | 9.3 ≤ w' ≤ 10.8 | scale across the spine only, by 10 / w' (at most ±8 %) | below the level at which type looks condensed or expanded [E, M] |
| 2 | 5 ≤ w' < 9.3 | scale by k, centre, pad both sides per row with the spine's edge colour; only if the spine's outer 0.5 mm columns are flat or a gradient | no pixel of the artwork is altered |
| 3 | w' > 10.8 and c' ≤ 7.0 | scale by k, crop 10 mm centred on the content | crop removes background only; keeps 1.5 mm to each fold [sib] |
| 4 | 7.0 < c' ≤ 8.2 | as 3, plus a squeeze across the spine of at most 15 % | last resort before re-typesetting |
| 5 | c' > 8.2 | **incompatible** | typical of spines over about 25 mm with the title set across or stacked |

Worked windows [calc]: with a poche card (178 mm high), rule 1 covers real spines of 9.3–10.9 mm for a poche (k = 1),
10.7–12.5 mm for a 140 × 205 broché (k = 0.87), 12.5–14.7 mm for a 155 × 240 (k = 0.74). So rule 1 is the exception;
rules 2, 3 and 5 are the normal cases.

Resolution [calc]: a wrap 1,000 px high printed at 178 mm is 143 ppi (reject); 1,400 px is 200 ppi (warn; type will be soft);
2,000 px is 285 ppi; 2,560 px is 365 ppi.

Bleed: crop the source to its trim, then rebuild 3 mm of bleed at head and foot from the source's own bleed if present,
otherwise by edge replication. The front panel is the real cover and the back is ours, so let the adapted spine's background
run 2 mm onto the back, or reuse it as the back colour: a fold drifts by about ±1 mm [sib].

**Seam carving — assessed, not recommended for launch.**

- Method: build an energy map, accumulate the minimum cost row by row over the three pixels above, remove the cheapest seam,
  repeat [V2, [caire](https://raw.githubusercontent.com/esimov/caire/master/README.md),
  [imageproc](https://raw.githubusercontent.com/image-rs/imageproc/master/src/seam_carving.rs)].
- Rust options: `seamcarving` 0.2.3 is **LGPL-3.0-or-later**, last published 2020-04-19
  [V1, [crates.io](https://crates.io/api/v1/crates/seamcarving)]: not allowed by the repo's licence rule. `imageproc`'s
  `shrink_width` is MIT and removes all seams in one pass, but uses luma differences and has **no protection mask and no
  forward energy** [V1]. caire (Go, MIT) has protect and remove masks and face detection [V1]: the mask is what a spine needs.
- Fit: when the content is centred, carving the margins equals a crop. It helps only when elements are staggered (logo offset
  from the title). If seams must cross letters, glyphs break.
- Law: it rearranges pixels inside the artwork; the legal notes rate cropping and alteration of a cover as high risk [sib].
  A US patent on seam carving (Avidan and Shamir, assigned to MERL, filed 2007) may still be in force [M, unverified].
- Cost if ever built in-house: about 150 lines; O(width × height) per seam [M]; a 260 × 2,100 px spine losing 140 seams is
  about 76 million pixel visits [calc].

**Re-flowing elements** (cut out title, author and logo blocks and re-pack them along the spine) is feasible with connected
components over a background model, but it is tracing by another name. Keep it for reseller mode with cleared rights, off by default.

## 5. Synthesising a spine when none exists

### 5.1 Pipeline and thresholds

| Step | Rule | Default | Conf. |
|---|---|---|---|
| S1 Normalise | Uniform border test per side: columns or rows within ΔE\*ab 6 of the corner colour for ≥ 98 % of their pixels. Bars on two opposite sides only = padding: crop. All four sides, each 1–12 % = design frame: keep and sample **inside the frame band** | tol 6, 98 %, 1–12 % | [L]: frame found at 3.4/3.4/2.1/2.1 %; side bars at 7.4/7.4/0/0 % |
| S2 Binding edge | Left edge of the front for left-to-right books; right edge for Arabic, Hebrew, Persian, Urdu and vertical-text Japanese or Chinese books | by book language | [M] |
| S3 Strip | Skip the outer 1 % (min 2 px), take the next 3 % of the width (min 8 px), full height; work in CIELAB | 1 % + 3 % | [E]; `ebc-color::edge_palette` takes the fraction |
| S4 Statistics | σ = RMS ΔE\*ab around the mean (the `ebc-color` definition). Low-pass the per-row median over height / 12. `resid` = RMS ΔE to that profile; `span` = its colour range; `wiggle` = its total variation / span; `edge` = share of vertical L\* steps > 8 | — | [L] |
| S5 Class | **flat**: σ ≤ 3, or span ≤ 3 and resid ≤ 6 · **gradient**: resid ≤ 4 and wiggle ≤ 1.5 · **texture**: resid ≤ 10, edge ≤ 3 %, span ≤ 10 · **busy**: otherwise | as stated | [L], table below; σ ≤ 3 matches `UNIFORM_EDGE_THRESHOLD` 0.9 |
| S6 Background | flat: strip median · gradient: the low-pass profile, up to 64 stops, as a shading · texture: its mean, no grain in print · busy: among the cover's five largest swatches, the one nearest the strip mean; if a known title colour gives < 4.5:1 on it, the next one | — | ladder from [sib], thresholds [E] |
| S6b Blurred mirror (opt-in) | Mirror a strip as wide as the spine; blur σ 2.5 mm across and 1 mm along; pull 35 % toward its mean. Only when edge ≤ 2 % and no text lies within 6 % of the edge | off | [E], not prototyped |
| S7 Text colour | Candidates in order: measured cover-title colour; lightest and darkest palette swatches; K100; paper white. Take the first with ratio ≥ 7.0; else the best ≥ 4.5; and \|ΔL\*\| ≥ 40. On a gradient test the 5th and 95th percentile rows under the text | 7.0 / 4.5 / 40 | 5.2 |
| S8 Rescue | No candidate passes: move the background L\* out of the dead zone by the smaller move, at most 25 L\*, keeping hue and chroma; if still failing, use the darkest or lightest palette swatch as a flat background | 25 L\* | [E] |
| S9 Type | Top open-font match from the identification stage [sib]; else by class: serif → Spectral SemiBold, sans → Atkinson Hyperlegible Bold, condensed caps → Bebas Neue. Reversed text: weight ≥ 600 and ≥ 8 pt [sib] | — | fonts are in `assets/fonts` |
| S10 Layout | Text band 6 mm (2 mm clear each side) [sib]; ink box ≤ 5.0 mm. Head and foot margins 8 mm (≥ `safe_margin` 4–5 mm of `ebc-layout`). Optional author at the head, ≤ 35 % of the live length, 0.7 × title size, gap ≥ 6 mm. Keep 12 mm free at the foot | — | [E]; author–title–publisher order is the French habit [M] |
| S11 Direction | By the **book's** language, not the UI locale. Bottom-to-top for French and most continental languages, top-to-bottom for English [M, sib]. CJK: upright glyphs, top-to-bottom | — | not verified, see below |
| S12 Marks | No publisher logo unless the reseller supplies and licenses it. No fake shading | — | trademark caution [sib] |

Classifier check on synthetic covers (800 × 1,295 px, JPEG 80) [L]:

| Cover | L\* | σ | resid | span | wiggle | Class |
|---|---|---|---|---|---|---|
| Flat cream | 97 | 0.0 | 0.0 | 0.0 | 0.0 | flat |
| Vertical gradient | 53 | 25.9 | 0.7 | 86.5 | 1.03 | gradient |
| Photo, low frequency | 42 | 25.2 | 4.1 | 81.5 | 2.79 | busy |
| Photo, detailed | 50 | 37.7 | 6.0 | 127.7 | 2.48 | busy |
| Photo, heavily blurred | 51 | 35.7 | 2.6 | 114.4 | 2.42 | busy |
| Flat colour + grain | 27 | 3.5 | 3.5 | 0.8 | 12.0 | flat |
| White frame around a photo, strip not corrected | 91 | 23.8 | 25.6 | 0.3 | 2.11 | busy (wrong: hence S1) |
| Title touching the binding edge | 20 | 32.3 | 19.1 | 130.3 | 2.00 | busy |

Without the `wiggle` test all three photographs were classed "gradient" and would have produced a streaked spine [L].

### 5.2 Contrast

- WCAG 2: L = 0.2126 R + 0.7152 G + 0.0722 B on linearised sRGB, threshold 0.04045 (0.03928 before May 2021, "no practical
  effect"), exponent 2.4 [V1, [WCAG term](https://raw.githubusercontent.com/w3c/wcag/main/guidelines/terms/20/relative-luminance.html)].
  Ratio = (L1 + 0.05) / (L2 + 0.05).
- 4.5:1 for body text and 3:1 for large or title text are the working thresholds in WCAG [sib] and in AndroidX Palette
  (`MIN_CONTRAST_BODY_TEXT = 4.5f`, `MIN_CONTRAST_TITLE_TEXT = 3.0f`)
  [V2, [Palette.java](https://raw.githubusercontent.com/androidx/androidx/androidx-main/palette/palette/src/main/java/androidx/palette/graphics/Palette.java)].
  Spine type is under 14 pt, so 4.5 is the floor and 7 the target [sib].
- **APCA is licensed for web content only**: "for web-delivered and web-based content only, and not for any other use";
  "Commercial use is prohibited without a written and signed commercial license agreement"; the name may be used only for
  compliant, up-to-date code [V1, [LICENSE](https://raw.githubusercontent.com/Myndex/apca-w3/master/LICENSE.md)]. Its guidance
  is still a useful reading aid: Lc 75 minimum for body text, Lc 60 for other content text, Lc 90 preferred
  [V1, [intro](https://raw.githubusercontent.com/Myndex/SAPC-APCA/master/documentation/APCAeasyIntro.md)]. Do not ship its code,
  constants or name. The |ΔL\*| ≥ 40 guard covers the known WCAG weakness on dark pairs, which the same document describes
  ("can be functionally unreadable when a color is near black").
- Limits by stock [calc; the L\* values of paper and K100 are from memory, as in the sibling note]:

| Condition | Best possible | Light text needs background L\* | Dark text needs background L\* | Dead zone for 7:1 |
|---|---|---|---|---|
| Ideal sRGB (white 100, black 0) | 21:1 | ≤ 50 (4.5) · ≤ 38 (7) | ≥ 49 (4.5) · ≥ 62 (7) | 38–62 |
| Coated print (paper 95, K100 16) | 13.0:1 | ≤ 46 · ≤ 34 | ≥ 59 · ≥ 73 | 34–73 |
| Uncoated print (paper 93, K100 31) | 7.5:1 | ≤ 45 · ≤ 33 | ≥ 74 · ≥ 90 | 33–90 |

  So thresholds must come from the print profile. On uncoated home prints target 4.5:1; 7:1 exists only for black on near-white.
- Viewing distance [calc]: 10 pt at 40 cm subtends what 23.7 CSS px do at the reference distance; at 1 m, 9.5 px. A shelf at
  1 m is below any body-text guidance: only maximum contrast and caps of 12 pt or more read there.
- When both polarities pass, prefer **dark on light**: reversed type under 8 pt is a registration risk [sib]. AndroidX does the
  opposite (white first), which suits screens, not print [V1].

### 5.3 Fitting the title (measured on the bundled fonts) [L]

| Font | d-to-p extent | Max size, 5.0 mm ink | Caps with É and Ç | Two-line max (2 × 2.6 mm) |
|---|---|---|---|---|
| Spectral Regular / SemiBold | 0.980 em | 14.5 pt | 1.102 em → 12.9 pt | 7.5 pt |
| Atkinson Hyperlegible Bold | 0.829 em | 17.1 pt | 1.072 em → 13.2 pt | 8.9 pt |
| Bebas Neue (caps) | 0.700 em | 20.2 pt | 1.009 em → 14.0 pt | 10.5 pt |

Fit ladder: (1) one line, largest size ≤ band maximum, preferred 11 pt; (2) if under 9 pt, drop the author; (3) if still under
9 pt, cut at the first subtitle delimiter (` : `, ` — `, ` - `, `, ` after 20 characters, `(`); (4) switch to the condensed
caps face; (5) two lines, each ≥ 7 pt, never splitting a word; (6) ellipsis at a word boundary, keeping ≥ 60 % of the
characters; (7) below 7 pt (8 pt reversed): report failure. Never drop words silently.

| Title (characters) | Width at 10 pt, Spectral | Largest one-line size in 162 mm: Spectral / Atkinson Bold / Bebas | With a 35 % author zone (100 mm): Spectral |
|---|---|---|---|
| Stoner (6) | 10.2 mm | 14.5 / 17.1 / 20.2 pt (band-limited) | 14.5 pt |
| Le Comte de Monte-Cristo (24) | 42.1 mm | 14.5 / 17.1 / 20.2 pt | 14.5 pt |
| Le Vieux qui ne voulait pas fêter son anniversaire (50) | 76.8 mm | 14.5 / 17.1 / 20.2 pt | 13.0 pt |
| L'Extraordinaire Voyage du fakir… armoire Ikea (77) | 119.8 mm | 13.5 / 12.6 / 17.9 pt | 8.3 pt → drop the author |
| La Vie et les aventures surprenantes de Robinson Crusoé, de York… (127) | 195.9 mm | 8.3 / 7.6 / 10.8 pt | 5.1 pt → cut at the comma (56 characters) |

Live length 162 mm = 178 − 2 × 8. A 77-character title still fits on one line above 12 pt; truncation is a rare path.

### 5.4 Scripts and shaping

- Coverage of the four bundled fonts [L, cmap tables]: French, Polish, Romanian, Turkish, German: complete (Atkinson Bold lacks
  capital ẞ). **U+202F missing in all four** → map to U+00A0 before shaping. Vietnamese: Spectral only. Cyrillic: Spectral
  only. Greek, Hebrew, Arabic, CJK: none. Rule: if any character has no glyph in the chosen font, fall back per script; if no
  font covers it, return a typed error. Never print a `.notdef` box.
- Vertical CJK: `rustybuzz` 0.20.1 accepts `Direction::TopToBottom` and applies only the `vert` feature [L, crate source].
  `unicode-vo` 0.1.0 (MIT/Apache-2.0) gives the per-character orientation [V1, [crates.io](https://crates.io/api/v1/crates/unicode-vo)].
  CSS Writing Modes states the default: in vertical text, characters of horizontal-only scripts are set sideways, 90° clockwise;
  `sideways-lr` exists for text rotated the other way [V1, [spec source](https://raw.githubusercontent.com/w3c/csswg-drafts/main/css-writing-modes-4/Overview.bs)].
- Dependency health: `rustybuzz` is archived, "not developed further", and points to HarfRust
  [V1, [README](https://raw.githubusercontent.com/harfbuzz/rustybuzz/main/README.md)]; `harfrust` 0.15.0 (MIT) was published
  2026-10-22 [V1, [crates.io](https://crates.io/api/v1/crates/harfrust)]. `ebc-text` depends on `rustybuzz` 0.20 [L].

## 6. Prior art

| Tool | What it does for spines | Parameters worth copying | Conf. |
|---|---|---|---|
| **calibre 9.0.0** Bookshelf view (2026-01-30: "books arranged on shelves showing their spines") | Synthesises a spine per book. Colour = dominant colour of the cover thumbnail. Text colour = whichever of two theme colours has the higher contrast ratio. Optional cover crop overlaid at 30 % opacity | Width 15–80 px on a 200 px spine, default 40, from pages through log₃(1 + 2f), 1,500 pages = maximum. Font between 0.75 × and 1.3 × base, shrink in 1 pt steps, wrap to two lines, then elide right. Default `bookshelf_up_to_down = False`, i.e. −90° rotation, bottom-to-top; a preference, not locale-driven | [V1, [view](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/gui2/library/bookshelf_view.py), [defaults](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/gui2/__init__.py), [changelog](https://raw.githubusercontent.com/kovidgoyal/calibre/master/Changelog.txt)]; GPLv3: read for facts only |
| calibre `dominant_color` | Downscale to ≤ 100 px; 32 levels per channel; most frequent bucket; if its saturation < 0.2, take one of the top five with saturation > 0.3 and > 10 % of pixels | a cheap deterministic "dominant colour" | [V1, [imageops.cpp](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/utils/imageops/imageops.cpp)] |
| calibre cover generator | Front covers only. Fixed sizes (120/80/80 px on 1,200 × 1,600), four fixed colour themes, random style, no contrast computation, no use of an existing image; lines that do not fit are not drawn and no ellipsis is added | a counter-example: silent loss of words | [V1, [covers.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/ebooks/covers.py)] |
| KOReader `FakeCover` | Text-only cover when none exists; no spine rendering in the file | Title 24 → 10, authors 20 → 6, step 2; accept when no word is split and > 20 % free height; second pass after cleaning the string | [V1, [mosaicmenu.lua](https://raw.githubusercontent.com/koreader/koreader/master/plugins/coverbrowser.koplugin/mosaicmenu.lua)] |
| Harvard Stack View (2012, MIT / GPL) | Library shelf of spines from metadata: thickness = pages clamped to 200–540 × 0.20 px; length = height clamped to 20–39 cm mapped to 59–100 %; colour = "heat" 1–10 from a usage rank; title, first author and year on the spine | clamping physical sizes to a readable range | [V1, [README](https://raw.githubusercontent.com/harvard-lil/stackview/master/README.md), [book.js](https://raw.githubusercontent.com/harvard-lil/stackview/master/src/js/types/book.js)] |
| virtual-bookshelf (CSS) | Spines built from markup, with title and author initials; height, colours and patterns randomised | — | [V1, [README](https://raw.githubusercontent.com/petargyurov/virtual-bookshelf/master/README.md)] |
| 3D book CSS generator (MIT) | 3D book from a cover URL with perspective and thickness; the README does not describe a spine | — | [V1, [README](https://raw.githubusercontent.com/scastiel/3d-book-image-css-generator/master/README.md)] |
| AndroidX Palette | Quantise to 16 colours on a bitmap of at most 112 × 112 px area; drop near-black, near-white and one hue range; dominant = largest population; six luma/saturation targets (dark ≤ 0.45, light ≥ 0.55) | swatch choice and text-on-swatch rule | [V1, [Palette.java](https://raw.githubusercontent.com/androidx/androidx/androidx-main/palette/palette/src/main/java/androidx/palette/graphics/Palette.java), [Target.java](https://raw.githubusercontent.com/androidx/androidx/androidx-main/palette/palette/src/main/java/androidx/palette/graphics/Target.java)] |
| Color Thief (MIT), Material Color Utilities | Dominant colour and palette (`quality` 10, `colorCount` 10, `ignoreWhite` true by default); quantiser "Celebi, which runs Wu, then WSMeans" | `ignoreWhite` is wrong for cream French covers | [V1, [color-thief](https://raw.githubusercontent.com/lokesh/color-thief/master/README.md), [MCU](https://raw.githubusercontent.com/material-foundation/material-color-utilities/main/README.md)] |
| Open Library | The coverstore README describes cover size variants only; nothing about spines | — | [V1, weak, [README](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/README.md)] |
| Kindle, Kobo, Apple Books, Google Play Books | Face-out covers; I know of no mainstream e-reader shelf that synthesises spines | — | [M] |
| Retail 3D packshots, mockup sites | Front cover with a generic or stretched spine | — | [M] |
| Academic spine work | Shelf photographs: segmentation by long near-vertical lines, then text reading or image matching (Stanford mobile book-spine recognition, about 2010–2011; Talker and Moses, "Viewpoint-independent book spine segmentation", 2014; "Smart Library", 2016–2017). They solve shelf photos, not flat wraps; useful only for the scan input | — | [M], nothing verified |

Three lessons. calibre shows the simplest scheme that works: one flat dominant colour and a two-way text colour. Its shading
(edges darkened 115 %, top highlight) imitates a round spine on screen and should not be printed on a real one. Nobody in this
list samples the binding edge or adapts a real spine: both parts of this note are new work.

## 7. Test cases

Fixtures must be synthetic (generated at build time), so that tests stay offline and no cover is redistributed.

| # | Input | Expected |
|---|---|---|
| T1 | Title "Stoner" | one line, size capped by the band (14.5 pt Spectral), centred in its zone |
| T2 | 77-character title + author | author dropped, title one line ≥ 12 pt |
| T3 | 127-character title | cut at the comma; without a delimiter, 8.3 pt Spectral; never under 7 pt; ellipsis only as the last step |
| T4 | Title with U+202F, U+00A0, `œ`, `«»` | no `.notdef`; U+202F normalised |
| T5 | Cyrillic title, sans font chosen | falls back to Spectral; note recorded |
| T6 | Greek, Arabic, Hebrew, CJK title, no extra font loaded | typed error or on-demand font; never tofu |
| T7 | Japanese title "1Q84"-style mix | kana and kanji upright, Latin run sideways, top-to-bottom |
| T8 | All-caps "ÉCOLE DES FEMMES, ÇA" | ink box ≤ 5.0 mm including accents and cedilla |
| T9 | Light cover (L\* 97) | flat background, dark K100 text; ratio ≥ 7 on coated, ≥ 4.5 on uncoated |
| T10 | Mid-tone flat cover (L\* 50) | background moved out of the dead zone (≤ 34 or ≥ 73); move ≤ 25 L\* |
| T11 | Busy photographic cover | class busy; flat palette colour; no streaks, no mirrored objects |
| T12 | Photo inside a white frame (3.5 %) | frame detected; white spine, dark text |
| T13 | Cover with white bars on two sides only | bars cropped before sampling |
| T14 | Title touching the binding edge | class busy; palette colour; no letter fragments |
| T15 | Sky gradient, L\* 30 → 80 | gradient kept only if one text colour passes at both ends; else flat |
| T16 | Real wrap, 17 mm spine, centred content 6 mm | rule 3: crop, no scaling across |
| T17 | Real wrap, 3 mm spine | incompatible; synthesised with the spine colour |
| T18 | Real wrap, 35 mm spine, stacked horizontal title | incompatible (c' > 8.2) |
| T19 | Wrap + eBook cover cropped on one side | low confidence; guides shown; never silently accepted |
| T20 | Wrap 1,000 px high | rejected for raster reuse (143 ppi) |
| T21 | Fold shifted by ±1 mm | no hard colour edge visible on the front |
| T22 | Hostile: 1 × 1 px, transparent PNG, 20,000 × 20 px, empty title, 10,000 characters, U+202E | no panic; typed error or safe default |
| T23 | Same input twice | byte-identical output |

## 8. Open questions

- Real-cover statistics: share of covers per background class; how often the eBook cover is a crop of the print front.
- Whether a publisher's eBook cover and print front are the same artwork often enough for the template match (series redesigns).
- Spine direction per country, including Dutch, Scandinavian, Arabic and Hebrew practice, and ISO 6357.
- The ±8 % stretch limit and the 60 % truncation rule need a visual test with readers.

## Sources

All fetched successfully in this session.

- calibre: [bookshelf_view.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/gui2/library/bookshelf_view.py), [gui2/__init__.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/gui2/__init__.py), [imageops.cpp](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/utils/imageops/imageops.cpp), [covers.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/ebooks/covers.py), [Changelog.txt](https://raw.githubusercontent.com/kovidgoyal/calibre/master/Changelog.txt)
- KOReader: [mosaicmenu.lua](https://raw.githubusercontent.com/koreader/koreader/master/plugins/coverbrowser.koplugin/mosaicmenu.lua)
- Shelf and mockup tools: [Stack View README](https://raw.githubusercontent.com/harvard-lil/stackview/master/README.md), [Stack View book.js](https://raw.githubusercontent.com/harvard-lil/stackview/master/src/js/types/book.js), [virtual-bookshelf](https://raw.githubusercontent.com/petargyurov/virtual-bookshelf/master/README.md), [3d-book-image-css-generator](https://raw.githubusercontent.com/scastiel/3d-book-image-css-generator/master/README.md), [Open Library coverstore](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/README.md)
- Colour: [Palette.java](https://raw.githubusercontent.com/androidx/androidx/androidx-main/palette/palette/src/main/java/androidx/palette/graphics/Palette.java), [Target.java](https://raw.githubusercontent.com/androidx/androidx/androidx-main/palette/palette/src/main/java/androidx/palette/graphics/Target.java), [color-thief](https://raw.githubusercontent.com/lokesh/color-thief/master/README.md), [material-color-utilities](https://raw.githubusercontent.com/material-foundation/material-color-utilities/main/README.md)
- Contrast: [WCAG relative luminance](https://raw.githubusercontent.com/w3c/wcag/main/guidelines/terms/20/relative-luminance.html), [apca-w3 README](https://raw.githubusercontent.com/Myndex/apca-w3/master/README.md), [apca-w3 LICENSE](https://raw.githubusercontent.com/Myndex/apca-w3/master/LICENSE.md), [APCA introduction](https://raw.githubusercontent.com/Myndex/SAPC-APCA/master/documentation/APCAeasyIntro.md)
- Seam carving and image processing: [caire](https://raw.githubusercontent.com/esimov/caire/master/README.md), [seamcarving README](https://raw.githubusercontent.com/lovasoa/seamcarving/master/README.md), [seamcarving on crates.io](https://crates.io/api/v1/crates/seamcarving), [imageproc seam_carving.rs](https://raw.githubusercontent.com/image-rs/imageproc/master/src/seam_carving.rs), [imageproc template_matching.rs](https://raw.githubusercontent.com/image-rs/imageproc/master/src/template_matching.rs), [imageproc on crates.io](https://crates.io/api/v1/crates/imageproc)
- Text: [rustybuzz README](https://raw.githubusercontent.com/harfbuzz/rustybuzz/main/README.md), [harfrust on crates.io](https://crates.io/api/v1/crates/harfrust), [unicode-vo on crates.io](https://crates.io/api/v1/crates/unicode-vo), [CSS Writing Modes 4 source](https://raw.githubusercontent.com/w3c/csswg-drafts/main/css-writing-modes-4/Overview.bs)
- Spine width and metadata: [Pressbooks class-spine.php](https://raw.githubusercontent.com/pressbooks/pressbooks/dev/inc/covergenerator/class-spine.php), [ONIX List 158 (im_onix)](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-158.yml)
- Taken from the sibling note `print-requirements.folding.md`, not re-read by me: [KDP help](https://kdp.amazon.com/en_US/help/topic/G201953020), [Lulu API help](https://help.api.lulu.com/en/support/solutions/articles/64000254616-how-is-spine-width-calculated-)
- Local: `assets/fonts/*.ttf` (metrics and cmap), `crates/ebc-color/src/palette.rs`, `crates/ebc-layout/src/profile.rs`, `docs/architecture.md`, cargo cache sources of `rustybuzz` 0.20.1, `imageproc` 0.27.0, `unicode-vo` 0.1.0. The four prototype scripts were throwaway files outside the repository.

## Could not verify

- Everything that needed a web search (none ran): KDP, Lulu and IngramSpark calculators (taken from the sibling note); bulk values
  of French offset and bouffant papers; KDP's paper weight.
- Spine reading direction by country and ISO 6357; binding side and spine practice for Arabic, Hebrew and vertical CJK books.
- The academic literature on spine detection and any dataset (names, sizes, licences); `arxiv.org` is blocked.
- LibraryThing and Goodreads shelf views, Open Library spine experiments, e-reader shelves that draw spines, commercial 3D mockup
  generators: no document read. The absence claims in section 6 are from memory.
- The seam-carving patent (owner, number, expiry, European counterparts).
- The typographic tolerance behind the ±8 % stretch limit.
- L\* of paper white and K100 on coated and uncoated stock (inputs of the contrast table).
- All thresholds marked [L] or [E] on real covers; the scan path (Hough) and the blurred mirror were not prototyped.
- How calibre maps `bookshelf_up_to_down` to the spine title's `downwards` argument: I read the call that passes the preference
  and the −90° branch, in separate excerpts of a long file.
