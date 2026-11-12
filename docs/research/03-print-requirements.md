# Professional print requirements for the gift-card PDF

_Status: draft — research date 2026-11-12_

**Scope.** What the PDF generator must produce so that the jacket card (back | 10 mm spine | front, QR code inside) is accepted by European online printers, by a local print shop working from a quote, and by a home printer. Printer selection and prices are in the printers notes; what may be done to a cover image is in `07-legal-covers-and-reseller-rights.md`.

**Confidence markers.** [V2] = verified in at least two independent sources during the research session; [V1] = one source; [M] = from memory, not verified in this session; [calc] = arithmetic from the stated inputs. "House rule" marks a design decision of ours, not a printer fact.

**Method limits.**

- This document synthesises two researcher notes: `notes/print-requirements.prepress.md` (24 searches) and `notes/print-requirements.folding.md` (25 searches). Both researchers ran out of the shared search budget. No page was read in full (WebFetch is blocked): every sourced figure comes from a search-result extract.
- For this synthesis four further searches were attempted (ECI licence text, profile ink limits, PDF/X OutputIntent keys, QR print sizing). **All four were refused: the budget was exhausted. Nothing new was verified on the web here.** No marker was upgraded.
- Gaps were filled, where possible, from sibling notes already in the repository: `notes/ereader-ux.ux.md` (QR facts read from library documentation), `notes/printers.web2print.md` and `notes/printers.print-api.md` (which products exist), `notes/printers.esg.md` (stock and finishing), and document 07 (cover integrity rules). Their markers are kept as they were.
- Every printer surveyed says the product data sheet or template overrides its general guide, for example Flyeralarm: "achten Sie stets auf die Vorgaben im jeweiligen Produkt-Datenblatt" ([Flyeralarm checklist](https://www.flyeralarm.com/blog/wp-content/uploads/Flyeralarm_Checkliste_Druckdatenerstellung-1.pdf)) [V2]. Every number below is a default to confirm per product before a printer profile is switched on.
- The sections on ICC licences, PDF/X dictionary keys, crop-mark geometry and QR sizing rest largely on memory. They are flagged, and listed again in §11.

---

## 1. Executive summary

### 1.1 Findings

1. **No catalogue product matches the object exactly.** No EU online printer or print API was found that sells, as a configurable product, a book-trim card with two creases 10 mm apart (three notes agree: folding, web2print, print-api) [V1]. The nearest stock product is the presentation folder with a 10 mm capacity spine [V2]. The generator therefore needs three outputs: an upload file without marks, a file with marks plus a dimensioned plan for quote jobs, and a home-print sheet.
2. **Bleed is not uniform in Europe**: 1, 2, 3, 3.175 and 5 mm are all published [V2]. Bleed is a per-printer parameter; 3 mm is the default.
3. **Most online printers reject printer's marks** (Pixartprinting, Flyeralarm, Printoclock, Bookvault, BoD, Lulu) [V1 to V2 per printer]. They expect a page of exactly trim + bleed. Marks are for traditional and local printers only.
4. **PDF flavour and colour condition are split**: PDF/X-4, PDF/X-3:2002, PDF/X-1a:2001 and "PDF 1.3"; FOGRA39 versus FOGRA51 [V2]. One content subset satisfies all of them: CMYK and grey only, no transparency, no layers, no spot colour, fonts embedded. Only the label and the output intent change per printer.
5. **Total ink: 300 % is the common ceiling** [V2]. Text, codes and the QR symbol are 100 % K.
6. **Minimums agree across printers**: 6 pt type, 0.25 pt positive lines, 0.5 pt reversed lines, 300 ppi images [V2].
7. **Cover resolution is the main prepress risk.** A retailer thumbnail 500 to 800 px wide gives about 112 to 203 ppi on the front panel [calc]. The cover must be sourced at 1,200 px wide or more, or the card falls back to the typographic layout.
8. **A 10 mm spine is about twice the threshold for spine text** (consensus 5 mm, range 4 to 6 mm) [V2]. With 2 mm clear on each side the text band is 6 mm. Creases drift by about ±1 mm [V1].
9. **Only the pocket formats can be printed at home.** A6, 108 × 177 and 110 × 178 mm are the only trims of §7 whose spread fits A4 at 100 % with bleed and cut ticks; all three also fit US Letter [calc]. The poche 110 × 178 mm, the standard French pocket size [V1], is the default format.
10. **Cover handling is fixed by document 07**: no crop, no stretch, no artwork in the bleed, spine and other panels built from our own material. This is why the default layout insets the cover on a flat ground, and why the bleed is always a flat colour.
11. **Unverified and load-bearing**: the ECI profile licence, the exact PDF/X keys, mark geometry and every QR size figure are [M]. They need one verification pass before a profile is bundled or a constant is frozen (§11).

### 1.2 The default print profile

Default = the upload profile for a professional printer (`upload_eu`), poche format. "Allowed range" is what a per-printer profile may set; anything outside it is a validation error.

| Parameter | Default | Allowed range | Source |
|---|---|---|---|
| Panel trim (front and back) | 110 × 178 mm | presets of §7; custom 50 to 300 mm per side | §7 [calc]; poche size [V1] |
| Spine width, crease to crease | 10.0 mm | 5 to 15 mm with text; no spine text below 5 mm | §6.2, §6.3 [V2] |
| Flat trim (spread) | 230 × 178 mm | (2 × width + spine) × height | [calc] |
| Bleed, each side | 3 mm | 1, 2, 3, 3.175 or 5 mm, set per printer | §2.3 [V2] |
| Bleed content | flat ground colour only | cover artwork in the bleed only with `trim_authorised` | document 07, rule 8 |
| Safety margin from trim (text, QR, logos) | 5 mm | 3 to 10 mm, never below 3 | §2.3: published values 3, 4 and 5 mm [V2] |
| Safety margin from a crease, on the cover panels | 5 mm | 3 to 10 mm | 5 mm crease-to-die-line rule by analogy [V1]; house rule |
| Cover image inset, from trim and from the crease | 5 mm | 3 mm or more; 0 only with `trim_authorised` | document 07 (3 mm floor); 4 mm frame rule [V2]; §1.4 row 9 |
| Spine text clear zone, each side | 2 mm (text band 6 mm) | 1.5 to 3.2 mm | §6.3: 1.5, 1.6 and 3.2 mm [V1 each] |
| Spine text clear zone, top and bottom | 5 mm | equal to the safety margin | house rule |
| Colour boundary near a crease | none within 2 mm of the crease | — | ±1 mm tolerance [V1]; §6.4 |
| Printer's marks, upload profile | none | — | §2.3 [V2] |
| Printer's marks, marks profile | crop and fold ticks, 0.25 pt, 5 mm long, starting at the bleed edge | length 3 to 5 mm; weight 0.25 to 0.5 pt | convention [M]; §2.4 |
| MediaBox, upload profile | = BleedBox = trim + bleed (236 × 184 mm) | — | §2.2 [V2] |
| TrimBox | the flat trim, on every page | — | [V1] and [M]; §2.2 |
| PDF label | PDF/X-4 | PDF/X-1a:2001, PDF/X-3:2002; plain PDF for home | §3.2 |
| PDF content | CMYK and grey only; no transparency, layers, spot colour, annotation or script | tagged sRGB images only in profiles with `allow_rgb` | §3.2 |
| Output intent | FOGRA39, "ISO Coated v2 300% (ECI)", profile embedded | FOGRA51 (PSO Coated v3), FOGRA52 (PSO Uncoated v3) | §3.1 [V2 for the conditions]; identifiers [M] |
| Total ink | 300 % or less | 260 % (Onlineprinters express), 270 % (Lulu) | §4.3 [V2] |
| Text, typed code, QR symbol | 100 % K only | — | §4.3 [V2] |
| Large black solids | K100 + C30 M30 Y30 (190 %) | K100 alone when the solid crosses a crease | Exaprint [V1]; the crease rule is a house rule |
| Lightest tint | 5 % in each channel used | — | Pixartprinting [V1] |
| Minimum type size | 6 pt positive; 8 pt reversed or multi-ink | — | [V2]; reversed figure [M] |
| Minimum line weight | 0.25 pt positive; 0.5 pt reversed | — | [V2] |
| Fonts | embedded subsets; spine title as outlines | outline all text (Printoclock) | §4.4 [V1 each] |
| Image resolution at final size | 300 ppi target; warning below 250; error below 150 | home: 200 / 150 / 100 | target [V2]; thresholds are house rules |
| QR symbol | 33 mm box including a 4-module quiet zone; version 4 or lower; level M, raised to Q when free | box 26 to 40 mm; module 0.5 mm or more, error below 0.4 mm | §5: quiet zone [V2]; sizes [M] |
| QR position | inside page, on white, 5 mm or more from trim and creases | — | house rule |
| Stock, professional | 300 g/m² board, grain parallel to the spine, two creases, delivered flat, no lamination | 250 to 350 g/m² | §6.5 [V1], [M] |
| Finishing | none | if any: 1.5 mm clear of each crease | §6.1 [V2] |

### 1.3 The three outputs

| | `upload_eu` | `professional_eu` | `home_a4` |
|---|---|---|---|
| Use | web-to-print upload, print APIs | local print shop, trade printer on quote | home or office printer |
| Page | trim + bleed | trim + bleed + 10 mm slug | A4 landscape, spread centred |
| Bleed | 3 mm (per printer) | 3 mm | 2 mm |
| Marks | none | crop ticks, fold ticks, job line in the slug | grey cut and fold guides, 100 mm scale ruler |
| PDF | PDF/X-4, label per printer | PDF/X-4 | plain PDF |
| Colour | CMYK + K | CMYK + K | sRGB images, grey text |
| Safety margin / spine clear zone | 5 mm / 2 mm | 5 mm / 2 mm | 5 mm / 2 mm |
| Resolution: target / warning / error | 300 / 250 / 150 ppi | same | 200 / 150 / 100 ppi |
| Companion file | dimensioned plan PDF, not for print | dimensioned plan PDF | none: instructions are on the sheet |

The repository already has these three profiles (`crates/ebc-layout/src/profile.rs`). Four of its values differ from this document: `professional_eu` uses a 4 mm safety margin, a 1.5 mm spine clear zone and registration targets; `home_*` assumes a 5 mm unprintable margin. The changes are listed in "Implications for the implementation".

### 1.4 Contradictions between the notes, and what is kept

| # | Topic | Source A | Source B | Kept | Reason |
|---|---|---|---|---|---|
| 1 | Default bleed | 3 mm: French guides, Pixartprinting, Helloprint, Bookvault | 2 mm: Onlineprinters, Saxoprint; 5 mm: BoD | 3 mm default, per-printer override | No single value exists. The bleed is a flat colour, so any value from 1 to 5 mm costs nothing |
| 2 | Exaprint bleed | charte PAO: 2 mm minimum | blog: at least 3 mm | 3 mm | Both are lower bounds; 3 mm satisfies both. Confirm on the product template |
| 3 | Printoclock bleed | FAQ: 2 mm | template table: 5 mm for most formats | the template value of the product ordered | A per-format template is more specific than a FAQ. Open until a product is chosen |
| 4 | Safety margin | prepress note: 5 mm | existing code: 4 mm | 5 mm | It is the maximum of the published 3, 4 and 5 mm values and costs 1 mm of live area |
| 5 | Spine text clear zone | EPFL 1.5 mm, IngramSpark 1.6 mm | Lulu 3.2 mm | 2 mm | Both notes propose it. It leaves a 6 mm band and covers the ±1 mm drift with 1 mm to spare |
| 6 | Where crop marks start | prepress note: at the bleed edge (3 mm from trim) | folding note's sheet-fit table: 3 mm beyond the bleed (6 mm from trim) | at the bleed edge | It is the usual "offset = bleed" instruction [M] and saves 3 mm per side on A4. The sheet-fit table in §7 is recomputed on this basis |
| 7 | Pixels needed for a full-bleed poche front | prepress note: about 1370 × 2170 | existing code: 1335 × 2174 | 1335 × 2174 | The front has bleed on three sides only: 113 × 184 mm at 300 ppi [calc] |
| 8 | Spine background past the crease | folding note: run the spine colour 2 mm onto front and back | document 07: nothing may overlap the cover | one continuous ground across back, spine and front; cover inset 5 mm from the crease | The overrun never touches the cover. With a full-bleed cover the overrun on the front side is zero and the drift is accepted |
| 9 | Cover inset | document 07: at least 3 mm | Onlineprinters and Flyeralarm: frames at least 4 mm from the edge; cutting shift 1 to 2 mm | 5 mm | A 3 mm border becomes 1 to 5 mm after a 2 mm shift, which is visibly uneven. 5 mm complies with both |
| 10 | "Artwork drawn to 5 mm bleed" | prepress note | document 07: no artwork in the bleed | the ground colour fills whatever bleed is asked | Same result without touching the cover |
| 11 | QR symbol width | prepress note: 25 mm or more [M] | UX note: 20 mm or more [M] | 33 mm box (symbol 25 to 27 mm); floor 26 mm box | Both figures are from memory. The quantity that matters is module size. The inside panel has room for the larger value |
| 12 | Smallest QR module | prepress note and code: 0.4 mm | UX note: 0.5 mm preflight floor | error below 0.4 mm, warning below 0.5 mm, default 0.8 to 1.0 mm | Keeps both; the default sits well above either |
| 13 | Lamination | folding note: laminate dark solids to stop cracking at creases [M] | ESG note: no film lamination, keeps the card recyclable | none by default | Cracking is handled by creasing, grain direction and low ink coverage at the creases. Lamination stays an option per printer |
| 14 | Flap width, single-sided home variant | folding note: 50 mm | this document: 45 mm | 45 mm | At 50 mm only 2.15 mm of printable sheet remains outside the trim on each side: room for the bleed, none for cut ticks [calc] |
| 15 | Pages for a 10 mm softcover spine | folding note: 150 to 250 [calc from published formulas] | web2print note: "100+" [M]; print-api note: 150 to 200 [M] | 150 to 250 | The only figure computed from sourced formulas |
| 16 | Fonts | Pixartprinting: embed, do not outline | Printoclock: outline | embed subsets; outline flag per printer | Embedding satisfies every printer but Printoclock. An outlined spine title is vector art to a preflight and is accepted everywhere |
| 17 | Default PDF label | X-4: Onlineprinters, Saxoprint, Gelato | X-3:2002 Flyeralarm; X-1a or X-3 Bookvault; PDF 1.3 Exaprint | X-4 by default, label per printer | §3.2 |
| 18 | Default output condition | FOGRA39: Flyeralarm, Exaprint, Pixartprinting | FOGRA51: Onlineprinters, Saxoprint | FOGRA39 | France first: the printer with confirmed production in France (Exaprint) and Pixartprinting name it. FOGRA51 is a per-printer option |
| 19 | Helloprint safety margin | help centre: 3 mm | blog: 5 mm | 5 mm | Satisfies both |
| 20 | KDP bleed | prepress note: 3.2 mm | folding note: 0.125 in | 3.175 mm | The same value; 0.125 in is exact |
| 21 | Which trims print on A4 at 100 % | folding note, summary: only 110 × 178 | folding note, table: A6 and 108 × 177 also fit | A6, 108 × 177 and 110 × 178 fit; 110 × 178 is the default | The table is the arithmetic; the summary overstated it. Recomputed in §7 |

---

## 2. Page boxes and marks

### 2.1 Coordinate model

Layout coordinates are in millimetres, origin at the top-left corner of the trim, x to the right, y down (as in `docs/architecture.md`). 1 mm = 72 / 25.4 = 2.834646 pt.

Poche format, both pages:

| Element | x from | x to | Width |
|---|---|---|---|
| Outside page: back cover / inside page: inside of the front cover | 0 | 110 | 110 mm = 311.811 pt |
| Crease A | 110 | — | — |
| Spine | 110 | 120 | 10 mm = 28.346 pt |
| Crease B | 120 | — | — |
| Outside page: front cover / inside page: inside of the back cover | 120 | 230 | 110 mm = 311.811 pt |

Height 178 mm = 504.567 pt. Flat trim 230 × 178 mm = 651.969 × 504.567 pt. The spread is symmetric, so the creases have the same x on both pages.

### 2.2 Page boxes

PDF user space, points, origin at the bottom-left corner of the MediaBox. Values for the poche format [calc].

| Box | `upload_eu` (bleed 3 mm) | `professional_eu` (bleed 3 mm, slug 10 mm) | `home_a4` (bleed 2 mm) |
|---|---|---|---|
| MediaBox | `[0 0 668.976 521.575]` (236 × 184 mm) | `[0 0 725.669 578.268]` (256 × 204 mm) | `[0 0 841.890 595.276]` (297 × 210 mm) |
| BleedBox | `[0 0 668.976 521.575]` | `[28.346 28.346 697.323 549.921]` | `[89.291 39.685 752.598 555.591]` |
| TrimBox | `[8.504 8.504 660.472 513.071]` | `[36.850 36.850 688.819 541.417]` | `[94.961 45.354 746.929 549.921]` |
| Crease A, x | 320.315 | 348.661 | 406.772 |
| Crease B, x | 348.661 | 377.008 | 435.118 |

General formulas, with trim W × H, bleed b and slug s, all in points: `TrimBox = [s+b, s+b, s+b+W, s+b+H]`, `BleedBox = [s, s, s+2b+W, s+2b+H]`, `MediaBox = [0, 0, 2s+2b+W, 2s+2b+H]`. For the upload profile s = 0. For a home sheet the trim is centred on the sheet.

Rules:

1. Printers that forbid marks expect the page to be exactly trim + bleed: Onlineprinters "data format (incl. 2 mm bleed)", Bookvault 154 × 216 mm for A5, Flyeralarm "Datenformat" [V2, sources in §2.3]. So for the upload profile `MediaBox = BleedBox = trim + bleed`.
2. A PDF/X file must state the trimmed and untrimmed page sizes and the output condition ([Macwelt on PDF/X-4](https://www.macwelt.de/article/956117/pdf-workflow-in-der-druckproduktion.html)) [V1]; box definitions in [pdf2go's box explainer](https://pdf2go.com/blog/what-are-pdf-boxes) and [Wikipedia PDF/X](https://Www.Wikipedia.org/wiki/PDF/X) [V1]. From the standard [M]: each page has a MediaBox and either a TrimBox or an ArtBox, never both; TrimBox lies inside BleedBox, which lies inside CropBox and MediaBox.
3. Write TrimBox and BleedBox on every page. Write no ArtBox. Either omit CropBox or set it equal to MediaBox; never make it smaller than the BleedBox.
4. `Rotate` is 0 and there is no `UserUnit`. All pages of a file have identical boxes (Onlineprinters requires identical page sizes, [PDF settings FAQ](https://onlineprinters.ie/c/faq/settings-print-pdf)) [V1].
5. Page 1 is the outside, page 2 the inside. The single-sided home variant has one page.
6. The same layout must export at bleeds of 1, 2, 3, 3.175 and 5 mm. Only the ground colour extends into the bleed.

### 2.3 What printers publish: geometry and marks (as of 2026-11)

| Printer | Bleed, per side | Safety margin from trim | Marks in the file? | Sources, confidence |
|---|---|---|---|---|
| Pixartprinting (IT) | 3 mm for perfect-bound and wire-o books; otherwise per product template (green line = bleed, magenta = cut, red = safe area) | "safe area" per template; says a 1 to 2 mm mechanical cutting shift is normal | No crop or registration marks | [instructions article](https://support.pixartprinting.com/hc/en-us/articles/26076155387026-What-do-I-find-in-the-instructions), [perfect binding spec](https://www.pixartprinting.co.uk/printing-magazines-books-catalogues/special-finish-perfect-binding/), [wire-o spec](https://www.pixartprinting.co.uk/printing-magazines-books-catalogues/special-finish-wire-o-binding/): bleed [V2]; "no marks" [V1], exact page not pinned (surfaced with the [flyer guide](https://www.pixartprinting.co.uk/blog/how-to-create-an-effective-flyer-the-complete-guide-from-idea-to-print/)) |
| Onlineprinters (DE) | 2 mm on flyers and postcards (DL flyer: data 10.9 × 21.4 cm for a 10.5 × 21 cm trim); 10 mm on large format | 4 mm on flyers and postcards | not found | [DL flyer](https://www.onlineprinters.ie/p/flyers-leaflets-dl-printed-on-one-side), [A5 postcard](https://www.onlineprinters.ie/p/exklusive-postcards-a5), [A6 insert](https://www.onlineprinters.ie/p/inserts-exclusive-4-4-colour-printing-a6) [V2] |
| Flyeralarm (DE) | 1 mm on small formats; 1 to 10 mm depending on format | per data sheet; frames and outlines at least 4 mm from the edge | No: "please do not create folding or cutting marks within the artwork format"; its InDesign tutorial says to select none of the marks | [UK artwork requirements](https://www.flyeralarm.com/images/upload/content/_pdf/uk/Artwork_Requirements_UK.pdf), [bleed tutorial](https://www.flyeralarm.com/blog/en/tutorial-setting-up-bleeds-in-indesign-cc-illustrator-cc-and-scribus-1-4-5/), [PDF tutorial](https://www.flyeralarm.com/blog/en/creating-print-ready-pdf-with-indesign/) [V2] |
| Saxoprint (DE) | 2 mm standard; 3 mm for packaging, folders, calendars; 5 mm for brochures (preflight minimum) | at least 3 mm; UK error page: 5 to 7 mm from the data edge when bleed is 2 mm; 10 mm from a binding edge | not found | [Druckdatenanleitung, May 2024](https://api.saxoprint.de/media-by-file-name/Druckdatenanleitung-1.pdf), [UK artwork instructions](https://www.saxoprint.co.uk/service-help/artwork-instructions), [FAQ](https://www.saxoprint.de/service-und-hilfe/faq), [pages and covers](https://www.saxoprint.co.uk/service-help/error-diagnosis/pages-and-covers) [V2] |
| Exaprint (FR) | "fonds perdus" 2 mm minimum (charte, labels article); blog says at least 3 mm | "zone tranquille" 3 mm on most products, 5 mm on stapled brochures, 10 mm on "dos collé" | not found | [charte PAO](https://www.exaprint.fr/imprimerie-en-ligne/charte-pao), [fond perdu article](https://blog.exaprint.fr/cadres-impression-contraintes-et-astuces/), [6 erreurs](https://blog.exaprint.fr/6-erreurs-courantes-a-eviter-en-impression/): zone tranquille [V2]; bleed 2 or 3 mm, see §1.4 row 2 |
| Printoclock (FR) | 2 mm (FAQ) or 5 mm (template table, most formats) | "marge technique" 2 mm; 5 mm for "dos carré collé" | No "traits de coupe", no colour bar, no "hirondelles", no white margins | [FAQ](https://support.printoclock.com/article/comment-faire-mon-fichier-dimpression), [bien réaliser son fichier](https://www.printoclock.com/bien-realiser-son-fichier), [fond perdu](https://www.printoclock.com/fond-perdu), [technical specs 63](https://www.printoclock.com/technical-specs-html/63) [V1 each]; bleed: §1.4 row 3 |
| Helloprint (NL) | at least 3 mm; 5 mm or more on large products | at least 3 mm; 4 mm on bookmarks; blog says 5 mm for flyers | a help-centre page on trim marks exists; content not retrieved | [help centre](https://www.helloprint.com/en-ie/help-centre/where-do-i-start-when-preparing-my-design-for-printing), [blog](https://www.helloprint.com/en-ie/blog/how-to-set-up-a-file-for-print), [bookmarks](https://www.helloprint.com/en-gb/bookmarksexclusivefinishing) [V2] |
| Vistaprint (NL/IE) | per product template ("Specs & Templates") | per template | not found | [business card file prep](https://www.vistaprint.com/hub/business-card-file-prep/) [V1]; the often-quoted 1.5 mm EU bleed is [M] |
| Lulu (US, prints in the EU) | 0.125 in = 3.175 mm | 0.5 in (12.7 mm) in the guide and API; one help article says 0.25 in | No trim, bleed or margin lines | [book creation guide](https://assets.lulu.com/media/guides/en/lulu-book-creation-guide.pdf), [API PDF settings](https://help.api.lulu.com/en/support/solutions/articles/64000254609-pdf-creation-settings), [cover upload](https://help.lulu.com/en/support/solutions/articles/64000282777-upload-your-cover-file) [V2] |
| BoD (DE) | 5 mm per side (a 12 × 19 cm page becomes 13 × 20 cm) | not found | Advises against crop marks | [BoD Grafiken und Farbmanagement](https://www.bod.ch/bodfiles/GLOBAL-Storage/documents/help-documents/bod-grafiken-und-farbmanagement.pdf) [V1]; the 5 mm figure also appears in an [11-year-old BoD cover guide](https://www.yumpu.com/de/document/view/41877228/wie-gestalte-ich-das-buchcover-richtig-bod) [V1] |
| Bookvault (UK) | 3 mm on every trimmed edge (A5 becomes 154 × 216 mm) | 5 mm on the cover template | No: the production software adds marks | [bleed](https://help.bookvault.app/what-is-bleed-and-why-do-i-need-it), [crop marks](https://help.bookvault.app/should-i-supply-crop-marks), [PDF guide](https://help.bookvault.app/hubfs/Guide%20To%20PDFs%20-%20Digital%20Version%20.pdf?hsLang=en) [V2] |
| Cloudprinter (NL) | 3 mm | not found | not found | [Canva export article](https://knowledge.cloudprinter.com/mastering-canva-print-export-for-cloudprinter.com-compatibility) [V1]; the authoritative page is [How to build production ready PDFs](https://docs.cloudprinter.com/client/how-to-build-the-pdf/), content not retrieved |
| Gelato (NO) | 3 mm, from an unreliable third-party page only; Gelato's own page says files are supplied unfolded with bleed | not found | not found | [third-party page](https://processor-5xq.pages.dev/print-ready/gelato): treat as unverified; [Gelato formats](https://support.gelato.com/en/articles/8996362-what-product-formats-and-dimensions-do-you-support) [V1] |

Context from printers outside the survey: druck.at defaults to 2 mm bleed and 2 mm safety ([Infoblatt](https://cdn.druck.at/sites/default/files/Infoblatt_Druckdatenerstellung.pdf)) [V1]; IngramSpark 3 mm ([file requirements](https://www.ingramspark.com/blog/file-requirements-for-print-books), [Ingram cover bleed sheet](https://marketing.ingramcontent.com/mrkng/documents/publishers/Cover-Bleed-Dimensions.pdf)) and KDP 0.125 in ([KDP help](https://kdp.amazon.com/help/topic/G201953020)) [V1]. French-language guides give 3 mm as the standard ([EPFL Repro guide](https://www.epfl.ch/campus/services/repro/wp-content/uploads/2025/11/REPRO-Guide-PAO.pdf), [J'imprime en France](https://www.jimprimeenfrance.fr/impression-chemise-rabat-fine)) [V2]; some packaging templates ask for 5 mm ([Smilepack](https://smilepack.fr/files/subscribers/d72b0b53-b3d0-49ff-adfd-37d51546ebde/ftp/gabarits/Coussin%20technique%20smilepack.pdf)) [V1].

Traditional printers are the exception on marks: Clays asks for bleed, crop marks and spine-width marks ([Clays cover guide](https://www.clays.co.uk/wp-content/uploads/supplying-cover-files-for-print.pdf)); Baumer asks for PDF/X-4 with 3 mm bleed and crop marks ([Baumer](https://www.baumer.ch/fileadmin/files/downloads/Baumer_UEbermittlung_von_Druckdaten_PSO_PSD_2021_DE.pdf)) [V1 each].

### 2.4 Marks specification (marks profile only)

No surveyed online printer publishes a mark offset, length or stroke, because they do not want marks. The values below are DTP convention [M] and house rules.

| Mark | Geometry | Stroke and colour |
|---|---|---|
| Crop ticks | Two per corner, on the extensions of the trim edges. Each starts at the bleed edge, that is at max(3 mm, bleed) from the trim, and is 5 mm long. Shorten to 3 mm when the slug is tight; omit below 2 mm | 0.25 pt, solid, butt caps |
| Fold ticks | At crease A and crease B, above and below the spread. Same start and length as crop ticks | 0.25 pt, dashed 1 mm on, 1 mm off, so that they cannot be taken for cut marks |
| Job line | One line of 6 pt text in the bottom slug, at least 1 mm outside the bleed: job reference, "230 × 178 mm, 2 rainages à 110 et 120 mm du bord gauche, livré à plat", profile name, date | 100 % K |
| Registration targets, colour bar | Off by default. No surveyed printer asks for them; Clays asks for crop and spine-width marks, Baumer for crop marks | — |
| Slug | 10 mm beyond the bleed on all sides | — |

Colour of the ticks: registration colour (all separations) for offset; 100 % K when the job is known to be digital [M]. Saxoprint warns that registration colour means 400 % ink and must never be used inside the printed area ([Saxoprint colours](https://www.saxoprint.de/service-und-hilfe/fehlerdiagnose/farben)) [V1]. No mark may enter the BleedBox.

The dashed style for fold ticks is a convention from memory; no printer page confirming it was retrieved [M].

### 2.5 How crease lines are communicated

There is no standard. The printers found disagree, including on colours.

| Printer or guide | Convention | Conf. |
|---|---|---|
| Pixartprinting instruction sheet (FR) | Cut, crease and fold lines as a **spot colour set to overprint** ([copy of the instruction sheet](https://www.cipherbliss.com/dessins/Qzines/maquettes/imprimeur%20docs/pixartprinting_instruction_7_fr.pdf)) | V1 |
| Papeo amalgam tutorial (FR) | **Magenta** line = *rainage* or *pli*, cyan = pre-cut, black = cut ([papeo](https://upload.prod.papeo.fr/media/tuto-pdf/tuto-imprimerie-amalgame.pdf)) | V1 |
| Pinguindruck folder data sheet (DE) | Cut edges **magenta**, crease lines **cyan** ([data sheet](https://pinguindruck.de/load/dateien/mappe_09-12_4s2ka6_datenblatt.pdf)) | V1 |
| German printer rules | Crease lines as an overprinting spot colour named "rillen", or shown in a separate view file; vector lines on a separate layer (result set: [printprinz](https://www.printprinz.de/support/weiterverarbeitung), [lazi-akademie](https://www.lazi-akademie.de/wiki/drucktechnik/weiterverarbeitung/rillung/)) | V1 |
| EPFL Repro guide | Cut path as a magenta spot colour named `CutContour` ([EPFL](https://www.epfl.ch/campus/services/repro/wp-content/uploads/2025/11/REPRO-Guide-PAO.pdf)) | V1 |
| WTTB (UK) folders | **Remove the "Guide" layer** before export: the printer applies creases from its own die ([WTTB](https://wttb.co.uk/support-guides/presentation-folder-styles-setup)) | V1 |

Magenta means "crease" at Papeo and "cut" at Pinguindruck. A fixed colour would be wrong at one of them. The generator therefore supports three encodings, chosen by the printer profile:

| Mode | What is written | When |
|---|---|---|
| `ticks` | Fold ticks outside the bleed (§2.4). Nothing inside the page | Default of the marks profile |
| `spot` | A stroke along each crease in a named separation set to overprint. Name and preview colour come from the profile (`rillen`, `Crease`, `Rainage`, `CutContour`). No layer in PDF/X-1a and X-3 files | Only for a printer whose template asks for it |
| `plan` | Nothing in the print file. A second PDF, marked "NE PAS IMPRIMER / DO NOT PRINT", shows trim, bleed, safety zones and both creases with dimensions | Always generated for professional orders; the only carrier of crease positions for the upload profile |

---

## 3. Colour management and PDF/X

### 3.1 What printers publish: PDF flavour, colour, ink (as of 2026-11)

| Printer | PDF standard or version | Colour and output condition | Total ink | Black rules | Sources, confidence |
|---|---|---|---|---|---|
| Onlineprinters | PDF/X-4:2008; identical page sizes; no password | CMYK; FOGRA51 (PSO Coated v3) for coated, FOGRA52 for uncoated | 300 % in normal production, 260 % for rush or express | black text under 12 pt is set to overprint automatically | [PDF settings FAQ](https://onlineprinters.ie/c/faq/settings-print-pdf), product pages of §2.3, [Überdrucken](https://www.onlineprinters.de/c/druckdaten/ueberdrucken) [V1 each]; the ink figure is from an onlineprinters.de page surfaced in the same search, exact page not pinned [V1] |
| Flyeralarm | PDF/X-3:2002 (no transparency); PDF/X-4 on some data sheets | "ISO Coated V2 300% (ECI)" in general, "ISO Coated V2 (ECI)" for large-format digital; the profile must be set as output intent | 300 %, implied by the profile | — | [checklist](https://www.flyeralarm.com/blog/wp-content/uploads/Flyeralarm_Checkliste_Druckdatenerstellung-1.pdf), [UK requirements](https://www.flyeralarm.com/images/upload/content/_pdf/uk/Artwork_Requirements_UK.pdf), [PDF/X article](https://flyeralarm.com/blog/en?p=24986) [V2] |
| Saxoprint | PDF/X-4 preferred, or PDF 1.6; avoid layers (Optional Content); keep native transparency | CMYK; PSO Coated v3 (FOGRA51) for coated | 300 % guaranteed maximum in offset | black text 100 % K, not four-colour | [guide 2024](https://api.saxoprint.de/media-by-file-name/Druckdatenanleitung-1.pdf), [UK instructions](https://www.saxoprint.co.uk/service-help/artwork-instructions), [colours](https://www.saxoprint.de/service-und-hilfe/fehlerdiagnose/farben) [V2 for the PDF flavour; V1 for ink and profile] |
| Exaprint | "PDF 1.3 optimal" | an old guide names ISO Coated V2 300 %; Exaprint supplies its own profiles | 300 %, excess corrected automatically | rich black for solids = K100 + C30 M30 Y30; text in K only; K100 always overprints | [Guide PAO PDF](https://static.exaprint.fr/marketing/cms/exaprint/guidePAO/GuidePAO-telechargement.pdf), [assistance PAO](https://www.exaprint.fr/page/assistance-pao), [conseils PAO](https://www.exaprint.fr/conseils-pao/), [brochure errors](https://blog.exaprint.fr/impression-brochure-erreurs-pao/): ink [V2]; the guide PDF is roughly 9 years old |
| Pixartprinting | PDF at 1:1 scale, not password-protected | CMYK, FOGRA39; non-CMYK files are converted automatically with a separation profile; Pantone not used in process printing | not found | text 100 % K; tints never below 5 % | [perfect binding spec](https://www.pixartprinting.co.uk/printing-magazines-books-catalogues/special-finish-perfect-binding/), [CMYK article](https://support.pixartprinting.com/hc/en-us/articles/360017440400-What-does-CMYK-mean), [inks advice](https://www.pixartprinting.co.uk/blog/inks-advices/), [2017 third-party checklist](https://printicom.wordpress.com/2017/07/06/final-checklist-before-uploading-your-documents/) [V2 for FOGRA39] |
| Printoclock | PDF, JPEG or TIFF; flatten images, outline fonts | CMJN preferred, RVB accepted | no number; warns against heavy ink, especially black solids | — | [bien réaliser son fichier](https://www.printoclock.com/bien-realiser-son-fichier) [V1] |
| Bookvault | PDF/X-1a:2001 or PDF/X-3:2002; transparency flattened | not found | not found | — | [PDF/X article](https://help.bookvault.app/what-is-pdf/x-1a2001-and-pdf/x-32002), [validation checks](https://help.bookvault.app/what-does-the-bookvault-file-validation-check-for) [V2] |
| Lulu | print-ready PDF, fonts embedded, layers flattened, single cover spread, no password | CMYK "Coated GRACoL 2006" (a US condition) | never above 270 % | — | [guide](https://assets.lulu.com/media/guides/en/lulu-book-creation-guide.pdf), [cover upload](https://help.lulu.com/en/support/solutions/articles/64000282777-upload-your-cover-file) [V1] |
| BoD | flatten transparency before export | keep CMYK images in CMYK; profile not found | not found | — | [BoD guide](https://www.bod.ch/bodfiles/GLOBAL-Storage/documents/help-documents/bod-grafiken-und-farbmanagement.pdf) [V1] |
| Gelato | PDF/X-4 for paper products | third-party claim: sRGB uploads are converted by Gelato (unverified) | not found | — | [Gelato support](https://connect-support.gelato.com/en/articles/9858024-gelatocreate-what-file-standard-is-used-for-paper-based-production) [V1] |
| Helloprint, Vistaprint | PDF (also AI, EPS, JPEG, PNG) | CMYK | not found | — | [Helloprint design solutions](https://www.helloprint.com/en-ie/design-solutions), [Vistaprint](https://www.vistaprint.com/hub/business-card-file-prep/) [V1] |

Spot colours: no general guide wants them for standard four-colour products. Pixartprinting uses a spot swatch only as a technical separation for finishes ("Finishing" = 0/50/100/0, vector, overprint) ([text for special finishes](https://support.pixartprinting.com/hc/en-us/articles/27655380376466-How-to-properly-set-up-text-for-special-finishes)) [V1].

File size limits: one figure only, Pixartprinting 1 GB per file, from a 2017 third-party post ([source](https://printicom.wordpress.com/2017/07/06/final-checklist-before-uploading-your-documents/)) [V1, dated]. A jacket PDF of a few megabytes is far below any plausible limit.

### 3.2 Decision: one content subset, a label per printer

**Content subset (all professional profiles).**

- Colour: DeviceCMYK and DeviceGray only. No DeviceRGB, no Lab, no spot colour. The one exception is the registration colour of marks, outside the bleed.
- No transparency of any kind: no soft mask, no opacity below 1, no blend mode, no transparency group.
- No layers (Optional Content): Saxoprint says to avoid them, Lulu and Bookvault want them flattened [V1 to V2].
- No annotation, form field, action, script, embedded file or encryption (Onlineprinters, Pixartprinting and Lulu refuse password protection [V1 each]).
- Fonts embedded as subsets (§4.4).
- 8-bit images, DCT or Flate compressed. No JPEG 2000, no 16-bit samples.
- File structure limited to PDF 1.3 features (no object streams, no cross-reference streams), so the same body can carry any of the three labels.

Such a file is valid content for PDF/X-1a:2001, PDF/X-3:2002 and PDF/X-4 at once [M for the standards' exact clauses; the printers' wording in §3.1 is consistent with it].

**Label.**

| Label | PDF version | Use it for | Reason |
|---|---|---|---|
| PDF/X-4 (default) | 1.6 | generic download, Onlineprinters, Saxoprint, Gelato, Baumer-type printers, Pixartprinting, Helloprint | It is what the printers with the most explicit current guides ask for [V1 to V2]. It requires the embedded profile, which we always write |
| PDF/X-3:2002 | 1.3 | Flyeralarm | Named on its checklist [V2] |
| PDF/X-1a:2001 | 1.3 | Bookvault, Exaprint ("PDF 1.3 optimal"), Printoclock (any PDF accepted, text outlined), any printer that rejects the X-4 label | The most conservative label; CMYK-only content is exactly what it describes |
| Plain PDF | 1.4 or later | home profile | No output condition applies to a home printer |

Why not PDF/X-1a everywhere: it would be accepted by every printer we know of [M], but it forbids the tagged-RGB variant of §3.4 and it is not what the German groups and Gelato publish. Why not X-4 everywhere: Bookvault and Flyeralarm name older parts, and a validator that checks the label literally would fail the file. The label is one field in the printer profile; the page content does not change.

Transparency is forbidden by PDF/X-3:2002 (Flyeralarm) and must be flattened for Bookvault, Lulu and BoD; Saxoprint and Onlineprinters keep it native under PDF/X-4 [V2]. PDF/X-4 may contain live transparency and layers, unlike X-1a and X-3 ([Cuno Druck PDF/X sheet](https://www.cunodruck.de/wp-content/uploads/2023/11/PDF-X-GCC_23.pdf)) [V1]. We use neither, so the difference does not affect us.

### 3.3 Output intent and PDF/X keys

Verified this session: which conditions printers ask for (§3.1), the exact Flyeralarm wording "ISO Coated V2 300% (ECI)", and that the profile must be present as the PDF output intent [V2]. **Everything else in this subsection is [M]** and must be checked against ISO 15930 and the ICC characterisation registry before release.

OutputIntent dictionary:

| Key | Value | Note |
|---|---|---|
| `/Type` | `/OutputIntent` | |
| `/S` | `/GTS_PDFX` | |
| `/OutputConditionIdentifier` | `(FOGRA39)`, `(FOGRA51)` or `(FOGRA52)` | Required. Should be the reference name in the ICC registry |
| `/RegistryName` | `(http://www.color.org)` | Present when the identifier is a registered characterisation |
| `/Info` | e.g. `(ISO Coated v2 300% (ECI))` | Human-readable; required when the identifier is not registered. We always write it |
| `/OutputCondition` | free text, e.g. `(Offset, premium coated, 115 g/m2, ISO 12647-2:2004)` | Optional |
| `/DestOutputProfile` | ICC stream, `/N 4` | PDF/X-1a and X-3 allow it to be omitted for a registered condition; PDF/X-4 requires it. We always embed it, because printer preflights look for it |

Other PDF/X requirements:

| Requirement | X-1a:2001 | X-3:2002 | X-4 |
|---|---|---|---|
| Version marker | Info: `GTS_PDFXVersion (PDF/X-1:2001)` and `GTS_PDFXConformance (PDF/X-1a:2001)` | Info: `GTS_PDFXVersion (PDF/X-3:2002)` | XMP: `pdfxid:GTS_PDFXVersion` = `PDF/X-4` |
| PDF header | 1.3 | 1.3 | 1.6 |
| XMP metadata | optional | optional | required, consistent with the Info dictionary |
| `Trapped` | `/True` or `/False`, never unknown. We write `/False` | same | same |
| `Title`, creation and modification dates, trailer `/ID` | required | required | required |
| Embedded ICC profile version | ICC v2 only | ICC v2 only | v2 or v4 |

One exactly-one rule applies to all three: a single `GTS_PDFX` output intent per file.

### 3.4 Colour conversion policy

| Object | Rule | Reason |
|---|---|---|
| Cover image, professional profiles | Converted from sRGB to the output-intent CMYK with a real ICC transform. Intent: relative colorimetric with black-point compensation by default, perceptual as an option [M]. Never the naive formula | Document 07, rule 5: no colour change other than ICC conversion to the output profile. A 300 % profile also guarantees the ink limit |
| Cover image, when the output profile is not available in the browser | Left as sRGB, tagged with the sRGB profile, in a PDF/X-4 file. Preflight warning "RGB image, the printer converts". Allowed only in profiles with `allow_rgb` | Pixartprinting converts non-CMYK files automatically [V2]; Printoclock accepts RVB [V1]; Gelato is said to convert sRGB (unverified). Onlineprinters, Saxoprint, Flyeralarm and Exaprint expect CMYK |
| Ground and spine colours | Defined in sRGB for the preview, converted through the same ICC transform, then cleaned: channels below 2.5 % go to 0, channels between 2.5 % and 5 % go to 5 % (house rule) | Preview and print stay consistent. Pixartprinting: no tint below 5 % [V1] |
| Black text, typed code, QR symbol, thin rules | Written directly as 0/0/0/100, never through the ICC transform | An ICC conversion of sRGB black gives a four-colour black. Printers want text in K only [V2] |
| White | 0/0/0/0 | |
| Home profile | Images stay sRGB (JPEG passed through unchanged); text in DeviceGray | Home printer drivers work in RGB [M] |

The ICC transform needs a colour-management module that reads LUT-based output profiles (B2A tables). A matrix-only module, sufficient for sRGB, cannot do it [M]. The existing `to_cmyk_naive` function is documented as not colour-managed and must not be applied to cover images.

A CMYK output profile is specific to a paper class. FOGRA39 and FOGRA51 are coated conditions; uncoated board (the ESG preference, §6.5) calls for FOGRA52 at printers that publish it (Onlineprinters) [V1]. For a quote job the printer must be asked which condition it runs.

### 3.5 ICC profiles and their licences

**All of this subsection is [M].** No search could be run on licence terms in either the research session or this synthesis.

| Profile (file) | Characterisation | Paper | Total ink | Use |
|---|---|---|---|---|
| ISO Coated v2 300% (ECI), `ISOcoated_v2_300_eci.icc` | FOGRA39L | coated, ISO 12647-2:2004 paper types 1 and 2 | 300 % | **default output intent** |
| ISO Coated v2 (ECI), `ISOcoated_v2_eci.icc` | FOGRA39L | same | 330 % | not used: exceeds the 300 % ceiling |
| PSO Coated v3, `PSOcoated_v3.icc` | FOGRA51 | premium coated, ISO 12647-2:2013 PS1, M1 measurement | 300 % | Onlineprinters, Saxoprint |
| PSO Uncoated v3 (FOGRA52), `PSOuncoated_v3_FOGRA52.icc` | FOGRA52 | wood-free uncoated, PS5 | 300 % | uncoated board at printers that name it |

| Source | Licence as recalled | Decision |
|---|---|---|
| ECI (eci.org) | Free of charge; shipped with a zlib/libpng-style licence text: use, alteration and redistribution permitted, origin not to be misrepresented, altered versions marked, notice kept. Compatible with a BSD-3-Clause repository if the licence file travels with the profile | Use these, **after reading the licence file inside the ECI download**. Until then, do not commit them and do not serve them from our host |
| basICColor variants of the same characterisations | The prepress note recalls no explicit redistribution grant. I recall that Debian ships basICColor-built files such as `ISOcoated_v2_300_bas.icc` in its free archive (package `icc-profiles-free`), which would imply a free licence. The two recollections disagree | Unresolved. The Debian package's copyright file is the quickest check |
| Adobe (Coated FOGRA39 and others) | Free download; redistribution only under Adobe's separate bundling licence | Do not bundle |
| CC0 "Compact ICC Profiles" already in `assets/icc/` | Public domain. The CMYK one is a US SWOP-compatible profile (CGATS TR 001) | Tests and last-resort fallback only. It is not a European condition and no surveyed EU printer names it |

Practical points:

- Embedding the profile in the PDF's output intent is the profile's intended use. Serving the `.icc` file to the browser is redistribution, so the licence check applies to the web application, not only to the repository.
- Each ECI CMYK profile is about 1.8 to 2.4 MB [M]. Load it on demand, when the user chooses a professional output, and cache it. The home path does not need it.
- The ECI offset profiles are ICC version 2 files [M], so they can be embedded in PDF 1.3 (X-1a, X-3) and PDF 1.6 (X-4) files alike. Check the header version at load time.
- If the licence check fails, the fallback is the tagged-sRGB PDF/X-4 path of §3.4 for printers that accept it, and a PDF/X-1a or X-3 file that names the registered condition without embedding the profile for the others. That second form is permitted by the standard [M] but some printer preflights will reject it, and the cover could then not be converted properly. It is a degraded mode.

### 3.6 Seed data for per-printer profiles

To be stored as data, one record per printer and product. "Used" is the value our layout applies; it is never looser than the published one. **Every row is unconfirmed until checked against the product template.**

| Printer | Bleed | Safety (published / used) | Marks | Label | Output intent | Ink limit | Text | Open point |
|---|---|---|---|---|---|---|---|---|
| Exaprint (FR) | 3 mm | 3 / 5 mm | none | PDF/X-1a:2001 | FOGRA39, 300 % | 300 % | embedded | bleed 2 or 3 mm; current profile |
| Pixartprinting (IT) | 3 mm or template | template / 5 mm | none | PDF/X-4 | FOGRA39 | 300 % assumed | embedded, not outlined | ink limit not published |
| Printoclock (FR) | 2 or 5 mm per template | 2 / 5 mm | none | PDF/X-1a:2001 (any PDF accepted) | FOGRA39 assumed | 300 % assumed | all outlined | bleed; profile |
| Onlineprinters (DE) | 2 mm | 4 / 5 mm | none | PDF/X-4 | FOGRA51 coated, FOGRA52 uncoated | 300 %, 260 % express | embedded | mark policy not found |
| Saxoprint (DE) | 2 mm, 3 mm on folders | 3 / 5 mm | none | PDF/X-4 | FOGRA51 | 300 % | embedded | mark policy not found |
| Flyeralarm (DE) | per data sheet (1 to 3 mm) | data sheet / 5 mm | none | PDF/X-3:2002 | FOGRA39, 300 % | 300 % | embedded | data sheet of the folder product |
| Helloprint (NL) | 3 mm | 3 to 5 / 5 mm | unknown: none | PDF/X-4 | FOGRA39 assumed | 300 % assumed | embedded | profile, ink limit, marks |
| Bookvault (UK) | 3 mm | 5 / 5 mm | none | PDF/X-1a:2001 | not found | not found | embedded | profile |
| Gelato | 3 mm (unverified) | not found / 5 mm | none | PDF/X-4 | not found; may accept sRGB | not found | embedded | first-party specification |
| Cloudprinter | 3 mm | not found / 5 mm | none | PDF/X-4 assumed | not found | not found | embedded | first-party specification |
| Lulu | 3.175 mm | 12.7 / 12.7 mm | none | any print-ready PDF | GRACoL 2006 (US) | 270 % | embedded | out of the default scope |
| BoD | 5 mm | not found | none | flattened PDF | not found | not found | embedded | current guide |

Where the mark policy was not found (Onlineprinters, Saxoprint, Exaprint, Helloprint), "none" is used because their product pages define the upload as the data format with bleed [V2 for Onlineprinters].

---

## 4. Fonts, images, resolution, ink limits

### 4.1 Published minimums

| Item | Published values | Confidence |
|---|---|---|
| Positive line (dark on light) | 0.25 pt = 0.09 mm at Onlineprinters ([Linienstärke](https://www.onlineprinters.de/c/druckdaten/linienstaerke)) and Flyeralarm ([UK requirements](https://www.flyeralarm.com/images/upload/content/_pdf/uk/Artwork_Requirements_UK.pdf)); 0.25 pt at Pixartprinting ([spec](https://www.pixartprinting.co.uk/printing-magazines-books-catalogues/special-finish-perfect-binding/)); Saxoprint 0.125 pt at 40 % ink or more, thinner lines are thickened automatically except 0 pt hairlines ([guide](https://api.saxoprint.de/media-by-file-name/Druckdatenanleitung-1.pdf)), while its blog recommends 0.75 pt ([blog](https://www.saxoprint.de/blog/druckvorlagen/schriften-und-linien)) | [V2] for 0.25 pt |
| Reversed line (light on dark) | 0.5 pt = 0.18 mm at Onlineprinters and Flyeralarm (same sources) | [V2] |
| Minimum font size | 6 pt at Onlineprinters ([Schriften](https://www.onlineprinters.de/c/data/schriften)), Flyeralarm ([checklist](https://www.flyeralarm.com/blog/wp-content/uploads/Flyeralarm_Checkliste_Druckdatenerstellung-1.pdf)), Saxoprint, Pixartprinting | [V2] |
| Stricter cases | Pixartprinting special finishes: lines 1 pt, type 10 pt; rigid media: lines 0.5 pt, type 10 pt ([legible text](https://support.pixartprinting.com/hc/en-us/articles/27654779357970-How-to-ensure-the-text-is-the-correct-size-to-be-legible-once-printed)); Saxoprint foil on packaging: 12 pt | [V1] |
| Reversed or four-colour text | no printer figure found; trade rule of thumb is 8 pt or more in a sturdy weight | [M] |
| Image resolution | 300 dpi at Pixartprinting, Onlineprinters, Printoclock, Bookvault, Vistaprint; 300 to 356 dpi at Flyeralarm; at least 250 dpi at Saxoprint; 100 to 150 dpi for large format | [V2] |
| Fonts | Pixartprinting: embedded, **not** outlined; Bookvault and Lulu: embedded, validated; Printoclock: outlined ("vectoriser") | [V1 each] |
| Overprint | Onlineprinters sets black text under 12 pt to overprint; Exaprint overprints all K100 | [V1 each] |

### 4.2 Images and resolution

Pixel budget for the cover on the poche format [calc]:

| Placement | Image box | 300 ppi (good) | 250 ppi (warning below) | 150 ppi (error below) |
|---|---|---|---|---|
| Default: inset 5 mm on a flat ground | 100 × 168 mm | 1181 × 1984 px | 984 × 1654 px | 591 × 992 px |
| Full bleed (`trim_authorised` only) | 113 × 184 mm | 1335 × 2174 px | 1112 × 1811 px | 667 × 1087 px |

The image is letterboxed in the box, so only one of the two dimensions is binding. For the usual eBook cover ratios of 1.5 to 1.6 [M] the width binds: **1,181 px wide is the target, 591 px the floor**.

Consequences:

- Retailer thumbnails 500 to 800 px wide give about 127 to 203 ppi on the inset box and 112 to 180 ppi on a full-bleed front [calc]. A 500 px image is an error; an 800 px image is a warning.
- Resolution is measured on the placed image (pixels divided by the printed size), on both axes, not read from the file's metadata.
- Upscaling does not add detail. If an upscaler is used, preflight must still report the source resolution, and the user must see the warning.
- Below the error threshold a professional order is blocked and the typographic card is offered instead. The home profile is more tolerant (200 / 150 / 100 ppi) because home devices and viewing expectations are lower (house rule).
- Images are 8-bit, without alpha. CMYK images are written without an embedded profile: they are already in the output-intent space.
- When a CMYK image is stored as JPEG, the writer must control the Adobe inversion convention (either write non-inverted data or add the matching `/Decode` array), and a test must render the page and compare colours. Getting this wrong produces a negative image [M].

### 4.3 Ink limits and black

| Rule | Value | Source |
|---|---|---|
| Total ink ceiling | 300 % | Onlineprinters, Saxoprint, Exaprint; Flyeralarm through its "ISO Coated V2 300 %" profile [V2] |
| Stricter products | 260 % for Onlineprinters express; 270 % for Lulu | [V1 each] |
| How the ceiling is met for images | by converting with a 300 % profile (§3.5), not by editing pixels afterwards | [M]; document 07 rule 5 |
| How it is met for flat colours | computed colours are checked and, if above the limit, reduced by scaling C, M and Y | house rule |
| Text, typed code, QR symbol | 0/0/0/100 | Saxoprint, Exaprint, Pixartprinting [V2] |
| Large black solids | K100 + C30 M30 Y30 | Exaprint [V1] |
| Solids crossing a crease | prefer a mid or light ground; a black ground that crosses a crease is K100 only, not rich black (house rule) | Print Pas Cher: with heavy ink coverage the paper may tear in the folds, and this is not a ground for complaint ([printpascher.com](https://www.printpascher.com/chemise-rabats.html)) [V1]; Printoclock warns against heavy black solids [V1] |
| Lightest tint | 5 % | Pixartprinting [V1] |
| Registration colour | marks only, outside the bleed | Saxoprint [V1] |
| Overprint | K100 text on a coloured ground is set to overprint; nothing else is. A white object must never overprint | Onlineprinters and Exaprint do this themselves [V1 each]; the white-object rule is standard preflight practice [M] |

A 260 % or 270 % limit cannot be met by a 300 % profile. For those two cases the options are a profile with a lower ink limit or a device-link transform; neither is in scope. Do not offer Onlineprinters express until this is solved.

### 4.4 Fonts and text

- Embed every font as a subset. This satisfies Pixartprinting, Bookvault and Lulu [V1 each] and is required by every PDF/X part [M].
- The spine title is drawn as outlines (decision of `08-legal-fonts-and-genai.md`). To a printer's preflight this is vector artwork, which every printer accepts.
- A profile flag `outline_all_text` converts all text to outlines for Printoclock [V1]. Cost: no selectable text, slightly heavier small type.
- Only fonts whose `fsType` allows embedding are used (already enforced in `ebc-text`). The bundled fonts are OFL 1.1 with `fsType` 0 (document 08).
- Sizes: nothing below 6 pt [V2]. Reversed or multi-ink text 8 pt or more, in a regular or heavier weight [M]. Inside-panel steps 12 pt or more, typed code 16 pt bold, all K100 (`notes/ereader-ux.ux.md` §12).
- Positive rules 0.25 pt or more, reversed rules 0.5 pt or more, no zero-width hairline [V2].
- Outlined small text is subject to the line-weight minimums: an outlined 6 pt light face has strokes under 0.25 pt. Keep outlined text at 8 pt or more.

---

## 5. QR code print rules

### 5.1 Facts

| Fact | Source | Conf. |
|---|---|---|
| The quiet zone is 4 modules: "the minimum according to the specs"; a second library defaults to four light modules | [python-qrcode README](https://raw.githubusercontent.com/lincolnloop/python-qrcode/main/README.rst), [segno docs](https://raw.githubusercontent.com/heuer/segno/master/docs/make.rst) (via `notes/ereader-ux.ux.md`) | [V2]; the ISO/IEC 18004 text itself was not read |
| Error-correction levels recover about 7 % (L), 15 % (M), 25 % (Q), 30 % (H) | python-qrcode README | [V1] |
| Alphanumeric mode covers digits, upper-case A to Z, space and `$%*+-./:`; lower case forces byte mode | [segno modes](https://raw.githubusercontent.com/heuer/segno/master/docs/qrcode-modes.rst); [BIP-173](https://raw.githubusercontent.com/bitcoin/bips/master/bip-0173.mediawiki) recommends upper case inside QR codes | [V1 each] |
| An encoder can raise the error-correction level at no cost when the version does not change | [Nayuki README](https://raw.githubusercontent.com/nayuki/QR-Code-generator/master/Readme.markdown), segno docs | [V2] |
| ZXing parses an upper-case `HTTPS://` payload as a URL | [URIResultParser.java](https://raw.githubusercontent.com/zxing/zxing/master/core/src/main/java/com/google/zxing/client/result/URIResultParser.java) | [V1]; stock iOS and Android cameras untested [M] |
| Fine elements and black text should be K only; positive detail 0.25 pt or more; cutting shift 1 to 2 mm | §3.1, §4.1, §2.3 | [V2], [V2], [V1] |
| Module size of at least 0.4 mm for phone scanning (GS1: about 0.396 mm minimum, 0.495 mm target) | prepress note | [M] |
| Symbol width of about one tenth of the scanning distance | prepress note, UX note | [M] |
| Symbol contrast grades of ISO/IEC 15415 (40 % for grade C, 70 % for grade A) | prepress note | [M] |

Symbol versions for our payloads, computed with an encoder in the UX note [calc]:

| Payload (upper case, alphanumeric) | Characters | Version at M | Version at Q | Modules per side at Q |
|---|---|---|---|---|
| `HTTPS://XXX.FR/K/` + 12-symbol code | 29 | 2 | 2 | 25 |
| `HTTPS://EBOOK-CADEAU.FR/K/` + code | 38 | 2 | 3 | 29 |
| `HTTPS://WWW.FNAC.COM/EBOOK-CADEAU/` + code | 46 | 3 | 3 | 29 |
| The 29-character payload in lower case (byte mode) | 29 | 3 | 3 | 29 |

### 5.2 Rules for the generator

1. **Payload**: upper-case `HTTPS://` + host + short path + code, no query string, so that it stays in alphanumeric mode.
2. **Level**: request M and let the encoder raise it to Q when the version does not change. Never H: it is for logo overlays, which we do not use.
3. **Version**: 4 (33 modules) at most. A larger symbol means the URL has grown; preflight warns.
4. **Quiet zone**: 4 modules on all four sides, white, with nothing else in it.
5. **Size**: the symbol and its quiet zone are fitted into a square box. Default box 33 mm; allowed 26 to 40 mm.

   | Version | Modules + quiet zone | Module at a 33 mm box | Symbol width | Smallest box for 0.5 mm modules |
   |---|---|---|---|---|
   | 2 | 25 + 8 = 33 | 1.00 mm | 25.0 mm | 16.5 mm |
   | 3 | 29 + 8 = 37 | 0.89 mm | 25.9 mm | 18.5 mm |
   | 4 | 33 + 8 = 41 | 0.80 mm | 26.6 mm | 20.5 mm |

   All [calc]. The 33 mm default satisfies both notes: symbol of 25 mm or more (prepress) and 20 mm or more (UX).
6. **Module size**: error below 0.4 mm, warning below 0.5 mm [M]. At 0.5 mm a module is about 6 device dots at 300 dpi [calc].
7. **Ink**: dark modules 0/0/0/100, drawn as vector rectangles merged into a single filled path (adjacent rectangles drawn separately can show hairline gaps in some renderers [M]). Light modules and quiet zone are paper white, 0/0/0/0. Never a four-colour black, never a reversed (light on dark) symbol.
8. **Place**: on the inside page, never on the cover panel (document 07, rule 7), never on or across the spine. Box edges 5 mm or more from the trim and from each crease.
9. **Clear text**: print the host and the typed code next to the symbol. It is the fallback without a phone and an accessibility requirement (`notes/legal-gifting-commerce.commerce.md`).
10. **Surface**: matt, uncoated or lightly coated stock scans better than gloss film [M, ESG note]. No varnish or foil over the symbol.
11. **Trade mark**: if the words "QR Code" are printed, add the Denso Wave acknowledgement line (document 07, rule 16).
12. **Secrecy**: the uploaded PDF shows the redemption code to the printer's staff and systems. Codes in professionally printed cards should stay inactive until the seller or buyer activates them (design suggestion from `notes/printers.web2print.md`, not a researched fact).

### 5.3 Verification

- Automated: render the inside page at 150 and at 300 dpi, decode the symbol with an independent decoder, and compare the payload with the expected URL. Do the same on a page scaled to 97 % to cover "fit to page" printing.
- Manual, once per stock and printer: scan a printed sample with at least three phones, including an old one, in dim light.

---
## 6. The spine and creases

### 6.1 Creasing, scoring, grain

| Topic | Finding | Conf. |
|---|---|---|
| Vocabulary | *Creasing* (FR *rainage*, DE *Rillung*) compresses the fibres into a hinge without cutting the surface; *scoring* cuts or ploughs a shallow channel (DE *Nuten* removes material); *folding* (FR *pliage*, DE *Falzen*) is the bend itself. Creasing is recommended for heavy coated and digitally printed sheets; scoring is adequate for lighter uncoated stock ([Mid-State Litho](https://blog.midstatelitho.com/creasing-vs-scoring/), [druckerei.de glossary](https://druckerei.de/glossar/nuten/)) | V2 |
| Threshold | Gmund: creasing recommended from **170 g/m²**, fold parallel to the fibres ([Gmund digital printing tips, FR](https://gmund.com/shop/media/pdf/instructions/Gmund_OG_mit_Digital_Printing_Tips_fr.pdf)). Heidelberg describes a scoring set-up "ideal for 170 gsm or more", other tooling up to 350 gsm ([Heidelberg tips](https://www.heidelberg.com/global/media/en/global_media/company___publications/heidelberg_news/tips_tricks/279_tips_tricks_de.pdf)) | V2 |
| Counter-rule | Mid-State Litho: coated stock above 80 lb cover (about 216 g/m²) should be creased; uncoated can be scored up to 100 lb cover ([source](https://blog.midstatelitho.com/creasing-vs-scoring/)). So 170 g/m² is a manufacturer's rule of thumb, not a standard | V1 |
| Grain | Paper folds cleanly with the grain and cracks across it. Digital presses often force cross-grain folds because of the sheet size, which makes creasing more important ([CreativePro](https://creativepro.com/paper-tips-going-against-the-grain/), [Gmund](https://gmund.com/shop/media/pdf/instructions/Gmund_OG_mit_Digital_Printing_Tips_fr.pdf)) | V2 |
| Ink at the fold | With heavy ink coverage the paper may tear in the folds; one French printer excludes this from complaints ([printpascher.com](https://www.printpascher.com/chemise-rabats.html)) | V1 |
| Varnish at the fold | Keep spot varnish off crease lines: about 1.5 mm clear at one German printer ([printprinz Veredelungen](https://www.printprinz.de/support/veredelungen)); same advice in a French packaging template ([Smilepack](https://smilepack.fr/files/subscribers/d72b0b53-b3d0-49ff-adfd-37d51546ebde/ftp/gabarits/Coussin%20technique%20smilepack.pdf)) | V2 |

Both creases run parallel to the spine, so the grain must run parallel to the spine, top to bottom of the cover.

### 6.2 Minimum distance between two parallel creases

| Source | Rule | Conf. |
|---|---|---|
| printing.com (UK) folder file guide | On a folder with capacity, **parallel creases must be at least 5 mm apart**; a crease must be at least 5 mm from a parallel die-cut line; gussets above 5 mm in whole millimetres ([printing.com](https://www.printing.com/uk/file-supply/how-to-supply-presentation-folder-files); identical text at the sister brand [Nettl](https://www.nettl.com/uk/shop/file-supply/how-to-supply-presentation-folder-files)) | V1 |
| Creasing machines | The hardware minimum is far lower: 0.1 mm on the Formax Atlas C200 and the Morgana AutoCreaser 50 ([Formax](https://manualedistruzioni.it/formax/binding-machine/atlas-c200/user-manual/v177), [Morgana](https://manuals.online/morgana/binding-machine/autocreaser-50/user-manual/1swt6)); 1.0 mm steps on Konica Minolta inline creasers ([KM manual](https://manuals.konicaminolta.eu/ineo-plus-2060L-2060-2070/EN/contents/id04-_102709920.html)) | V2 |
| Edge distance | Morgana DigiFold Pro: minimum crease distance from the tail edge 50 mm ([manual](https://media.mybinding.com/media/manuals/Morgana-DigiFold-Pro-User-Manual.pdf)); rule of thumb 5 mm between a crease and the sheet edge ([lazi-akademie wiki](https://www.lazi-akademie.de/wiki/drucktechnik/weiterverarbeitung/rillung/)) | V1 each |
| Machine *folding* | One German online printer gives 50 mm minimum fold to fold and 50 mm to the final format for machine folds (result set: [printprinz Weiterverarbeitung](https://www.printprinz.de/support/weiterverarbeitung), [viperprint Falzflyer](https://viperprint.de/flyer/falzflyer); exact page not pinned) | V1 |
| Catalogue spines | Online printers sell folders with a double crease forming a 5 mm or 10 mm spine (§8) | V2 |

Reading: a 10 mm spine made of two creases is twice the 5 mm practical minimum and is a catalogue-standard geometry. It cannot be produced as a machine *fold*, because the middle panel is far below 50 mm. It must be ordered as **crease only, delivered flat** ("2 rainages, livré à plat"), or as a folder product.

### 6.3 Spine text: what platforms and printers allow

Platform cover templates (as of 2026-11):

| Platform | Bleed | Spine width formula | Spine text allowed from | Spine text side margin | Hinge and other |
|---|---|---|---|---|---|
| Amazon KDP paperback | 0.125 in (3.2 mm) top, bottom, outside; cover text 0.125 in or more inside the trim ([KDP help](https://kdp.amazon.com/en_US/help/topic/G201953020)) [V2] | pages × 0.002252 in (0.0572 mm) for white and standard colour; 0.002347 in (0.0596 mm) for premium colour ([KDP help](https://kdp.amazon.com/en_US/help/topic/G201953020)) [V1]; cream 0.0025 in per third parties ([kdpeasy](https://www.kdpeasy.com/fr/tools/calculateur-dos-kdp)) [V1] | 79 pages; KDP pages are inconsistent ("at least 79", "more than 79", Cover Creator says 80) ([KDP help](https://kdp.amazon.com/en_US/help/topic/A1L46HVPPIYTE), [ebookpbook](https://www.ebookpbook.com/2026/06/24/book-spine-text-requirements/)) [V2] | 0.0625 in (1.6 mm) each side [M], not confirmed by search | 0.4 in (10 mm) between the spine and the safe area on front and back ([KDP help](https://kdp.amazon.com/en_US/help/topic/G201953020)) [V1]; probably the hardcover hinge [M] |
| Lulu paperback | 0.125 in ([M] in the folding note; [V2] in the prepress note, §2.3) | pages / 444 + 0.06 in; pages / 460 + 0.06 in for magazine and comic paper ([Lulu API help](https://help.api.lulu.com/en/support/solutions/articles/64000254616-how-is-spine-width-calculated-)) [V1] | No spine text at 80 pages or fewer ([Lulu Book Creation Guide](https://assets.lulu.com/media/guides/en/lulu-book-creation-guide.pdf), read through a [Scribd copy](https://www.scribd.com/document/615961952/lulu-book-creation-guide)) [V1] | 0.125 in (3.2 mm) to each spine edge (same guide) [V1] | Text spanning the full spine is risky: small shifts push it onto the covers; spine width differs between Lulu's own network and its distribution partners ([Lulu help](https://help.lulu.com/en/support/solutions/articles/64000255462)) [V1] |
| IngramSpark | 0.125 in (3 mm) ([Ingram cover bleed sheet](https://marketing.ingramcontent.com/mrkng/documents/publishers/Cover-Bleed-Dimensions.pdf)) [V1] | Template generator only ([IngramSpark help](https://help.ingramspark.com/hc/en-us/articles/35303841585933-Cover-Template-Generator)) | Paperback 48 pages, hardcover 18 pages ([IngramSpark help](https://help.ingramspark.com/hc/en-us/articles/35306346735245-Cover-File-Creation), [ebookpbook](https://www.ebookpbook.com/2026/06/24/book-spine-text-requirements/)) [V2] | Spine of 0.35 in (9 mm) or more: 0.0625 in; thinner: 0.03125 in (about 1 mm) ([IngramSpark blog](https://www.ingramspark.com/blog/file-requirements-for-print-books), [Ingram dust-jacket template](https://marketing.ingramcontent.com/mrkng/documents/publishers/Dust-Jacket-Template.pdf)) [V2] | All text 0.25 in (6 mm) or more from the trim; duplex covers keep the spine + 0.0625 in each side white for the glue; a 0.5 in (13 mm) hinge is quoted for Ingram covers [V1] |
| BoD (DE/FR) | **5 mm** on the four outer sides ([BoD cover guide, about 11 years old](https://www.yumpu.com/de/document/view/41877228/wie-gestalte-ich-das-buchcover-richtig-bod)) [V1] | Online *Umschlagsberechnung*; minimum 24 pages (same guide) | 80 pages or a **5 mm** spine (same guide) [V1] | not found | Keep text 8 mm away from the fold area next to the spine (same guide) [V1] |
| Bookvault (UK) | 3 mm ([V2] in the prepress note, §2.3) | Sizing calculator and generated template ([Bookvault templates](https://bookvault.app/downloading-bookvault-templates/)) | "About 10 mm or more leaves room for spine text"; skip spine text on low page counts ([Bookvault help](https://help.bookvault.app/book-cover-spine-misaligned)) [V1] | not found | The help page treats slight cover and spine misalignment as normal production variance |

French, Swiss and German printers on spine text:

| Source | Rule | Conf. |
|---|---|---|
| Graphiscann (Lyon) | Title or logo on the spine from 5 mm, about 80 pages; perfect binding starts around 40 pages ([graphiscann.fr](https://www.graphiscann.fr/brochures-collees-dos-carre-impression.html)) | V1 |
| Infoprint (Montpellier) | Below 5 mm a spine title is hard; estimate = pages / 2 × grammage / 1000 (mm); 3 mm bleed; 5 mm unprinted reserve each side of the spine inside (glue does not hold on ink); a few millimetres of error visibly decentre the title ([article](https://www.montpellier-imprimeur.com/imprimer-un-titre-sur-la-tranche-ce-que-le-dos-peut-accueillir/), [checklist](https://www.montpellier-imprimeur.com/checklist-dos-carre-colle)) | V1 |
| EPFL Repro | Title only on a spine of 5 mm or more, **1.5 mm margin or more on each side**; a 5 mm spine carries a 2 mm cap height; comfort creases (*rainages de confort*) 6 mm either side of the spine; glue zone = spine + 9 mm left blank ([EPFL](https://www.epfl.ch/campus/services/repro/wp-content/uploads/2025/11/REPRO-Guide-PAO.pdf)) | V1 |
| Booksfactory.fr | No text on a spine under 6 mm ([booksfactory.fr](https://booksfactory.fr/article_info.php?articles_id=53)) | V1 |
| Veoprint | No elements on a spine under 4 mm ([Veoprint data sheet](https://www.veoprint.com/dl?t=template&id=34)) | V1 |
| Reset St. Pauli (DE) | Spine lettering sensible from 5 mm; a 3 mm spine usually gets no crease ([checklist](https://www.resetstpauli.de/checkliste-klebebindung/)) | V1 |
| printiteasy (DE) | A spine off by 2 mm is immediately visible on the cover ([printiteasy](https://printiteasy.eu/de/wissensdatenbank/druckvorlagen/)) | V1 |

Synthesis:

- "No spine text below about 5 mm" is the consensus, range 4 to 6 mm (KDP's 79 pages is about 4.5 mm, Lulu's 80 pages about 6.1 mm [calc]) [V2]. A 10 mm spine is about twice the threshold and matches Bookvault's comfortable figure.
- Side margins for spine text: 1.5 mm (EPFL), 1.6 mm (IngramSpark, spines of 9 mm or more), 3.2 mm (Lulu) [V1 each]. On 10 mm these leave a band of 7 mm, 6.8 mm and 3.6 mm [calc]. **We keep 2 mm each side and a 6 mm band.**
- Tolerance: generic spine guides quote about ±1 mm (result set including [pdfpress](https://www.pdfpress.app/blog/spine-width-calculator-guide) and [bookdesigner.ai](https://www.bookdesigner.ai/guides/book-spine-width); exact page not pinned) [V1]. Pixartprinting states a 1 to 2 mm cutting shift [V1].
- Hinge zones (5 mm at Infoprint, 8 mm at BoD, 10 to 13 mm on hardcovers at KDP and Ingram; [Mixam](https://mixam.co.uk/support/hardcoversetup) adds 5 mm each side) exist because of glue. The card has no glue, so no hinge zone applies. Optional "paperback look" hinge creases 6 mm outside the spine (EPFL) would respect the 5 mm spacing rule; they are not part of the default.

### 6.4 Spine geometry and tolerances for the card

Poche format, trim coordinates in millimetres:

```
x (mm)  0             105 110 112         118 120 125               225 230
        |             |   |   |           |   |   |                 |   |
        | back live   |   |cz | text band |cz |   | cover box       |   |
        | area        |   |   | 6 mm      |   |   | (inset 5 mm)    |   |
        trim              A                   B                         trim
```

A and B are the creases; cz is the 2 mm clear zone; 105 is the safety line of the back panel; 125 and 225 are the edges of the cover box.

| Zone | x from | x to | y from | y to |
|---|---|---|---|---|
| Back cover live area | 5 | 105 | 5 | 173 |
| Crease A | 110 | — | 0 | 178 |
| Spine clear zone | 110 | 112 | — | — |
| **Spine text band** | 112 | 118 | 5 | 173 |
| Spine clear zone | 118 | 120 | — | — |
| Crease B | 120 | — | 0 | 178 |
| Cover image box, default inset layout | 125 | 225 | 5 | 173 |
| Cover image box, full bleed (`trim_authorised`) | 120 | 233 | −3 | 181 |

Rules:

1. **Text band.** Spine text sits in the central 6 mm, 168 mm long. The band is 17.0 pt wide: one line of all-caps text up to about 24 pt, or mixed case up to about 17 pt [calc, assuming a cap height of 0.7 em and ascender plus descender of 0.95 em]. Floor 8 pt (§4.4). The text is centred on x = 115.
2. **One continuous ground.** By default the back, the spine and the front share one flat ground colour, which may be a single dominant colour computed from the cover (document 07, rule 11). A crease that drifts by 1 mm is then invisible.
3. **If the spine has its own colour**, the colour runs 2 mm past each crease, to x = 108 and x = 122, never ending on the crease. The cover box starts at x = 125, so the overrun never touches it.
4. **Full-bleed cover** (`trim_authorised`): the cover's inner edge lies exactly on crease B. The spine colour cannot overrun onto the front. A drift of up to 1 mm will show as a sliver of spine colour on the front or of cover on the spine. Choosing the spine colour from the cover's dominant colour makes this less visible. This is the accepted cost of the full-bleed option.
5. **Nothing important within 5 mm of a crease on the cover panels**, on either page. The inside spine strip (x 110 to 120 on page 2) carries no content.
6. **Reading direction** follows the book's language: bottom to top for French, top to bottom for English [M]. It is a parameter, decided outside this document.
7. **Ink on the creases**: prefer light or mid ground colours; do not use rich black across a crease (§4.3).
8. **Spine width is 10.0 mm in the file.** A 97 % "fit to page" print turns it into 9.7 mm; this is why the home sheet carries a scale ruler (§9).

### 6.5 Stock, grain, finishing

| Item | Recommendation | Basis |
|---|---|---|
| Weight | 300 g/m² board; 250 to 350 g/m² acceptable | Folder products are sold on about 300 to 350 g/m² board ([Helloprint product name](https://www.helloprint.com/en-gb/folders-a3foldedtoa4-1mmcover-300gsilk-bothsides-nofinishing-nofinish-250)) [V1]; weight bands: 130 to 170 g flyers, 170 to 250 g brochure covers and postcards, board above about 200 g ([graphiste.com](https://graphiste.com/blog/impression-comment-choisir-le-bon-grammage-et-taux-dencrage)) [V1] |
| Surface | matt, uncoated or lightly coated, recycled where offered | ESG note [M]; also better for QR scanning [M] |
| Lamination, varnish, foil | none by default | ESG note: keeps the card recyclable and label-compatible [M]. The folding note's suggestion to laminate dark solids against cracking [M] is kept as an option only (§1.4 row 13) |
| Creasing | two creases, always: 300 g/m² is far above the 170 g/m² threshold | §6.1 [V2] |
| Grain | parallel to the creases | §6.1 [V2] |
| Delivery | flat, creased, not folded | §6.2 [V1] |
| Colour condition | uncoated board needs an uncoated output condition (FOGRA52 where the printer names it) | §3.4 |

On an SRA3 sheet (320 × 450 mm), a spread laid with its spine along the 450 mm edge needs long-grain sheets; laid with its spine along the 320 mm edge it needs short-grain sheets [calc]. The grain direction of stocked sheets was not researched [M]; it must be stated in a quote request.

Structural caveat (design inference): a hollow U-shaped jacket has nothing inside its 10 mm spine. It stands, but it tends to spring open. Flaps folded inward, a folded insert, or a glued sleeve fix this. A prototype on 300 g/m² board should settle whether the bare jacket is acceptable.

### 6.6 How many pages make a 10 mm spine (softcover or notebook carrier)

| Basis | Pages for about 10 mm | Conf. |
|---|---|---|
| KDP white, 0.0572 mm per page | about 175 | calc |
| KDP cream, 0.0635 mm per page | about 158 | calc |
| Lulu, pages / 444 + 0.06 in | about 148 | calc |
| Infoprint estimate, 80 g/m² | about 250 | calc |
| Infoprint estimate, 90 / 100 / 120 g/m² | about 222 / 200 / 167 | calc |

A real softcover with a 1 cm spine is a 150 to 250 page book or notebook. Real papers have a bulk above 1.0, so the Infoprint estimate overstates the page count [M].

---

## 7. Trim sizes, spread sizes and sheet fit

Verified by search: French *poche* is 11 × 18 cm and *broché* about 14 × 21 cm ([publiersonlivre.fr](https://publiersonlivre.fr/publier-livre/format-livre-poche-broche/)); A5 148 × 210 is Lulu's most popular softcover size ([Lulu FR](https://www.lulu.com/fr/create/print-books)) [V1 each]. The other sizes are the usual French trade sizes from memory [M].

Sheet data: SRA3 = 320 × 450 mm; a Xerox DocuColor prints to within 4 mm of each edge, image area 312 × 442 mm, bleed of 3 mm or more recommended ([Xerox manual p.28](https://www.manualslib.com/manual/195677/Xerox-2000.html?page=28), [p.30](https://www.manualslib.com/manual/195677/Xerox-2000.html?page=30), [papersizes.io](https://papersizes.io/raw/sra3)) [V2 for the sheet size, V1 for the image area]. Larger presses exist (Canon varioPRINT iX3200: 337 × 504 mm printable, [Canon](https://www.canon.co.uk/business/products/production-printers/digital-presses/varioprint-ix3200/specifications/)) [V1].

Assumptions for the table, all [calc]:

- Spread = (2 × width + 10) × height.
- "Upload page" = spread + 3 mm bleed per side.
- "With marks" = spread + 8 mm per side (3 mm bleed, then 5 mm ticks starting at the bleed edge; §1.4 row 6).
- Home A4: sheet 297 × 210 mm landscape, 6.35 mm unprintable on each side, so 284.3 × 197.3 mm printable. The home layout needs the spread + 7 mm per side (2 mm bleed, 5 mm ticks).
- US Letter: 279.4 × 215.9 mm landscape, 266.7 × 203.2 mm printable with the same margins.
- SRA3: image area 312 × 442 mm.

| Panel trim w × h (mm) | h / w | Spread | Upload page | With marks | Home A4 at 100 % | US Letter at 100 % | SRA3 |
|---|---|---|---|---|---|---|---|
| 105 × 148 (A6) | 1.41 | 220 × 148 | 226 × 154 | 236 × 164 | yes (234 × 162) | yes | 2-up |
| 108 × 177 (poche) | 1.64 | 226 × 177 | 232 × 183 | 242 × 193 | yes (240 × 191) | yes | 2-up |
| **110 × 178 (poche, default)** | 1.62 | 230 × 178 | 236 × 184 | 246 × 194 | **yes** (244 × 192) | **yes** | 2-up |
| 125 × 190 | 1.52 | 260 × 190 | 266 × 196 | 276 × 206 | no: 204 mm high needs more than 197.3; bleed fits, top and bottom ticks do not | no: 274 mm wide | 2-up |
| 130 × 200 | 1.54 | 270 × 200 | 276 × 206 | 286 × 216 | no: the trim alone is 200 mm high | no: 270 mm wide exceeds 266.7 | 2-up |
| 140 × 205 | 1.46 | 290 × 205 | 296 × 211 | 306 × 221 | no | no | 2-up at the limit (2 × 221 = 442); 1-up is safe |
| 148 × 210 (A5) | 1.41 | 306 × 210 | 312 × 216 | 322 × 226 | no: wider than the sheet | no | 1-up |
| 155 × 240 | 1.55 | 320 × 240 | 326 × 246 | 336 × 256 | no | no | 1-up |

Observations [calc]:

- **Home A4 envelope**: with 5 mm ticks the panel may be at most 130 mm wide and 183 mm high; with 3 mm ticks, 187 mm high. Only A6 and the two poche sizes fit. The folding note listed 125 × 190 as "bleed fits, side ticks only" and 110 × 178 as "exactly at the limit"; with ticks starting at the bleed edge the poche fits with 5 mm to spare in height, and 125 × 190 still fails.
- **Poche 110 × 178 fits A4 and US Letter** at 100 % with bleed and cut ticks. On A4 it leaves 33.5 mm at the left and right and 16 mm at the top and bottom of the sheet.
- **Single-sided variant with a flap** (A4 only): back 110 + spine 10 + front 110 + flap 45 = 275 mm. That leaves 11 mm per side: 6.35 mm unprintable, 2 mm bleed, 2.65 mm for ticks. A 50 mm flap leaves 2.15 mm, enough for the bleed only. It does not fit US Letter.
- The poche ratio (1.62) is the closest to common eBook cover ratios of 1.5 to 1.6 [M], so the letterbox bands are the smallest.
- A5 and larger need A3 or SRA3: professional print only.
- In the 2-up SRA3 layouts the spine runs along the 450 mm edge (long-grain sheets); in the 1-up A5 layout it runs along the 320 mm edge (short-grain sheets).
- The repository's `digest` preset (130 × 200) cannot be printed at home on A4. The home profile must refuse it rather than scale it.

---

## 8. Candidate catalogue products

### 8.1 Products that can carry the design

| Product (FR / DE) | How the spine is made | Evidence | Fit for the gift jacket |
|---|---|---|---|
| **Presentation folder with capacity spine** (*chemise à rabats double rainage*, *chemise à dos* / *Mappe mit Füllhöhe*) | Two parallel creases; standard spines **5 mm and 10 mm** | Helloprint: double crease 5 mm (about 30 to 50 sheets of 80 g) and 10 mm (about 60 to 100 sheets), templates for A4 and A5 at 1, 5 and 10 mm ([FR-BE](https://www.helloprint.com/fr-be/chemisesrabats), [UK](https://www.helloprint.com/en-gb/folders)); Flyeralarm folders A5 or A4 with *Füllhöhe* 1, 5, 10 or 15 mm ([Flyeralarm](https://www.flyeralarm.com/de/p/mappen-2-teilig-mit-laschen-4390809.html)); viaprinto 1, 5 or 10 mm ([viaprinto](https://www.viaprinto.at/praesentationsmappen)); DruckDiscount24 5 or 10 mm ([DD24](https://www.druckdiscount24.de/angebotsmappen)); Tradeprint none, 3, 4, 5 or 10 mm ([Tradeprint](https://replatform.tradeprint.co.uk/presentation-folders/a4-glued-folder)); Veoprint 1, 6 or 12 mm ([Veoprint CH](https://www.veoprint.ch/impression-chemises-a-rabats)); Imprimeur du Roi A4 double crease 5 mm with PDF and AI template ([Imprimeur du Roi](https://imprimeurduroi.com/fr/impression-chemise-double-rainage/126-a4-double-rainage-5mm.html)); Pinguindruck **A6** folder, file 277 × 200 mm at 10 mm ([data sheet](https://pinguindruck.de/load/dateien/mappe_09-12_4s2ka6_datenblatt.pdf)) [V2] | **Best catalogue match.** A 10 mm spine is a stock option; A5 and A6 exist; the flaps can hold an insert with the QR code and stop the jacket springing open. Limits: the formats are made to hold A4, A5 or A6 sheets, so the panels are larger than book trims; die shapes are fixed; minimums apply (Imprimerie du Net from 50 copies, [source](https://imprimeriedunet.fr/601-chemise-a-rabats-standard.html)). Capacity figures vary between printers (10 to 30 versus 30 to 50 sheets for 5 mm: [Personaliz](https://www.personaliz.com/impressions-tout-support/papeterie-personnalisee/chemise-a-rabat.html) versus Helloprint). At Prontaprint a two-pocket folder has spine = 2 × capacity ([Prontaprint](https://printers.prontaprint.com/product/glued-folders/)) |
| **Flat card with two creases** (*carte avec 2 rainages, livrée à plat*) | Two creases 10 mm apart on 300 to 350 g board | Crease spacing rule of 5 mm or more (§6.2). Not a standard fold type. One lead: Exaprint "Carte postale double avec pliure sur mesure" ([page](https://www.exaprint.fr/la-carterie/carte-postale/double/)) [V1]; whether "sur mesure" allows two creases 10 mm apart is not confirmed | The exact object at the exact book trim. Today a **custom quote**, not a configurator product |
| **Flat custom-format card, creased by us** | Any printer's custom-size flyer or card on 300 to 350 g, uncreased; creases added with a manual creasing tool before dispatch | Design inference; no source | Removes the dependency on a printer's finishing options for batches fulfilled by the operator or a reseller. Needs a hands-on test |
| **Single-fold card** (*carte pliée, 1 rainage central*) | One central crease | Onlineprinters creased A6 cards, delivered flat by default ([page](https://www.onlineprinters.fr/p/cartes-pliables-format-paysage-a6-rainage-sur-le-cote-long)); Exaprint "2 ou 3 volets", 350 g, 1 rainage central ([page](https://www.exaprint.fr/Carterie/2-ou-3-volets-classique/Carte-de-visite-32-a-92-cm2-surface-a-plat/350g-couche-demi-mat/Non-pellicule/1-rainage-central)); Gelato fold options `fold-ver`, `fold-hor` only ([Gelato product UID article](https://apisupport.gelato.com/hc/en-us/articles/360010095259-What-is-a-product-UID-)); Merci Facteur API card `folded` = 15 × 21 cm closed ([README](https://github.com/MerciFacteur/Merci-facteur-API/blob/master/README.md)) [V1 each, sibling notes] | Available everywhere, including at quantity 1 through APIs. **It has no spine**: the title cannot show on a shelf. A degraded fallback only |
| **Softcover book or notebook** (*carnet dos carré collé*) | A real perfect-bound block | 150 to 250 pages for 10 mm (§6.6); KDP, Lulu, BoD and Bookvault print single copies | The most book-like object, the highest unit cost and weight; the QR code goes on page 1. Cover rights weigh more here: it is a "book" bearing the cover |
| **Dust jacket** (*jaquette*) | Folds around an existing book; no crease needed on 130 to 150 g paper [M] | Ingram flaps 3.25 in (83 mm) + 0.25 in (6 mm) strip, bleed 0.125 in ([Ingram template](https://marketing.ingramcontent.com/mrkng/documents/publishers/Dust-Jacket-Template.pdf)); Mixam flaps 2.5 in; PrintNinja minimum 2.83 in ([PrintNinja](https://printninja.com/dust-jacket-file-setup-guide/)) [V1 each] | Needs a book or a dummy block underneath; alone it does not stand. Useful as a **layout model**: flaps of 65 to 85 mm are the natural place for QR code and instructions |
| **DVD-case inlay** (*jaquette DVD*) | Paper sleeve slid into a plastic case, spine 14 mm (7 mm slim) | Trim 273 × 183 mm, panels about 129.5 mm, 279 × 189 mm with bleed; slim 266 × 183 mm ([4over4](https://www.4over4.com/guide/dvd-cover-size-dimensions-and-print-specs)); Avery sells pre-cut A4 inkjet inserts 273 × 183 mm ([Avery J8437](https://www.avery.co.uk/product/dvd-case-inserts-j8437-25)) [V2] | Fits A4 at 100 %, rigid, stands on a shelf, panel ratio 1.41. The spine is 14 mm, not 10, and it is a plastic case. A proven precedent for "print at home, slip into a case" |
| **Book-shaped box** (*boîte livre*, *étui*) | Die-cut folding carton | not searched (budget) | Could hold a voucher card or a small gift; die-cut minimums; needs a packaging printer [M] |
| **Bellyband** (*bandeau*, *Banderole*) | A strip wrapped around an object | not searched (budget) | A complement only; cannot carry cover and spine alone [M] |

### 8.2 What this means for the file

- **Folder products** have their own die template. The printer's panel sizes, flap shapes and glue zones are fixed, and WTTB-type printers want no guide lines at all [V1]. To use one, the generator must lay our panels onto a printer-specific template: panel width and height, spine width, flap outline, per-product bleed. This is per-product data, not a new layout engine, but each template has to be obtained and tested.
- **Quote jobs** need the marks profile, the plan PDF and a written brief. Suggested brief, to adapt:

  > Carte 4 pages à dos. Format ouvert 230 × 178 mm (fermé 110 × 178 mm, dos 10 mm). 2 rainages parallèles au petit côté, à 110 mm et 120 mm du bord gauche (vu de l'extérieur). Livraison à plat, non pliée. Impression quadri recto verso, sans vernis ni pelliculage. Carton 300 g/m², sens des fibres parallèle aux rainages. Fond perdu 3 mm. Fichier PDF/X-4 CMJN, 2 pages (extérieur, intérieur), profil à préciser par l'imprimeur (par défaut ISO Coated v2 300 %).

- **Single-fold products** can only be served by a different layout (no spine). That layout is outside this document; it must not be presented as the same object.
- **Minimum quantities** rule out web-to-print for single personalised cards: Pixartprinting invitations from 10, Helloprint greeting cards from 10, Rapid-Flyer double cards from 25, Realisaprint from 50 (`notes/printers.web2print.md`) [V1 each]. The professional route is a batch route unless a quote printer accepts singles.

---

## 9. Print-at-home profile

### 9.1 Evidence

| Point | Finding | Conf. |
|---|---|---|
| Unprintable margins | Model-specific: 4.23 mm per side on one HP LaserJet ([HP forum](https://h30434.www3.hp.com/t5/LaserJet-Printing/What-does-the-quot-Wide-A4-quot-setting-do-HP-Laserjet/td-p/6862566)); 3.37 mm left and right on A4 for a Lexmark inkjet, 6.35 mm on other sizes ([Lexmark Z52 manual](https://www.manualslib.com/manual/291880/Lexmark-Z52.html?page=50)); HP forum: "some 3 mm, some 5 mm" ([HP forum](https://h30434.www3.hp.com/t5/Inkjet-Printing/Minimum-Margins-Envy-5030/m-p/7329955)). Range **3 to 6.35 mm** | V2 |
| Bottom edge | Older inkjets have a larger bottom margin than the sides (an old HP manual in the results); 12 to 15 mm on some models [M] | V1 / M |
| Laser versus inkjet | Laser printers usually allow smaller margins; borderless printing is an inkjet feature and cannot be assumed ([Label Planet](https://labelplanet.co.uk/glossary/printer-margins)) | V1 |
| Paper | 160 to 200 g/m² matt card is the practical ceiling of most home printers; 120 g works but will not stand well | M |
| Duplex registration | 1 to 2 mm between front and back on home printers | M |
| Grain | Ordinary A4 office paper is long grain, so an A4-landscape spread folds across the grain | M |

### 9.2 Sheet specification (A4 landscape, poche format)

| Item | Value |
|---|---|
| Page | A4 landscape, 297 × 210 mm; boxes in §2.2 |
| Spread position | centred: 33.5 mm from the left and right sheet edges, 16 mm from top and bottom |
| Printable frame assumed | 6.35 mm on every side; nothing we draw may lie outside it |
| Bleed | 2 mm of ground colour |
| Cut guides | corner ticks from the bleed edge outward, 5 mm, 0.5 pt, light grey (K only) |
| Fold guides | at both creases, above and below, same length, dashed, light grey |
| Scale ruler | a 100 mm line with a tick every 10 mm, labelled "100 mm", 5 mm above the trim, over the back panel |
| Printed instruction | a short text in a side margin (27 mm of printable width is free on each side): "Imprimer en taille réelle (100 %), recto verso, retourner sur le bord court" |
| PDF | plain PDF, sRGB images, grey text; no PDF/X claim |
| Inside page | white ground; all content 5 mm or more inside the trim and away from the creases |
| Resolution thresholds | 200 ppi target, warning below 150, error below 100 |

A lower margin of 15 mm on an old inkjet still leaves the trim area intact (16 mm free at the bottom), but clips the bottom bleed and ticks [calc].

### 9.3 Instructions to give the user

1. Print at "actual size" or 100 % (*taille réelle*), never "fit to page". Check the ruler: it must measure 100 mm. At 97 % the spine becomes 9.7 mm and the fold ticks move.
2. Duplex: flip on the **short edge**. The creases are parallel to the short edge of the landscape sheet, so the sheet turns like a book page. With a long-edge flip the two sides still overlap, because the spread is centred and symmetric, but the inside prints upside down relative to the cover [calc].
3. Paper: 160 to 200 g/m² matt card if the printer takes it [M].
4. Score before cutting, while the fold ticks are still on the sheet: lay a ruler between the top and bottom ticks and draw a blunt point (bone folder, empty ballpoint) along each of the two lines. Above 170 g/m² scoring is needed (§6.1), and on A4 landscape the fold runs across the grain. See also the [Cardstock Warehouse scoring guide](https://cardstock-warehouse.com/pages/paper-101-scoring-and-folding) (content not read).
5. Cut along the corner ticks with a craft knife and a metal ruler.
6. Fold each crease to 90 degrees, away from the scored groove, so that the raised bead ends up inside the fold [M].

### 9.4 Single-sided variant with a flap

For printers without duplex, and to avoid duplex misregistration:

- One page. Left to right: back 110 mm | spine 10 mm | front 110 mm | flap 45 mm. Trim 275 × 178 mm, centred on A4 (11 mm at the left and right). Three creases, at 110, 120 and 230 mm from the left trim edge.
- The flap folds inward behind the front cover, as on a dust jacket. It carries the QR code (33 mm box in a 35 mm live width), the typed code and the host. The secret is hidden when the card is closed.
- The gift message and the three steps go on the back panel, which is our own material.
- Side ticks are 2.65 mm long at most (§7). A4 only.

The DVD-inlay precedent (273 × 183 mm on A4, pre-cut Avery sheets, §8.1) shows that an A4 slip-in format is an accepted home-print pattern [V2].

---
## 10. Preflight checklist

Severity: **E** = error, blocks a professional order; **W** = warning, shown to the user; **I** = information. "Pro" = `upload_eu` and `professional_eu` and every per-printer profile.

### 10.1 Machine-checkable

**A. File and structure**

| # | Check | Pass condition | Sev. | Profiles |
|---|---|---|---|---|
| A1 | File integrity | Parses; not encrypted; no incremental update | E | all |
| A2 | Active content | No script, action, form field, annotation, embedded file or multimedia | E | all |
| A3 | Page count | 2 pages (outside, inside); 1 for the single-sided home variant; all pages have identical boxes | E | all |
| A4 | Version and features match the label | Header 1.3 and no object or cross-reference streams for X-1a:2001 and X-3:2002; header 1.6 for X-4 | E | pro |
| A5 | PDF/X markers | Version marker of §3.3 present, consistent between Info and XMP; `Trapped` is `/False`; `Title`, dates and trailer `/ID` present | E | pro |
| A6 | Output intent | Exactly one `GTS_PDFX` intent; identifier equals the printer profile's; embedded profile is CMYK, output class, 4 components; ICC v2 when the header is 1.3 | E | pro |
| A7 | Geometry keys | The `EBC.*` Info keys (trim, spine width, crease positions) equal the page geometry | E | all |

**B. Boxes and geometry**

| # | Check | Pass condition | Sev. | Profiles |
|---|---|---|---|---|
| B1 | Boxes present | MediaBox, BleedBox, TrimBox on every page; no ArtBox; CropBox absent or equal to MediaBox; `Rotate` 0; no `UserUnit` | E | all |
| B2 | Trim size | TrimBox equals the expected spread within 0.01 mm | E | all |
| B3 | Bleed | Bleed on each of the four sides equals the profile's value within 0.01 mm; TrimBox inside BleedBox inside MediaBox | E | all |
| B4 | Upload page | MediaBox equals BleedBox; no object in the registration colour | E | upload |
| B5 | Marks | Every mark lies outside the BleedBox and inside the MediaBox | E | marks |
| B6 | Spine | Distance between the two creases equals the spine width within 0.01 mm | E | all |
| B7 | Bleed is painted | The ground fills the whole BleedBox of page 1, unless the ground is white | E | pro |
| B8 | Colour boundary | No colour boundary parallel to a crease within 2 mm of it, except the edge of a full-bleed cover | W | all |
| B9 | Home sheet | MediaBox equals the sheet; every drawn object lies inside the 6.35 mm frame; the ruler is 100.00 mm long | E | home |

**C. Colour and ink**

| # | Check | Pass condition | Sev. | Profiles |
|---|---|---|---|---|
| C1 | Colour spaces | DeviceCMYK and DeviceGray only. Tagged RGB only when the profile has `allow_rgb`. A separation only for marks (registration) or for the profile's crease colour | E | pro |
| C2 | Transparency | No soft mask, no opacity below 1, blend mode Normal, no transparency group | E | pro |
| C3 | Layers | No Optional Content | E | pro |
| C4 | Total ink | Every vector fill and stroke at or below the profile limit; for images, the 99.5th percentile of per-pixel total ink at or below the limit | E | pro |
| C5 | Black text | Text below 8 pt, the typed code and the QR symbol are 0/0/0/100 | E | pro |
| C6 | Light tints | No flat colour with a non-zero channel below 5 % | W | pro |
| C7 | Overprint | Overprint set only on K100 objects; no white object set to overprint | E | pro |
| C8 | Registration colour | Only outside the BleedBox | E | pro |

**D. Fonts, text, lines**

| # | Check | Pass condition | Sev. | Profiles |
|---|---|---|---|---|
| D1 | Fonts | All embedded and subset; no Type 3 font; no text at all when the profile has `outline_all_text` | E | all |
| D2 | Type size | 6 pt or more | E | all |
| D3 | Reversed or multi-ink text | 8 pt or more | W | all |
| D4 | Line weight | 0.25 pt or more (0.5 pt reversed); no zero-width stroke | W / E for zero width | all |
| D5 | Safety zones | Text, logos and the QR box at least 5 mm (profile value) from the trim and from each crease on the cover panels | E | all |
| D6 | Spine band | Spine text inside the central band (x 112 to 118 for the poche) and 5 mm or more from top and bottom | E | all |

**E. Images and cover integrity**

| # | Check | Pass condition | Sev. | Profiles |
|---|---|---|---|---|
| E1 | Resolution | Effective ppi on both axes: at or above target = pass; between warning and target = I; between error and warning = W; below error = E. Pro 300 / 250 / 150; home 200 / 150 / 100 | I / W / E | all |
| E2 | Uniform placement | Horizontal and vertical scale equal within 0.1 %; no rotation or skew; the clip box equals the image box (document 07, rule 5) | E | all |
| E3 | Nothing on the cover | No object intersects the cover's bounding box (document 07, rule 7) | E | all |
| E4 | Inset | The cover box is inside the trim and clear of the crease by the profile inset, unless `trim_authorised` | E | all |
| E5 | Encoding | 8-bit; no alpha; CMYK or grey in pro profiles; DCT or Flate only; no embedded profile on CMYK images | E | pro |

**F. QR symbol**

| # | Check | Pass condition | Sev. | Profiles |
|---|---|---|---|---|
| F1 | Presence and ink | One symbol on the inside page (or the flap); modules 0/0/0/100 on white; all modules square and equal | E | all |
| F2 | Module size | 0.5 mm or more = pass; 0.4 to 0.5 mm = W; below 0.4 mm = E | W / E | all |
| F3 | Quiet zone | 4 modules or more on every side, empty | E | all |
| F4 | Position | Box 5 mm or more from trim and creases; not on the cover panel; not across a crease | E | all |
| F5 | Version | 4 or lower | W | all |
| F6 | Decodes | The page rendered at 150 dpi and at 300 dpi decodes to the expected URL | E | all |

### 10.2 Manual, once per printer profile and stock

| # | Check |
|---|---|
| M1 | Page size and bleed compared with the printer's template for the exact product |
| M2 | Printed proof measured: creases at 110 and 120 mm, spine 10 mm, trim 230 × 178 mm |
| M3 | The folded card stands on a shelf; note whether it springs open |
| M4 | No cracking of the ink or the board at the creases; grain runs parallel to the spine |
| M5 | Colour compared with the screen preview; black text sharp, no colour fringe |
| M6 | QR symbol scanned with three phones, one of them old, in dim light |
| M7 | Output condition confirmed by the printer, especially for uncoated stock |
| M8 | One file per PDF/X label checked in an independent preflight tool or by a printer; no open-source PDF/X validator is assumed to exist [M] |
| M9 | Home sheet printed on one inkjet and one laser printer: ruler measures 100 mm, front and back align within 2 mm, ticks are visible |
| M10 | Licence file present next to every ICC profile served or committed |

---

## 11. Open questions

Ordered by how much they block.

1. **The printed product.** Does any EU online printer accept two creases 10 mm apart on a custom-format card in its configurator? The only lead is Exaprint's "pliure sur mesure" double card. Failing that: obtain the A5 and A6 folder templates (Helloprint, Flyeralarm, Pinguindruck) and compare their panel sizes with book trims; ask two or three French printers for a quote on the brief of §8.2.
2. **ICC licences.** The ECI licence text and redistribution right; the basICColor terms (the two recollections disagree, §3.5); profile file sizes, ICC versions and ink limits. Nothing may be bundled or served before this is read.
3. **PDF/X keys.** The OutputIntent keys, the version-marker strings, the registered identifier names (`FOGRA39`, `FOGRA51`, `FOGRA52`) and the box rules are from memory. They need a check against ISO 15930 or an independent validator.
4. **QR sizing.** The 0.4 mm module floor, the GS1 figures, the distance / 10 rule and the contrast grades are from memory in three notes. No standard or printer guide was read. Upper-case `HTTPS://` on stock iOS and Android cameras is untested.
5. **Mark geometry.** No surveyed printer publishes offsets or lengths. The dashed fold tick is a recollection. Confirm with the first quote printer.
6. **Per-printer values still open**: Exaprint (2 or 3 mm bleed, current profile); Printoclock (2 or 5 mm bleed for a folded card); Pixartprinting (ink limit, current upload size limit, the page stating "no crop marks"); Onlineprinters (the page carrying the 300 % and 260 % figures, mark policy); Saxoprint (mark policy, rich-black recipe; its May 2024 guide was seen only in summary); Flyeralarm (bleed and safety on a current card or folder data sheet; an archived 2015 A6 sheet was ambiguous between 1 mm and 4 mm; explicit ink statement); Helloprint and Vistaprint (mark policy, profile, ink limit; Vistaprint's 1.5 mm EU bleed); Gelato and Cloudprinter (bleed, colour space, ink limit and boxes from first-party documents; whether Gelato converts sRGB); BoD and Bookvault (profile, ink limit).
7. **Tolerances.** The ±1 mm binding tolerance and the 50 mm fold-to-fold minimum each rest on one search extract whose page could not be pinned down. The 5 mm crease-spacing rule is a single source.
8. **Stock.** Whether an unlaminated, uncoated 300 g/m² board cracks at the creases under a mid-tone ground; the grain direction of stocked SRA3 sheets; which printers run FOGRA52.
9. **Ink limits below 300 %.** No method is specified for the 260 % (Onlineprinters express) and 270 % (Lulu) cases.
10. **Home printing.** Maximum paper weight of common home printers, duplex registration, bottom margins and the hand-scoring technique are from memory.
11. **Trim sizes.** The French trade sizes 108 × 177, 125 × 190, 130 × 200, 140 × 205 and 155 × 240 are from memory; only 11 × 18, 14 × 21 and A5 were confirmed. eBook cover aspect ratios are from memory.
12. **Reference platforms.** The KDP spine-text margin (0.0625 in) and spine variance, the KDP formula's cover allowance, Lulu's 32-page minimum, the IngramSpark 0.5 in hinge and the BoD figures (from a guide about 11 years old) are from memory, mirrors or old documents. They are references, not targets, so they do not block.
13. **Not searched at all** (budget): bellyband specifications, book-shaped boxes, catalogue notebooks with a 10 mm spine and their prices, Saxoprint and Onlineprinters perfect-binding data sheets, hinge-score practice.
14. **Reversed text.** No printer figure for the minimum size of reversed text was found; 8 pt is a rule of thumb.
15. **Structure.** Whether the bare U-shaped jacket stands acceptably without flaps or an insert.

---

## 12. Sources

All retrieved as search-result summaries on 2026-11-12 unless stated. Links marked "sibling" were found by the researchers of other notes in this repository.

**Printer file-preparation documents**

- Pixartprinting: [instructions](https://support.pixartprinting.com/hc/en-us/articles/26076155387026-What-do-I-find-in-the-instructions), [perfect binding](https://www.pixartprinting.co.uk/printing-magazines-books-catalogues/special-finish-perfect-binding/), [wire-o](https://www.pixartprinting.co.uk/printing-magazines-books-catalogues/special-finish-wire-o-binding/), [special finishes text](https://support.pixartprinting.com/hc/en-us/articles/27655380376466-How-to-properly-set-up-text-for-special-finishes), [legible text](https://support.pixartprinting.com/hc/en-us/articles/27654779357970-How-to-ensure-the-text-is-the-correct-size-to-be-legible-once-printed), [CMYK](https://support.pixartprinting.com/hc/en-us/articles/360017440400-What-does-CMYK-mean), [inks advice](https://www.pixartprinting.co.uk/blog/inks-advices/), [flyer guide](https://www.pixartprinting.co.uk/blog/how-to-create-an-effective-flyer-the-complete-guide-from-idea-to-print/), [2017 third-party checklist](https://printicom.wordpress.com/2017/07/06/final-checklist-before-uploading-your-documents/), [copy of instruction sheet 7, FR](https://www.cipherbliss.com/dessins/Qzines/maquettes/imprimeur%20docs/pixartprinting_instruction_7_fr.pdf)
- Onlineprinters: [PDF settings FAQ](https://onlineprinters.ie/c/faq/settings-print-pdf), [DL flyer](https://www.onlineprinters.ie/p/flyers-leaflets-dl-printed-on-one-side), [A5 postcard](https://www.onlineprinters.ie/p/exklusive-postcards-a5), [A6 insert](https://www.onlineprinters.ie/p/inserts-exclusive-4-4-colour-printing-a6), [Überdrucken](https://www.onlineprinters.de/c/druckdaten/ueberdrucken), [Überdrucken magazine](https://www.onlineprinters.de/magazin/ueberdrucken-und-aussparen-indesign/), [Linienstärke](https://www.onlineprinters.de/c/druckdaten/linienstaerke), [Schriften](https://www.onlineprinters.de/c/data/schriften), [creased A6 card, FR](https://www.onlineprinters.fr/p/cartes-pliables-format-paysage-a6-rainage-sur-le-cote-long) (sibling)
- Flyeralarm: [checklist](https://www.flyeralarm.com/blog/wp-content/uploads/Flyeralarm_Checkliste_Druckdatenerstellung-1.pdf), [UK artwork requirements](https://www.flyeralarm.com/images/upload/content/_pdf/uk/Artwork_Requirements_UK.pdf), [ready for printing](https://www.flyeralarm.com/blog/uk/ready-for-printing/), [bleed tutorial](https://www.flyeralarm.com/blog/en/tutorial-setting-up-bleeds-in-indesign-cc-illustrator-cc-and-scribus-1-4-5/), [PDF tutorial](https://www.flyeralarm.com/blog/en/creating-print-ready-pdf-with-indesign/), [PDF/X article](https://flyeralarm.com/blog/en?p=24986), [folders with Füllhöhe](https://www.flyeralarm.com/de/p/mappen-2-teilig-mit-laschen-4390809.html)
- Saxoprint: [Druckdatenanleitung May 2024](https://api.saxoprint.de/media-by-file-name/Druckdatenanleitung-1.pdf), [UK artwork instructions](https://www.saxoprint.co.uk/service-help/artwork-instructions), [FAQ](https://www.saxoprint.de/service-und-hilfe/faq), [colours](https://www.saxoprint.de/service-und-hilfe/fehlerdiagnose/farben), [fonts and lines blog](https://www.saxoprint.de/blog/druckvorlagen/schriften-und-linien), [pages and covers UK](https://www.saxoprint.co.uk/service-help/error-diagnosis/pages-and-covers), [Seiten und Umschläge](https://www.saxoprint.de/service-und-hilfe/fehlerdiagnose/seiten-umschlaege)
- Exaprint: [Guide PAO PDF](https://static.exaprint.fr/marketing/cms/exaprint/guidePAO/GuidePAO-telechargement.pdf), [charte PAO](https://www.exaprint.fr/imprimerie-en-ligne/charte-pao), [assistance PAO](https://www.exaprint.fr/page/assistance-pao), [conseils PAO](https://www.exaprint.fr/conseils-pao/), [fond perdu](https://blog.exaprint.fr/cadres-impression-contraintes-et-astuces/), [6 erreurs](https://blog.exaprint.fr/6-erreurs-courantes-a-eviter-en-impression/), [brochure errors](https://blog.exaprint.fr/impression-brochure-erreurs-pao/), [carte postale double, pliure sur mesure](https://www.exaprint.fr/la-carterie/carte-postale/double/) (sibling), [2 ou 3 volets, 1 rainage central](https://www.exaprint.fr/Carterie/2-ou-3-volets-classique/Carte-de-visite-32-a-92-cm2-surface-a-plat/350g-couche-demi-mat/Non-pellicule/1-rainage-central) (sibling)
- Printoclock: [FAQ](https://support.printoclock.com/article/comment-faire-mon-fichier-dimpression), [bien réaliser son fichier](https://www.printoclock.com/bien-realiser-son-fichier), [fond perdu](https://www.printoclock.com/fond-perdu), [technical specs 63](https://www.printoclock.com/technical-specs-html/63)
- Helloprint: [help centre](https://www.helloprint.com/en-ie/help-centre/where-do-i-start-when-preparing-my-design-for-printing), [blog](https://www.helloprint.com/en-ie/blog/how-to-set-up-a-file-for-print), [bookmarks](https://www.helloprint.com/en-gb/bookmarksexclusivefinishing), [design solutions](https://www.helloprint.com/en-ie/design-solutions), [chemises à rabats FR-BE](https://www.helloprint.com/fr-be/chemisesrabats), [folders UK](https://www.helloprint.com/en-gb/folders), [folder product, 300 g silk](https://www.helloprint.com/en-gb/folders-a3foldedtoa4-1mmcover-300gsilk-bothsides-nofinishing-nofinish-250)
- Vistaprint: [business card file prep](https://www.vistaprint.com/hub/business-card-file-prep/)
- Lulu: [book creation guide](https://assets.lulu.com/media/guides/en/lulu-book-creation-guide.pdf), [Scribd copy](https://www.scribd.com/document/615961952/lulu-book-creation-guide), [API PDF settings](https://help.api.lulu.com/en/support/solutions/articles/64000254609-pdf-creation-settings), [cover upload](https://help.lulu.com/en/support/solutions/articles/64000282777-upload-your-cover-file), [spine width](https://help.api.lulu.com/en/support/solutions/articles/64000254616-how-is-spine-width-calculated-), [spine help](https://help.lulu.com/en/support/solutions/articles/64000255462), [Lulu FR formats](https://www.lulu.com/fr/create/print-books)
- BoD: [Grafiken und Farbmanagement](https://www.bod.ch/bodfiles/GLOBAL-Storage/documents/help-documents/bod-grafiken-und-farbmanagement.pdf), [cover guide, about 11 years old](https://www.yumpu.com/de/document/view/41877228/wie-gestalte-ich-das-buchcover-richtig-bod)
- Bookvault: [bleed](https://help.bookvault.app/what-is-bleed-and-why-do-i-need-it), [crop marks](https://help.bookvault.app/should-i-supply-crop-marks), [PDF/X](https://help.bookvault.app/what-is-pdf/x-1a2001-and-pdf/x-32002), [validation](https://help.bookvault.app/what-does-the-bookvault-file-validation-check-for), [PDF guide](https://help.bookvault.app/hubfs/Guide%20To%20PDFs%20-%20Digital%20Version%20.pdf?hsLang=en), [spine misaligned](https://help.bookvault.app/book-cover-spine-misaligned), [templates](https://bookvault.app/downloading-bookvault-templates/)
- Gelato: [file standard](https://connect-support.gelato.com/en/articles/9858024-gelatocreate-what-file-standard-is-used-for-paper-based-production), [formats](https://support.gelato.com/en/articles/8996362-what-product-formats-and-dimensions-do-you-support) (sibling), [product UID](https://apisupport.gelato.com/hc/en-us/articles/360010095259-What-is-a-product-UID-) (sibling), [third-party page, unreliable](https://processor-5xq.pages.dev/print-ready/gelato)
- Cloudprinter: [Canva export](https://knowledge.cloudprinter.com/mastering-canva-print-export-for-cloudprinter.com-compatibility), [production PDF docs](https://docs.cloudprinter.com/client/how-to-build-the-pdf/)
- KDP: [cover help G201953020](https://kdp.amazon.com/help/topic/G201953020), [same, en_US](https://kdp.amazon.com/en_US/help/topic/G201953020), [spine text](https://kdp.amazon.com/en_US/help/topic/A1L46HVPPIYTE), [kdpeasy calculator](https://www.kdpeasy.com/fr/tools/calculateur-dos-kdp), [ebookpbook](https://www.ebookpbook.com/2026/06/24/book-spine-text-requirements/)
- IngramSpark: [file requirements](https://www.ingramspark.com/blog/file-requirements-for-print-books), [cover file creation](https://help.ingramspark.com/hc/en-us/articles/35306346735245-Cover-File-Creation), [template generator](https://help.ingramspark.com/hc/en-us/articles/35303841585933-Cover-Template-Generator), [cover bleed sheet](https://marketing.ingramcontent.com/mrkng/documents/publishers/Cover-Bleed-Dimensions.pdf), [dust-jacket template](https://marketing.ingramcontent.com/mrkng/documents/publishers/Dust-Jacket-Template.pdf)
- Other printers and guides: [druck.at Infoblatt](https://cdn.druck.at/sites/default/files/Infoblatt_Druckdatenerstellung.pdf), [Clays](https://www.clays.co.uk/wp-content/uploads/supplying-cover-files-for-print.pdf), [Baumer](https://www.baumer.ch/fileadmin/files/downloads/Baumer_UEbermittlung_von_Druckdaten_PSO_PSD_2021_DE.pdf), [Mixam hardcover](https://mixam.co.uk/support/hardcoversetup), [PrintNinja dust jacket](https://printninja.com/dust-jacket-file-setup-guide/), [Merci Facteur API README](https://github.com/MerciFacteur/Merci-facteur-API/blob/master/README.md) (sibling)

**PDF/X and boxes**

- [Macwelt on PDF/X-4](https://www.macwelt.de/article/956117/pdf-workflow-in-der-druckproduktion.html), [Cuno Druck PDF/X sheet](https://www.cunodruck.de/wp-content/uploads/2023/11/PDF-X-GCC_23.pdf), [pdf2go boxes](https://pdf2go.com/blog/what-are-pdf-boxes), [Wikipedia PDF/X](https://Www.Wikipedia.org/wiki/PDF/X)

**Creasing, grain, crease spacing**

- https://gmund.com/shop/media/pdf/instructions/Gmund_OG_mit_Digital_Printing_Tips_fr.pdf
- https://www.heidelberg.com/global/media/en/global_media/company___publications/heidelberg_news/tips_tricks/279_tips_tricks_de.pdf
- https://blog.midstatelitho.com/creasing-vs-scoring/
- https://creativepro.com/paper-tips-going-against-the-grain/
- https://druckerei.de/glossar/nuten/
- https://www.lazi-akademie.de/wiki/drucktechnik/weiterverarbeitung/rillung/
- https://www.printing.com/uk/file-supply/how-to-supply-presentation-folder-files
- https://www.nettl.com/uk/shop/file-supply/how-to-supply-presentation-folder-files
- https://manualedistruzioni.it/formax/binding-machine/atlas-c200/user-manual/v177
- https://manuals.online/morgana/binding-machine/autocreaser-50/user-manual/1swt6
- https://manuals.konicaminolta.eu/ineo-plus-2060L-2060-2070/EN/contents/id04-_102709920.html
- https://media.mybinding.com/media/manuals/Morgana-DigiFold-Pro-User-Manual.pdf
- https://www.printprinz.de/support/weiterverarbeitung
- https://www.printprinz.de/support/veredelungen
- https://viperprint.de/flyer/falzflyer
- https://www.printpascher.com/chemise-rabats.html
- https://cardstock-warehouse.com/pages/paper-101-scoring-and-folding

**Crease-line and technical-line conventions**

- https://upload.prod.papeo.fr/media/tuto-pdf/tuto-imprimerie-amalgame.pdf
- https://pinguindruck.de/load/dateien/mappe_09-12_4s2ka6_datenblatt.pdf
- https://www.epfl.ch/campus/services/repro/wp-content/uploads/2025/11/REPRO-Guide-PAO.pdf
- https://wttb.co.uk/support-guides/presentation-folder-styles-setup
- https://smilepack.fr/files/subscribers/d72b0b53-b3d0-49ff-adfd-37d51546ebde/ftp/gabarits/Coussin%20technique%20smilepack.pdf
- https://www.jimprimeenfrance.fr/impression-chemise-rabat-fine

**Spine rules**

- https://www.graphiscann.fr/brochures-collees-dos-carre-impression.html
- https://www.montpellier-imprimeur.com/imprimer-un-titre-sur-la-tranche-ce-que-le-dos-peut-accueillir/
- https://www.montpellier-imprimeur.com/checklist-dos-carre-colle
- https://booksfactory.fr/article_info.php?articles_id=53
- https://www.veoprint.com/dl?t=template&id=34
- https://www.resetstpauli.de/checkliste-klebebindung/
- https://printiteasy.eu/de/wissensdatenbank/druckvorlagen/
- https://www.pdfpress.app/blog/spine-width-calculator-guide
- https://www.bookdesigner.ai/guides/book-spine-width

**Trim sizes, sheets, home printers, paper**

- https://publiersonlivre.fr/publier-livre/format-livre-poche-broche/
- https://www.manualslib.com/manual/195677/Xerox-2000.html?page=28
- https://www.manualslib.com/manual/195677/Xerox-2000.html?page=30
- https://papersizes.io/raw/sra3
- https://www.canon.co.uk/business/products/production-printers/digital-presses/varioprint-ix3200/specifications/
- https://h30434.www3.hp.com/t5/LaserJet-Printing/What-does-the-quot-Wide-A4-quot-setting-do-HP-Laserjet/td-p/6862566
- https://h30434.www3.hp.com/t5/Inkjet-Printing/Minimum-Margins-Envy-5030/m-p/7329955
- https://www.manualslib.com/manual/291880/Lexmark-Z52.html?page=50
- https://labelplanet.co.uk/glossary/printer-margins
- https://graphiste.com/blog/impression-comment-choisir-le-bon-grammage-et-taux-dencrage

**Catalogue products**

- https://www.viaprinto.at/praesentationsmappen
- https://www.druckdiscount24.de/angebotsmappen
- https://replatform.tradeprint.co.uk/presentation-folders/a4-glued-folder
- https://www.veoprint.ch/impression-chemises-a-rabats
- https://imprimeurduroi.com/fr/impression-chemise-double-rainage/126-a4-double-rainage-5mm.html
- https://imprimeriedunet.fr/601-chemise-a-rabats-standard.html
- https://www.personaliz.com/impressions-tout-support/papeterie-personnalisee/chemise-a-rabat.html
- https://printers.prontaprint.com/product/glued-folders/
- https://www.4over4.com/guide/dvd-cover-size-dimensions-and-print-specs
- https://www.avery.co.uk/product/dvd-case-inserts-j8437-25

**QR code (sibling note `ereader-ux.ux.md`, fetched from raw.githubusercontent.com)**

- [python-qrcode README](https://raw.githubusercontent.com/lincolnloop/python-qrcode/main/README.rst), [segno make](https://raw.githubusercontent.com/heuer/segno/master/docs/make.rst), [segno modes](https://raw.githubusercontent.com/heuer/segno/master/docs/qrcode-modes.rst), [Nayuki README](https://raw.githubusercontent.com/nayuki/QR-Code-generator/master/Readme.markdown), [ZXing URIResultParser](https://raw.githubusercontent.com/zxing/zxing/master/core/src/main/java/com/google/zxing/client/result/URIResultParser.java), [BIP-173](https://raw.githubusercontent.com/bitcoin/bips/master/bip-0173.mediawiki)

**Repository documents used**

- `docs/research/notes/print-requirements.prepress.md`, `docs/research/notes/print-requirements.folding.md`
- `docs/research/notes/ereader-ux.ux.md`, `docs/research/notes/printers.web2print.md`, `docs/research/notes/printers.print-api.md`, `docs/research/notes/printers.esg.md`, `docs/research/notes/legal-gifting-commerce.commerce.md`
- `docs/research/07-legal-covers-and-reseller-rights.md`, `docs/research/08-legal-fonts-and-genai.md`
- `docs/architecture.md`, `crates/ebc-layout/src/profile.rs`, `crates/ebc-qr/src/lib.rs`, `assets/icc/README.md`

---

## Implications for the implementation

**Profiles and data**

1. Keep three built-in profiles: `upload_eu` (the default for any professional output), `professional_eu` (with marks) and `home_<sheet>`. Their values are those of §1.2 and §1.3.
2. Change `professional_eu` and `upload_eu`: safety margin from 4 to 5 mm; spine clear zone from 1.5 to 2 mm; registration targets and colour bar off by default. Keep: bleed 3 mm, slug 10 mm, ticks 5 mm long starting at the bleed edge, 0.25 pt, PDF/X-4, 300 % ink.
3. Change `home_*`: printable margin from 5 to 6.35 mm. Keep: bleed 2 mm, safety margin 5 mm, spine clear zone 2 mm, grey K-only guides at 0.5 pt, scale ruler, plain PDF.
4. Add a third resolution threshold to every profile. Professional: target 300, warning below 250, error below 150 ppi. Home: 200, 150, 100.
5. Add per-printer profiles as data records, not code: bleed, safety margin, marks on or off, crease encoding (`ticks`, `spot`, `plan`), crease separation name and preview colour, PDF label, output condition, ink limit, `allow_rgb`, `outline_all_text`, status (`unconfirmed` or `confirmed`), source URL, date checked. Seed them from §3.6, all `unconfirmed`. An unconfirmed profile shows a warning and cannot be used for an automated order.
6. Validate profiles: bleed from 1 to 5 mm (published values: 1, 2, 3, 3.175 and 5 mm); safety margin 3 to 10 mm; spine width 5 to 15 mm when the spine carries text; two creases never closer than 5 mm.
7. The default format is the poche, 110 × 178 mm, spine 10.0 mm. Page creation fails with a clear error when a format does not fit the home sheet at 100 %: panel at most 130 × 183 mm on A4 with 5 mm ticks. Never scale the layout to fit. The `digest` and `a5` presets are professional-only.

**Boxes and marks**

8. Write MediaBox, BleedBox and TrimBox on every page with the formulas of §2.2. No ArtBox. CropBox absent or equal to MediaBox. `Rotate` 0. Identical boxes on all pages. Tests assert the point values of the §2.2 table to 0.001 pt.
9. Upload profile: no marks, no slug, `MediaBox = BleedBox`.
10. Marks profile: crop ticks and fold ticks as in §2.4; fold ticks dashed 1 mm on, 1 mm off; a 6 pt job line in the bottom slug; nothing inside the BleedBox. Tick colour is a profile option: registration colour (default) or 100 % K.
11. Implement the three crease encodings of §2.5. For every professional output also generate the plan PDF: a separate plain-PDF file showing trim, bleed, safety zones and creases with dimensions in millimetres, marked "NE PAS IMPRIMER / DO NOT PRINT".
12. Generate the French printer brief of §8.2 from the actual format, so that the numbers in the brief, the plan and the print file cannot disagree.
13. Keep writing trim size, spine width and crease positions in the `EBC.*` Info keys; preflight compares them with the boxes.

**Layout**

14. Default cover placement: uniform scale, letterboxed in a box inset 5 mm from the trim and from the crease (100 × 168 mm on the poche), on a flat ground that fills the bleed. Full bleed only when the title carries `trim_authorised`; then the cover's inner edge lies exactly on the crease.
15. By default the back, spine and front share one flat ground colour. A spine with its own colour extends 2 mm past each crease and never reaches the cover box.
16. Text, logos and the QR box stay 5 mm or more from the trim and from each crease on the cover panels. Spine text stays in the central 6 mm band and 5 mm from top and bottom, at 8 pt or more; the fitting routine caps all-caps text at about 24 pt and mixed case at about 17 pt on a 10 mm spine.
17. The inside spine strip carries nothing. On the home profile the inside page has a white ground.
18. Add the single-sided home variant: back | spine | front | 45 mm flap, three creases, A4 only, QR code and typed code on the flap, message and steps on the back panel.

**PDF and colour**

19. All professional outputs use the content subset of §3.2. Violations are errors raised by the writer, not silently repaired.
20. Write the page content and file structure with PDF 1.3 features only, whatever the label. The label (PDF/X-4 by default, PDF/X-1a:2001, PDF/X-3:2002) changes only the header version, the version markers and the metadata.
21. Always embed the output-intent profile. Default: FOGRA39 with "ISO Coated v2 300% (ECI)"; FOGRA51 and FOGRA52 by profile. Check the profile header on load: CMYK, output class, ICC version 2.
22. Load the output profile on demand, when a professional output is requested; do not link it into the WebAssembly binary. Do not commit or serve the ECI profiles until their licence file has been read and stored next to them. The CC0 SWOP-compatible profile in `assets/icc/` is for tests only and must never be the output intent of an order sent to a European printer.
23. Convert cover images with a real ICC transform to the output-intent space (relative colorimetric with black-point compensation by default). If the profile is unavailable, write the image as tagged sRGB in a PDF/X-4 file only for profiles with `allow_rgb`; otherwise refuse the professional output and say why. Never apply the naive conversion to a cover.
24. Write text, the typed code, the QR symbol and thin rules directly as 0/0/0/100, and white as 0/0/0/0. Convert flat colours through the ICC transform, then set channels below 2.5 % to 0 and channels from 2.5 % to 5 % to 5 %, then reduce C, M and Y if the total exceeds the profile's ink limit.
25. Default ink limit 300 %. Keep the 260 % and 270 % profiles disabled until an ink-limiting transform exists.
26. Set overprint on K100 text that lies on a non-white ground, and nowhere else.
27. Embed fonts as subsets; draw the spine title as outlines; honour `outline_all_text`. Enforce at layout time: no text under 6 pt, no reversed, multi-ink or outlined text under 8 pt, no stroke under 0.25 pt (0.5 pt reversed).
28. If CMYK images are stored as JPEG, add a test that renders the page and compares colours, to catch an inverted-channel file.
29. Home profile: plain PDF, sRGB JPEG passed through unchanged, text in DeviceGray, guides in light grey, the 100 mm ruler, and one instruction line: actual size, duplex on the short edge.

**QR symbol**

30. Encode the payload in upper case, request level M, accept the encoder's free upgrade to Q, and warn above version 4.
31. Fit the symbol and a 4-module quiet zone into a 33 mm box (allowed 26 to 40 mm). Keep the existing constants: error below 0.4 mm per module, warning below 0.5 mm. Draw the dark modules as one merged path in 0/0/0/100 on white.
32. Place the box on the inside page (or the flap), 5 mm or more from the trim and from every crease, never on the cover panel or the spine, with the host and the typed code in clear text beside it.
33. Add a test that renders the inside page at 150 dpi, at 300 dpi and at 97 % scale, decodes the symbol with an independent decoder, and compares the payload.

**Preflight, tests, release gates**

34. Implement the checks of §10.1 with stable finding codes. Run preflight in the browser before any download and on the server before any order. Any error blocks a professional order.
35. Add golden tests per profile and format: box values, crease positions, absence of marks in the upload profile, byte-for-byte determinism.
36. Do not implement folder-product templates until a printer's template has been obtained. When one is, store it as per-product data: panel size, spine width, flap outline, bleed.
37. A printer profile moves to `confirmed` only after the product template has been compared with our page (M1) and one printed proof has passed the manual list of §10.2. Record the date and the result in the profile record.
38. Before release, resolve open questions 2, 3 and 4 of §11 (ICC licence, PDF/X keys, QR sizes): each is a constant or an asset in the code that currently rests on memory.
