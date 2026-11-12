# Spine design — conventions, adapting a real spine to 10 mm, and synthesising one

_Status: synthesised from researcher notes, not independently fact-checked — research date 2026-11-12_

**Scope.** What goes on the 10 mm spine of the gift card: text direction by country, elements and their order, type sizes, long titles, French typography, legibility of rotated text; how to locate and adapt a real spine taken from a flat wraparound cover; how to synthesise a spine (background, text colour, fitting) when none exists. Not covered: the legal limits on reusing cover pixels (`07-legal-covers-and-reseller-rights.md` and `08-legal-fonts-and-genai.md` govern; they are applied here), choosing the title font (`10-font-identification-tech.md`), printer tolerances (`03-print-requirements.md`).

**Inputs.** `notes/spine-design.conventions.md` (conventions; written entirely from memory) and `notes/spine-design.adaptation.md` (algorithms and thresholds; 31 primary files read on `raw.githubusercontent.com` and `crates.io`, five throwaway prototypes on synthetic images). Facts the two notes take from siblings (`print-requirements.*`, `font-id-tech.generation.md`) are marked [sib]. The code description at the end comes from `crates/ebc-spine` and `crates/ebc-i18n` (spine direction), read on 2026-11-12.

**Evidence and its limits — read first.**

- **No web search ran** for either note. The conventions note cites no URL on purpose; every country rule, standard and publisher habit in §2–§5 is recalled, not verified. The adaptation note's sourced facts are mostly single-source primary files.
- **No real cover was used.** Every threshold marked [L] was checked on synthetic images only; [E] values are engineering defaults.
- Markers: [V2] two independent documents read, [V1] one, [M-hi] / [M-med] / [M-lo] memory by confidence, [M] memory, [L] measured or computed locally, [calc] arithmetic, [E] engineering default, [sib] taken from a sibling note.
- Later evidence elsewhere in this folder, applied here: the fact-check of doc 02 found that ONIX List 158 has spine codes 56 and 57 (the adaptation note read a stale copy that stopped at 55); the editor of doc 02 saw ISO 6357:1985 listed by ISO in a search result (title and existence only); doc 08's corrections fix the text band at 6 mm and outlined text at 8 pt or more.

---

## 1. Executive summary

| Question | Answer | Confidence |
|---|---|---|
| Which way does the title run? | **Bottom-to-top** (ascending) for French, German, Italian, Spanish and Portuguese books; **top-to-bottom** (descending; ISO 6357, ANSI/NISO Z39.41) for English, Dutch and Scandinavian books. Chosen from the **book** (language, country, ISBN group), never from the site locale; a user toggle always | rule [M-hi] for FR and EN; others [M-med]; German and Spanish practice is a majority, not uniform |
| What is on it? | Author then title on one line, centred across the spine; no collection number, no publisher logo; subtitle omitted | [M-med] |
| How big? | One line in the 6 mm text band; ink box ≤ 5.0 mm; typical 9–12 pt mixed case, up to 13–14 pt for short all-caps titles (accented capitals limit it); floor 7 pt positive, 8 pt reversed or outlined | [L] on the bundled fonts; floors [sib] |
| Long titles? | Fit ladder: drop the author below 9 pt, cut at the subtitle delimiter, condensed caps face, two lines (≥ 7 pt), then an ellipsis at a word boundary keeping ≥ 60 % of the characters. A 77-character title still fits on one line above 12 pt | [L] |
| Can a real spine be reused? | Only inside narrow windows: stretch across by ≤ 8 %, pad, or crop background; otherwise "incompatible" and re-typeset, keeping only colours. Rejected below 150 ppi or below 1.7 mm cap height | [L]/[E] |
| How is a spine found in a flat wrap? | It is centred by construction: match the known front cover inside the wrap; width within 0.7 mm for spines of 3–30 mm on synthetic wraps; error = 2 × any asymmetric crop of the eBook cover, so show draggable guides | [L] |
| What background? | Classical ladder on the strip next to the binding edge: flat → edge colour; smooth gradient → gradient; texture → its mean; busy → a palette swatch. A frame/padding detector must run first. Never mirror, stretch or outpaint by default | [L] classes; rule from doc 08 |
| Text colour? | WCAG 2 ratio with a polarity rule: paper-white text on dark and mid-tones (L* ≤ 46), black only on light grounds (L* ≥ 70); floor 4.5:1, target 7:1, **computed on printed colours**; move the background's lightness when nothing passes | [calc] on recalled paper and ink L* |
| Prior art? | calibre 9.0's Bookshelf view (2026-01-30) synthesises screen spines from the dominant cover colour with a contrast-picked text colour; nothing found samples the binding edge or adapts a real spine | [V1] |

Findings that shape the code:

