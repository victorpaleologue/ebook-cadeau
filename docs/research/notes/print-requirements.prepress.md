# Print requirements — prepress specifications published by European online printers

Research notes for the "eBook Cadeau" gift-card PDF (front cover + 10 mm spine + back, QR code inside).
Angle: what printers actually publish in their file-preparation guides. Date of research: 2026-11-12.

## 0. Method, confidence markers, and a hard limitation

- **[V2]** = found in two independent documents (for a printer's own spec: two distinct documents of that printer, or printer + third party). **[V1]** = one document. **[M]** = from memory, not verified in this session.
- Search results arrive as summaries, not full pages: numbers below are as reported in those summaries. Where a summary did not say which page a figure came from, I say so.
- **Limitation: the shared web-search budget for this run was exhausted after 24 queries (23 returned results).** The planned searches on ECI licence terms, basICColor, PDF/X OutputIntent keys, QR print sizing, Vistaprint EU, and deeper Gelato/Cloudprinter specs could not be run. Sections 6 and 7 are therefore almost entirely **[M]** and must be re-verified before anything is bundled or hard-coded.
- All printer specs are product-specific. Every printer surveyed says the product data sheet / template overrides the general guide (e.g. Flyeralarm's checklist: "achten Sie stets auf die Vorgaben im jeweiligen Produkt-Datenblatt" — [Flyeralarm checklist](https://www.flyeralarm.com/blog/wp-content/uploads/Flyeralarm_Checkliste_Druckdatenerstellung-1.pdf)) [V2].

## 1. Headline findings

1. **There is no single European bleed value.** Published values: 1 mm (Flyeralarm small formats), 2 mm (Onlineprinters, Saxoprint, Exaprint minimum, Printoclock FAQ), 3 mm (Pixartprinting books, Helloprint, Bookvault, Cloudprinter, Saxoprint packaging), 3.175 mm (Lulu), 5 mm (BoD, Printoclock template table, Saxoprint brochures). A generator must make bleed a per-printer parameter; 3 mm is the best default, with artwork that can be extended to 5 mm.
2. **Most online printers do not want crop marks at all.** Explicit "no marks" statements: Pixartprinting, Flyeralarm, Printoclock, Bookvault, BoD, Lulu. Marks are wanted by traditional book printers (Clays) and some trade printers (Baumer). Default for upload-to-web-printer output = no marks, correct TrimBox/BleedBox; marks only in the "print at home / local print shop" variant.
3. **Colour condition is split between FOGRA39 and FOGRA51.** FOGRA39 / ISO Coated v2 (300 %): Flyeralarm, Exaprint, Pixartprinting. FOGRA51 / PSO Coated v3 (and FOGRA52 for uncoated): Onlineprinters, Saxoprint.
4. **Total ink limit: 300 % is the common ceiling** (Onlineprinters, Saxoprint, Exaprint, Flyeralarm via its "ISO Coated V2 300 %" profile); 260 % for Onlineprinters express jobs; 270 % for Lulu.
5. **PDF flavour is split too:** PDF/X-4 (Onlineprinters, Saxoprint, Gelato), PDF/X-3:2002 (Flyeralarm), PDF/X-1a:2001 or X-3:2002 (Bookvault), "PDF 1.3" (Exaprint). A file with no transparency, CMYK/gray only and embedded fonts satisfies all of them and can be labelled either way.
6. **Minimums that agree across printers:** font size 6 pt; positive lines 0.25 pt, reversed lines 0.5 pt; images 300 dpi target (250 dpi floor at Saxoprint).

## 2. Comparison table — geometry and marks (as of 2026-11)

| Printer | Bleed (per side) | Safety margin (from trim) | Marks in file? | Sources / confidence |
|---|---|---|---|---|
| Pixartprinting (IT) | 3 mm for perfect-bound / wire-o books; otherwise per product template (green line = bleed, magenta = cut, red = safe area) | "safe area" per template; states a 1–2 mm mechanical cutting shift is normal | No crop or registration marks | [instructions article](https://support.pixartprinting.com/hc/en-us/articles/26076155387026-What-do-I-find-in-the-instructions), [perfect binding spec](https://www.pixartprinting.co.uk/printing-magazines-books-catalogues/special-finish-perfect-binding/), [wire-o spec](https://www.pixartprinting.co.uk/printing-magazines-books-catalogues/special-finish-wire-o-binding/) — bleed [V2]; "no marks" [V1], exact page not pinned (surfaced with the [flyer guide](https://www.pixartprinting.co.uk/blog/how-to-create-an-effective-flyer-the-complete-guide-from-idea-to-print/)) |
| Onlineprinters (DE) | 2 mm on flyers/postcards (DL flyer: data 10.9 x 21.4 cm for 10.5 x 21 cm trim); 10 mm on large format | 4 mm on flyers/postcards | not found | [DL flyer](https://www.onlineprinters.ie/p/flyers-leaflets-dl-printed-on-one-side), [A5 postcard](https://www.onlineprinters.ie/p/exklusive-postcards-a5), [A6 insert](https://www.onlineprinters.ie/p/inserts-exclusive-4-4-colour-printing-a6) [V2] |
| Flyeralarm (DE) | 1 mm on small formats; 1–10 mm depending on format | per data sheet; frames/outlines at least 4 mm from edge | No: "please do not create folding or cutting marks within the artwork format"; InDesign tutorial says select none of the Marks | [UK artwork requirements](https://www.flyeralarm.com/images/upload/content/_pdf/uk/Artwork_Requirements_UK.pdf), [bleed tutorial](https://www.flyeralarm.com/blog/en/tutorial-setting-up-bleeds-in-indesign-cc-illustrator-cc-and-scribus-1-4-5/), [PDF tutorial](https://www.flyeralarm.com/blog/en/creating-print-ready-pdf-with-indesign/) [V2] |
| Saxoprint (DE) | 2 mm standard; 3 mm packaging/folders/calendars; 5 mm brochures (preflight minimum) | at least 3 mm; UK error page: 5–7 mm from the data edge when bleed is 2 mm; 10 mm from a binding edge | not found | [Druckdatenanleitung, May 2024](https://api.saxoprint.de/media-by-file-name/Druckdatenanleitung-1.pdf), [UK artwork instructions](https://www.saxoprint.co.uk/service-help/artwork-instructions), [FAQ](https://www.saxoprint.de/service-und-hilfe/faq), [pages and covers](https://www.saxoprint.co.uk/service-help/error-diagnosis/pages-and-covers) [V2] |
| Exaprint (FR) | "fonds perdus" 2 mm minimum (charte, labels article); blog says at least 3 mm | "zone tranquille" 3 mm on most products, 5 mm stapled brochures, 10 mm "dos collé" | not found | [charte PAO](https://www.exaprint.fr/imprimerie-en-ligne/charte-pao), [fond perdu article](https://blog.exaprint.fr/cadres-impression-contraintes-et-astuces/), [6 erreurs](https://blog.exaprint.fr/6-erreurs-courantes-a-eviter-en-impression/) — zone tranquille [V2]; bleed conflicting 2 vs 3 mm |
| Printoclock (FR) | 2 mm (FAQ) vs 5 mm in the template table for most formats | "marge technique" 2 mm; 5 mm for "dos carré collé" | No "traits de coupe", no colour bar, no "hirondelles", no white margins | [FAQ](https://support.printoclock.com/article/comment-faire-mon-fichier-dimpression), [bien réaliser son fichier](https://www.printoclock.com/bien-realiser-son-fichier), [fond perdu](https://www.printoclock.com/fond-perdu), [technical specs 63](https://www.printoclock.com/technical-specs-html/63) [V1 each; bleed conflicting] |
| Helloprint (NL) | at least 3 mm; 5 mm or more on large products | at least 3 mm; 4 mm on bookmarks; blog says 5 mm for flyers | help-centre page on trim marks exists, content not retrieved | [help centre](https://www.helloprint.com/en-ie/help-centre/where-do-i-start-when-preparing-my-design-for-printing), [blog](https://www.helloprint.com/en-ie/blog/how-to-set-up-a-file-for-print), [bookmarks](https://www.helloprint.com/en-gb/bookmarksexclusivefinishing) [V2] |
| Vistaprint (NL/IE) | per product template ("Specs & Templates") | per template | not found | [business card file prep](https://www.vistaprint.com/hub/business-card-file-prep/) [V1]; the commonly quoted 1.5 mm EU bleed is [M], unverified |
| Lulu (US, prints in EU) | 0.125 in = 3.175 mm | 0.5 in (12.7 mm) in guide/API; one help article says 0.25 in | No trim, bleed or margin lines | [book creation guide](https://assets.lulu.com/media/guides/en/lulu-book-creation-guide.pdf), [API PDF settings](https://help.api.lulu.com/en/support/solutions/articles/64000254609-pdf-creation-settings), [cover upload](https://help.lulu.com/en/support/solutions/articles/64000282777-upload-your-cover-file) [V2] |
| BoD (DE) | 5 mm per side (12 x 19 cm page becomes 13 x 20 cm) | not found | Advises against crop marks | [BoD Grafiken und Farbmanagement](https://www.bod.ch/bodfiles/GLOBAL-Storage/documents/help-documents/bod-grafiken-und-farbmanagement.pdf) [V1] |
| Bookvault (UK) | 3 mm on every trimmed edge (A5 becomes 154 x 216 mm) | 5 mm on the cover template | No: production software adds marks | [bleed](https://help.bookvault.app/what-is-bleed-and-why-do-i-need-it), [crop marks](https://help.bookvault.app/should-i-supply-crop-marks), [PDF guide](https://help.bookvault.app/hubfs/Guide%20To%20PDFs%20-%20Digital%20Version%20.pdf?hsLang=en) [V2] |
| Cloudprinter (NL) | 3 mm | not found | not found | [Canva export article](https://knowledge.cloudprinter.com/mastering-canva-print-export-for-cloudprinter.com-compatibility) [V1]; the authoritative page is [How to build production ready PDFs](https://docs.cloudprinter.com/client/how-to-build-the-pdf/), content not retrieved |
| Gelato (NO) | 3 mm, from an unreliable third-party page only | not found | not found | [third-party page](https://processor-5xq.pages.dev/print-ready/gelato) — treat as unverified |

Context from non-surveyed printers: druck.at defaults to 2 mm bleed / 2 mm safety ([Infoblatt](https://cdn.druck.at/sites/default/files/Infoblatt_Druckdatenerstellung.pdf)) [V1]; IngramSpark 3 mm ([file requirements](https://www.ingramspark.com/blog/file-requirements-for-print-books)) and KDP 3.2 mm ([KDP help](https://kdp.amazon.com/help/topic/G201953020)) [V1]. Traditional book printers are the exception on marks: Clays asks for bleed, crop marks and spine-width marks ([Clays cover guide](https://www.clays.co.uk/wp-content/uploads/supplying-cover-files-for-print.pdf)) and Baumer asks for PDF/X-4 with 3 mm bleed and crop marks ([Baumer](https://www.baumer.ch/fileadmin/files/downloads/Baumer_UEbermittlung_von_Druckdaten_PSO_PSD_2021_DE.pdf)) [V1].

**Crop-mark geometry.** None of the surveyed online printers publishes mark offset / length / stroke, because they do not want marks. Conventional values, for the home-print and local-print-shop variant only [M]: stroke 0.25 pt; length about 5 mm; offset from trim at least equal to the bleed (3 mm) so marks never sit inside the bleed; colour "Registration" (prints on all separations) for offset, or 100 % K for digital-only output; fold marks for the two spine folds drawn the same way, dashed. Note that Saxoprint warns registration colour equals 400 % ink and must never be used inside the printed area ([Saxoprint colours](https://www.saxoprint.de/service-und-hilfe/fehlerdiagnose/farben)) [V1].

## 3. Comparison table — PDF flavour, colour, ink

| Printer | PDF standard / version | Colour and output condition | Total ink (TAC) | Black rules | Confidence / source |
|---|---|---|---|---|---|
| Onlineprinters | PDF/X-4:2008; identical page sizes; no password | CMYK, FOGRA51 (PSO Coated v3) for coated, FOGRA52 for uncoated | 300 % normal production, 260 % for rush/express | black text under 12 pt is automatically set to overprint | [PDF settings FAQ](https://onlineprinters.ie/c/faq/settings-print-pdf), product pages above, [Überdrucken](https://www.onlineprinters.de/c/druckdaten/ueberdrucken) [V1 each]; TAC figure is from an onlineprinters.de page surfaced in the same search, exact page not pinned [V1] |
| Flyeralarm | PDF/X-3:2002 (no transparency); PDF/X-4 on some data sheets | "ISO Coated V2 300% (ECI)" in general, "ISO Coated V2 (ECI)" for large-format digital; profile must be set as output intent | implied 300 % by the profile | — | [checklist](https://www.flyeralarm.com/blog/wp-content/uploads/Flyeralarm_Checkliste_Druckdatenerstellung-1.pdf), [UK requirements](https://www.flyeralarm.com/images/upload/content/_pdf/uk/Artwork_Requirements_UK.pdf), [PDF/X article](https://flyeralarm.com/blog/en?p=24986) [V2] |
| Saxoprint | PDF/X-4 preferred, or PDF 1.6; avoid layers (Optional Content); keep native transparency | CMYK; PSO Coated v3 (FOGRA51) for coated | 300 % guaranteed maximum in offset | black text 100 % K, not 4c | [guide 2024](https://api.saxoprint.de/media-by-file-name/Druckdatenanleitung-1.pdf), [UK instructions](https://www.saxoprint.co.uk/service-help/artwork-instructions), [colours](https://www.saxoprint.de/service-und-hilfe/fehlerdiagnose/farben) [V2 for PDF flavour; V1 for TAC and profile] |
| Exaprint | "PDF 1.3 optimal" | old guide names ISO Coated V2 300 %; Exaprint supplies its own profiles | 300 %, excess corrected automatically | rich black for solids = K100 + C30 M30 Y30; text in K only; K100 always overprints | [Guide PAO PDF](https://static.exaprint.fr/marketing/cms/exaprint/guidePAO/GuidePAO-telechargement.pdf), [assistance PAO](https://www.exaprint.fr/page/assistance-pao), [conseils PAO](https://www.exaprint.fr/conseils-pao/), [brochure errors](https://blog.exaprint.fr/impression-brochure-erreurs-pao/) — TAC [V2]; guide PDF is roughly 9 years old |
| Pixartprinting | PDF at 1:1 scale, not password-protected | CMYK, FOGRA39; non-CMYK files are converted automatically with a separation profile; Pantone not used in process printing | not found | text 100 % K; tints never below 5 % | [perfect binding spec](https://www.pixartprinting.co.uk/printing-magazines-books-catalogues/special-finish-perfect-binding/), [CMYK article](https://support.pixartprinting.com/hc/en-us/articles/360017440400-What-does-CMYK-mean), [inks advice](https://www.pixartprinting.co.uk/blog/inks-advices/), [2017 third-party checklist](https://printicom.wordpress.com/2017/07/06/final-checklist-before-uploading-your-documents/) [V2 for FOGRA39] |
| Printoclock | PDF, JPEG or TIFF; flatten images, outline fonts | CMJN preferred, RVB accepted | no number; warns against heavy ink, especially black solids | — | [bien réaliser son fichier](https://www.printoclock.com/bien-realiser-son-fichier) [V1] |
| Bookvault | PDF/X-1a:2001 or PDF/X-3:2002; transparency flattened | not found | not found | — | [PDF/X article](https://help.bookvault.app/what-is-pdf/x-1a2001-and-pdf/x-32002), [validation checks](https://help.bookvault.app/what-does-the-bookvault-file-validation-check-for) [V2] |
| Lulu | print-ready PDF, fonts embedded, layers flattened, single cover spread, no password | CMYK "Coated GRACoL 2006" (a US condition) | never above 270 % | — | [guide](https://assets.lulu.com/media/guides/en/lulu-book-creation-guide.pdf), [cover upload](https://help.lulu.com/en/support/solutions/articles/64000282777-upload-your-cover-file) [V1] |
| BoD | flatten transparency before export | keep CMYK images in CMYK; profile not found | not found | — | [BoD guide](https://www.bod.ch/bodfiles/GLOBAL-Storage/documents/help-documents/bod-grafiken-und-farbmanagement.pdf) [V1] |
| Gelato | PDF/X-4 for paper products | third-party claim: sRGB uploads converted by Gelato (unverified) | not found | — | [Gelato support](https://connect-support.gelato.com/en/articles/9858024-gelatocreate-what-file-standard-is-used-for-paper-based-production) [V1] |
| Helloprint / Vistaprint | PDF (also AI, EPS, JPEG, PNG) | CMYK | not found | — | [Helloprint design solutions](https://www.helloprint.com/en-ie/design-solutions), [Vistaprint](https://www.vistaprint.com/hub/business-card-file-prep/) [V1] |

Spot colours: none of the general guides wants them for standard 4c products; Pixartprinting only uses a spot swatch as a technical separation for finishes ("Finishing" = 0/50/100/0, vector, overprint) ([text for special finishes](https://support.pixartprinting.com/hc/en-us/articles/27655380376466-How-to-properly-set-up-text-for-special-finishes)) [V1]. The generator should emit no spot colours.

File size limits: only one figure found — Pixartprinting 1 GB per file, from a 2017 third-party post ([source](https://printicom.wordpress.com/2017/07/06/final-checklist-before-uploading-your-documents/)) [V1, dated]. Nothing found for the others; a jacket PDF of a few MB is far below any plausible limit.

## 4. Minimum line weights, type sizes, resolution, fonts

| Item | Published values | Confidence |
|---|---|---|
| Positive line (dark on light) | 0.25 pt = 0.09 mm at Onlineprinters ([Linienstärke](https://www.onlineprinters.de/c/druckdaten/linienstaerke)) and Flyeralarm ([UK requirements](https://www.flyeralarm.com/images/upload/content/_pdf/uk/Artwork_Requirements_UK.pdf)); 0.25 pt at Pixartprinting ([spec](https://www.pixartprinting.co.uk/printing-magazines-books-catalogues/special-finish-perfect-binding/)); Saxoprint 0.125 pt at 40 % ink or more, thinner lines are thickened automatically except 0 pt hairlines ([guide](https://api.saxoprint.de/media-by-file-name/Druckdatenanleitung-1.pdf)), while its blog recommends 0.75 pt ([blog](https://www.saxoprint.de/blog/druckvorlagen/schriften-und-linien)) | [V2] for 0.25 pt |
| Reversed line (light on dark) | 0.5 pt = 0.18 mm at Onlineprinters and Flyeralarm (same sources) | [V2] |
| Minimum font size | 6 pt at Onlineprinters ([Schriften](https://www.onlineprinters.de/c/data/schriften)), Flyeralarm ([checklist](https://www.flyeralarm.com/blog/wp-content/uploads/Flyeralarm_Checkliste_Druckdatenerstellung-1.pdf)), Saxoprint, Pixartprinting | [V2] |
| Stricter cases | Pixartprinting special finishes: lines 1 pt, type 10 pt; rigid media: lines 0.5 pt, type 10 pt ([legible text](https://support.pixartprinting.com/hc/en-us/articles/27654779357970-How-to-ensure-the-text-is-the-correct-size-to-be-legible-once-printed)); Saxoprint foil on packaging: 12 pt | [V1] |
| Reversed / 4c text minimum | no printer figure found; trade rule of thumb is 8 pt or more and a sturdy weight | [M] |
| Image resolution | 300 dpi at Pixartprinting, Onlineprinters, Printoclock, Bookvault, Vistaprint; 300–356 dpi at Flyeralarm; at least 250 dpi at Saxoprint; 100–150 dpi for large format | [V2] |
| Fonts | Pixartprinting: embedded, **not** outlined; Bookvault and Lulu: embedded (validated); Printoclock: outline ("vectoriser") | [V1 each] — embed-and-subset satisfies all but Printoclock, where outlining is also harmless |
| Transparency | forbidden by PDF/X-3:2002 (Flyeralarm) and flattened for Bookvault, Lulu, BoD; kept native for Saxoprint and Onlineprinters (PDF/X-4) | [V2] |
| Overprint | Onlineprinters auto-overprints black text under 12 pt; Exaprint overprints all K100 | [V1 each] |

Consequence for scraped covers: a 110 x 178 mm front plus 3 mm bleed needs about 1370 x 2170 px at 300 dpi (about 1140 x 1810 px at 250 dpi). Typical retailer thumbnails (500–800 px wide) land at 110–185 dpi and will trigger low-resolution warnings. Cover sourcing or upscaling is a prepress requirement, not a nicety.

## 5. Page boxes

- Printers that forbid marks expect the page to be exactly trim + bleed: Onlineprinters "data format (incl. 2 mm bleed)", Bookvault 154 x 216 mm for A5, Flyeralarm "Datenformat" [V2, sources above]. So for the upload variant: MediaBox = BleedBox = trim + bleed, TrimBox = trim, no CropBox.
- A PDF/X file must carry the trimmed and untrimmed page sizes and the intended output condition ([Macwelt on PDF/X-4](https://www.macwelt.de/article/956117/pdf-workflow-in-der-druckproduktion.html)); general box definitions in [pdf2go's box explainer](https://pdf2go.com/blog/what-are-pdf-boxes) and [Wikipedia PDF/X](https://Www.Wikipedia.org/wiki/PDF/X) [V1]. Exact rule from the standard [M]: each page has a MediaBox and either a TrimBox or an ArtBox, never both; nesting is TrimBox ⊆ BleedBox ⊆ CropBox ⊆ MediaBox.
- Variant with marks (home / local printer): MediaBox = trim + 2 x (bleed + mark offset + mark length + about 2 mm slack), BleedBox = trim + bleed, TrimBox = trim [M].
- Bleed is a parameter: the same layout must be exportable at 1, 2, 3, 3.175 and 5 mm. Background art should always be generated to 5 mm and clipped by the BleedBox.

## 6. ICC profiles and the PDF/X OutputIntent (mostly unverified — search budget ran out)

Verified in this session: which conditions printers ask for (section 3), the exact Flyeralarm wording "ISO Coated V2 300% (ECI)", and that the profile must be present as the PDF output intent [V2]. PDF/X-4 may contain live transparency and layers, unlike X-1a and X-3 ([Cuno Druck PDF/X sheet](https://www.cunodruck.de/wp-content/uploads/2023/11/PDF-X-GCC_23.pdf)) [V1].

Everything below is **[M]** and needs a verification pass against eci.org, color.org and ISO 15930 before implementation:

| Profile (file) | Characterisation | Paper | TAC |
|---|---|---|---|
| ISO Coated v2 (ECI) — `ISOcoated_v2_eci.icc` | FOGRA39L | coated, ISO 12647-2:2004 PT1/2 | 330 % |
| ISO Coated v2 300% (ECI) — `ISOcoated_v2_300_eci.icc` | FOGRA39L | same | 300 % |
| PSO Coated v3 — `PSOcoated_v3.icc` | FOGRA51 | premium coated, ISO 12647-2:2013 PS1, M1 | 300 % |
| PSO Uncoated v3 (FOGRA52) — `PSOuncoated_v3_FOGRA52.icc` | FOGRA52 | wood-free uncoated, PS5 | 300 % |

- **ECI licence [M]:** the ECI offset profiles are free of charge and ship with a zlib/libpng-style licence text (use, alteration and redistribution permitted, origin not to be misrepresented, altered versions marked, notice kept). Debian redistributes them on that basis. That is compatible with a BSD-3-Clause repository, provided the licence file travels with the profile. Not verified this session.
- **basICColor [M]:** offers free-to-download offset profiles built on the same FOGRA data; I recall no explicit redistribution grant. Do not bundle without reading the terms.
- **Adobe profiles (Coated FOGRA39) [M]:** free download, redistribution only under Adobe's separate bundling licence. Avoid.
- **Size [M]:** each ECI CMYK profile is roughly 1.8–2.4 MB — material for a WASM bundle; fetch lazily.
- **OutputIntent dictionary [M]:** `/Type /OutputIntent`, `/S /GTS_PDFX`, `/OutputConditionIdentifier` (required, e.g. `FOGRA39`, `FOGRA51`), `/RegistryName (http://www.color.org)` when the identifier is a registered characterisation, `/Info` (human-readable, required when the identifier is not registered), `/OutputCondition` (optional), `/DestOutputProfile` (ICC stream, `/N 4`).
- **Embedded or not [M]:** PDF/X-1a and X-3 allow omitting the profile when the identifier names a registered characterisation; PDF/X-4 requires it embedded (X-4p allows an external reference). Embedding always is the safe choice because printer preflights look for it.
- **Other PDF/X keys [M]:** `GTS_PDFXVersion` (Info dictionary for X-1a/X-3; XMP `pdfxid:GTS_PDFXVersion` for X-4), `/Trapped` True or False, title, trailer `/ID`, XMP metadata for X-4, no encryption, all fonts embedded, PDF 1.3 for the :2001/:2002 parts, PDF 1.6 for X-4.

## 7. QR codes in print (unverified — no searches could be run)

Supported by this session's findings: black text and fine elements should be K-only (Saxoprint, Exaprint, Pixartprinting) [V2]; positive detail of at least 0.25 pt [V2]; cutting shift of 1–2 mm [V1].

From memory, to be verified [M]:

- **Quiet zone:** 4 modules on all sides (ISO/IEC 18004). Many phone readers tolerate 2; do not rely on it.
- **Module size:** at least 0.4 mm for reliable phone scanning (GS1 gives about 0.396 mm minimum, 0.495 mm target for consumer scanning); 0.5 mm or more is a comfortable house rule.
- **Overall size:** rule of thumb is symbol width of about scan distance / 10. A card held at 20–30 cm needs 20–30 mm; use a symbol of 25 mm or more plus quiet zone (about 32 mm box), 30 mm if space allows.
- **Error correction:** L 7 %, M 15 %, Q 25 %, H 30 %. Higher levels add modules for the same payload, so modules shrink at a fixed size. M is the default trade-off; Q if the card is likely to be scuffed; H only for a logo overlay.
- **Capacity (bytes / alphanumeric):** v2 M 26/38, v3 M 42/61, v3 Q 32/47, v4 M 62/90, v4 Q 46/67, v5 Q 60/87. A short uppercase URL in alphanumeric mode fits v3 (29 modules): 25 mm wide gives 0.86 mm modules.
- **Ink and contrast:** dark modules 100 % K only, as vector rectangles merged into one path, on white or a very light tint; never 4c black (misregistration blurs edges) and no reversed code. ISO/IEC 15415 grades symbol contrast: 40 % or more for grade C, 70 % for A.
- **Placement:** keep the symbol and its quiet zone at least 5 mm from trim and from the spine folds.

## 8. Proposed house spec for the generator (derived, not a printer fact)

| Parameter | Default | Rationale |
|---|---|---|
| Bleed | 3 mm, configurable 1–5 mm; art always drawn to 5 mm | covers every surveyed printer |
| Safety margin | 5 mm from trim and from each spine fold line where space allows | maximum of the published 3 / 4 / 5 mm values |
| Spine (10 mm) | type confined to the central 6 mm | 1–2 mm cutting and folding shift per side |
| Marks | none for web-printer profiles; crop and fold marks only for home / local print | section 2 |
| Boxes | MediaBox = BleedBox = trim + bleed; TrimBox = trim | section 5 |
| Content | no transparency, no layers, no spot colours, CMYK and gray only, fonts embedded and subset | valid as X-1a, X-3 and X-4 content |
| Label | PDF/X-4 by default; PDF/X-1a:2001 for Bookvault, Flyeralarm, Exaprint | section 3 |
| Output intent | FOGRA39 (ISO Coated v2 300 %) default; FOGRA51 for Onlineprinters and Saxoprint | section 3 |
| TAC | 300 % or less; 260 % profile for express; 270 % for Lulu | section 3 |
| Black | text, short code and QR in K100 only; large black solids K100 + C30 M30 Y30 | Exaprint, Saxoprint |
| Minimums | type 6 pt (8 pt reversed), lines 0.25 pt (0.5 pt reversed), images 300 dpi target / 250 dpi warning / 150 dpi error | section 4 |

Per-printer profiles belong in data (one record per printer and product), because every value above is overridden by the product data sheet.

## Sources

Printer documents (all retrieved as search-result summaries on 2026-11-12):

- Pixartprinting: [instructions](https://support.pixartprinting.com/hc/en-us/articles/26076155387026-What-do-I-find-in-the-instructions), [perfect binding](https://www.pixartprinting.co.uk/printing-magazines-books-catalogues/special-finish-perfect-binding/), [wire-o](https://www.pixartprinting.co.uk/printing-magazines-books-catalogues/special-finish-wire-o-binding/), [special finishes text](https://support.pixartprinting.com/hc/en-us/articles/27655380376466-How-to-properly-set-up-text-for-special-finishes), [legible text](https://support.pixartprinting.com/hc/en-us/articles/27654779357970-How-to-ensure-the-text-is-the-correct-size-to-be-legible-once-printed), [CMYK](https://support.pixartprinting.com/hc/en-us/articles/360017440400-What-does-CMYK-mean), [inks advice](https://www.pixartprinting.co.uk/blog/inks-advices/), [flyer guide](https://www.pixartprinting.co.uk/blog/how-to-create-an-effective-flyer-the-complete-guide-from-idea-to-print/), [2017 third-party checklist](https://printicom.wordpress.com/2017/07/06/final-checklist-before-uploading-your-documents/)
- Onlineprinters: [PDF settings FAQ](https://onlineprinters.ie/c/faq/settings-print-pdf), [DL flyer](https://www.onlineprinters.ie/p/flyers-leaflets-dl-printed-on-one-side), [A5 postcard](https://www.onlineprinters.ie/p/exklusive-postcards-a5), [A6 insert](https://www.onlineprinters.ie/p/inserts-exclusive-4-4-colour-printing-a6), [Überdrucken](https://www.onlineprinters.de/c/druckdaten/ueberdrucken), [Überdrucken magazine](https://www.onlineprinters.de/magazin/ueberdrucken-und-aussparen-indesign/), [Linienstärke](https://www.onlineprinters.de/c/druckdaten/linienstaerke), [Schriften](https://www.onlineprinters.de/c/data/schriften)
- Flyeralarm: [checklist](https://www.flyeralarm.com/blog/wp-content/uploads/Flyeralarm_Checkliste_Druckdatenerstellung-1.pdf), [UK artwork requirements](https://www.flyeralarm.com/images/upload/content/_pdf/uk/Artwork_Requirements_UK.pdf), [ready for printing](https://www.flyeralarm.com/blog/uk/ready-for-printing/), [bleed tutorial](https://www.flyeralarm.com/blog/en/tutorial-setting-up-bleeds-in-indesign-cc-illustrator-cc-and-scribus-1-4-5/), [PDF tutorial](https://www.flyeralarm.com/blog/en/creating-print-ready-pdf-with-indesign/), [PDF/X article](https://flyeralarm.com/blog/en?p=24986)
- Saxoprint: [Druckdatenanleitung May 2024](https://api.saxoprint.de/media-by-file-name/Druckdatenanleitung-1.pdf), [UK artwork instructions](https://www.saxoprint.co.uk/service-help/artwork-instructions), [FAQ](https://www.saxoprint.de/service-und-hilfe/faq), [colours](https://www.saxoprint.de/service-und-hilfe/fehlerdiagnose/farben), [fonts and lines blog](https://www.saxoprint.de/blog/druckvorlagen/schriften-und-linien), [pages and covers UK](https://www.saxoprint.co.uk/service-help/error-diagnosis/pages-and-covers), [Seiten und Umschläge](https://www.saxoprint.de/service-und-hilfe/fehlerdiagnose/seiten-umschlaege)
- Exaprint: [Guide PAO PDF](https://static.exaprint.fr/marketing/cms/exaprint/guidePAO/GuidePAO-telechargement.pdf), [charte PAO](https://www.exaprint.fr/imprimerie-en-ligne/charte-pao), [assistance PAO](https://www.exaprint.fr/page/assistance-pao), [conseils PAO](https://www.exaprint.fr/conseils-pao/), [fond perdu](https://blog.exaprint.fr/cadres-impression-contraintes-et-astuces/), [6 erreurs](https://blog.exaprint.fr/6-erreurs-courantes-a-eviter-en-impression/), [brochure errors](https://blog.exaprint.fr/impression-brochure-erreurs-pao/)
- Printoclock: [FAQ](https://support.printoclock.com/article/comment-faire-mon-fichier-dimpression), [bien réaliser son fichier](https://www.printoclock.com/bien-realiser-son-fichier), [fond perdu](https://www.printoclock.com/fond-perdu), [technical specs 63](https://www.printoclock.com/technical-specs-html/63)
- Helloprint: [help centre](https://www.helloprint.com/en-ie/help-centre/where-do-i-start-when-preparing-my-design-for-printing), [blog](https://www.helloprint.com/en-ie/blog/how-to-set-up-a-file-for-print), [bookmarks](https://www.helloprint.com/en-gb/bookmarksexclusivefinishing), [design solutions](https://www.helloprint.com/en-ie/design-solutions)
- Vistaprint: [business card file prep](https://www.vistaprint.com/hub/business-card-file-prep/)
- Lulu: [book creation guide](https://assets.lulu.com/media/guides/en/lulu-book-creation-guide.pdf), [API PDF settings](https://help.api.lulu.com/en/support/solutions/articles/64000254609-pdf-creation-settings), [cover upload](https://help.lulu.com/en/support/solutions/articles/64000282777-upload-your-cover-file)
- BoD: [Grafiken und Farbmanagement](https://www.bod.ch/bodfiles/GLOBAL-Storage/documents/help-documents/bod-grafiken-und-farbmanagement.pdf)
- Bookvault: [bleed](https://help.bookvault.app/what-is-bleed-and-why-do-i-need-it), [crop marks](https://help.bookvault.app/should-i-supply-crop-marks), [PDF/X](https://help.bookvault.app/what-is-pdf/x-1a2001-and-pdf/x-32002), [validation](https://help.bookvault.app/what-does-the-bookvault-file-validation-check-for), [PDF guide](https://help.bookvault.app/hubfs/Guide%20To%20PDFs%20-%20Digital%20Version%20.pdf?hsLang=en)
- Gelato: [file standard](https://connect-support.gelato.com/en/articles/9858024-gelatocreate-what-file-standard-is-used-for-paper-based-production); Cloudprinter: [Canva export](https://knowledge.cloudprinter.com/mastering-canva-print-export-for-cloudprinter.com-compatibility), [production PDF docs](https://docs.cloudprinter.com/client/how-to-build-the-pdf/)
- Context: [druck.at Infoblatt](https://cdn.druck.at/sites/default/files/Infoblatt_Druckdatenerstellung.pdf), [Clays](https://www.clays.co.uk/wp-content/uploads/supplying-cover-files-for-print.pdf), [Baumer](https://www.baumer.ch/fileadmin/files/downloads/Baumer_UEbermittlung_von_Druckdaten_PSO_PSD_2021_DE.pdf), [IngramSpark](https://www.ingramspark.com/blog/file-requirements-for-print-books), [KDP](https://kdp.amazon.com/help/topic/G201953020), [Macwelt](https://www.macwelt.de/article/956117/pdf-workflow-in-der-druckproduktion.html), [Cuno Druck](https://www.cunodruck.de/wp-content/uploads/2023/11/PDF-X-GCC_23.pdf), [pdf2go boxes](https://pdf2go.com/blog/what-are-pdf-boxes), [Wikipedia PDF/X](https://Www.Wikipedia.org/wiki/PDF/X)

## Could not verify

- **Everything in sections 6 and 7 marked [M]:** ECI licence wording and redistribution right, basICColor and Adobe terms, profile TAC values and file sizes, exact OutputIntent and PDF/X key requirements, all QR sizing, capacity and contrast figures. No search could be run on these (budget exhausted).
- Crop-mark geometry (offset, length, stroke, colour): no surveyed printer publishes it; values given are convention from memory.
- Pixartprinting: total ink limit, current upload size limit, the exact page stating "no crop marks".
- Onlineprinters: the exact page carrying the 300 % / 260 % ink figures; its crop-mark policy.
- Flyeralarm: bleed and safety values on a current flyer or card data sheet (an archived 2015 A6 sheet was ambiguous: 1 mm and 4 mm, order unclear); explicit TAC statement.
- Saxoprint: crop-mark policy and rich-black recipe (the May 2024 guide was only seen in summary).
- Exaprint: currently required ICC profile and whether 2 mm or 3 mm bleed applies to cards (sources conflict).
- Printoclock: which of 2 mm or 5 mm bleed applies to a folded card.
- Helloprint, Vistaprint: trim-mark policy, ICC profile, TAC; Vistaprint's 1.5 mm EU bleed.
- Gelato and Cloudprinter: bleed, colour space, TAC and box requirements from first-party documents.
- BoD and Bookvault: ICC profile and TAC.
- Minimum reversed-text size: no printer figure found.
- Whether any of these printers offers a folded-card product with a 10 mm spine (two parallel creases); that is a product question, not covered here.
