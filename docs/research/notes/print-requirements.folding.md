# Print requirements — the physical object (wraparound cover with a 10 mm spine)

Research notes, 2026-11-12. Scope: creasing/folding rules, spine rules, cover templates of the POD platforms, trim sizes vs sheet sizes, catalogue products that can carry the design, print-at-home case.

**Method and limits.** 25 web searches were run (EN/FR/DE). WebFetch is blocked, so every sourced fact below comes from search-result extracts, not from a full read of the page. The shared search budget ran out before the last topics (bellyband, book-shaped box, notebook catalogue specs, Lulu bleed, Saxoprint data sheet, hinge-score practice) could be searched; these are marked [M] and listed under "Could not verify".

Confidence markers: **[V2]** = seen in >= 2 independent sources, **[V1]** = single source, **[M]** = from memory, not verified in this session, **[calc]** = my own arithmetic from sourced inputs.

---

## 1. Creasing, scoring, folding, grain

| Topic | Finding | Conf. |
|---|---|---|
| Vocabulary | *Creasing* (FR *rainage*, DE *Rillung*) compresses the fibres into a hinge without cutting the surface; *scoring* cuts/ploughs a shallow channel (DE *Nuten* removes material); *folding* (FR *pliage*, DE *Falzen*) is the actual bend. Creasing is recommended for heavy coated and digitally printed sheets, scoring is adequate for lighter uncoated stock ([Mid-State Litho](https://blog.midstatelitho.com/creasing-vs-scoring/), [druckerei.de glossary](https://druckerei.de/glossar/nuten/)) | V2 |
| Threshold | Gmund: creasing recommended from **170 g/m2**, fold parallel to the fibres ([Gmund digital printing tips, FR](https://gmund.com/shop/media/pdf/instructions/Gmund_OG_mit_Digital_Printing_Tips_fr.pdf)). Heidelberg describes a scoring set-up "ideal for 170 gsm or more", other tooling up to 350 gsm ([Heidelberg tips](https://www.heidelberg.com/global/media/en/global_media/company___publications/heidelberg_news/tips_tricks/279_tips_tricks_de.pdf)) | V2 |
| Counter-rule | Mid-State Litho: coated stock above 80 lb cover (about 216 g/m2) should be creased, uncoated can be scored up to 100 lb cover ([source](https://blog.midstatelitho.com/creasing-vs-scoring/)). So 170 g/m2 is a manufacturer rule of thumb, not a standard | V1 |
| Grain | Paper folds cleanly with the grain and cracks across it; digital presses often force cross-grain folds because of the sheet size, which makes creasing more important ([CreativePro](https://creativepro.com/paper-tips-going-against-the-grain/), [Gmund](https://gmund.com/shop/media/pdf/instructions/Gmund_OG_mit_Digital_Printing_Tips_fr.pdf)) | V2 |
| Ink at the fold | Print Pas Cher warns that with heavy ink coverage the paper may tear in the folds and that this is not a ground for complaint ([printpascher.com](https://www.printpascher.com/chemise-rabats.html)) | V1 |
| Varnish at the fold | Keep spot varnish off crease lines (about 1.5 mm clear at one German printer, [printprinz Veredelungen](https://www.printprinz.de/support/veredelungen); same advice in a French packaging template, [Smilepack](https://smilepack.fr/files/subscribers/d72b0b53-b3d0-49ff-adfd-37d51546ebde/ftp/gabarits/Coussin%20technique%20smilepack.pdf)) | V2 |

Consequence for the jacket: both creases run parallel to the spine, so the grain must run **parallel to the spine** (top-to-bottom of the cover). On an SRA3 sheet (320 x 450 mm) a spread laid with its spine along the 450 mm edge needs long-grain sheets; laid with the spine along the 320 mm edge it needs short-grain sheets [calc]. Ordinary A4 office paper is normally long grain [M], so an A4-landscape home print folds *across* the grain: scoring before folding matters there.

### Minimum distance between two parallel creases

| Source | Rule | Conf. |
|---|---|---|
| printing.com (UK) folder file guide | On a folder with capacity/gusset, **parallel creases must be at least 5 mm apart**; a crease must be at least 5 mm from a parallel die-cut line; gussets above 5 mm in whole millimetres ([printing.com](https://www.printing.com/uk/file-supply/how-to-supply-presentation-folder-files); identical text at sister brand [Nettl](https://www.nettl.com/uk/shop/file-supply/how-to-supply-presentation-folder-files)) | V1 |
| Creasing machines | Hardware minimum between creases is far lower: 0.1 mm on Formax Atlas C200 and Morgana AutoCreaser 50 ([Formax](https://manualedistruzioni.it/formax/binding-machine/atlas-c200/user-manual/v177), [Morgana](https://manuals.online/morgana/binding-machine/autocreaser-50/user-manual/1swt6)); 1.0 mm steps on Konica Minolta inline creasers ([KM manual](https://manuals.konicaminolta.eu/ineo-plus-2060L-2060-2070/EN/contents/id04-_102709920.html)) | V2 |
| Edge distance | Morgana DigiFold Pro: minimum crease distance from the tail edge 50 mm ([manual](https://media.mybinding.com/media/manuals/Morgana-DigiFold-Pro-User-Manual.pdf)); rule of thumb 5 mm between a crease and the sheet edge ([lazi-akademie wiki](https://www.lazi-akademie.de/wiki/drucktechnik/weiterverarbeitung/rillung/)) | V1 each |
| Machine *folding* | One German online printer gives 50 mm minimum fold-to-fold and 50 mm to the final format for machine folds (result set: [printprinz Weiterverarbeitung](https://www.printprinz.de/support/weiterverarbeitung), [viperprint Falzflyer](https://viperprint.de/flyer/falzflyer); exact page not pinned) | V1 |
| Standard catalogue spines | Online printers sell folders with a double crease forming a 5 mm or 10 mm spine (section 5) | V2 |

Reading: a 10 mm spine made of two creases is twice the 5 mm practical minimum and is a catalogue-standard geometry. It cannot be produced as a machine *fold* (panel far below 50 mm): order **crease only, delivered flat** ("rainage, livré à plat") or a folder product.

---

## 2. Crease and fold marks in the PDF

There is no single convention; the printers found disagree on colours.

| Printer / guide | Convention | Conf. |
|---|---|---|
| Pixartprinting instruction sheet (FR) | Cut, crease and fold lines defined as a **spot colour set to overprint** ([copy of instruction sheet](https://www.cipherbliss.com/dessins/Qzines/maquettes/imprimeur%20docs/pixartprinting_instruction_7_fr.pdf)) | V1 |
| Papeo amalgam tutorial (FR) | **Magenta** line = *rainage* / *pli*, cyan = pre-cut, black = cut ([papeo](https://upload.prod.papeo.fr/media/tuto-pdf/tuto-imprimerie-amalgame.pdf)) | V1 |
| Pinguindruck folder data sheet (DE) | Cut edges **magenta**, crease lines **cyan** ([data sheet](https://pinguindruck.de/load/dateien/mappe_09-12_4s2ka6_datenblatt.pdf)) | V1 |
| German printer rules | Crease lines as overprinting spot colour named "rillen", or shown in a separate view file; vector lines on a separate layer (result set: [printprinz](https://www.printprinz.de/support/weiterverarbeitung), [lazi-akademie](https://www.lazi-akademie.de/wiki/drucktechnik/weiterverarbeitung/rillung/)) | V1 |
| EPFL Repro guide | Cut path as magenta spot colour named `CutContour` ([EPFL](https://www.epfl.ch/campus/services/repro/wp-content/uploads/2025/11/REPRO-Guide-PAO.pdf)) | V1 |
| WTTB (UK) folders | **Remove the "Guide" layer** before exporting the PDF: the printer applies creases from its own die ([WTTB](https://wttb.co.uk/support-guides/presentation-folder-styles-setup)) | V1 |

Implications for the generator (design inference):

- Technical lines must be **switchable per printer profile**: (a) none inside the page, short tick marks outside the trim only; (b) a separate optional-content layer with a named overprinting spot colour (name and colour configurable: `rillen`, `Crease`, `Rainage`, `CutContour`); (c) a separate proof/"view" PDF with the lines drawn.
- The default safe output is (a): fold ticks at top and bottom, outside the trim, never touching live artwork. Dashed fold ticks vs solid crop marks is the DTP convention I remember [M]; no printer page confirming "dashed" was retrieved.
- Standard bleed in France is 3 mm ([EPFL](https://www.epfl.ch/campus/services/repro/wp-content/uploads/2025/11/REPRO-Guide-PAO.pdf), [J'imprime en France](https://www.jimprimeenfrance.fr/impression-chemise-rabat-fine)) [V2]; some packaging templates ask 5 mm ([Smilepack](https://smilepack.fr/files/subscribers/d72b0b53-b3d0-49ff-adfd-37d51546ebde/ftp/gabarits/Coussin%20technique%20smilepack.pdf)) [V1].

---

## 3. Spine rules: width, tolerance, text

### 3.1 Platform cover templates compared (as of 2026-11)

| Platform | Bleed | Spine width formula | Spine text allowed from | Spine text side margin | Hinge / other |
|---|---|---|---|---|---|
| Amazon KDP paperback | 0.125 in (3.2 mm) top, bottom, outside; cover text >= 0.125 in inside trim ([KDP help](https://kdp.amazon.com/en_US/help/topic/G201953020)) [V2] | pages x 0.002252 in (0.0572 mm) white / standard colour; 0.002347 in (0.0596 mm) premium colour ([KDP help](https://kdp.amazon.com/en_US/help/topic/G201953020)) [V1]; cream 0.0025 in per third parties ([kdpeasy](https://www.kdpeasy.com/fr/tools/calculateur-dos-kdp)) [V1] | 79 pages; KDP pages are inconsistent ("at least 79" vs "more than 79", Cover Creator says 80) ([KDP help](https://kdp.amazon.com/en_US/help/topic/A1L46HVPPIYTE), [ebookpbook](https://www.ebookpbook.com/2026/06/24/book-spine-text-requirements/)) [V2] | 0.0625 in (1.6 mm) each side [M], not confirmed by search | 0.4 in (10 mm) between spine and safe area on front/back ([KDP help](https://kdp.amazon.com/en_US/help/topic/G201953020)) [V1]; I believe this is the hardcover hinge [M] |
| Lulu paperback | 0.125 in [M] | pages / 444 + 0.06 in; pages / 460 + 0.06 in for magazine/comic paper ([Lulu API help](https://help.api.lulu.com/en/support/solutions/articles/64000254616-how-is-spine-width-calculated-)) [V1] | No spine text at 80 pages or fewer ([Lulu Book Creation Guide](https://assets.lulu.com/media/guides/en/lulu-book-creation-guide.pdf), read via [Scribd copy](https://www.scribd.com/document/615961952/lulu-book-creation-guide)) [V1] | 0.125 in (3.2 mm) to each spine edge (same guide) [V1] | Text spanning the full spine is risky: small shifts push it onto the covers; spine width differs between Lulu's own network and distribution partners ([Lulu help](https://help.lulu.com/en/support/solutions/articles/64000255462)) [V1] |
| IngramSpark | 0.125 in (3 mm) ([Ingram cover bleed sheet](https://marketing.ingramcontent.com/mrkng/documents/publishers/Cover-Bleed-Dimensions.pdf)) [V1] | Template generator only ([IngramSpark help](https://help.ingramspark.com/hc/en-us/articles/35303841585933-Cover-Template-Generator)) | Paperback 48 pages, hardcover 18 pages ([IngramSpark help](https://help.ingramspark.com/hc/en-us/articles/35306346735245-Cover-File-Creation), [ebookpbook](https://www.ebookpbook.com/2026/06/24/book-spine-text-requirements/)) [V2] | Spine >= 0.35 in (9 mm): 0.0625 in; thinner: 0.03125 in (about 1 mm) ([IngramSpark blog](https://www.ingramspark.com/blog/file-requirements-for-print-books), [Ingram dust-jacket template](https://marketing.ingramcontent.com/mrkng/documents/publishers/Dust-Jacket-Template.pdf)) [V2] | All text >= 0.25 in (6 mm) from trim; duplex covers keep spine + 0.0625 in each side white for the glue; 0.5 in (13 mm) hinge quoted for Ingram covers [V1] |
| BoD (DE/FR) | **5 mm** on the four outer sides ([BoD cover guide, about 11 years old](https://www.yumpu.com/de/document/view/41877228/wie-gestalte-ich-das-buchcover-richtig-bod)) [V1] | Online *Umschlagsberechnung*; minimum 24 pages (same guide) | 80 pages or **5 mm** spine (same guide) [V1] | not found | Keep text 8 mm away from the fold area next to the spine (same guide) [V1] |
| Bookvault (UK) | 3 mm [M], not confirmed | Sizing calculator / generated template ([Bookvault templates](https://bookvault.app/downloading-bookvault-templates/)) | "About 10 mm or more leaves room for spine text"; skip spine text on low page counts ([Bookvault help](https://help.bookvault.app/book-cover-spine-misaligned)) [V1] | not found | Help page acknowledges slight cover/spine misalignment as normal production variance |

### 3.2 French and German printers on spine text

| Source | Rule | Conf. |
|---|---|---|
| Graphiscann (Lyon) | Title/logo on the spine from 5 mm, about 80 pages; perfect binding starts around 40 pages ([graphiscann.fr](https://www.graphiscann.fr/brochures-collees-dos-carre-impression.html)) | V1 |
| Infoprint (Montpellier) | Below 5 mm a spine title is hard; estimate = pages / 2 x grammage / 1000 (mm); 3 mm bleed; 5 mm unprinted reserve each side of the spine inside (glue does not hold on ink); a few mm of error visibly decentres the title ([article](https://www.montpellier-imprimeur.com/imprimer-un-titre-sur-la-tranche-ce-que-le-dos-peut-accueillir/), [checklist](https://www.montpellier-imprimeur.com/checklist-dos-carre-colle)) | V1 |
| EPFL Repro | Title only on a spine >= 5 mm, **>= 1.5 mm margin each side**; 5 mm spine carries a 2 mm cap height; comfort creases (*rainages de confort*) 6 mm either side of the spine; glue zone = spine + 9 mm left blank ([EPFL](https://www.epfl.ch/campus/services/repro/wp-content/uploads/2025/11/REPRO-Guide-PAO.pdf)) | V1 |
| Booksfactory.fr | No text on a spine under 6 mm ([booksfactory.fr](https://booksfactory.fr/article_info.php?articles_id=53)) | V1 |
| Veoprint | No elements on a spine under 4 mm ([Veoprint data sheet](https://www.veoprint.com/dl?t=template&id=34)) | V1 |
| Reset St. Pauli (DE) | Spine lettering sensible from 5 mm; a 3 mm spine usually gets no crease ([checklist](https://www.resetstpauli.de/checkliste-klebebindung/)) | V1 |
| printiteasy (DE) | A spine off by 2 mm is immediately visible on the cover ([printiteasy](https://printiteasy.eu/de/wissensdatenbank/druckvorlagen/)) | V1 |

Synthesis:

- "No spine text below about 5 mm" is the consensus (range 4-6 mm; KDP's 79 pages is about 4.5 mm, Lulu's 80 pages about 6.1 mm [calc]) [V2]. A **10 mm spine is about twice the threshold**, and matches Bookvault's "comfortable" figure.
- Side margin for spine text: 1.5 mm (EPFL), 1.6 mm (IngramSpark, spine >= 9 mm), 3.2 mm (Lulu). On 10 mm this leaves a text band of 6.8-7 mm (Ingram/EPFL) or 3.65 mm (Lulu) [calc]. Recommended for the gift card: **2 mm each side, 6 mm text band**: all-caps up to about 24 pt, mixed case about 17 pt [calc, assuming 0.7 em cap height and 0.95 em ascender+descender].
- Tolerance: a binding tolerance of about +-1 mm is quoted by generic spine guides (result set incl. [pdfpress](https://www.pdfpress.app/blog/spine-width-calculator-guide), [bookdesigner.ai](https://www.bookdesigner.ai/guides/book-spine-width); exact page not pinned) [V1]. Design for a crease drift of +-1 mm: let the spine background run at least 2 mm onto front and back (or use a continuous background), never a hard colour edge exactly on the crease.
- Hinge: on perfect-bound covers a second pair of creases sits about 6 mm outside the spine (EPFL) and the area is glue-affected (5 mm at Infoprint, 8 mm at BoD, 10-13 mm on hardcovers at KDP/Ingram; [Mixam](https://mixam.co.uk/support/hardcoversetup) adds 5 mm each side). The gift card has no glue, so no hinge is needed; if a "paperback look" is wanted, optional hinge creases at 6 mm respect the 5 mm crease-spacing rule.
- Spine reading direction: French and continental titles usually read bottom-to-top, English-language ones top-to-bottom [M]. Should follow the site locale.

### 3.3 How many pages make a 10 mm spine (softcover / notebook)

| Basis | Pages for about 10 mm | Conf. |
|---|---|---|
| KDP white, 0.0572 mm/page | about 175 | calc |
| KDP cream, 0.0635 mm/page | about 158 | calc |
| Lulu, pages / 444 + 0.06 in | about 148 | calc |
| Infoprint estimate, 80 g/m2 | about 250 | calc |
| Infoprint estimate, 90 g / 100 g / 120 g | about 222 / 200 / 167 | calc |

So a real softcover with a 1 cm spine is a 150-250-page book or notebook, about 75-125 sheets. Real papers have bulk above 1.0, so the Infoprint estimate overstates the page count [M].

---

## 4. Trim sizes and sheet fit

Verified by search: French *poche* is 11 x 18 cm and *broché* about 14 x 21 cm ([publiersonlivre.fr](https://publiersonlivre.fr/publier-livre/format-livre-poche-broche/)); A5 148 x 210 is Lulu's most popular softcover size ([Lulu FR](https://www.lulu.com/fr/create/print-books)) [V1 each]. The other sizes in the table are the usual French trade sizes from memory [M].

Sheet data: SRA3 = 320 x 450 mm; a Xerox DocuColor prints to within 4 mm of each edge, image area 312 x 442 mm, bleed >= 3 mm recommended ([Xerox manual p.28](https://www.manualslib.com/manual/195677/Xerox-2000.html?page=28), [p.30](https://www.manualslib.com/manual/195677/Xerox-2000.html?page=30), [papersizes.io](https://papersizes.io/raw/sra3)) [V2 for the sheet size, V1 for the image area]. Larger presses exist (Canon varioPRINT iX3200: 337 x 504 mm printable, [Canon](https://www.canon.co.uk/business/products/production-printers/digital-presses/varioprint-ix3200/specifications/)) [V1].

Spread = back + 10 mm spine + front = (2w + 10) x h. "Bleed" adds 3 mm per side, "marks" a further 8 mm per side (3 mm offset + 5 mm mark). A4 landscape = 297 x 210, assumed home printable area 287 x 200 (5 mm margins). All [calc].

| Trim w x h (mm) | Ratio h/w | Spread | + bleed | + marks | A4 home, 100 % | SRA3 (312 x 442) |
|---|---|---|---|---|---|---|
| 105 x 148 (A6) | 1.41 | 220 x 148 | 226 x 154 | 242 x 170 | fits with bleed and marks | 2-up |
| 108 x 177 (poche) | 1.64 | 226 x 177 | 232 x 183 | 248 x 199 | fits with bleed and marks | 2-up |
| 110 x 178 (poche) | 1.62 | 230 x 178 | 236 x 184 | 252 x 200 | fits, marks exactly at the limit: shorten marks to 3 mm | 2-up |
| 125 x 190 | 1.52 | 260 x 190 | 266 x 196 | 282 x 212 | bleed fits, top/bottom marks do not: side ticks only | 2-up |
| 130 x 200 | 1.54 | 270 x 200 | 276 x 206 | 292 x 216 | trim only, zero bleed, zero margin reserve: not recommended | 2-up |
| 140 x 205 | 1.46 | 290 x 205 | 296 x 211 | 312 x 227 | does not fit | 2-up with bleed, marks tight |
| 148 x 210 (A5) | 1.41 | 306 x 210 | 312 x 216 | 328 x 232 | does not fit (wider than the sheet) | 1-up |
| 155 x 240 | 1.55 | 320 x 240 | 326 x 246 | 342 x 262 | does not fit | 1-up |

Observations [calc]:

- **Poche 110 x 178 is the only common French trim that prints on A4 at 100 % with bleed and marks**, and also on US Letter landscape (279 x 216 mm, printable about 266 x 203 with 6.35 mm margins). It leaves 57 mm of spare width: enough for a 50 mm inward-folding flap carrying the QR code, which makes a **single-sided** home print possible (back 110 + spine 10 + front 110 + flap 50 = 280 mm).
- Poche ratio (1.62) is also the closest to common eBook cover ratios (1.5-1.6) [M], so the cover image needs the least cropping.
- A5 and larger need A3/SRA3: professional print only.

---

## 5. Catalogue products that can carry the design

| Product (FR / DE) | How the spine is made | Evidence | Fit for the gift jacket |
|---|---|---|---|
| **Presentation folder with capacity spine** (*chemise à rabats double rainage*, *chemise à dos* / *Mappe mit Füllhöhe*) | Two parallel creases; standard spines **5 mm and 10 mm** | Helloprint: double crease 5 mm (about 30-50 sheets 80 g) and 10 mm (about 60-100 sheets), templates for A4 and A5 at 1/5/10 mm ([FR-BE](https://www.helloprint.com/fr-be/chemisesrabats), [UK](https://www.helloprint.com/en-gb/folders)); Flyeralarm folders A5 or A4 with *Füllhöhe* 1/5/10/15 mm ([Flyeralarm](https://www.flyeralarm.com/de/p/mappen-2-teilig-mit-laschen-4390809.html)); viaprinto 1/5/10 mm ([viaprinto](https://www.viaprinto.at/praesentationsmappen)); DruckDiscount24 5 or 10 mm ([DD24](https://www.druckdiscount24.de/angebotsmappen)); Tradeprint none/3/4/5/10 mm ([Tradeprint](https://replatform.tradeprint.co.uk/presentation-folders/a4-glued-folder)); Veoprint 1/6/12 mm ([Veoprint CH](https://www.veoprint.ch/impression-chemises-a-rabats)); Imprimeur du Roi A4 double crease 5 mm with PDF/AI template ([Imprimeur du Roi](https://imprimeurduroi.com/fr/impression-chemise-double-rainage/126-a4-double-rainage-5mm.html)); Pinguindruck **A6** folder, file 277 x 200 mm at 10 mm ([data sheet](https://pinguindruck.de/load/dateien/mappe_09-12_4s2ka6_datenblatt.pdf)) [V2] | **Best match.** 10 mm spine is a stock option; A5 and A6 exist; flaps can hold an insert with the QR code. Limits: formats are oversize relative to book trims (made to hold A4/A5 sheets), die shapes are fixed, minimums apply (Imprimerie du Net from 50 copies, [source](https://imprimeriedunet.fr/601-chemise-a-rabats-standard.html)). Capacity figures vary widely between printers (10-30 vs 30-50 sheets for 5 mm: [Personaliz](https://www.personaliz.com/impressions-tout-support/papeterie-personnalisee/chemise-a-rabat.html) vs Helloprint). Watch the definition: at Prontaprint a two-pocket folder has spine = 2 x capacity ([Prontaprint](https://printers.prontaprint.com/product/glued-folders/)) |
| **Flat card with two creases** (*carte / dépliant avec 2 rainages, livré à plat*) | Two creases 10 mm apart on 300-350 g board | Crease spacing rule >= 5 mm (section 1); not a standard fold type (50 mm fold-to-fold minimum for machine folding) | Closest to the design at exact book trim, but usually a **custom quote**, not a configurator product. No catalogue listing found |
| **Softcover book / notebook** (*carnet dos carré collé*) | Real perfect-bound block | 150-250 pages for 10 mm (section 3.3); KDP/Lulu/BoD/Bookvault print one copy | A blank or printed notebook with the eBook's cover: the most book-like object, the highest unit cost and weight; the QR goes on page 1. Cover rights are a bigger question here (a "book" bearing the cover) |
| **Dust jacket** (*jaquette*) | Folds around an existing book; no crease needed on 130-150 g paper [M] | Ingram flaps 3.25 in (83 mm) + 0.25 in (6 mm) strip, bleed 0.125 in ([Ingram template](https://marketing.ingramcontent.com/mrkng/documents/publishers/Dust-Jacket-Template.pdf)); Mixam flaps 2.5 in, PrintNinja minimum 2.83 in ([PrintNinja](https://printninja.com/dust-jacket-file-setup-guide/)) [V1 each] | Needs a book or dummy block underneath; alone it does not stand. Useful as a **layout model**: flaps of 65-85 mm are the natural place for QR + instructions |
| **DVD-case inlay** (*jaquette DVD*) | Paper sleeve slid into a plastic case, spine 14 mm (7 mm slim) | Trim 273 x 183 mm, panels about 129.5 mm, 279 x 189 with bleed; slim 266 x 183 ([4over4](https://www.4over4.com/guide/dvd-cover-size-dimensions-and-print-specs)); Avery sells pre-cut A4 inkjet inserts 273 x 183 ([Avery J8437](https://www.avery.co.uk/product/dvd-case-inserts-j8437-25)) [V2] | Fits A4 at 100 %, rigid, stands on a shelf, panel ratio 1.41. Spine is 14 mm not 10, and it is a plastic case (poor environmental story). A proven precedent for "print at home, slip into a case" |
| **Book-shaped box** (*boîte livre*, *étui*) | Die-cut folding carton | not searched (budget) | Could hold a voucher card or small gift; die-cut minimums; needs a packaging printer [M] |
| **Bellyband** (*bandeau*, *Banderole*) | Strip wrapped around an object | not searched (budget) | Only a complement (around a notebook or box); cannot carry cover + spine alone [M] |

Stock for a free-standing jacket: folder products are sold on about 300-350 g/m2 board (Helloprint product name lists 300 g silk, [Helloprint](https://www.helloprint.com/en-gb/folders-a3foldedtoa4-1mmcover-300gsilk-bothsides-nofinishing-nofinish-250)) [V1]. General weight bands: 130-170 g flyers and inner pages, 170-250 g brochure covers and postcards, board above about 200 g ([graphiste.com](https://graphiste.com/blog/impression-comment-choisir-le-bon-grammage-et-taux-dencrage)) [V1]. Lamination (*pelliculage*) over dark solids to stop toner cracking at the creases is my recommendation [M].

Structural caveat (design inference): a hollow U-shaped jacket has nothing inside its 10 mm spine. It will stand, but it tends to spring open. Flaps folded inward, a folded insert leaflet, or a glued sleeve fix this.

---

## 6. Print-at-home case

| Point | Finding | Conf. |
|---|---|---|
| Unprintable margins | Model-specific: 4.23 mm per side on one HP LaserJet ([HP forum](https://h30434.www3.hp.com/t5/LaserJet-Printing/What-does-the-quot-Wide-A4-quot-setting-do-HP-Laserjet/td-p/6862566)); 3.37 mm left/right on A4 for a Lexmark inkjet, 6.35 mm on other sizes ([Lexmark Z52 manual](https://www.manualslib.com/manual/291880/Lexmark-Z52.html?page=50)); HP forum: "some 3 mm, some 5 mm" ([HP forum](https://h30434.www3.hp.com/t5/Inkjet-Printing/Minimum-Margins-Envy-5030/m-p/7329955)). Range **3-6.35 mm** | V2 |
| Bottom edge | Older inkjets have a larger bottom margin than the sides (old HP manual in the results); 12-15 mm on some models [M] | V1 / M |
| Laser vs inkjet | Laser printers usually allow smaller margins; borderless is an inkjet feature and cannot be assumed ([Label Planet](https://labelplanet.co.uk/glossary/printer-margins)) | V1 |

Recommendations (design inference from the above):

1. **Layout**: A4 landscape, poche spread 230 x 178 mm centred: 33 mm free left/right and 16 mm top/bottom, so even a 15 mm bottom margin is survived. Keep everything, marks included, inside a 6.35 mm safe frame.
2. **Print dialogue**: "Actual size / 100 % / *Taille réelle*", never "Fit to page". Print a 100 mm calibration ruler on the sheet so the user can check the scale, because a 97 % shrink makes the spine 9.7 mm and misplaces the fold ticks.
3. **Paper**: 160-200 g/m2 matt card is the practical ceiling of most home printers [M]; 120 g works but will not stand well. Above 170 g, score before folding (section 1).
4. **Cut**: craft knife and metal ruler along the crop ticks; a 2-3 mm bleed makes the cut forgiving.
5. **Score and fold**: lay a ruler between the top and bottom fold ticks, draw a blunt point (bone folder, empty ballpoint, back of a knife) along each of the two lines, then fold away from the groove so the raised bead ends up inside the fold [M; see also [Cardstock Warehouse scoring guide](https://cardstock-warehouse.com/pages/paper-101-scoring-and-folding), content not read]. Fold each crease to 90 degrees, not flat.
6. **Duplex**: front/back registration on home printers is poor (1-2 mm) [M]. Either keep the inside content well away from creases and edges, or offer the single-sided variant with a 50 mm flap (section 4).
7. **Alternative**: the DVD-inlay precedent (273 x 183 mm on A4, pre-cut Avery sheets) shows that an A4 "slip-in" format is an accepted home-print pattern.

---

## 7. Proposed numeric defaults for the generator (derived, to be validated with a real printer)

| Parameter | Default | Basis |
|---|---|---|
| Spine | 10.0 mm between two creases | catalogue-standard double crease; >= 5 mm spacing rule |
| Bleed | 3 mm (profile option 5 mm for BoD-like specs, 3.175 mm for US POD) | sections 2, 3.1 |
| Spine text band | 6 mm centred (2 mm clear each side) | EPFL 1.5 mm, Ingram 1.6 mm, Lulu 3.2 mm |
| Spine background | extends >= 2 mm past each crease | +-1 mm tolerance |
| Varnish / foil | none within 1.5 mm of a crease | printprinz |
| QR code and typed code | >= 5 mm from any crease or trim | printing.com 5 mm die-line rule, by analogy |
| Crease lines | profile-dependent: outside ticks only / spot-colour overprint layer / separate view PDF | section 2 |
| Board (pro) | 300-350 g/m2, grain parallel to spine, laminated | sections 1, 5 |
| Default trim | 110 x 178 mm (A4-printable, 2-up on SRA3); A5 as the "pro only" size | section 4 |

---

## Sources

Creasing, grain, crease spacing
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

Marks and file conventions
- https://www.cipherbliss.com/dessins/Qzines/maquettes/imprimeur%20docs/pixartprinting_instruction_7_fr.pdf
- https://upload.prod.papeo.fr/media/tuto-pdf/tuto-imprimerie-amalgame.pdf
- https://pinguindruck.de/load/dateien/mappe_09-12_4s2ka6_datenblatt.pdf
- https://www.epfl.ch/campus/services/repro/wp-content/uploads/2025/11/REPRO-Guide-PAO.pdf
- https://wttb.co.uk/support-guides/presentation-folder-styles-setup
- https://smilepack.fr/files/subscribers/d72b0b53-b3d0-49ff-adfd-37d51546ebde/ftp/gabarits/Coussin%20technique%20smilepack.pdf
- https://www.jimprimeenfrance.fr/impression-chemise-rabat-fine

Spine rules and platform templates
- https://kdp.amazon.com/en_US/help/topic/G201953020
- https://kdp.amazon.com/en_US/help/topic/A1L46HVPPIYTE
- https://www.ebookpbook.com/2026/06/24/book-spine-text-requirements/
- https://www.kdpeasy.com/fr/tools/calculateur-dos-kdp
- https://assets.lulu.com/media/guides/en/lulu-book-creation-guide.pdf
- https://www.scribd.com/document/615961952/lulu-book-creation-guide
- https://help.api.lulu.com/en/support/solutions/articles/64000254616-how-is-spine-width-calculated-
- https://help.lulu.com/en/support/solutions/articles/64000255462
- https://www.ingramspark.com/blog/file-requirements-for-print-books
- https://help.ingramspark.com/hc/en-us/articles/35306346735245-Cover-File-Creation
- https://help.ingramspark.com/hc/en-us/articles/35303841585933-Cover-Template-Generator
- https://marketing.ingramcontent.com/mrkng/documents/publishers/Cover-Bleed-Dimensions.pdf
- https://marketing.ingramcontent.com/mrkng/documents/publishers/Dust-Jacket-Template.pdf
- https://www.yumpu.com/de/document/view/41877228/wie-gestalte-ich-das-buchcover-richtig-bod
- https://help.bookvault.app/book-cover-spine-misaligned
- https://bookvault.app/downloading-bookvault-templates/
- https://mixam.co.uk/support/hardcoversetup
- https://printninja.com/dust-jacket-file-setup-guide/
- https://www.graphiscann.fr/brochures-collees-dos-carre-impression.html
- https://www.montpellier-imprimeur.com/imprimer-un-titre-sur-la-tranche-ce-que-le-dos-peut-accueillir/
- https://www.montpellier-imprimeur.com/checklist-dos-carre-colle
- https://booksfactory.fr/article_info.php?articles_id=53
- https://www.veoprint.com/dl?t=template&id=34
- https://www.resetstpauli.de/checkliste-klebebindung/
- https://printiteasy.eu/de/wissensdatenbank/druckvorlagen/
- https://www.pdfpress.app/blog/spine-width-calculator-guide
- https://www.bookdesigner.ai/guides/book-spine-width

Trim sizes, sheets, home printers
- https://publiersonlivre.fr/publier-livre/format-livre-poche-broche/
- https://www.lulu.com/fr/create/print-books
- https://www.manualslib.com/manual/195677/Xerox-2000.html?page=28
- https://www.manualslib.com/manual/195677/Xerox-2000.html?page=30
- https://papersizes.io/raw/sra3
- https://www.canon.co.uk/business/products/production-printers/digital-presses/varioprint-ix3200/specifications/
- https://h30434.www3.hp.com/t5/LaserJet-Printing/What-does-the-quot-Wide-A4-quot-setting-do-HP-Laserjet/td-p/6862566
- https://h30434.www3.hp.com/t5/Inkjet-Printing/Minimum-Margins-Envy-5030/m-p/7329955
- https://www.manualslib.com/manual/291880/Lexmark-Z52.html?page=50
- https://labelplanet.co.uk/glossary/printer-margins
- https://graphiste.com/blog/impression-comment-choisir-le-bon-grammage-et-taux-dencrage

Catalogue products
- https://www.helloprint.com/fr-be/chemisesrabats
- https://www.helloprint.com/en-gb/folders
- https://www.helloprint.com/en-gb/folders-a3foldedtoa4-1mmcover-300gsilk-bothsides-nofinishing-nofinish-250
- https://www.flyeralarm.com/de/p/mappen-2-teilig-mit-laschen-4390809.html
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

## Could not verify

- **Nothing was read in full.** All sourced facts come from search extracts; quoted numbers should be re-checked against the printer's current template before being hard-coded.
- **Search budget exhausted after 25 searches.** Not searched at all: bellyband (*bandeau*) specs, book-shaped boxes (*boîte livre*), catalogue notebooks with a 10 mm spine and their prices, Saxoprint / Onlineprinters perfect-binding data sheets, hinge-score practice on perfect-bound covers, home-paper recommendations and hand-scoring technique.
- **KDP spine-text side margin (0.0625 in) and spine variance (0.125 in)**: from memory; the search found only unnumbered "spine safe area" wording. The 0.4 in (10 mm) hinge was returned under KDP help but I believe it applies to hardcovers.
- **KDP formula**: KDP's page gives per-page factors with no cover allowance; a third-party French calculator adds 0.06 in. Not reconciled.
- **Lulu bleed (0.125 in)** and Lulu's 32-page paperback minimum (mirror site only). The Lulu spine-text rules come from copies of the Book Creation Guide, not the live PDF.
- **Bookvault bleed (3 mm), spine-text minimum and hinge**: no official figure found, only "about 10 mm or more leaves room".
- **BoD figures** come from a guide about 11 years old; current BoD bleed and spine-text threshold not confirmed.
- **IngramSpark 0.5 in hinge**: reported by one extract as a perfect-bound figure; may be the case-laminate hinge.
- **+-1 mm binding tolerance** and the **50 mm fold-to-fold minimum**: each appeared in a search summary without the exact page being identifiable among two candidates.
- **Dashed fold marks outside the trim** as a convention: no printer page retrieved; colours are demonstrably not standardised.
- **French trade trim sizes** 108 x 177, 125 x 190, 130 x 200, 140 x 205, 155 x 240: from memory; only "11 x 18 poche", "14 x 21 broché" and A5 were confirmed.
- **Grain direction of stocked SRA3 and A4 sheets**, home-printer maximum paper weight, duplex registration, dust-jacket paper weight, eBook cover aspect ratios: from memory.
- **Whether any EU online printer sells a book-trim-sized card with two creases 10 mm apart as a configurable product** (rather than an A4/A5/A6 folder or a custom quote): not found. This is the biggest open point for the "order a printed one" flow.