1. **Direction belongs to the book.** A French novel bought from a Dutch shop still gets an ascending spine [calc].
2. **French poches and literary collections do not continue the cover image on the spine**: pocket collections use a series template, typographic literary collections the same flat ground as the cover [M-med]. A flat colour taken from the binding edge is therefore both the most plausible look and the legally clean option (doc 08 R9).
3. **Size by the ink of the actual string**: accented capitals (É, Ç) reach 1.0–1.1 em, not the 0.66–0.70 em cap height, so an all-caps title tops out at 13–14 pt in a 5 mm ink box, not 24 pt [L].
4. **Contrast limits come from print, not sRGB.** On coated stock 7:1 needs a background of L* ≤ 34 (light text) or ≥ 73 (dark text); between L* 46 and 59 neither black nor white reaches even 4.5:1 [calc].
5. **Do not ship APCA**: its licence limits use to web content and forbids commercial use without a signed agreement [V1]. Use WCAG 2 with the polarity rule.
6. **The bundled fonts lack U+202F** (narrow no-break space, standard in French titles); only Spectral covers Cyrillic; none covers Greek, Hebrew, Arabic or CJK [L]. Coverage must be checked before layout.

---

## 2. Conventions

### 2.1 Vocabulary

| English | French | German | Italian | Spanish | Dutch |
|---|---|---|---|---|---|
| spine | **dos** (popularly *tranche*) | Buchrücken | dorso, costa | lomo | rug |
| spine title | titre au dos | Rückentitel | titolo sul dorso | título de lomo | rugtitel |
| along / across the spine | titre en long / en travers | Längstitel / Quertitel | — | — | — |
| head / foot | tête / queue (pied) | Kopf / Fuß | testa / piede | cabeza / pie | kop / staart |
| front / back cover | première / quatrième de couverture | Vorderdeckel / Rückseite | prima / quarta di copertina | cubierta / contracubierta | voorplat / achterplat |

All [M-hi] except secondary Italian, Spanish and Dutch terms [M-med]. French UI copy should say "dos"; doc 06 covers user-facing wording.

### 2.2 Standards

| Standard | What it says (recalled) | Conf. |
|---|---|---|
| ISO 6357:1985, *Spine titles on books and other publications* | Defines transverse, descending and ascending titles; a transverse title when the spine is wide enough, otherwise **descending** (reads normally with the book lying front cover up). Existence and title seen in a search result by doc 02's editor | rule [M-hi]; "transverse preferred" [M-med] |
| ISO 6357, other provisions | Edge title when the spine is too narrow; a zone at the foot (~30 mm) left for the library mark | [M-lo] |
| ANSI/NISO Z39.41-1997, *Printed Information on Spines* | US counterpart, descending; author, title, publisher as elements | [M-hi]; status [M-med] |
| BS 6738:1986; UNE 50-120-92 | British and Spanish adoptions of ISO 6357 (so the Spanish norm says descending although practice does not) | [M-lo] |
| DIN 1429 | German standard on title pages and binding lettering; direction rule not recalled | [M-lo] |
| France | No standard recalled; tradition | [M-lo] |

Pillar lettering (upright letters stacked) is rejected by typographic manuals for Latin script [M-med].

### 2.3 Direction by country

| Area | Practice | Conf. |
|---|---|---|
| UK, Ireland, USA, Canada (English), Australia, NZ | Descending (early Penguins ran ascending until the late 1940s [M-lo]) | [M-hi] |
| Netherlands, Flanders | Descending | [M-med] |
| Denmark, Norway, Sweden | Descending; Finland unknown | [M-med] |
| France, Wallonia, French-speaking Switzerland | **Ascending**, near-universal in trade publishing; isolated exceptions (co-editions, some art, technical and comics publishers) | rule [M-hi]; exceptions [M-lo] |
| Quebec | Ascending, expect a mix | [M-med] |
| Germany, Austria, German-speaking Switzerland | Ascending in the majority, **not uniform** (science publishers, co-editions print descending); no share known | [M-med] |
| Italy | Ascending | [M-med] |
| Spain | Ascending traditionally, mixed in practice | [M-lo] |
| Portugal, Brazil, Spanish-speaking Latin America | Ascending | [M-med] / [M-lo] |
| Central and Eastern Europe | Unknown | — |

**Decision in software** [calc]: (1) a real spine supplied with a flat cover keeps its own direction; (2) explicit user or reseller setting; (3) ONIX `CountryOfPublication` or the publisher's country; (4) the book's language; (5) ISBN registration group (below); (6) ISO default = descending.

| ISBN prefix | Group | Default |
|---|---|---|
| 978-0, 978-1; 979-8 | English; USA | descending |
| 978-2; 979-10 | French; France | ascending |
| 978-3 | German | ascending |
| 978-84 | Spain | ascending |
| 978-88; 979-12 | Italy | ascending |
| 978-90, 978-94 | Netherlands and Flanders | descending |
| 978-87 / 82 / 91 | Denmark / Norway / Sweden | descending |
| 978-972, 989 / 85, 65 | Portugal / Brazil | ascending |

Group identifiers [M-hi]; the direction inherits the table above. Group 2 includes French-language publishers in Belgium, Switzerland and Canada, so the proxy follows language, not borders.

**Geometry, to avoid a 180° bug** [calc]: the flat outside spread is back (left) | spine | front (right). Ascending: rotate 90° counter-clockwise, reads tail to head, letter tops towards the back cover, baseline on the front-cover side. Descending: rotate 90° clockwise, letter tops towards the front cover. Test: lay the folded card front-up, spine towards you — a descending title reads upright, an ascending one upside down.

### 2.4 Publishers and trade dress (understanding, not imitation)

All [M-med] or [M-lo]; directions follow the country rule in every recalled case.

| Collection | Spine as recalled |
|---|---|
| Gallimard Folio (1972, Massin) | White ground; number at the head; author and title along; logo at the foot. A pure series template |
| Gallimard Blanche; Minuit; P.O.L; Actes Sud/Babel | Same flat ground as the cover, typographic |
| Livre de Poche, Pocket, 10/18, Points | Numbers, logos, colours coded by series or genre |
| Bibliothèque de la Pléiade | Transverse gilt lettering on leather coloured by century |
| edition suhrkamp (Fleckhaus) | One flat colour per volume from a 48-colour spectrum |
| dtv, Reclam, Einaudi, Adelphi | White or pastel typographic spines; Reclam colours by text type |
| Penguin | Horizontal bands, colour by genre; descending |
| Anagrama | Direction not recalled |

Pattern [calc]: no pocket collection continues the cover image on the spine. Reproducing a collection's trade dress or logo is the risky option (doc 07); the table explains what looks plausible, nothing more.

### 2.5 Elements and order

| Element | Usual place | Gift-card decision |
|---|---|---|
| Collection number | Head (French poches) | Omit |
| Author | First along the reading direction in French, German, Italian paperbacks | Keep; surname alone acceptable |
| Title | After the author | Keep; main title only |
| Subtitle | Normally left off | Omit |
| Series / volume | Short form after the title | Keep in short form |
| Publisher name or device | Foot | Only with publisher opt-in, as text; otherwise nothing or the service's own small mark |

Proposed zones on a 10 × 178 mm spine [calc]: head and foot margins 8 mm; text zone about 120–162 mm; author and title on one baseline, separated by space plus a change of weight or colour, not by punctuation.

### 2.6 French and neighbouring typography

| Rule | Detail | Conf. |
|---|---|---|
| Accented capitals | Capitals keep accents, cedilla, diaeresis, including À: "ÉLÉGANCE", "LEÇON" | [M-hi] |
| Ligatures | œ and æ compulsory, also in capitals: "CŒUR" | [M-hi] |
| Title case | Traditional (capital on first noun after an initial article: *Le Rouge et le Noir*) and simplified systems coexist: **never re-case by algorithm**, copy the publisher's form | [M-hi]/[M-med] |
| Punctuation | Typographic apostrophe; narrow no-break space before ; ! ?; no-break space before : and inside « » | [M-hi] |
| Uppercasing in code | Unicode case mapping (`str::to_uppercase`), locale-independent: fine for French | [M-hi] |
| German, Dutch, Spanish, Italian | Capital ẞ allowed since 2017; IJ as a pair; accents on capitals; È not E' | [M-hi] |

### 2.7 Legibility of rotated text

| Study (recalled) | Finding | Conf. |
|---|---|---|
| Byrne 2002 | Rotated text read faster than stacked (marquee); no reliable difference between up and down | [M-med] |
| Yu, Park, Gerold, Legge 2010 | Horizontal fastest; rotated 90° either way markedly slower; marquee slowest | direction [M-med]; magnitudes [M-lo] |
| Legge and Bigelow 2011 | Fluent reading needs an x-height of about 0.2° of visual angle: 1.4 mm at 40 cm, 3.5 mm at 1 m | [M] |

Consequences: never stack letters; follow the convention, no legibility penalty either way; prefer mixed case for long titles and capitals only for short ones; one line, one sweep of the eye. A shelf at 1 m needs about 14 pt capitals.

---

## 3. Sizes and fitting on a 10 mm spine

### 3.1 Band

Type lives in the central 6 mm; 2 mm clear on each side absorb cutting and fold drift of about ±1 mm [sib, doc 03]. Keep the ink box at 5.0 mm or less for breathing room [E].

### 3.2 Measured on the bundled fonts [L]

| Font | d-to-p extent | Max size, 5.0 mm ink | All caps with É and Ç | Two lines (2 × 2.6 mm) | Characters in 120 mm at 10 pt (mixed / caps) |
|---|---|---|---|---|---|
| Spectral Regular / SemiBold | 0.980 em | 14.5 pt | 1.102 em → 12.9 pt | 7.5 pt | 68 / 53 |
| Atkinson Hyperlegible Bold | 0.829 em | 17.1 pt | 1.072 em → 13.2 pt | 8.9 pt | 65 / 56 |
| Bebas Neue (caps only) | 0.700 em | 20.2 pt | 1.009 em → 14.0 pt | 10.5 pt | 96 |

Size guidance: mixed case 9–12 pt (cap height 2.1–2.8 mm); all caps 12–14 pt for short titles; floors 7 pt positive and 8 pt reversed or outlined, in a weight of 600 or more for reversed text [sib, doc 03; doc 08 item 7].

### 3.3 Fit ladder and worked titles

Ladder [L/E], stopping at the first step that works: (1) one line at the largest size ≤ band maximum, preferred about 11 pt; (2) under 9 pt → drop the author; (3) still under 9 pt → cut at the first subtitle delimiter (` : `, ` — `, ` - `, `, ` after 20 characters, `(`); (4) switch to the condensed caps face; (5) two lines, each ≥ 7 pt, never splitting a word; (6) ellipsis at a word boundary keeping ≥ 60 % of the characters; (7) below 7 pt (8 pt reversed) → report failure. Never drop words silently.

The conventions note proposes a slightly different order (drop the subtitle first, then reduce the author to initials and surname, then shrink to 8 pt with at most −2 % tracking and no horizontal squeeze under 92 %, then two lines at 7.5 pt, then truncate keeping at least four words). Both agree on: subtitle before truncation, author before title, two lines before an ellipsis, never stacked letters.

| Title (characters) | Width at 10 pt, Spectral | Largest one-line size in 162 mm: Spectral / Atkinson / Bebas | With a 35 % author zone: Spectral |
|---|---|---|---|
| Stoner (6) | 10.2 mm | 14.5 / 17.1 / 20.2 pt (band-limited) | 14.5 pt |
| Le Comte de Monte-Cristo (24) | 42.1 mm | 14.5 / 17.1 / 20.2 pt | 14.5 pt |
| Le Vieux qui ne voulait pas fêter son anniversaire (50) | 76.8 mm | 14.5 / 17.1 / 20.2 pt | 13.0 pt |
| L'Extraordinaire Voyage du fakir… (77) | 119.8 mm | 13.5 / 12.6 / 17.9 pt | 8.3 pt → drop the author |
| La Vie et les aventures surprenantes de Robinson Crusoé… (127) | 195.9 mm | 8.3 / 7.6 / 10.8 pt | 5.1 pt → cut at the comma |

Live length 162 mm = 178 − 2 × 8 (150 mm with a foot mark). Truncation is rare.

### 3.4 Scripts and shaping [L]

- Bundled fonts cover French, Polish, Romanian, Turkish and German completely (Atkinson Bold lacks capital ẞ); **U+202F is missing in all four** — map it to U+00A0 before shaping. Vietnamese and Cyrillic: Spectral only. Greek, Hebrew, Arabic, CJK: none.
- If a character has no glyph, fall back per script; if no font covers it, return a typed error or omit with a visible note — never a `.notdef` box.
- Vertical CJK: shaping engines apply the `vert` feature in top-to-bottom direction; Unicode vertical orientation data gives per-character orientation [V1]. Out of scope for launch.
- The adaptation note recorded `ebc-text` on `rustybuzz` 0.20 (archived upstream); `ebc-text` now depends on **HarfRust** 0.15 [L, Cargo.toml read 2026-11-12].

---

## 4. Adapting a real spine

### 4.1 Where a real spine comes from

Reseller feeds (ONIX List 158: 29 full cover, 51 flap, **56 spine image, 57 spine panorama image** — corrected per doc 02's fact-check), a self-publisher's wraparound PDF, or a scan. No public source serves spines by ISBN [sib]. Rights: doc 07; reusing a spine is an opt-in that needs the rights-holder's authorisation.

### 4.2 Width priors

| Rule | Formula | Conf. |
|---|---|---|
| Physics | sheet thickness (µm) = grammage × bulk; spine = pages / 2 × thickness | [calc] |
| Pressbooks | spine (in) = pages / PPI, PPI = 2 / caliper | [V1] |
| KDP | pages × 0.002252 in (white), × 0.0025 (cream) | [sib] |
| Lulu | pages / 444 + 0.06 in | [sib] |

Pages for 10 mm range from about 125 (bouffant 80 g, bulk 2.0) to 205 (coated 115 g) [calc on recalled bulk values]. Use the formula as a prior window (0.8–1.25 ×), never as the answer.

### 4.3 Locating the spine in a flat wrap

1. PDF: spine = spread trim width − 2 × front trim width; no image analysis.
2. Raster, front cover known: downscale to 128 px height, slide the cover over the right half of the wrap at scales 0.90–1.10, score by normalised cross-correlation, refine at 512 px; correct for a symmetric crop of the eBook cover; spine width by symmetry.
3. Cross-checks inside the prior window: colour-step band edges (accepted only if within 1 mm of step 2) and the extent of rotated text (a lower bound).
4. Confidence high if NCC ≥ 0.95, right gap < 0.5 mm and the cross-checks agree; otherwise show two draggable guides.
5. Scans: Hough lines; not prototyped; start with a manual crop.

Prototype on synthetic wraps [L]:

| Estimator | Error over spines of 3–30 mm, 3 styles |
|---|---|
| Geometry only, matching aspect | −0.11 mm |
| Geometry only, aspect off by 2.4 % | +6.66 mm (unusable alone) |
| Template match | −0.67 to +0.40 mm, NCC 0.986–0.999 |
| Same, eBook cover cropped 3 % on one side | +7.3 to +8.4 mm (2 × asymmetry, undetectable) |
| Band edges, distinct band | 0.00 to ±0.17 mm |
| Band edges, continuous artwork | none or false hits: must be gated |

### 4.4 Adaptation ladder

`k` = card height / book height; `w'` = k × real spine width; `c'` = k × content extent across; `t'` = k × cap height of the largest spine text.

| Order | Condition | Action | Conf. |
|---|---|---|---|
| 0 | < 150 ppi at final size, or t' < 1.7 mm, or w' < 5 mm, or no text | **incompatible**: synthesise, reuse the spine's colour | [sib]/[E] |
| 1 | 9.3 ≤ w' ≤ 10.9 | scale across only, by at most ±8 % | [E] |
| 2 | 5 ≤ w' < 9.3, outer columns flat or gradient | scale by k, centre, pad with the spine's edge colour | [E] |
| 3 | w' > 10.9 and c' ≤ 7.0 | crop 10 mm centred on the content | [E] |
| 4 | 7.0 < c' ≤ 8.2 | crop plus a squeeze across of at most 15 % | [E] |
| 5 | c' > 8.2 | **incompatible** | [E] |

Rule 1 covers real spines of 9.3–10.9 mm for a poche, 10.7–12.5 mm for a 140 × 205 broché, 12.5–14.7 mm for 155 × 240 [calc]: it is the exception; padding, cropping and rejection are the normal cases. Resolution: a wrap 1,000 px high at 178 mm is 143 ppi (reject); 1,400 px is 200 ppi (warn).

Seam carving: assessed, not recommended — the only Rust crate with masks is LGPL (not allowed), `imageproc`'s version has no protection mask, it alters the artwork more deeply, and a US patent may still be in force [V1/M]. Re-flowing cut-out elements is tracing by another name: reseller mode with cleared rights only.

---

## 5. Synthesising a spine

### 5.1 Background ladder

| Step | Rule | Default | Conf. |
|---|---|---|---|
| S1 Normalise | Uniform border per side (ΔE*ab ≤ 6 for ≥ 98 % of pixels): two opposite bars = padding, crop; all four sides 1–12 % = design frame, sample inside it | tol 6, 98 %, 1–12 % | [L] |
| S2 Binding edge | Left edge of the front for left-to-right books | by book language | [M] |
| S3 Strip | Skip the outer 1 %, take the next 3 % of the width, full height, in CIELAB | 1 % + 3 % | [E] |
| S4–S5 Classes | **flat** σ ≤ 3 (or span ≤ 3 and resid ≤ 6); **gradient** resid ≤ 4 and wiggle ≤ 1.5; **texture** resid ≤ 10, edge ≤ 3 %, span ≤ 10; **busy** otherwise | as stated | [L] |
| S6 Ground | flat: strip median; gradient: low-pass profile as a shading (≤ 64 stops); texture: its mean, no grain; busy: among the five largest cover swatches, the one nearest the strip mean (next one if the title colour fails 4.5:1) | — | [E] |
| S6b Blurred mirror (opt-in, authorisation) | Strip as wide as the spine, mirrored, blur σ 2.5 mm across and 1 mm along, pulled 35 % towards its mean; only when edge density ≤ 2 % and no text within 6 % of the edge | off | [E], not prototyped |
| S7 Text colour | Printed colours. Ground L* ≤ 46 (45 uncoated): paper white; ≥ 70 (74 uncoated): K100; coloured text only at ≥ 7:1; floor 4.5, target 7; on a gradient test the 5th and 95th percentile rows | — | [calc] |
| S8 Rescue | Mid-tone or nothing passes: move the ground's L* to the nearer bound (to the 7:1 bound if ≤ 25 L*), keep hue and chroma; else darkest or lightest swatch | 25 L* | [E] |
| S9 Type | Top open-font match (doc 10); else by class: serif → Spectral SemiBold, sans → Atkinson Hyperlegible Bold, condensed caps → Bebas Neue | — | — |
| S10 Layout | 6 mm band, ink ≤ 5.0 mm; head and foot 8 mm; author ≤ 35 % of the live length at 0.7 × title size, gap ≥ 6 mm | — | [E] |
| S11 Direction | §2.3; an adapted real spine keeps its own | — | [sib] |
| S12 Marks | No publisher logo unless licensed; no fake shading | — | [sib] |

Classifier check on synthetic covers [L]: flat cream → flat; vertical gradient → gradient; three photographs → busy (without the `wiggle` test all three were classed "gradient" and would have produced streaks); a white frame around a photo is misread as busy unless S1 runs first.

Use the same ground on the back panel, and let it run at least 2 mm past each crease so a fold drift shows no hard edge [sib].

### 5.2 Contrast on paper

| Condition | Best ratio | Light text needs ground L* | Dark text needs ground L* | Dead zone for 7:1 |
|---|---|---|---|---|
| Ideal sRGB | 21:1 | ≤ 50 (4.5) · ≤ 38 (7) | ≥ 49 · ≥ 62 | 38–62 |
| Coated (paper 95, K100 16) | 13.0:1 | ≤ 46 · ≤ 34 | ≥ 59 · ≥ 73 | 34–73 |
| Uncoated (paper 93, K100 31) | 7.5:1 | ≤ 45 · ≤ 33 | ≥ 74 · ≥ 90 | 33–90 |

[calc; paper and K100 L* values recalled]. At equal WCAG ratio, white text on a mid-tone reads far better than black on it (APCA's lesson, evaluated once with its published constants, not shipped) [calc]. Any pair at 4.5:1 is at least 42 L* apart, so a separate lightness guard adds nothing. On uncoated home prints, target 4.5:1; 7:1 exists only for black on near-white.

### 5.3 Spines for eBook-only titles

No reference object exists, so plausibility comes from the language's conventions. A 10 mm spine corresponds to a 150–250-page paperback, believable for a novel [sib]. KDP-ratio covers (1.6:1) fit the poche trim (1.618:1) within about 1 % [calc]. A small head mark showing that this is a digital gift keeps the object honest: it should look like a book on a shelf without passing for a pirated edition [E]. Self-published authors can be offered a spine editor (short title, colours, direction), since they hold the rights.

---

## 6. Prior art

| Tool | What it does | Worth copying | Conf. |
|---|---|---|---|
| calibre 9.0 Bookshelf view (2026-01-30) | Screen spines: dominant cover colour, text colour by higher contrast, font 0.75–1.3 × base, shrink, two lines, elide; default bottom-to-top as a preference | Simplest working scheme; its shading imitates a round spine on screen and must not be printed. GPLv3: facts only | [V1] |
| calibre `dominant_color` | ≤ 100 px, 32 levels per channel, most frequent bucket; low-saturation fallback | Cheap deterministic dominant colour | [V1] |
| calibre cover generator, KOReader FakeCover | Text-only covers; calibre silently drops lines that do not fit | Counter-example: silent loss of words | [V1] |
| AndroidX Palette | 16-colour quantisation, swatch targets; white text first, black only if white fails | Swatch choice and polarity | [V1] |
| Harvard Stack View, virtual-bookshelf, CSS 3D books | Screen shelves from metadata | Clamping sizes to a readable range | [V1] |
| Kindle, Kobo, Apple Books, Google Play Books | Face-out covers; no printed spine synthesis known | — | [M] |

Nothing found samples the binding edge or adapts a real spine: both are new work.

---

## 7. Test cases (synthetic fixtures, offline)

| # | Input | Expected |
|---|---|---|
| T1 | "Stoner" | one line, size capped by the band |
| T2 | 77-character title + author | author dropped, title one line ≥ 12 pt |
| T3 | 127-character title | cut at the comma; never under 7 pt; ellipsis last |
| T4 | U+202F, U+00A0, œ, « » | no `.notdef`; U+202F normalised |
| T5–T6 | Cyrillic; Greek/Arabic/Hebrew/CJK without extra font | fallback to Spectral; typed error or note, never tofu |
| T8 | "ÉCOLE DES FEMMES, ÇA" in caps | ink box ≤ 5.0 mm including accents |
| T9 | Light cover (L* 97) | flat ground, K100 text, ≥ 7:1 coated |
| T10 | Mid-tone cover (L* 50) | ground darkened to L* ≤ 34, white text ≥ 7:1; never black on a mid-tone |
| T11–T14 | Busy photo; white frame; side bars; title touching the binding edge | palette colour; frame detected; bars cropped; no letter fragments |
| T15 | Sky gradient L* 30 → 80 | gradient kept only if one text colour passes at both ends |
| T16–T20 | Real wraps: 17 mm spine with 6 mm content; 3 mm spine; 35 mm stacked title; asymmetric eBook crop; 1,000 px high | crop; incompatible; incompatible; low confidence with guides; rejected (143 ppi) |
| T21 | Fold shifted ±1 mm | no hard colour edge on the front |
| T22 | Hostile: 1 × 1 px, transparent PNG, 20,000 × 20 px, empty title, 10,000 characters, U+202E | no panic; typed error or safe default |
| T23 | Same input twice | byte-identical output |

---

## 8. Open questions

1. Direction in practice: German and Spanish shares, Dutch and Scandinavian confirmation, French exceptions, Portugal, Brazil, Quebec, Finland, Central and Eastern Europe; exact wording and status of ISO 6357 and Z39.41 (edge title, library zone, "transverse preferred").
2. Real-cover statistics: share of covers per background class (about 200 covers to classify), how often the eBook cover is a crop of the print front, whether series redesigns break the template match.
3. Visual tests with readers of the ±8 % stretch limit, the 60 % truncation rule and the 9 pt author-drop threshold.
4. Paper-white and K100 L* on coated and uncoated stock (inputs of §5.2); real printed contrast of sample cards.
5. Typical point sizes on real 9–12 mm spines (observation only).
6. Element order and foot marks on current French poche spines.
7. The seam-carving patent; the typographic tolerance behind ±8 %.
8. Whether a reseller feed actually fills ONIX codes 56 and 57.

---

## 9. Sources

Primary files read by the adaptation researcher on `raw.githubusercontent.com` and `crates.io` (calibre `bookshelf_view.py`, `gui2/__init__.py`, `imageops.cpp`, `covers.py`, `Changelog.txt`; KOReader `mosaicmenu.lua`; AndroidX `Palette.java`, `Target.java`; WCAG relative-luminance term; apca-w3 README and LICENSE; APCA introduction; imageproc `template_matching.rs` and `seam_carving.rs`; caire README; seamcarving on crates.io; rustybuzz README; harfrust, unicode-vo on crates.io; CSS Writing Modes 4 source; Pressbooks `class-spine.php`) are linked in full in `notes/spine-design.adaptation.md` (Sources). The most load-bearing:

- [calibre bookshelf_view.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/gui2/library/bookshelf_view.py) and [Changelog](https://raw.githubusercontent.com/kovidgoyal/calibre/master/Changelog.txt)
- [AndroidX Palette.java](https://raw.githubusercontent.com/androidx/androidx/androidx-main/palette/palette/src/main/java/androidx/palette/graphics/Palette.java)
- [WCAG relative luminance](https://raw.githubusercontent.com/w3c/wcag/main/guidelines/terms/20/relative-luminance.html)
- [apca-w3 LICENSE](https://raw.githubusercontent.com/Myndex/apca-w3/master/LICENSE.md)
- [imageproc template_matching.rs](https://raw.githubusercontent.com/image-rs/imageproc/master/src/template_matching.rs)
- [Pressbooks class-spine.php](https://raw.githubusercontent.com/pressbooks/pressbooks/dev/inc/covergenerator/class-spine.php)
- [Jonix ResourceContentTypes.java](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ResourceContentTypes.java) (ONIX List 158 codes 56 and 57; via doc 02's fact-check)
- [ISO 6357:1985 catalogue entry](https://www.iso.org/standard/12665.html) (existence and title only; via doc 02's editor)

Recalled references, not located (conventions note): ISO 6357:1985; ANSI/NISO Z39.41-1997; Wikipedia "Bookbinding", "Buchrücken", "Dos (livre)"; *Chicago Manual of Style* ch. 1; *Lexique des règles typographiques en usage à l'Imprimerie nationale*; Byrne 2002; Yu et al. 2010; Legge and Bigelow 2011; Baines, *Penguin by Design*; Hochuli and Kinross, *Designing Books*.

---

## Implications for the implementation

1. The spine direction is computed from the book (real spine, explicit setting, country of publication, language, ISBN group), never from the UI locale, with a user toggle; unknown → top-to-bottom (ISO 6357).
2. One line, author then title, centred across the spine; never stacked letters; no collection number, no publisher logo unless licensed.
3. Sizes are computed on the ink of the actual string, inside the 6 mm band with an ink box ≤ 5.0 mm; floors 7 pt positive, 8 pt reversed or outlined.
4. The fit ladder never drops words silently and reports every step taken (author dropped, subtitle cut, tightened, truncated).
5. Titles are never re-cased by algorithm; uppercasing uses Unicode mapping; U+202F is normalised before shaping; missing glyphs fall back per script, never `.notdef`.
6. A real spine is reused only as an opt-in with recorded authorisation, through the adaptation ladder; "incompatible" falls back to a typeset spine with the spine's colour.
7. The default background is a flat colour computed from the binding-edge strip after frame/padding detection; mirrored, stretched or outpainted artwork is never the default.
8. Text colour follows the polarity rule and is checked on printed colours (output profile); when nothing passes, the ground's lightness moves.
9. APCA code, constants and name are not shipped.
10. Output is deterministic (byte-identical for the same input) and safe on hostile images and strings.
11. Locating the spine in a flat wrap shows two draggable guides whenever confidence is not high.
12. ONIX spine resources (List 158 codes 56, and 29 for full covers) feed `SpineSource::Provided` / `FullWrap` in reseller mode, under the per-title rights flags.

---

## What the code implements today

Read from `crates/ebc-spine` (crate and module documentation, `SpineOptions::default()`, constants) and `crates/ebc-i18n/src/spine.rs` on 2026-11-12.

- **Output**: `design` returns the spine's items in page millimetres: a background covering the rectangle with bleed, and the title (and the author when there is room) as filled outlines — no font embedded.
- **Sources**: `SpineSource::Synthesize` (default); `Provided(pixmap)` and `FullWrap { image, spine_px }` are opt-ins documented as requiring the rights-holder's authorisation; `SpineDesign::requires_authorisation` reports it. The caller supplies the spine's pixel columns of a wrap.
- **Background** (`BackgroundStyle::Auto`, default): (1) the left 4 % strip of the cover is near-uniform (`edge_uniformity` ≥ 0.9) → its colour; (2) a uniform frame → the frame's colour; (3) otherwise the swatch, among the five largest covering ≥ 10 % of the cover, nearest the edge's mean colour; (4) no cover → white. Also `Solid`, `DominantColour`, and `EdgeContinuation` (opt-in, authorisation: mirrored strip blurred 2.5 mm across and 3 mm along, pulled 35 % to its mean, with a flat band behind the text if contrast fails). Flat fills are sRGB by default, device CMYK on request (PDF/X-1a).
- **Direction**: `ebc_i18n::spine_direction_for_book` — the book's language (fr, de, it, es, pt → bottom-to-top; en, nl, da, sv, nb → top-to-bottom), else its ISBN registration group, else the UI locale's convention. `ebc-spine` only applies the direction it is given.
- **Typography** (`SpineOptions::default()`): one line, author then title, centred along between 8 mm head and foot margins; optically centred across on the title's baseline-to-cap-height box; all ink inside the band between the two safe margins (`ebc-layout` default `spine_safe_margin` 2.0 mm), measured on real outlines. Title 7–14 pt; author at 0.7 × title, ≥ 6 pt, ≤ 35 % of the length, 6 mm gap; the title is not reduced below 9 pt to keep the author (the author is dropped instead); letter-spacing may tighten by 0.02 em; then cut at the first subtitle separator; then truncation at a word boundary with an ellipsis. Text is never invented, re-cased or stacked. Missing characters are left out and reported (`MissingGlyphs`), after trying `fallback_font`.
- **Contrast**: WCAG 2 ratio on sRGB values, minimum 4.5; white or black text by default; a preferred text colour is used only if it reaches the minimum; notes for low contrast, mid-tone ground (L* 46–59, estimate) and reversed text under 8 pt.
- **Adaptation of a provided spine** (`adapt.rs`): incompatible below 150 ppi, without detected text, or when `w'` < 0.5 W; stretched across when 0.92 W ≤ `w'` ≤ 1.08 W; padded with the side colours when narrower with plain sides; cropped when wider with plain sides and content ≤ W − 2 × safe margin; shrunk uniformly with head and foot filled when `w'` ≤ 1.3 W; incompatible above; the text across must stay ≥ 1.7 mm. The text detector is a heuristic (ink per column, transitions along the spine).
- **Limits** (stated by the crate): left-to-right books only; no two-line setting; no vertical CJK; no publisher mark at the foot; contrast figures are not ink-on-paper measurements. Deterministic integer or basic IEEE arithmetic so that pixels are identical on every platform, including wasm.

**Where the implemented default differs from the research:**

| Point | Research (§§ above) | Code today | Note |
|---|---|---|---|
| Direction fallback | Unknown language and ISBN → top-to-bottom (ISO 6357); also doc 02 implication 45 | Falls back to the **UI locale's** convention | Contradicts "never from the site locale" for books with no language or known ISBN group; country of publication (ONIX) is not consulted |
| Background classes | flat / gradient / texture / busy, with a gradient rendered as a shading | Flat colour only (edge, frame, swatch, white); no gradient class | Smooth gradients become a flat colour — legally safe, visually less faithful |
| Edge strip | Skip outer 1 %, take next 3 % | Outer 4 % | Minor: JPEG edge artefacts and bleed slivers are included |
| Frame/padding pre-step | Bars on two sides cropped; frame sampled inside | Uniform frame used as the ground colour; padding bars not cropped | Covers with side bars may be misclassified |
| Mirror blur (opt-in) | σ 2.5 mm across, 1 mm along | 2.5 mm across, 3 mm along | Stronger smoothing along the spine |
| Text colour | Printed colours via the output profile; polarity rule; target 7:1; move the ground's L* when nothing passes | WCAG 2 on sRGB; floor 4.5:1, no 7:1 target; white or black; a note (no correction) for mid-tones | The main typographic gap; uncoated home prints may fall below 4.5:1 on paper |
| Sizes | Preferred ~11 pt; floor 7 pt, 8 pt reversed/outlined; ink box ≤ 5.0 mm | 7–14 pt; reversed-text note under 8 pt; ink inside the band between safe margins (6 mm at the default 2 mm margin) | Code allows ink up to 6 mm; outlined positive text may go to 7 pt although doc 03/08 put outlined text at ≥ 8 pt |
| Fit ladder | Author drop, subtitle cut, condensed face, **two lines**, ellipsis keeping ≥ 60 % | Author drop, tightening, subtitle cut, ellipsis | No condensed-face step and no two-line setting; minimum kept length after truncation not documented |
| Author shortening | Initials, surname only, "et al." / "Collectif" | First author only | Partial |
| Missing glyphs | Map U+202F → U+00A0; typed error when no font covers a script | Fallback font, then left out with a note | No silent tofu in either; the code may omit characters rather than fail |
| Adaptation windows | Stretch 9.3–10.9 mm (±8 %); crop when content ≤ 7.0 mm; squeeze ≤ 15 % across when content 7.0–8.2 mm; incompatible above | Stretch 9.2–10.8 mm; crop when content ≤ W − 2 × safe margin (6 mm at 2 mm); **uniform shrink** up to 13 mm with head/foot fill; incompatible above 1.3 W | The code never squeezes anisotropically — closer to doc 08's "uniform scaling only" rule than the research |
| Locating the spine in a wrap | Template match + cross-checks + guides | Caller provides the spine columns | Locator not implemented |
| Marks | Optional small "digital gift" head mark, service mark at the foot | None | As the crate's limits state |
