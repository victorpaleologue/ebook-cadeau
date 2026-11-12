# Online printers and print APIs: prices, EU hosting/production, environmental and social (ESG) ranking

_Status: fact-check corrections applied (the fact-check had limited web access) — research date 2026-11-12_

## Evidence base and limits (read first)

- This document merges three researcher notes: `notes/printers.web2print.md` (13 searches), `notes/printers.print-api.md` (30 searches) and `notes/printers.esg.md` (0 searches: the shared search budget was already spent, so that note is memory only). Two sibling notes were cross-read for facts the printer notes could not search: `notes/print-requirements.folding.md` (catalogue folders with a 10 mm spine) and `notes/market-business.competitors-costs.md` (La Poste tariffs).
- **The lead analyst could add no new evidence.** Five further searches were attempted on 2026-11-12 (Print.com ownership, date of Directive (EU) 2024/825, Imprim'Vert criteria, Exaprint labels, Gelato card prices). All five were refused because the run's shared web-search budget was exhausted. Page fetching is blocked in this sandbox. No confidence marker has been upgraded.
- **There is no hard quote in this document.** No live configurator was opened and no printer was asked for a written quote. Every price is a figure seen in a search-result extract of a vendor page, a third-party reading, or our own estimate. Section 3 labels each one.
- Confidence markers: **[V2]** seen in at least two independent sources during the run, **[V1]** one source, **[M]** from memory (knowledge cutoff mid-2026), **[M-low]** weak memory, **[est.]** / **[calc]** our arithmetic on sourced or remembered figures. Markers are the researchers'; "sibling" means the fact was verified by another note's author, not re-checked here.
- Abbreviations: HT = excluding VAT, TTC = including VAT, POD = print on demand, DPA = data processing agreement (GDPR Art. 28), SCOP = French worker cooperative, EA = *entreprise adaptée* (employer with at least 55 % disabled employees on ordinary contracts), ESAT = sheltered medico-social workshop.
- Prices are HT unless stated. "As of 2026-11" means the figure appeared in a search extract on 2026-11-12; the age of the underlying page is unknown unless stated.
- **Fact-check pass (2026-11-12, `factcheck/printers.md`).** The checker ran no web search (all refused) and fetched no page; its verdicts rest on its own memory ("checker's recall" below), on recomputed arithmetic and on the notes. The corrections are applied in place and listed in the Verification log at the end. The editor added three web searches and one primary file: the Merci Facteur API README on GitHub (read in full for the card section), which changes the launch recommendation (§3.4, §4, §7.1). Where the checker's recall is the only basis, the text says so and the claim stays unverified.

## 1. Executive summary

| Question | Pick | One-line justification | Confidence |
|---|---|---|---|
| **Best overall** | No single supplier covers both jobs. Stack: **print at home** (default) + **Merci Facteur API** (one card mailed to one recipient) + **Exaprint**, with **Onlineprinters** as the documented alternative (batches of one title) | The six web-to-print shops read had minimums of 10 to 50 [V1]; quantity-1 upload shops (viaprinto, print24, WIRmachenDRUCK) were not examined. The only per-card route to a French letterbox found is Merci Facteur, about €4.29–6.3 [est. on V1 prices, single vendor]; its documented card API takes **one JPEG visual**, not our four-panel artwork (fact-check, README read), so the launch route is its **letter** service with our A4 PDF until a test proves otherwise. Exaprint has a plant in France [V2], but whether our job is printed there is not established | Medium on the structure, low on every price |
| **Best for the environment** | Single card: **print at home or in store**, then **French print + letter post** (Merci Facteur; Maileva for a plain letter). Batch: a **Blauer Engel DE-UZ 195 / EMAS printer** (Lokay, oeding print, dieUmweltDruckerei) if its licences check out; best *evidenced* mainstream printer is **Onlineprinters** | For one card, logistics outweigh the print shop: a courier parcel is worth several hundred grams CO2e against tens of grams for the card [est., M]. Only product-level ecolabels cover paper, inks and recyclability together [M]. Onlineprinters has an ESG report, Imprim'Vert and PEFC/Blue Angel papers [V1] | Low: no label register was checked |
| **Best for society** | An **entreprise adaptée** print shop (Copiver Express is listed by UNEA [V1]; Handiprint, ANRH, APF Entreprises [M]) or a **SCOP** printer (Laballery, La Contemporaine, Média Graphic [M]) | Highest social value (inclusive employment, worker ownership); a reseller with 20+ employees can deduct part of EA/ESAT purchases from its disabled-employment contribution [M] | Low: no price, no capability check, quote-based only |
| **Best API for one-off cards** | **Merci Facteur** for France at launch (letter route certain; card route only if a test shows our artwork can be printed); **Gelato** for EU scale-out | Merci Facteur: folded card €2.99 + postage from €1.30, printed and posted the same day via La Poste, public REST docs, €19.95/month [V1, single vendor]. Its card API accepts one JPEG visual (ratio 0.71, 350 g/m² board, folded card 14 × 19.5 cm closed); its letter API accepts colour PDFs, recto-verso [V1, API README]. Gelato: folded cards, catalogue/price/order APIs, production in 32 countries, public DPA [V2/V1]; no unit price obtained | Low for Merci Facteur cards, medium for its letters; low for Gelato (no price) |
| **Cheapest for short runs** | 100 folded, creased A6 cards: **about €52 HT delivered** (Onlineprinters eco 300 g PEFC €52.41 HT; Flyeralarm €52.27 HT delivered for a smaller format). Flat A6: **Pixartprinting €36.15 HT per 100** (third-party reading). 50 flat cards: **Realisaprint from €25 HT** | Only Onlineprinters priced the actual product (folded A6, €52.41–59.24 per 100); the other figures are a business-card-size folded card (Flyeralarm), a flat A6 read by a competitor (Pixartprinting) and a TTC price without shipping (Vistaprint), so they are not an agreement (fact-check) [V1 per row] | Low on the band and on each figure |

Five findings that shape every decision:

1. **No public catalogue or API sells a book-sized card with two creases 10 mm apart at quantity 1.** Card products are single-fold [V1]. The 10 mm double crease is a stock option only on presentation folders in A4, A5 and A6 [V2, sibling]; at book trim it is a quote job.
2. **Cimpress plc owns most of the candidate list**: Vistaprint, Pixartprinting, Exaprint, Easyflyer, WIRmachenDRUCK and, since 2 July 2026, SAXOPRINT and viaprinto [V2 for the 2026 deal and WIRmachenDRUCK; the rest of the list is the checker's recall and the researchers' memory]. The brief's "Saxoprint (DE, Cewe)" is out of date.
3. **Hosting location is known for two providers only**: Maileva (France) and Gelato (AWS, EU residency options) [V1]. For every other printer it must be obtained in writing.
4. **"Climate-neutral printing" claims must not be relayed.** Offset-based neutrality claims to consumers are blacklisted under Directive (EU) 2024/825, to be applied from 27 September 2026 through national transposition. France had not finished transposing it when last reported (DDADUE bill, arts. 20–21; commentary of late 2026 says not yet transposed) [V1, editor's search, law-firm and blog sources], but French law already restricts "neutre en carbone" and generic claims (§6.2). ClimatePartner itself warns printers [V1].
5. **The printed QR code is a bearer secret** readable by the printer, its subcontractor and the postal chain. Codes must stay inactive until the buyer or seller activates them (design consequence, not a researched fact).

## 2. Product variants we priced

Default trim is the French *poche* format 110 × 178 mm (the only common French trim that prints on A4 at 100 % with bleed and marks [calc, sibling]).

| ID | Variant | Geometry | Stock | Where it can be bought | Minimum | Is the spine real? |
|---|---|---|---|---|---|---|
| P0 | **Home print** | A4 landscape, spread 230 × 178 mm at 100 %, fold ticks outside the trim | 160–200 g/m² card [M] | User's own printer; walk-in copy shop (Copy-Top, COREP [M], not priced) | 1 | Yes, hand-scored |
| P1 | **True jacket** (*carte à 2 rainages, livrée à plat*) | Flat sheet (2w + 10) × h: 230 × 178 mm for poche, 306 × 210 mm for A5; two creases 10 mm apart; crease only, delivered flat | 300–350 g/m², 4/4; 250 g/m² when it must stay under 20 g in the post | Trade printer **on quote**; no configurator listing found | Not known; expect 50–100 [M] | Yes |
| P2 | **Single-fold card with a printed spine band** | One centre crease. A6 closed (open 210 × 148 mm), A5 closed (Merci Facteur: 15 × 21 cm closed, 30 × 21 cm open per its price page [V1]; its API README gives 14 × 19.5 cm closed, 28 × 19.5 cm open after cutting, on 350 g/m² [V1]), 14 × 14 cm (Popcarte) | 270–350 g/m², 4/4 | Every web-to-print shop; card APIs | 10–50 in batches; 1 through card APIs | No: the 10 mm band is printed next to the fold |
| P3 | **Folder with capacity spine** (*chemise à dos / double rainage*, *Mappe mit Füllhöhe*) | Presentation folder, two creases, stock spines 5 and 10 mm; A4, A5, A6 | about 300–350 g/m² [V1, sibling] | Helloprint, Flyeralarm, viaprinto, DruckDiscount24, Pinguindruck and others [V2, sibling] | 50 at one French printer [V1, sibling] | Yes, but formats are sized to hold A4/A5/A6 sheets, not book trims |
| P4 | **One card printed and mailed** | P2 produced singly, enveloped, posted to the recipient | Provider's card stock | Merci Facteur, Gelato, Printful, Print&Bind, Prodigi; consumer apps as benchmarks | 1 | No |
| P5 | **Dummy book / notebook** | Perfect-bound block; 10 mm spine needs about 150–250 pages [calc, sibling] | Book paper + 250–300 g cover | BoD, Lulu, Bookvault, Print API (NL) | 1 | Yes |
| P6 | **Plain letter** | A4 colour sheet (the home-print layout) in an envelope | 80 g office paper | Maileva, Pingen | 1 | No: the recipient cuts and folds |

Contradiction resolved: the web2print note says a 10 mm perfect-bound spine needs "roughly 100+ pages of 90 g paper" [M]; the print-api note says 150–200 pages [M]; the folding note computes 148 pages with Lulu's published formula and 158–175 with KDP's per-page factors [calc]. We keep **150–250 pages** because it is derived from published formulas, not memory.

## 3. Price tables (as of 2026-11, delivery to one address in metropolitan France)

Evidence classes used in the tables:

- **Q — hard quote** (live configurator total or a written quote): **none exists in this document.**
- **L — list price** seen in a search extract of the vendor's own page on 2026-11-12.
- **T — third-party reading** (comparison site, review, competitor page).
- **E — estimate** (our arithmetic or memory).

### 3.1 Batches of one design: single-fold or flat card (P2)

| Printer | Product, stock | Qty | Total | Per card | Delivery to France | Class | Source and caveat |
|---|---|---|---|---|---|---|---|
| Onlineprinters FR | Folded card A6 landscape, crease on the long side, 300 g offset "eco" PEFC | 100 | €52.41 HT | €0.52 | Several product pages titled "Livraison gratuite" ([example](https://www.onlineprinters.fr/p/cartes-postales-a6)) [V1] | L | [page](https://www.onlineprinters.fr/p/cartes-pliables-en-papier-eco-naturel-paysage-a6) [V1] |
| Onlineprinters FR | Same format, 300 g coated PEFC (Christmas variant) | 100 | €54.59 | €0.55 | as above | L | [page](https://www.onlineprinters.fr/p/cartes-pliables-de-no%EBl-format-paysage-a6-rainage-sur-le-cote-long); HT/TTC unclear; pre-folded +€1.50, otherwise shipped creased and flat [V1] |
| Onlineprinters FR | Same format, 270 g chromo or 300 g coated PEFC | 100 | about €59.24 | €0.59 | as above | L | [page](https://www.onlineprinters.fr/p/cartes-pliables-format-paysage-a6-rainage-sur-le-cote-long); HT/TTC unclear [V1] |
| Flyeralarm FR | Folded loyalty card 8.5 × 5.5 cm (smaller than A6), recycled 300 g, 4/4 | 100 | €52.27 HT (€62.72 TTC) | €0.52 | **Included**, 5–7 working days [V2] | L | [page](https://www.flyeralarm.com/fr/p/cartes-de-fidelite-525038), [comparatif-imprimerie.fr](https://comparatif-imprimerie.fr/avis/flyeralarm/) |
| Pixartprinting FR | A6 card 350 g, **flat**, no envelope | 100 | €36.15 HT (slowest) to €50.31 HT (fastest) | €0.36–0.50 | Included for some products [V1] | T | [Manuscry](https://manuscry.com/manuscry-ou-pixartprinting) (a competitor; reading date unknown) [V1] |
| Exaprint | "Carte postale 2 ou 3 volets", special finishes, format not captured | 50 | from €80.46 HT | €1.61 | Single-point delivery free in FR, BE, LU [V1] | L | [page](https://www.exaprint.fr/impression/carte-postale-2-3-volets-finitions-speciales), [comparatif-imprimerie.fr, updated 2026-09-30](https://comparatif-imprimerie.fr/avis/exaprint/) [V1] |
| Exaprint | "Carte à message" | 250 | from €172.38 HT | €0.69 | as above | L | [page](https://www.exaprint.fr/la-carterie/carte-message) [V1] |
| Rapid-Flyer | Double (2-panel) greeting card | 25 | from €53 HT | €2.12 | not found | L | [page](https://www.rapid-flyer.com/p/carte-de-voeux-double) [V1] |
| Printshot | Folded A6 flyer, 2 panels, 10 × 15 cm closed | 50 | €46.80 HT | €0.94 | not found | L | [page](https://www.printshot.fr/1185-flyer-2-volets) [V1] |
| Realisaprint | Correspondence card (flat), 350 g coated matt or 250 g bristol | 50 | from €25.00 HT | €0.50 | not found | L | [page](https://www.realisaprint.com/carte-correspondance-personnalisee-r2284.html) [V1] |
| EOZ | A6 2-panel greeting card | not captured | from €36.56 HT | — | not found | L | [page](https://www.eoz.eu/72-carte-de-voeux-2-volets) [V1] |
| Helloprint FR | Greeting card, option "Pliée" (creased); configuration not captured | min. 10 | displayed total €36.49 HT; express +€14.50 / +€42.50 | — | "Livraison gratuite" on some pages; UPS, DPD [V1] | L | [page](https://www.helloprint.com/fr-fr/cartesdevoeux) [V1] |
| Vistaprint FR | Greeting card 18.2 × 11.7 cm, folded or flat | 10 / 100 | €0.74 at quantity 10 (unit or lot unclear); about €0.61 per card at 100, TTC, excl. shipping | €0.61–0.74 | not found | L / T | [vistaprint.fr](https://www.vistaprint.fr/fetes/cartes-de-voeux), [L'Essentiel de l'Éco](https://lessentieldeleco.fr/8883-quel-est-le-meilleur-site-de-carte-de-voeux-2027) [V1] |
| Printoclock | A6 card 350 g, colour one side, flat | 100 / 500 | about €0.20 per card "indicatif"; €25.00 HT per 500 (format not stated) | €0.05–0.20 | not found | L / T | [printoclock.com](https://www.printoclock.com/cartes-de-voeux), [monpremierclient.fr](https://monpremierclient.fr/impression/avis-printoclock/) [V1] |
| Pixartprinting FR | Promo: 250 invitations 10 × 15 cm for €1, new customers with an EU VAT number, 7 working days | 250 | €1 | — | — | L | [promo page](https://www.pixartprinting.fr/essai-gratuit/250-invitations-10x15/) [V1]; not a real price |
| Market cross-check | Personalised greeting cards on French sites, TTC, excl. shipping | 30 / 100 | — | €0.82–3.96 at 30; €0.61–2.40 at 100 | — | T | [L'Essentiel de l'Éco](https://lessentieldeleco.fr/8883-quel-est-le-meilleur-site-de-carte-de-voeux-2027/) [V1, sibling market note] |
| Reference floor | 1,000 flyers A5, 130 g, 4/4 at Exaprint | 1,000 | €38.00 HT (economy) / €44.40 HT (standard) | €0.04 | free single point | T | read 2026-09-30 by [comparatif-imprimerie.fr](https://comparatif-imprimerie.fr/avis/exaprint/) [V1] |

Minimum order quantities seen: Pixartprinting invitations from 10 ([product page](https://www.pixartprinting.com/digital-litho-printing/customized-invitation-cards-postcards/invitations/)) [V1], Helloprint 10, Vistaprint 10, Rapid-Flyer 25, Realisaprint 50, Exaprint entry tiers 50 and 250 [V1]. Contradiction: Manuscry says Pixartprinting's classic invitation cards start at 100. We keep **10** because it comes from Pixartprinting's own page, and treat the 100 as a different product or an old reading; unresolved.

### 3.2 Planning matrix by quantity (P2, per card, delivered)

| Quantity | Route | Per card | Class | Comment |
|---|---|---|---|---|
| 1 | Card API or consumer service (see 3.4) | €4.29–6.94 | L + E | Batch printers do not sell 1 |
| 10 | Lowest web-to-print tier | **not usable**: Helloprint total €36.49 HT with unknown configuration; Vistaprint €0.74 with unit/lot unclear | L, ambiguous | Needs a configurator read |
| 25–50 | Web-to-print | €0.50–2.12 (totals €25–80 HT) | L | Setup cost dominates |
| 100 | Web-to-print | **€0.36–0.60** (totals €36–60 HT) | L / T | Four different products (fact-check): only Onlineprinters' €0.52–0.59 is a folded A6 card |
| 250–500 | Web-to-print | €0.69 at 250 (Exaprint "carte à message"); €0.05 at 500 for a flat card of unstated format (Printoclock) | L | Not captured for a folded card |
| 1,000 | Web-to-print | €0.07–0.13 | **E [M]** | Extrapolated from the flyer floor above; not a price |

### 3.3 True jacket (P1) and folder with spine (P3)

**No price was captured for either.** What is known:

- The folder product exists as a stock item with a 10 mm double crease: Helloprint (A4 and A5 templates at 1, 5 and 10 mm) ([FR-BE](https://www.helloprint.com/fr-be/chemisesrabats), [UK](https://www.helloprint.com/en-gb/folders)); Flyeralarm A5 or A4 folders with *Füllhöhe* 1/5/10/15 mm ([page](https://www.flyeralarm.com/de/p/mappen-2-teilig-mit-laschen-4390809.html)); viaprinto 1/5/10 mm ([page](https://www.viaprinto.at/praesentationsmappen)); DruckDiscount24 5 or 10 mm ([page](https://www.druckdiscount24.de/angebotsmappen)); Pinguindruck A6 folder, file 277 × 200 mm at 10 mm ([data sheet](https://pinguindruck.de/load/dateien/mappe_09-12_4s2ka6_datenblatt.pdf)); Imprimeur du Roi A4 with a 5 mm double crease ([page](https://imprimeurduroi.com/fr/impression-chemise-double-rainage/126-a4-double-rainage-5mm.html)) [V2, sibling folding note].
- Minimum seen for folders: 50 copies at Imprimerie du Net ([page](https://imprimeriedunet.fr/601-chemise-a-rabats-standard.html)) [V1, sibling].
- For a card at book trim, the only catalogue lead is Exaprint's "Carte postale double avec pliure sur mesure" ([page](https://www.exaprint.fr/la-carterie/carte-postale/double/)) [V1]; whether "sur mesure" accepts two creases 10 mm apart is not confirmed. Exaprint also lists "2 ou 3 volets", 350 g couché demi-mat, 1 rainage central ([page](https://www.exaprint.fr/Carterie/2-ou-3-volets-classique/Carte-de-visite-32-a-92-cm2-surface-a-plat/350g-couche-demi-mat/Non-pellicule/1-rainage-central)) [V1].
- Two parallel creases 10 mm apart are technically ordinary (practical minimum 5 mm between creases [V1, sibling]); the obstacle is commercial, not technical.
- **Planning assumption** used by the business model (`12-market-and-business-inputs.md`): €0.60 / 1.00 / 1.50 per jacket in runs of 100–500, identical copies, no variable data. Class **E, pure guess**: no quote supports it and no priced product is an analogue. A poche jacket (230 × 178 mm) has about 30 % more sheet area than an open A6 card and an A5 jacket about twice as much [calc, fact-check], so even the A6 prices of §3.1 do not transfer.
- Unique codes per jacket need variable-data printing, which was not priced anywhere. The alternative is one shared code with a redemption cap.

### 3.4 One card printed and mailed to one French address (P4, P6)

| Provider | Product | Card | Postage / shipping | Landed, one card | Class | Source and caveat |
|---|---|---|---|---|---|---|
| **Merci Facteur** (FR) | Folded card 15 × 21 cm closed, envelope included, La Poste | €2.99 | from €1.30; tracking +€0.50 | **about €4.29** (€4.79 tracked) + plan €19.95/month (€39.95 above about 2,000 cards/month) | L + E | [tarifs](https://www.merci-facteur.com/tarifs.php), [API page](https://www.merci-facteur.com/pro/lp/api/api-carte-postale.php), [Manuscry comparison](https://manuscry.com/manuscry-ou-merci-facteur) [V1: one vendor's pages plus a competitor's reading; downgraded from V2 by the fact-check]; HT/TTC not stated. API README: one JPEG visual per card, no documented inside or back artwork (§4). Other formats: postcard €1.00, flat card €1.99, giant €3.99; discount from 11 cards |
| Popcarte (FR, no API) | Folded 14 × 14 cm, envelope, sent direct | €3.99 | €2.95 direct-send fee (1–49 cards); €1.75 at 1,000+; tracking +€1.50 | **€6.94**, 4–5 working days | L | [envoi direct](https://www.popcarte.com/envoi-direct-destinataire) [V1]; page may be about 1.5 years old. Fully custom artwork can be uploaded ([custom upload](https://www.popcarte.com/votre-photo-seule/importez-votre-graphisme-personalise)) |
| Fizzer (FR, no API) | Flat 10 × 15 cm card in envelope, printed in France | 1 credit | included worldwide | €1.70–2.99 | L / T | [Fizzer](https://www.fizzer.com/fr/carte-postale-en-ligne/), [pvt.fr](https://www.pvt.fr/code-promo/fizzer-avis-test-carte-postale-code-promo) [V2] |
| MyPostcard (DE [M]) | Folded greeting card | — | included worldwide | €3.69–4.29 (own FAQ pages disagree) | L | [FAQ](https://support.mypostcard.com/en/support/solutions/articles/75000054648-how-much-does-mypostcard-cost-) [V1]; a "Print API" exists without public price ([OMR](https://omr.com/en/reviews/product/mypostcard-print-api)) |
| Printful (US/LV) | Greeting card, "Made in EU" collection | €2.36 list (the same page also showed €1.62, discounted) | from €3.49 (to NL); apparel reference about €4.79 first item in Europe | **about €5.85** | L + E | [Printful](https://www.printful.com/custom/collections/made-in-eu/greeting-card), [podvector](https://podvector.ai/articles/printful/shipping/printful-global-shipping-times-costs-and-what-to-expect) [V1]; card shipping rate to France not found |
| Print&Bind (NL) | Folded / greeting card, 4-page PDF | "from €2.82" (quantity basis not confirmed) | not found | about €6 | L + E | [greeting card](https://www.printenbind.nl/en/card/greeting-card), [folded card](https://www.printenbind.nl/en/card/gevouwen-kaart) [V1]; one customer review mentions about €30 for 50 with envelopes |
| Prodigi (UK) | Fine-art greetings card | from £0.75 excl. tax and shipping | not found for France | unknown | L | [page](https://www.prodigi.com/products/cards-and-stationery/greetings-cards/fine-art-greetings-cards/) [V1] |
| Gelato (NO [M]) | Folded card | **not found** | not found | unknown | — | A roundup gives "from $6.16 for a pack of 10" and "from $6 to the EU" ([merchize](https://merchize.com/top-etsy-print-on-demand-cards-companies/)) [V1, low; attribution to Gelato uncertain] |
| Printify (US/LV [M]) | Folded card | "$1 folded" headline | only a Canadian provider surfaced: from $2.97, shipping from $16.79 | not viable | L | [Printify](https://printify.com/custom-greeting-card-printing/) [V1] |
| Stannp (UK) | A5 greetings card | from £0.50 (UK volume price) | not found for France | unknown | L | [Stannp UK](https://www.stannp.com/uk) [V1], snapshot age unclear |
| **Maileva** (FR) | A4 letter (P6), not card stock | print from €0.219–0.224 HT per black-and-white envelope (rates of 2026-01-01); colour first page €0.28 at a reseller | 0–50 g: €0.747 to €1.538 HT depending on speed | **about €1.0–1.9 HT** | L + E | [Maileva](https://www.maileva.com/nos-solutions/courrier-en-ligne/), [Index Education](https://www.index-education.com/fr/service-maileva-edt.php), intermediary [postal-api.com](https://postal-api.com/tarifs) (colour page at €1.477 HT incl. DL envelope) [V1] |
| Pingen (CH) | Letter (P6) | CHF 0.40 per letter under 500/month | $1.10 per letter to France excl. VAT | about €1.4–1.6 | L + E | [prices](https://www.pingen.com/en/prices/), [price list dated 2026-01-03](https://www.pingen.fr/en/price-list/) [V1] |
| Buyer mails a home print | P0 in the buyer's own envelope | paper and ink | Lettre verte ≤ 20 g: €1.52 in 2026, **€1.67 from 1 Jan 2027** | about €1.7–2.0 | L + E | [La Poste tarifs](https://www.laposte.fr/tarifs-postaux-courrier-lettres-timbres), [La Poste Lettre verte](https://www.laposte.fr/tarif-lettre-verte) [V2, sibling market note] |

Three cautions on the Merci Facteur figure, which carries most of the launch plan:

- **Artwork (added by the fact-check, settled by the editor).** The API README documents cards as one visual sent as a JPEG URL or base64: recommended 1200 × 1690 px, minimum 800 × 1124, maximum 4000 × 2840, short side 0.71 × long side, 4 MB, "RVB conseillé"; formats `postcard`, `folded`, `classic`, `square`, `large`, `large-a4`; 350 g/m² board; envelopes C5 for folded cards [V1, [README](https://github.com/MerciFacteur/Merci-facteur-API/blob/master/README.md), read on raw.githubusercontent.com]. No parameter for an inside or back visual is documented there (the OpenAPI file it links was not reachable). So our layout — cover front, printed spine, designed back, QR code inside — **cannot be assumed printable** through the card API. The letter API takes up to 10 colour PDFs, recto or recto-verso, which fits the home-print A4 layout (P6) [V1, same README].
- "Postage from €1.30" is **below** La Poste's public Lettre verte rate for 20 g (€1.52 in 2026, €1.67 in 2027) [V2, sibling; 2026 rates re-confirmed by the editor's search], so Merci Facteur is evidently not passing on retail stamps, and whether retail weight bands apply to its pricing is unknown (fact-check). Weight: the folded card per the README (open 28 × 19.5 cm on 350 g/m²) is about 19.1 g before a C5 envelope of 7–8 g, so about 27 g mailed [calc]; the price page's 30 × 21 cm on 300 g/m² gives 18.9 g before the envelope [calc, fact-check]. If retail bands applied, the public rate would be €3.10 (2026) or €3.34 (2027). Until a test order shows the real invoice, treat **€4.29 as an unverified floor and about €6.10 (2026) to €6.33 (2027) as a guessed ceiling** [est.: €2.99 + public 100 g rate]; the sibling note's Maileva postage (0–50 g bands at €0.747–1.538 HT) shows bulk senders face different bands altogether.
- Whether €2.99 and €1.30 are HT or TTC is not stated, and API orders are billed "at the same price as a letter sent from the Merci Facteur PRO interface" [V1, README], which was not retrieved; the API has a `/getPostagePrice` endpoint that would answer it with an account.

Contradictions between the notes on single-card prices, and what we keep:

- Printful: the web2print note read €2.36 on the product page, the print-api note read €1.62 "shown (discounted)" on the same page. We keep **€2.36** as the planning figure because a discount cannot be assumed to last.
- MyPostcard: its own FAQ pages give €3.69 and €4.29 for a folded card, and an App Store listing "from €3.99" [V1, sibling market note]. We keep the range.
- Print&Bind and Printful "about €6 landed" are estimates that borrow Printful's €3.49 shipping to the Netherlands; no shipping rate to France was found for either.

### 3.5 Dummy book or notebook with a 10 mm spine, one copy to France (P5)

| Provider | Verified facts | Estimate, about 160 pages, black-and-white interior, colour cover | Class |
|---|---|---|---|
| BoD (DE, bod.fr) | Print from €2.05 per copy ([prix](https://www.bod.fr/prix)); a 300-page paperback on 90 g paper cited at €9.50 ([blog](https://www.mecanismes-dhistoires.fr/imprimer-son-livre-a-la-demande-avec-bod-avis-complet/)); shipping to France €4.50 excl. VAT up to 499 g, 6–8 days ([livraison](https://www.bod.fr/mybod/livraison-et-paiement.html)) [V1 each]. No public order API found | **about €10–11 HT** [est., linear interpolation] | L + E |
| Lulu Print API (US) | No subscription or setup fee ([Lulu help](https://help.api.lulu.com/en/support/solutions/articles/64000254631-are-there-fees-to-use-lulu-s-print-api-)); routes to the nearest facility, EUR supported ([API docs](https://api.lulu.com/docs/?_gl=1)); 200-page black-and-white paperback about $5.34 ([chapter.pub](https://blog.chapter.pub/lulu-pricing/)) [V1 each] | about $4.5 print + €4–7 post [M] = **about €9–12** [est.] | T + E |
| Bookvault (UK) | 256-page paperback £3.70 ([cost comparison](https://karldrinkwater.substack.com/p/weekly-writers-print-on-demand-cost)); per-binding and per-page pricing ([Author Media](https://www.authormedia.com/printing-the-future-print-on-demand-with-offset-quality-by-bookvault/)); title setup £19.95 on the free plan, Pro £89.95/month, quoted inconsistently ([directory](https://selfpublishingdirectory.com/print-on-demand/bookvault)) [V1 each] | unknown, probably ≥ €10 [est.]; the per-title fee is a blocker when every order is a new title | T + E |
| Print API (NL) | Softcover A6 landscape €5.44 excl. VAT with 24 pages, +€0.25 per extra page; A5 €8.98 ([page](https://www.printapi.nl/drukwerk/softcover-boeken)) [V1] | 150 pages would cost about €36.94 [calc, if the per-page add-on was read correctly] — not viable | L + E |
| Print&Bind (NL) | Softcover A4, 61 colour pages, 100 g, 300 g cover: €17.46 (example) ([page](https://www.printenbind.nl/en/book/softcover-book)) [V1] | not estimated | L |
| Helloprint | "Carnet A6 personnalisé" exists ([page](https://www.helloprint.com/fr-fr/spectruma6carnet)) [V1]; price and spine not captured | — | — |

Reading: a dummy book costs **about €9–12 delivered**, against about €4.3–7 for a card; it takes longer (BoD shipping alone is 6–8 days) and wastes about 150 blank pages unless they are useful as a notebook.

### 3.6 Postage and the 20 g band

La Poste public tariffs [V2, sibling market note; the 2026 column, including €3.10 / €2.97 at 100 g, re-confirmed by the editor's search (La Poste and Tout sur mes finances result summaries); the 2027 column rests on third-party reports of a La Poste release of late July 2026, unverified on laposte.fr]:

| Product | 2026 | From 1 January 2027 |
|---|---|---|
| Lettre verte ≤ 20 g | €1.52 (€1.49 printed online) | €1.67 |
| Lettre verte ≤ 100 g | €3.10 (€2.97 online) | €3.34 |
| Lettre verte suivie ≤ 20 g / ≤ 100 g | €2.02 / €3.60 | €2.17 / €3.84 |
| International letter ≤ 20 g / ≤ 100 g | €2.25 / €4.85 | €2.45 / €5.25 |

Sources: [tarifs courrier](https://www.laposte.fr/tarifs-postaux-courrier-lettres-timbres), [Lettre verte](https://www.laposte.fr/tarif-lettre-verte), [Lettre suivie](https://www.laposte.fr/tarif-lettre-suivie), [Tout sur mes finances](https://www.toutsurmesfinances.com/argent/a/prix-du-timbre), [étranger](https://www.laposte.fr/tarifs-postaux-etranger).

A poche jacket (230 × 178 mm, about 0.041 m²) weighs about 10 g at 250 g/m², 12 g at 300 g/m² and 14 g at 350 g/m²; a C5 envelope adds about 7–8 g [calc, sibling]. So 250 g/m² stays under 20 g, 300 g/m² is borderline, and 350 g/m², flaps or an A5 jacket move to the 100 g band. **The band jump costs €1.58 per card in 2026 and €1.67 in 2027**, as much as the stamp itself. (The checker doubted the €3.10 figure from memory, expecting twice the 20 g rate, €3.04; the editor's search found €3.10 and €2.97 online on La Poste-derived pages, so the figure stands.) These are retail stamps: API providers and mailing houses buy industrial rates with other weight bands, and other EU posts price by format as well as weight (a C5 envelope is not a standard letter in Germany) [checker's recall], so the 20 g logic is France-specific and retail-specific.

## 4. Print API comparison

"Custom crease" means two creases 10 mm apart at quantity 1. It was found nowhere.

| Provider (seat) | Product for us | Min. qty | Custom crease | API | Price transparency | Unit price evidence | Production for France | DPA / hosting found | Verdict |
|---|---|---|---|---|---|---|---|---|---|
| **Merci Facteur** (FR) | Folded card 15 × 21 cm closed (price page) or 14 × 19.5 cm (API README), envelope, La Poste [V1]; **one JPEG visual per card** in the API; colour PDF letters [V1, README] | 1 | No, one centre fold | REST, public docs on [GitHub](https://github.com/MerciFacteur/Merci-facteur-API) ([README](https://github.com/MerciFacteur/Merci-facteur-API/blob/master/README.md)); production from the €19.95/month plan [V2] | Public price list; `/getPostagePrice` endpoint [V1] | €2.99 + from €1.30 [V1, single vendor] | Printed and posted the same day [V1]; site not named | None found | **Launch choice for France as a letter (A4 PDF)**; as a card only if a test order shows that inside and back artwork can be printed; subject to a DPA |
| **Gelato** (NO [M]) | Flat and folded cards, envelopes ([catalogue](https://www-api.gelato.com/en-US/products)); fold options `fold-ver`, `fold-hor`, `8pp-roll-fold-ver`, `three-fold-hor` ([UID article](https://apisupport.gelato.com/hc/en-us/articles/360010095259-What-is-a-product-UID-)); files unfolded with bleed ([formats](https://support.gelato.com/en/articles/8996362-what-product-formats-and-dimensions-do-you-support)) [V1] | 1 | No | Mature: catalogue, price and order v3 ([get product](https://dashboard.gelato.com/docs/products/product/get/), [create order](https://dashboard.gelato.com/docs/orders/v3/create/)) [V1] | Price API only; optional Gelato+ subscription quoted $14.99 to about $30/month ([ecommerce-platforms](https://ecommerce-platforms.com/articles/gelato-print-on-demand-pricing)) [V1; $14.99 is an older price per the checker's recall] | Not found | 32 countries ([mega.management](https://mega.management/gelato-print-on-demand/), [stylefactory](https://www.stylefactoryproductions.com/blog/gelato-review), [syncost](https://www.syncost.com/blogs/gelato-print-on-demand-review)) [V2]; France named ([merchtitans](https://merchtitans.com/blog/print-on-demand-europe-guide)) [V1, one review blog; French production of *cards* unconfirmed]; FR to FR 2–4 business days for a poster [V1] | Public [data processing terms](https://gelato.com/legal/data-processing-terms) (updated 2024-11-29), [GelatoConnect DPA](https://gelato.com/legal/data-processing-agreement-for-gelatoconnect), AWS with EU residency options ([trust page](https://www.gelato.com/es/trust)) [V1] | **EU scale-out choice**, after a live quote |
| Cloudprinter (NL [M]) | Commercial print "up to books on demand" ([DE page](https://www.cloudprinter.com/countries/local-printing-in-germany-with-global-print-api)); card SKUs not confirmed [V1] | 1 for books | Not found | Mature CloudCore REST, [PHP SDK](https://github.com/cloudprintercom/cloudapps-php-sdk) [V1] | Quote endpoint only ([docs](https://docs.cloudprinter.com/client/how-to-check-prices)) [V1] | None; says it adds no margin, charges a small job fee ([Shopify listing](https://apps.shopify.com/cloudprinter-com)) [V1] | "170 print partners", "104 countries" (own claims) [V1] | None retrieved | Candidate for the book variant only |
| Printful (US/LV) | Greeting card | 1 | No | Mature ([docs](https://developers.printful.com/docs/)) | Public | €2.36 list / €1.62 discounted [V1] | Riga and Barcelona ([Wikipedia](https://en.wikipedia.org/wiki/Printful,_Inc), [merchtitans](https://merchtitans.com/blog/print-on-demand-europe-guide)) [V2]; no French site | Not found; US parent | Fallback only |
| Prodigi (UK) | Fine-art greetings cards, postcards ([cards](https://www.prodigi.com/products/cards-and-stationery/greetings-cards/)) | 1 | Not found | Mature, no setup fee ([reference](https://www.prodigi.com/print-api/docs/reference/)) [V1] | "From" prices | from £0.75 [V1] | Cards "fulfilled from UK, EU, US"; postcards UK only [V1] | Not found | Weak: customs and VAT friction on UK-origin items |
| Printify (US/LV [M]) | Flat and folded cards through third-party providers | 1 | No | Mature ([docs](https://developers.printify.com/)) | Public | "$0.33 flat, $1 folded" [V1] | No EU card provider found | Not found | Exclude on current evidence |
| Print&Bind (NL) | Folded card as a 4-page PDF; books | 1 likely [M] | Not found | Order API for webshops ([page](https://www.printenbind.nl/en/order-api)) [V1] | Public "from" price | from €2.82 [V1] | Not found (Amsterdam [M]) | Not found | Worth a quote; undocumented |
| Peecho (NL) | Books, magazines, wall art; no greeting cards ([products](https://www.peecho.com/products)) | 1 | n/a | Free API, pay per order ([Print API](https://www.peecho.com/solutions/print-api)) [V1] | Partial | magazines from €2.90 [V1] | "Dozens of countries" (own claim) | Not found | No card product |
| Lulu (US) | Paperback with spine | 1 | Spine by page count | Print API, cost endpoint needs a full address [V1] | Calculator / endpoint | about $5.34 for 200 pages (third party) [V1] | Nearest facility [V1]; EU plants not confirmed | Not found; US parent | Book variant only |
| BoD (DE) | Paperback | 1 | Spine by page count | No public order API found; publisher services ([editeurs](https://www.bod.fr/editeurs)) | Public price page | from €2.05 [V1] | Norderstedt [M] | Not researched | Book variant, manual only |
| Bookvault (UK) | Paperback, minimum 28 pages [V1] | 1 | Spine by page count | Exists; terms not retrieved | Partial | £3.70 for 256 pages [V1] | UK | Not found | Exclude: per-title fee, UK origin |
| Blurb (US) | Books, magazines | 1 | n/a | Self-service API ships to the US only; international via custom API ([Blurb](https://www.blurb.com/print-api-software)); active as of 2026-11-10 ([press release](https://www.einpresswire.com/article/955606544/blurb-launches-instant-store-a-fast-way-to-sell-direct-to-consumer-maximize-earnings)) [V1] | Not found | Not found | Not found | Not found | Exclude |
| Mixam (UK, DE site) | Booklets, zines, books; PrintLink + Print API ([mixam.de](https://mixam.de/print-on-demand), [PrintLink guide](https://mixam.com/support/printlink-guide.pdf)) [V1] | n/a | Not found | Endpoints not public | Not found | Not found | mixam.de advertises printing "in Germany" ([mixam.de](https://mixam.de/?locale=en_DE)) [V1]; France not confirmed | Not found | Not assessed further |
| **Maileva** (FR) | A4 letter, not card stock | 1 | n/a | Yes, plus intermediaries ([postal-api.com](https://postal-api.com/tarifs)) [V1] | Public | about €1.0–1.9 HT [est.] | France; handed to La Poste the same day before 14:00 in 98 % of cases ([brochure](https://www.maileva.com/app/uploads/2024/06/Plaquette_plateforme_envoi_courrier_Maileva.pdf)) [V1] | Production and hosting in France [V1] | Fallback: mail the home-print sheet |
| Pingen (CH) | Letter | 1 | n/a | Mature REST | Public | about €1.4–1.6 [est.] | Letters to FR, DE, BE, ES, UK, AT, CH, NL printed locally ([post API](https://www.pingen.nl/en/post-api/)); French products "Lettre verte", "Recommandé" ([help](https://help.pingen.com/en/submit-and-send-letters/introduction-delivery-products)) [V1] | Not found | Letter fallback outside France |
| Stannp (UK) | Postcards, A5 greetings cards; API sizes `A6`, `A5`, `A5-ENV`, `A5-PORT` ([API](https://www.stannp.com/uk/direct-mail-api/postcards?lang=php)) [V1] | 1 | No | Mature, free test mode | Public (UK) | from £0.50 [V1] | UK/US/CA; no EU site found | Not found | Weak for France |
| Lob (US) | Postcards, letters | 1 | No | Mature | Public | — | International = USPS First-Class, 4 × 6 in postcards only ([Lob help](https://help.lob.com/print-and-mail/building-a-mail-strategy/international-mail)) [V1] | US | **Exclude** |
| Flyeralarm PRO (DE) | Catalogue in run-length tiers | 100, 250, ... [V1] | Folder with 10 mm spine is a stock item [V2, sibling] | Reseller REST API, neutral shipping, activation required ([flyeralarm.pro](https://flyeralarm.pro/), [migration doc](https://flyeralarm.pro/wp-content/uploads/media/2025-02-10_Client-API-Documentation_Reseller-Migration_v3.pdf), [be.print wiki](https://wiki.beprint.app/pl/add-ons/reseller-api-dropshipping/informationen-dienstleister-flyeralarm-pro.html)) [V2] | Behind login | see 3.1 | Germany | Not found | Batches only |
| SAXOPRINT (DE, Cimpress) | Catalogue in run-length tiers | tiers | Not found | Reseller interface, key + secret from customer service, free neutral shipping ([saxoprint.de](https://www.saxoprint.de/), [FAQ](https://m.saxoprint.at/faq), [key2print](https://dropshipping.key2print.com/how-to-start-with-api/)) [V2] | Behind login | Not found | Dresden | Not found | Batches only |
| Onlineprinters (DE) | Catalogue; folded cards shipped creased and flat | not found | Not found | `op.connect`, free for resellers, documentation on request ([op.connect](https://www.onlineprinters.de/c/lp/op-connect)) [V1] | Public shop prices | see 3.1 | Germany | Not found | Batches only |
| Helloprint (NL) | "10,000+ products", folders with 10 mm spine [V2, sibling] | 10 on cards [V1] | Folder stock item | Direct API, white-label parcels, 300+ suppliers in 30+ countries ([reseller page](https://www.helloprint.com/en-us/reseller-solutions), [Printweek](https://www.printweek.com/content/news/helloprint-adds-new-api)) [V2] | Behind reseller login | Not found | Partner printers, unknown per order | Not found | Small runs to the reseller, not per recipient |
| Print.com (NL) | Catalogue | not found | Not found | Reseller API ([print.com/api](https://www.print.com/en/api/)) [V1] | Not found | Not found | Not found | Not found | Not assessed |
| Pixartprinting (IT, Cimpress) | Catalogue | 10 [V1] | Not found | **No public API evidence found** | Public shop prices | see 3.1 | Italy | Not found | Manual batches only |
| Also surfaced | [printapi.io](https://www.printapi.io/) (NL), [Tradeprint Connect](https://www.tradeprint.co.uk/tradeprint-api) (UK), MyPostcard Print API — existence only [V1] | — | — | — | — | — | — | — | Not assessed |

Reading: card APIs give quantity 1 but only a single fold; trade-printer APIs give custom finishing only on quote and only in runs. The two needs — one personalised card to one recipient, and a true 10 mm spine — are not met by the same supplier.

## 5. Who is in the EU, precisely

"Exposure" combines three things: a parent or listing under US jurisdiction, a US cloud host, and production outside the EU. The CLOUD Act binds providers under US jurisdiction for data in their "possession, custody or control", wherever stored; its direct lever is the **cloud host** more than the printer, and a US-parented or US-listed group is also reachable by ordinary US subpoenas [M]. Transfers to US entities certified under the EU–US Data Privacy Framework are lawful today; a challenge (*Latombe v Commission*, T-553/23) was dismissed by the General Court on 3 September 2025 and an appeal was lodged in autumn 2025 [M; confirmed by the checker's recall to mid-2026, status since then unknown]. UK adequacy was renewed in December 2025 and Swiss adequacy confirmed in January 2024 [checker's recall].

| Candidate | Seat (legal entity) | Owner | Production relevant to France | Hosting / processors found | CLOUD Act and third-country exposure |
|---|---|---|---|---|---|
| **Exaprint** | Exaprint SAS, Mauguio (FR) | Cimpress since 2015 ([PIWorld](https://piworld.com/article/cimpress-acquires-french-web-print-company-exagroup)) [V1] | **Mauguio, near Montpellier**: about 10,000 m², production re-insourced since 2019, about 400 staff, about €80M revenue. Fact-check: size, staff and revenue not re-verified; historically a platform subcontracting to partner printers, with the in-house plant covering part of the catalogue only, so a French plant does not prove that our job is printed there; trade customers only (SIRET) — both points the checker's recall ([Journal des Entreprises](https://www.lejournaldesentreprises.com/article/limprimeur-exaprint-investit-dans-la-qualite-de-vie-au-travail-2119064), [€15M plan](https://www.lejournaldesentreprises.com/occitanie/breve/limprimeur-en-ligne-exaprint-investit-15-millions-deuros-2053466), [La Réclame](https://lareclame.fr/exaprint-factory-cartes-visite-232104), [Businesscoot](https://businesscoot.com/en/company/exaprint)) [V2] | Not found; shared Cimpress platform likely [M-low] | Indirect: Cimpress plc is Irish-domiciled, Nasdaq-listed, with large US operations and, from memory, AWS infrastructure [M] |
| Pixartprinting | Pixartprinting S.p.A., Quarto d'Altino (IT) | Cimpress | Quarto d'Altino (Venice) and Lavis (Trento); "facilities" in France and Tunisia of unspecified nature; Pittsburgh (US) since 2025 ([Wikipedia](https://en.wikipedia.org/wiki/Pixartprinting)); packaging line at Quarto d'Altino ([World of Print](https://www.worldofprint.com/?p=2364436)) [V1]; billing and shipping address must be in the same country per UK terms ([terms](https://pixartprinting.co.uk/sales-terms-conditions)) [V1] | Not found | Indirect, via Cimpress |
| Vistaprint | Cimpress plc, Dundalk (IE); EU sales entity in Venlo (NL) [M] | Cimpress | Venlo (NL) [M]; no French production ([L'Essentiel de l'Éco](https://lessentieldeleco.fr/8883-quel-est-le-meilleur-site-de-carte-de-voeux-2027)) [V1] | AWS, from memory [M]; customer care outside the EU [M] | High |
| WIRmachenDRUCK | GmbH, Backnang (DE) | Cimpress since 2015/16 ([WhatTheyThink](https://whattheythink.com/news/76915-cimpress-agrees-acquire-leading-german-web-print-company-wirmachendruck), [deutsche-startups](https://www.deutsche-startups.de/2015/12/21/cimpress-kauft-wirmachendruck/)) [V2] | Own plants plus outsourced partner printers ([fastlancer](https://www.fastlancer.org/fastlancer-blog/wirmachendruck-review/)) [V2]; plant per order not predictable | Not found | Indirect |
| SAXOPRINT, viaprinto | SAXOPRINT GmbH, Dresden (DE) | **Cimpress since 2026-07-02**, bought from CEWE ([Nasdaq](https://www.nasdaq.com/press-release/cimpress-agrees-purchase-saxoprint-and-viaprinto-cewe-2026-05-11), [TipRanks](https://www.tipranks.com/news/company-announcements/cimpress-completes-acquisition-of-saxoprint-and-viaprinto), [CEWE via Yahoo Finance](https://finance.yahoo.com/markets/stocks/articles/cewe-completes-sale-commercial-online-213408958.html), [Printweek](https://www.printweek.com/content/news/cimpress-to-acquire-saxoprint-and-viaprinto)) [V2] | Dresden, Enderstr. 92c, about 500 staff, 21,000 m² ([CEWE site page](https://www.cewe-group.com/en/about-us/corporate-group/sites/dresden-site-saxoprint.html)) [V1]; sells in France among nine countries [V1] | Not found | Indirect, new |
| Easyflyer | French brand | Cimpress [V1] | Not found | Not found | Indirect |
| **Onlineprinters** | Onlineprinters GmbH, Fürth (DE) | Bregal Unternehmerkapital (private equity, family capital based in Switzerland/NL) [M] | Main plant Neustadt an der Aisch; group sites listed inconsistently: Fürth, Waldbüttelbrunn, Southend (Solopress, UK), Aarhus and Skarpnäck (LaserTryk, DK/SE), Szczecin (PL), Leganés (Copysell, ES) ([overview](https://onlineprinters.ie/c/about-us/the-company/overview), [group press release](https://www.onlineprinters.ie/c/about-us/press-info-and-pr/press-releases/onlineprinters-group-bundles-international-activities-and-pushes-strategic-realignment)) [V1] | Not found | No US parent known; depends on its cloud vendors |
| **Flyeralarm** | GmbH, Würzburg (DE) | Founder-owned [M] | Eight sites per one source (three in Würzburg, Greußenheim, Marktheidenfeld, Veitshöchheim, Klipphausen, Kesselsdorf), six per an older one ([Lifeverde](https://www.lifeverde.de/nachhaltigkeitsmagazin/news-tipps/trendentwicklung-co2-reduziertes-drucken), [Druckmarkt](https://www.druckmarkt.com/archiv/pdf/125/41_Flyeralarm_CO2.pdf)) [V1] | Not found | No US parent known |
| Helloprint | B.V., Rotterdam (NL) [M] | Venture/PE-backed [M-low] | Marketplace over 300+ partner printers [V2]; plant unknown per order | Not found | Unknown |
| Print.com | NL | See contradiction below | Not found | Not found | Unknown |
| print24 / Unitedprint | Unitedprint SE, Radebeul (DE) [M] | Founder-owned [M-low] | Not researched | Not found | Unknown |
| Printoclock | Toulouse (FR), founded 2008 ([Aleou](https://www.aleou.fr/xg/contact/supplier/58697)) [V1] | Not established | Brochure mentions a new workshop in Toulouse ([PDF](https://www.printoclock.com//media/file/1e/1c/08fddf0a58413472f4b21d371925.pdf)) [V1]; in-house share unknown | Not found | Unknown |
| Realisaprint, Rapid-Flyer | FR (Alpes-Maritimes; Carros) [M] | Not established | Not found | Not found | Unknown |
| Print&Bind, Print API | NL | Not established | Not found (Amsterdam [M]) | Not found | Unknown |
| **Merci Facteur** | French company; legal entity not retrieved | Not established | Printed and posted the same day via La Poste [V1]; site not named | **No DPA or hosting statement found** | Unknown — must be asked before launch |
| Popcarte, Fizzer | French companies | Not established | Fizzer prints in France [V1]; Popcarte assumed | Not found | Unknown |
| **Maileva** | Docaposte, La Poste group (FR) | State-owned (Caisse des Dépôts, État); "entreprise à mission" since 2021 [M; ownership confirmed by the checker's recall] | France [V1] | **France** [V1, one 2024 vendor brochure; no DPA or sub-processor list read] | Lowest |
| Pingen | Switzerland (adequacy decision [M]) | Not established | Prints French letters locally [V1] | Not found | Low |
| **Gelato** | Gelato AS, Oslo (NO, EEA) [M] | Venture-backed [M] | Partner in the destination country where available [V1]; address data necessarily goes to that partner | AWS with EU data-residency options; transfers under DPF or SCC; sub-processor list is Schedule 2, contents not retrieved [V1] (Oslo seat and 32 countries match the checker's recall) | US cloud host; local partner unknown per order |
| Cloudprinter, Peecho | Netherlands [M; V1 for Peecho, Amsterdam] | Not established | Partner networks | No DPA text retrieved | Unknown |
| Printful | US corporation with Latvian operations; merger with Printify announced late 2024 [M] | Private | Riga (LV), Barcelona (ES) [V2] | Not found | High: US parent |
| Lulu | US corporation; certified B Corp [M] | Private | Nearest facility [V1] | Not found | High: US parent |
| Printify, Blurb, Lob | US | — | No EU card production found | Not found | High |
| BoD | GmbH, Norderstedt (DE) [M] | Not established | Norderstedt [M] | Not found | Low |
| dieUmweltDruckerei, Lokay, oeding print | German owner-managed SMEs: Hannover, Reinheim, Braunschweig [M] | Owners [M] | Germany [M] | Not found | Low |
| French SCOP and EA printers | France | Workers (SCOP) or associations/companies (EA) [M] | France | Not found | Low |
| MOO, Solopress, Mixam UK, Prodigi, Stannp, Bookvault | UK | Solopress belongs to Onlineprinters since 2017 [V1] | UK; MOO "made in the UK", Camden ([MOO blog](https://www.moo.com/blog/uk/inside-moo/made-in-the-uk), [EU promise page](https://www.moo.com/eu/about/moo-promise), [EU delivery](https://www.moo.com/uk/about/free-delivery-EU)) [V1] | Not found | Third country under an adequacy decision [M]; customs border |
| CEWE | Oldenburg (DE), listed | Left commercial online print in 2026 [V2]; keeps photo products [M] | — | — | Not a candidate any more |

Contradiction on **Print.com**: the web2print note calls it "the former Drukwerkdeal, Cimpress-owned" [M]; the ESG note recalls it as independent, started by Drukwerkdeal's founder after he sold Drukwerkdeal (now Printdeal) to Cimpress [M]. Both are memory and the search meant to settle it was refused. We keep the **ESG note's version** (independent; the Cimpress brand is Printdeal) because it is the more specific recollection and is consistent with the Cimpress brand list in the same note. The fact-checker's independent recall agrees (Drukwerkdeal sold to Cimpress in 2014). It stays memory; do not rely on it.

What this means in practice:

- **Batches carry no personal data** if codes are inactive at print time: the printer receives artwork only. Group ownership then matters for ESG, not for privacy.
- **Per-recipient mailing sends a name and a postal address** to the printer, which becomes a sub-processor (reseller = controller, eBook Cadeau = processor, printer = sub-processor, Art. 28 contract and prior authorisation needed [M]). Here hosting and ownership matter. Only Maileva is verified as French-owned with French hosting; Merci Facteur must answer in writing.
- No printer's website hosting location appeared in any search result. "Hosted in the EU" is stated for **Maileva only**, and only in a 2024 vendor brochure: that is not the written, contractual statement the `eu_only` rule requires (fact-check), so nobody passes that condition yet.
- **Standalone mode** (added by the fact-check, checker's recall): when an individual orders a mailed card, eBook Cadeau is the controller for the recipient's address and owes the recipient the information of GDPR Art. 14; the postal operator is normally an independent controller, not a sub-processor. The processor chain above applies to reseller mode only.

## 6. ESG: method, scoring grid and ranking

### 6.1 What each label guarantees

All rows [M] unless tagged; registers are named, not linked, because their addresses were not confirmed in this run.

| Label | What is certified | What it does not cover | Weight in the grid |
|---|---|---|---|
| **Blauer Engel DE-UZ 195**, EU Ecolabel for printed paper (Decision (EU) 2020/1803), Nordic Swan, Austrian UZ 24 | The printed product: recycled or ecolabelled paper, restricted inks and chemicals, VOC limits, de-inkability, energy management | Social aspects, transport | Highest (E1) |
| Cradle to Cradle Certified (e.g. gugler* AT, licensed partners such as Lokay) | A defined paper + ink + coating system, including social fairness | Only the certified combinations; gugler*'s current status unverified | Highest (E1), niche |
| EMAS | Site: ISO 14001 plus a verified public environmental statement | Product properties | High (E2) |
| ISO 14001 / ISO 50001 | Management process for environment / energy | Any performance level | Medium (E2) |
| **Imprim'Vert** (FR) | Site: hazardous-waste disposal, secured storage, no CMR-labelled products, awareness, energy monitoring; held by well over a thousand sites and by some foreign ones such as Onlineprinters ([Pressebox](https://www.pressebox.com/pressrelease/onlineprinters-gmbh/ImprimVert-Environmental-certification-renewed-for-the-sixth-time/boxid/1110757)) [V1] | Paper origin, carbon, social | Baseline (E2) |
| FSC / PEFC | Fibre chain of custody; the printer needs its own licence to print the logo | Printing process, inks, energy | Medium (E3) |
| Print'Ethic (UNIIC), B Corp, Gemeinwohl-Bilanz, EcoVadis scorecard | Company CSR practice | Product properties | S4 |
| ClimatePartner "klimaneutral", PrintReleaf, tree planting | Offsets or compensation | Reduction of the product's own emissions | **Zero** |

Process notes [M]: mineral-oil inks are restricted by law in France — loi AGEC art. 112 bans specified mineral-oil fractions above thresholds (arrêté of 13 April 2022) in printed matter for the public since 1 January 2025. It does not mandate vegetable inks (corrected by the fact-check, checker's recall), mainly concerns offset, and barely touches toner and inkjet, the likely processes for runs of 1–300. It applies to what is distributed in France, so a German or Italian batch printer must be asked for compliance. Low-mineral-oil ink is a baseline, not a merit. Film lamination, UV varnish and foil hinder recycling and are restricted by product ecolabels. Dry toner de-inks well; HP Indigo liquid toner historically does not, and much inkjet output also fails de-inking tests that DE-UZ 195 requires, so criterion E1 is probably unattainable for any single-card service (fact-check, checker's recall). Recycled 300 g board exists in mainstream catalogues: Flyeralarm lists a recycled 300 g stock ([product page](https://www.flyeralarm.com/fr/p/cartes-de-fidelite-525038)) [V1]; Onlineprinters lists Blue Angel, FSC and PEFC papers ([environment page](https://www.onlineprinters.ie/c/about-us/certificates-and-awards/environment-and-sustainability)) [V1].

### 6.2 What may be claimed

All [M]; a lawyer must confirm before launch.

- Directive (EU) 2024/825 ("EmpCo"): transposition due by 27 March 2026, application from **27 September 2026** (dates confirmed by the checker's recall and the editor's search). It binds traders through national law: France's transposition (DDADUE bill, arts. 20–21) was reported as not yet complete in late 2026 [V1, editor's search, law-firm commentary; check Légifrance], so "blacklisted in France since 27 September 2026" is not established. Blacklisted towards consumers: generic environmental claims without recognised excellent performance (EU Ecolabel or an official Type I label); claims about a whole product when only one aspect is concerned; **climate-neutral claims based on offsetting**; sustainability labels not backed by a certification scheme; presenting a legal requirement as a distinctive feature.
- ClimatePartner itself warns that advertising "klimaneutral drucken" without disclosed reductions now risks warnings and injunctions ([ClimatePartner](https://www.climatepartner.com/de/klimaneutral-drucken)) [V1].
- France already restricts "neutre en carbone" advertising (Code de l'environnement art. L. 229-68) and forbids "respectueux de l'environnement", "biodégradable" and similar wording on products and packaging (loi AGEC art. 13, codified at art. L. 541-9-1), so on the card itself. Germany: BGH, 27 June 2024, I ZR 98/23. (Citations confirmed by the checker's recall.)
- The Green Claims Directive proposal: the Commission announced in June 2025 that it intended to withdraw it [checker's recall, unverified].
- Whether Imprim'Vert counts as a "certification scheme" under the directive is an open question (researcher's analysis).
- Safe pattern: specific, checkable facts only, for example "imprimé à [ville] par [imprimeur], licence FSC-C…, papier 100 % recyclé".

### 6.3 What drives the footprint of one card

Orders of magnitude only [est., built on M]; no life-cycle study was consulted.

| Item | Rough CO2e |
|---|---|
| Board, 15 g | 10–20 g |
| Digital printing, both sides | 5–30 g |
| Envelope + letter post in France | tens of grams |
| Courier parcel within Europe, one card | 300–1,000 g |
| Batch of 100 cards in one parcel | 3–10 g per card |
| Home inkjet, one A4 sheet | 10–30 g |
| Dedicated offset makeready for a 50-card job | more than the product itself (100–250 waste sheets plus plates) |

Consequences (an opinion, not a finding: the fact-check rates the ranking UNSUPPORTED because no life-cycle study was used): for single cards the order is **home or in-store print < French print + letter post < one batch parcel < cross-border parcel per card**. For batches, paper and process matter and transport becomes minor, so a certified printer in Germany is defensible for a French batch. Runs of 1–300 belong on digital presses or gang-run offset. A dummy book multiplies paper mass by about 15.

### 6.4 Scoring grid (100 points)

Environment 70, social 25, data 5. The grid is the ESG researcher's; the partial-credit rules are added here so that two people scoring the same supplier get the same number.

| # | Criterion | Max | Rule |
|---|---|---|---|
| E1 | Product-level Type I ecolabel or C2C usable on our job | 15 | 15 if a licence covering our job is seen; else 0 |
| E2 | Audited site system | 10 | EMAS 10; ISO 14001 6; ISO 50001 +2; Imprim'Vert alone 3; capped at 10 |
| E3 | Board | 10 | 10 = recycled Blue Angel / FSC Recycled 250–350 g by default; 7 = recycled board offered as an option; 4 = FSC Mix or PEFC board; 0 = nothing seen |
| E4 | Process fit | 10 | 10 = evidence of digital or gang-run production for our run length and unlaminated, de-inkable output; 5 = digital or gang-run by construction of the product (single-copy service, mass web-to-print); 0 = unknown, dedicated makeready, or about 150 blank pages |
| E5 | Energy | 5 | 5 = renewable electricity with evidence |
| E6 | Transport to a French address | 15 | 15 = printed in France, letter post or pick-up; 12 = printed in France, one parcel, or French letter post with the print site not named; 8 = neighbouring EU country, one batch parcel, or French company with unknown in-house share; 6 = network claiming in-country routing; 4 = unknown EU plant; 3 = one item per order from another EU country; 0 = UK or US origin |
| E7 | Claims hygiene | 5 | 5 = publishes measured data and makes no offset-based neutrality claim; 3 = publishes a report but also markets offsetting, or report not read; 0 = otherwise |
| S1 | Ownership and form | 5 | 5 = SCOP, foundation or employee ownership; 4 = public; 3 = family or founder; 1 = listed, PE/VC-backed or not established |
| S2 | Inclusive employment | 8 | 8 = EA; 6 = ESAT or insertion structure; 3 = documented partnership |
| S3 | Labour transparency | 7 | 7 = own named EU plants with collective-agreement evidence; 5 = own named EU plants; 2 = own plants plus partners, or site only partly identified; 0 = opaque network or unknown |
| S4 | Third-party social rating | 5 | 5 = B Corp, Print'Ethic, GWÖ or a shared EcoVadis scorecard seen |
| G1 | Data and jurisdiction | 5 | +2 controlled from the EU/EEA with no US/UK parent and no US listing known (1 if EU seat in a US-listed group; 0 if US/UK parent); +2 EU hosting stated (1 if only "EU residency options" on a US cloud); +1 DPA with sub-processor list published |

Evidence rules:

- **Floor score** = points for facts seen in a source during this run ([V1]/[V2]). Company-published claims without a register check count half, rounded up. Memory counts zero. Offsetting counts zero.
- **Ceiling score** = floor plus the points that would be added if the researchers' [M] recollections were confirmed in a register.
- The gap between the two is the verification work still to do. A supplier with no verified fact is listed with a ceiling only.

### 6.5 Scores

**A. Batches (25–1,000 copies) delivered to one French address**

| Supplier | E1 | E2 | E3 | E4 | E5 | E6 | E7 | S1 | S2 | S3 | S4 | G1 | **Floor** | **Ceiling** | What the ceiling assumes |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Onlineprinters | 0 | 2 | 4 | 5 | 0 | 8 | 3 | 1 | 0 | 5 | 0 | 2 | **30** | 34 | Imprim'Vert in the directory; Blue Angel board at 300 g |
| Flyeralarm | 0 | 0 | 7 | 5 | 0 | 8 | 0 | 1 | 0 | 5 | 0 | 2 | **28** | 30 | Founder ownership |
| Exaprint | 0 | 0 | 0 | 5 | 0 | 12 | 0 | 1 | 0 | 5 | 0 | 1 | **24** | 31 | Imprim'Vert; FSC/PEFC board |
| SAXOPRINT | 0 | 4 | 0 | 5 | 0 | 8 | 0 | 1 | 0 | 5 | 0 | 1 | **24** | 28 | ISO 14001 and 50001 still current after the takeover |
| Entreprise adaptée (Copiver Express as the one verified example) | 0 | 0 | 0 | 0 | 0 | 12 | 0 | 1 | 8 | 0 | 0 | 2 | **23** | 40 | Imprim'Vert, PEFC board, digital press, named site |
| Printoclock | 0 | 2 | 2 | 5 | 0 | 8 | 0 | 1 | 0 | 2 | 0 | 2 | **22** | 32 | Toulouse workshop prints our job; labels in registers |
| Pixartprinting | 0 | 0 | 0 | 5 | 0 | 8 | 0 | 1 | 0 | 5 | 0 | 1 | **20** | 20 | Nothing further recalled |
| dieUmweltDruckerei | 0 | 0 | 0 | 0 | 0 | 8 | 3 | 1 | 0 | 0 | 0 | 2 | **14** | 58 | DE-UZ 195 licence, recycled Blue Angel board by default, green electricity, owner-managed, own plant |
| WIRmachenDRUCK | 0 | 0 | 0 | 5 | 0 | 4 | 0 | 1 | 0 | 2 | 0 | 1 | **13** | 13 | — |
| Helloprint | 0 | 0 | 0 | 5 | 0 | 4 | 0 | 1 | 0 | 0 | 0 | 2 | **12** | 12 | — |
| Realisaprint, Rapid-Flyer | 0 | 0 | 0 | 5 | 0 | 4 | 0 | 1 | 0 | 0 | 0 | 2 | **12** | 25 | Own plant in France |
| Vistaprint | 0 | 0 | 0 | 5 | 0 | 4 | 0 | 1 | 0 | 0 | 0 | 1 | **11** | 20 | Venlo plant |
| Lokay, oeding print | — | — | — | — | — | — | — | — | — | — | — | — | **not scorable** | 68 | DE-UZ 195 licence, EMAS, recycled board, green electricity, owner-managed |
| French SCOP printers (Laballery, La Contemporaine, Média Graphic) | — | — | — | — | — | — | — | — | — | — | — | — | **not scorable** | 36 | SCOP status, Imprim'Vert, PEFC board, French plant |

Evidence behind the floor points: Onlineprinters — Imprim'Vert renewal (own press release), PEFC 300 g on product pages, ESG report 2022 ([PDF](https://de.onlineprinters.ch/$WS/diedruckerei/shopdata/web/images/landingpages/unternehmen/was-uns-wichtig-ist/op_esg-report-2022_en.pdf)), carbon-neutral option since 2013 ([press release](https://www.pr.com/press-release/555928)), named sites [V1]. Flyeralarm — recycled 300 g stock, named sites [V1]; its only climate document found is a 2012 offsetting leaflet naming a Ghana cookstove project ([leaflet](https://www.flyeralarm.com/images/upload/content/_pdf/de/de/FA_Infoblatt_KlimaneutralesDrucken_12122012_WEB.pdf)) [V1]. Exaprint — Mauguio plant [V2]. SAXOPRINT — ISO 14001 and ISO 50001 in a 2022 corporate presentation ([PDF](https://www.heidelberg.com/global/media/global_media/company___press_lounge/press_kits_1/2022_15/saxoprint/presentation/SAXOPRINT_Corporate_presentation-2022.pdf)) [V1], before the Cimpress takeover. Copiver Express — UNEA directory sheet ([PDF](https://www.unea.fr/sites/default/files/doc_ea_4657_copiver_express_.pdf)) [V1]. Printoclock — own brochure [V1]. dieUmweltDruckerei — a sustainability report dated 02/2023 exists ([PDF](https://www.dieumweltdruckerei.de/assets/content_files/10580/Nachhaltigkeitsbericht_02_23_dieUmweltDruckerei.pdf)) [V1], content not read.

**B. One personalised card to one French recipient**

| Supplier | E1 | E2 | E3 | E4 | E5 | E6 | E7 | S1 | S2 | S3 | S4 | G1 | **Floor** | **Ceiling** | Note |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Maileva (plain letter, P6) | 0 | 0 | 0 | 5 | 0 | 15 | 0 | 1 | 0 | 2 | 0 | 4 | **27** | 30 | Ceiling: public ownership confirmed. Not a card |
| Fizzer (flat card) | 0 | 0 | 0 | 5 | 0 | 15 | 0 | 1 | 0 | 2 | 0 | 2 | **25** | 25 | No API: benchmark only |
| **Merci Facteur** (folded card) | 0 | 0 | 0 | 5 | 0 | 12 | 0 | 1 | 0 | 0 | 0 | 2 | **20** | 28 | Ceiling: French print site named |
| Popcarte (folded card) | 0 | 0 | 0 | 5 | 0 | 12 | 0 | 1 | 0 | 0 | 0 | 2 | **20** | 28 | No API: benchmark only |
| **Gelato** | 0 | 0 | 0 | 5 | 0 | 6 | 0 | 1 | 0 | 0 | 0 | 4 | **16** | 22 | Ceiling: French partner confirmed for cards |
| Printful | 0 | 0 | 0 | 5 | 0 | 3 | 0 | 1 | 0 | 5 | 0 | 0 | **14** | 14 | Riga or Barcelona to France |
| Print&Bind | 0 | 0 | 0 | 5 | 0 | 3 | 0 | 1 | 0 | 0 | 0 | 2 | **11** | 11 | — |
| Lulu dummy book | 0 | 0 | 0 | 0 | 0 | 6 | 0 | 1 | 0 | 0 | 0 | 0 | **7** | 12 | Ceiling: B Corp confirmed |
| BoD dummy book | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 1 | 0 | 0 | 0 | 2 | **6** | 6 | — |
| Prodigi, Stannp (UK) | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | **6** | 9 | Ceiling: Prodigi fulfils from the EU |

Print at home and in-store printing are not suppliers and are not scored; they beat every row on transport (no shipment) and on data (no address leaves the buyer).

### 6.6 Ranking and how to read it

**No supplier scores above 30 out of 100 on verified evidence.** That reflects what could be checked (locations, a few papers, a few company documents), not how good the suppliers are. The ranking below is therefore provisional and names what would change it.

Batches:

1. **To verify first, highest potential — environment:** Lokay and oeding print (ceiling 68), dieUmweltDruckerei (58). One register lookup each (Blauer Engel product search, EMAS register) decides whether they lead.
2. **To verify first, highest potential — society:** entreprises adaptées (ceiling 40, the only verified S2 points) and SCOP printers (36). The grid gives the environment 70 % of the points, which is why they sit below the German eco-printers; on social criteria alone they come first.
3. **Best on verified evidence today:** Onlineprinters (30), Flyeralarm (28), Exaprint and SAXOPRINT (24).
4. **Low or opaque:** Pixartprinting (20), WIRmachenDRUCK, Helloprint, Vistaprint (11–13): little evidence, or the plant is not predictable per order.
5. **Deprioritise:** UK production (MOO, Solopress, Mixam UK): customs border and longer transport.

Singles: French print with letter post leads (Maileva 27 for a letter, Merci Facteur 20 for a card), then Gelato (16), then Printful (14); dummy books and UK or US origin come last (6–7).

Contradiction resolved: the web2print note ranks Exaprint as the best mainstream option for France; the grid puts Onlineprinters six points ahead. Both are right about different things. Onlineprinters has more **evidence** (report, Imprim'Vert, papers); Exaprint has the better **transport** score and French jobs, and its labels are simply unverified (three points of Imprim'Vert and four of PEFC would close the gap to 31 against 34). We keep the grid order for ESG and keep Exaprint as the first batch supplier to quote, because free delivery in France, lead time and the custom-fold lead are decision criteria outside the grid.

### 6.7 Ownership, labour and other social notes

- Ownership [M unless tagged]: Cimpress plc (listed, leveraged) owns Vistaprint, Pixartprinting, Exaprint, Easyflyer, WIRmachenDRUCK, Printdeal, druck.at, Tradeprint, and SAXOPRINT and viaprinto since 2026-07-02 [V2 for the 2026 deal]. Onlineprinters belongs to Bregal. Flyeralarm is founder-owned. La Poste group (Maileva) is State-owned.
- Exaprint invested in workplace quality ([Journal des Entreprises](https://www.lejournaldesentreprises.com/article/limprimeur-exaprint-investit-dans-la-qualite-de-vie-au-travail-2119064)) [V1].
- Marketplaces and partner networks (Helloprint, WIRmachenDRUCK's partners, Gelato, Cloudprinter) do not let the customer see working conditions at the plant that prints. The grid scores that opacity at 0–2 on S3.
- Letter post is delivered by postal employees; parcel delivery in France relies largely on subcontracted drivers [M-low]. This is a second reason to prefer envelope formats.
- EA versus ESAT: an EA is an ordinary employer paying at least the minimum wage; ESAT workers are users of a medico-social service, paid a guaranteed fraction of it [M]. The EA is the stronger social claim. Prison print workshops are excluded.
- **Recollections not to be repeated publicly** (no source in this run): union criticism of Flyeralarm over collective agreements and works councils [M-low]; offshored customer care and restructurings at Cimpress/Vistaprint [M].

## 7. Recommendation and phased sourcing plan

### 7.1 Recommendation

1. **Default to print at home.** It costs nothing, ships nothing, exposes no address, and is the only route for a standalone user whose cover is not licensed for professional printing (see `07-legal-covers-and-reseller-rights.md`: an unlicensed cover is never sent to a printer).
2. **Mailed single card (reseller mode, or a rights holder gifting one copy): Merci Facteur**, with the single-fold A5 layout (P2). Planning cost €4.29–4.79 per card plus €19.95 per month; ceiling about €6.3 if the card lands in the 100 g postage band.
3. **Batches of one title (authors, publishers, reseller stock): quote Exaprint first, Onlineprinters second**, and ask both for the true jacket (P1). Buy the single-fold card (P2) from the catalogue while waiting: €0.36–0.60 per card at 100.
4. **Offer two labelled alternatives to resellers**, once verified: an entreprise adaptée (social case, OETH deduction [M]) and a Blauer Engel printer (environmental case).
5. **Do not use** book POD as the default object (cost about €9–12, about 150 blank pages), UK or US origin for single items, or any supplier's "climate-neutral" wording.

### 7.2 Phases

| Phase | Volume trigger | Singles (one recipient) | Batches (one title) | Actions and gates |
|---|---|---|---|---|
| **0. Before launch** (weeks 0–2) | none | Home print only | None; fake printer in tests | Send the quote request of section 8.2 to Exaprint, Onlineprinters, Flyeralarm (A5 folder, 10 mm), one EA, one SCOP. Order samples: one Merci Facteur folded card (about €4.29), 100 folded A6 cards at Onlineprinters (€52.41 HT). **Weigh them.** Ask Merci Facteur for a DPA, hosting country, print site, HT/TTC and the postage band |
| **1. Launch** (months 1–3) | first orders | Merci Facteur API, layout P2 A5, tracking optional (+€0.50) | Catalogue P2 at Onlineprinters or Exaprint, 50–250 copies, delivered creased and flat | Gate to open mailed cards: DPA signed, test invoice matches the price model, codes inactive until activation. Retail price must cover €4.29–6.33 of cost. At €6.90 and a €4.29 cost, the contribution is about €2.26 after payment fees (1.5 % + €0.25), about €1.41 after micro-entreprise charges of 12.3 % of turnover; the €19.95 plan is then covered from about 15 cards a month. If the cost is €6.33, the same contribution needs a price of about €9.00 before charges and €9.30 after them [all est.] |
| **2. Reseller pilot** (months 3–9) | 100–500 mailed cards a month, or a first reseller | Keep Merci Facteur; negotiate a direct mailing contract (*routage*) with an EA or a mailing house: in-house style cost about €1.08 print and envelope + €1.67 stamp = €2.75 [est., business-model placeholders], a saving of about €1.5–2.0 per card | True jacket (P1) from the winning quote, 250–1,000 per title, variable data if unique codes are required | Reseller is controller; list each printer as an authorised sub-processor. Publish only verified facts (site, licence numbers). At 500 cards a month the saving is about €770–1,000 a month [est.], enough to justify the contract work |
| **3. EU scale-out** (month 9+) | first non-French market, or more than about 2,000 mailed cards a month (Merci Facteur plan step to €39.95) | Gelato folded card per country after a live price check; Pingen or a national postal API for the letter fallback | One certified printer per region: a DE-UZ 195 printer for German-speaking markets; Exaprint or an EA for France | Gate: Gelato sub-processor list read; French partner and delivery mode (letter or parcel) confirmed; no country served by a cross-border parcel per card |

In-store printing at a reseller's photo counter would remove shipping altogether; whether Fnac counters can print and crease a 300 g card is unknown [M-low]. Keep it as a question for the first reseller meeting.

### 7.3 What would change the recommendation

- Merci Facteur cannot provide a DPA or prints outside France: move singles to Gelato (if a French partner is confirmed) or to Maileva with the letter format.
- A French printer quotes the true jacket at quantity 1–10 with mailing: it replaces both Merci Facteur and the P2 compromise.
- Exaprint's "pliure sur mesure" does not accept two creases: use a folder product with a 10 mm spine (Flyeralarm or Helloprint, A5) or a regional printer on quote.
- A register lookup confirms a DE-UZ 195 licence at dieUmweltDruckerei with delivery to France: it becomes the default "certified" batch option, since it is an upload-and-pay shop [M].

## 8. What could not be verified, and how to get real quotes

### 8.1 Not verified

Prices and products:

- **Any live configurator price or written quote.** Prices at 1, 10, 500 and 1,000 for a folded card are missing for almost every printer.
- **Any price for the true jacket (P1) or for a folder with a 10 mm spine (P3)**, with or without variable data. Whether any configurator accepts two creases 10 mm apart at book trim; Exaprint's "pliure sur mesure" is the only lead.
- Official unit price and shipping to France for Gelato, Cloudprinter, Prodigi, Printful (card shipping), Lulu, Helloprint, Flyeralarm PRO, SAXOPRINT, Onlineprinters op.connect, Print.com, Mixam. Whether the "$6.16 per pack of 10" figure refers to Gelato.
- Merci Facteur: HT or TTC, API-side price, postal product and weight band, accepted artwork formats, print site, DPA, hosting.
- HT versus TTC on the Onlineprinters €59.24 and €54.59 figures and on Vistaprint's €0.74. Pixartprinting's minimum (10 versus 100). Whether an Exaprint business account is mandatory.
- Lulu: minimum page count, EU plants, postage to France. BoD: minimum page count, order API. Bookvault: EU shipping after Brexit, current fees. Gelato and Printful notebooks as spine carriers.
- Dust jackets, small packaging, bellybands: not searched. Not researched at all: print24 / Unitedprint, Drukland, Copy-Top, COREP, PostGrid, Docsaway, Touchnote, Cewe cards; Easyflyer beyond ownership; MOO, Solopress and Mixam prices.

Locations and data:

- **Website and data hosting for every printer except Maileva and Gelato.** Sub-processor lists for all; Gelato's Schedule 2 was not retrieved. Cimpress on AWS is from memory.
- Production site per order for WIRmachenDRUCK, Helloprint, Printoclock, Realisaprint, Rapid-Flyer, Print&Bind, Vistaprint (Venlo is from memory), Cloudprinter, Peecho, and for Gelato's French partner. The nature of Pixartprinting's "facilities in France and Tunisia". Flyeralarm's site count (six or eight). Onlineprinters' site list (company pages disagree).
- Seats marked [M]: Gelato (Norway), Cloudprinter (Netherlands), MyPostcard (Germany); UK and Swiss adequacy status; ownership of Bregal/Onlineprinters, Flyeralarm, Unitedprint, Helloprint, Print.com, and of Merci Facteur, Popcarte, Fizzer, Printoclock, Realisaprint, Rapid-Flyer; the Printful–Printify merger.

ESG and law:

- **Every label**: Imprim'Vert, FSC, PEFC for Exaprint, Pixartprinting, Printoclock and the French card services; SAXOPRINT's ISO certificates after the takeover; Flyeralarm's current reporting; all B Corp, EcoVadis and Print'Ethic statuses.
- **Every eco or social printer profile** (Lokay, oeding print, dieUmweltDruckerei beyond the existence of its report, gugler*, Printzipia, Laballery, La Contemporaine, Média Graphic, Imprimerie 34, Imprimerie Villière, Pure Impression, Escourbiac, Handiprint, ANRH, APF Entreprises): labels, minimums, prices, delivery to France. "Ecoprint", "Greenprinter" and "ICI Imprimerie écologique" were not identified.
- Label mechanics (exact Imprim'Vert criteria, DE-UZ 195 version, FSC Mix and PEFC thresholds); the application date of Directive (EU) 2024/825 and its French transposition; the status of the Green Claims Directive; the OETH deduction rules; the Data Privacy Framework appeal.
- All carbon figures in 6.3; all labour recollections in 6.7.

### 8.2 How to get real quotes

1. **Read the configurators.** Allow these hosts in the session's network settings (the first three were tested and blocked): `www.onlineprinters.fr`, `www.printenbind.nl`, `comparatif-imprimerie.fr`, then `www.exaprint.fr`, `www.flyeralarm.com`, `www.pixartprinting.fr`, `www.helloprint.com`, `www.merci-facteur.com`, `www.saxoprint.de`. Record for each: product URL, format, stock, quantity tiers 10 / 25 / 50 / 100 / 250 / 500 / 1,000, HT and TTC, delivery fee, lead time, date.
2. **Use the price endpoints**, which need an account but no order: Gelato's product and price API, Cloudprinter's CloudCore price lookup, Lulu's cost endpoint (needs a full French address for VAT). For Merci Facteur, open an account and read the PRO price list that API orders are billed at. Store the raw responses with a date.
3. **Send one written request for quotation** to Exaprint, Onlineprinters, Flyeralarm, one EA and one SCOP. Text to send:

   > Demande de devis — carte « jaquette » à deux rainages.
   > Format ouvert 230 × 178 mm (format fermé 110 × 178 mm, dos 10 mm). Deux rainages parallèles à 110 mm et 120 mm du bord gauche, **livré à plat, rainé non plié**. Sens des fibres parallèle aux rainages.
   > Impression quadri recto verso (4/4), fonds perdus 3 mm, PDF/X-4 fourni.
   > Papier : carton 300 g/m² (variante 250 g/m²), de préférence 100 % recyclé non couché ou couché mat ; **sans pelliculage** (variante : pelliculage mat recto).
   > Quantités : 50, 100, 250, 500, 1 000 exemplaires, un seul modèle. Variante : données variables (un code et un QR code uniques par exemplaire, fichier PDF multipage fourni).
   > Variante format A5 : ouvert 306 × 210 mm, rainages à 148 et 158 mm.
   > Livraison en un point en France métropolitaine. Merci d'indiquer : prix HT par quantité, frais de livraison, délai, site de production, et de joindre : attestation Imprim'Vert de l'année, numéro de licence FSC ou PEFC, licence Blauer Engel / Écolabel européen le cas échéant, certificat ISO 14001 ou enregistrement EMAS.

4. **Check registers before any public statement**: the Imprim'Vert directory, the FSC and PEFC certificate searches, the Blauer Engel product search, the EU EMAS register, the EU Ecolabel catalogue, the B Corp directory, and for inclusive suppliers UNEA and the State's "Marché de l'inclusion" [register names from memory].
5. **Send a data questionnaire** to any supplier that will receive addresses: legal entity and owner, hosting provider and country, list of sub-processors, retention period for artwork and addresses, deletion on request, incident contact.
6. **Order and weigh samples** before fixing a paper weight: the 20 g band decides the unit economics.

## 9. Sources

No source was retrieved by the lead analyst. Every link below appeared in a researcher's search results during this workflow run on 2026-11-12.

Ownership and sites
- https://www.nasdaq.com/press-release/cimpress-agrees-purchase-saxoprint-and-viaprinto-cewe-2026-05-11
- https://www.tipranks.com/news/company-announcements/cimpress-completes-acquisition-of-saxoprint-and-viaprinto
- https://finance.yahoo.com/markets/stocks/articles/cewe-completes-sale-commercial-online-213408958.html
- https://www.printweek.com/content/news/cimpress-to-acquire-saxoprint-and-viaprinto
- https://www.cewe-group.com/en/about-us/corporate-group/sites/dresden-site-saxoprint.html
- https://www.heidelberg.com/global/media/global_media/company___press_lounge/press_kits_1/2022_15/saxoprint/presentation/SAXOPRINT_Corporate_presentation-2022.pdf
- https://en.wikipedia.org/wiki/Pixartprinting
- https://www.worldofprint.com/?p=2364436
- https://pixartprinting.co.uk/sales-terms-conditions
- https://piworld.com/article/cimpress-acquires-french-web-print-company-exagroup
- https://www.lejournaldesentreprises.com/article/limprimeur-exaprint-investit-dans-la-qualite-de-vie-au-travail-2119064
- https://www.lejournaldesentreprises.com/occitanie/breve/limprimeur-en-ligne-exaprint-investit-15-millions-deuros-2053466
- https://lareclame.fr/exaprint-factory-cartes-visite-232104
- https://businesscoot.com/en/company/exaprint
- https://whattheythink.com/news/76915-cimpress-agrees-acquire-leading-german-web-print-company-wirmachendruck
- https://www.deutsche-startups.de/2015/12/21/cimpress-kauft-wirmachendruck/
- https://www.fastlancer.org/fastlancer-blog/wirmachendruck-review/
- https://onlineprinters.ie/c/about-us/the-company/overview
- https://www.onlineprinters.ie/c/about-us/press-info-and-pr/press-releases/onlineprinters-group-bundles-international-activities-and-pushes-strategic-realignment
- https://www.lifeverde.de/nachhaltigkeitsmagazin/news-tipps/trendentwicklung-co2-reduziertes-drucken
- https://www.druckmarkt.com/archiv/pdf/125/41_Flyeralarm_CO2.pdf
- https://www.aleou.fr/xg/contact/supplier/58697
- https://www.printoclock.com//media/file/1e/1c/08fddf0a58413472f4b21d371925.pdf
- https://mixam.de/?locale=en_DE
- https://www.moo.com/blog/uk/inside-moo/made-in-the-uk
- https://www.moo.com/eu/about/moo-promise
- https://www.moo.com/uk/about/free-delivery-EU
- https://en.wikipedia.org/wiki/Printful,_Inc

Web-to-print products and prices
- https://www.onlineprinters.fr/p/cartes-pliables-format-paysage-a6-rainage-sur-le-cote-long
- https://www.onlineprinters.fr/p/cartes-pliables-de-no%EBl-format-paysage-a6-rainage-sur-le-cote-long
- https://www.onlineprinters.fr/p/cartes-pliables-en-papier-eco-naturel-paysage-a6
- https://www.onlineprinters.fr/p/cartes-postales-a6
- https://www.flyeralarm.com/fr/p/cartes-de-fidelite-525038
- https://comparatif-imprimerie.fr/avis/flyeralarm/
- https://comparatif-imprimerie.fr/avis/exaprint/
- https://www.pixartprinting.com/digital-litho-printing/customized-invitation-cards-postcards/invitations/
- https://www.pixartprinting.fr/essai-gratuit/250-invitations-10x15/
- https://manuscry.com/manuscry-ou-pixartprinting
- https://www.exaprint.fr/impression/carte-postale-2-3-volets-finitions-speciales
- https://www.exaprint.fr/la-carterie/carte-message
- https://www.exaprint.fr/la-carterie/carte-postale/double/
- https://www.exaprint.fr/Carterie/2-ou-3-volets-classique/Carte-de-visite-32-a-92-cm2-surface-a-plat/350g-couche-demi-mat/Non-pellicule/1-rainage-central
- https://www.helloprint.com/fr-fr/cartesdevoeux
- https://www.helloprint.com/fr-fr/spectruma6carnet
- https://www.vistaprint.fr/fetes/cartes-de-voeux
- https://lessentieldeleco.fr/8883-quel-est-le-meilleur-site-de-carte-de-voeux-2027
- https://www.printoclock.com/cartes-de-voeux
- https://monpremierclient.fr/impression/avis-printoclock/
- https://www.realisaprint.com/carte-correspondance-personnalisee-r2284.html
- https://www.rapid-flyer.com/p/carte-de-voeux-double
- https://www.eoz.eu/72-carte-de-voeux-2-volets
- https://www.printshot.fr/1185-flyer-2-volets
- https://www.printenbind.nl/en/card/gevouwen-kaart
- https://www.printenbind.nl/en/card/greeting-card
- https://www.printenbind.nl/en/order-api
- https://www.printenbind.nl/en/book/softcover-book
- https://www.printapi.nl/drukwerk/softcover-boeken
- https://www.printful.com/custom/collections/made-in-eu/greeting-card

Folders with a spine (sibling note `print-requirements.folding.md`)
- https://www.helloprint.com/fr-be/chemisesrabats
- https://www.helloprint.com/en-gb/folders
- https://www.flyeralarm.com/de/p/mappen-2-teilig-mit-laschen-4390809.html
- https://www.viaprinto.at/praesentationsmappen
- https://www.druckdiscount24.de/angebotsmappen
- https://pinguindruck.de/load/dateien/mappe_09-12_4s2ka6_datenblatt.pdf
- https://imprimeurduroi.com/fr/impression-chemise-double-rainage/126-a4-double-rainage-5mm.html
- https://imprimeriedunet.fr/601-chemise-a-rabats-standard.html

POD networks and print APIs
- https://www-api.gelato.com/en-US/products
- https://apisupport.gelato.com/hc/en-us/articles/360010095259-What-is-a-product-UID-
- https://support.gelato.com/en/articles/8996362-what-product-formats-and-dimensions-do-you-support
- https://dashboard.gelato.com/docs/products/product/get/
- https://dashboard.gelato.com/docs/orders/v3/create/
- https://gelato.com/legal/data-processing-terms
- https://gelato.com/legal/data-processing-agreement-for-gelatoconnect
- https://www.gelato.com/es/trust
- https://mega.management/gelato-print-on-demand/
- https://www.stylefactoryproductions.com/blog/gelato-review
- https://www.syncost.com/blogs/gelato-print-on-demand-review
- https://ecommerce-platforms.com/articles/gelato-print-on-demand-pricing
- https://merchize.com/top-etsy-print-on-demand-cards-companies/
- https://merchtitans.com/blog/print-on-demand-europe-guide
- https://docs.cloudprinter.com/client/how-to-check-prices
- https://www.cloudprinter.com/countries/local-printing-in-germany-with-global-print-api
- https://apps.shopify.com/cloudprinter-com
- https://github.com/cloudprintercom/cloudapps-php-sdk
- https://www.prodigi.com/products/cards-and-stationery/greetings-cards/
- https://www.prodigi.com/products/cards-and-stationery/greetings-cards/fine-art-greetings-cards/
- https://www.prodigi.com/print-api/docs/reference/
- https://podvector.ai/articles/printful/shipping/printful-global-shipping-times-costs-and-what-to-expect
- https://developers.printful.com/docs/
- https://printify.com/custom-greeting-card-printing/
- https://developers.printify.com/
- https://www.peecho.com/solutions/print-api
- https://www.peecho.com/products
- https://www.blurb.com/print-api-software
- https://www.einpresswire.com/article/955606544/blurb-launches-instant-store-a-fast-way-to-sell-direct-to-consumer-maximize-earnings
- https://mixam.de/print-on-demand
- https://mixam.com/support/printlink-guide.pdf

Book POD
- https://help.api.lulu.com/en/support/solutions/articles/64000254631-are-there-fees-to-use-lulu-s-print-api-
- https://api.lulu.com/docs/?_gl=1
- https://blog.chapter.pub/lulu-pricing/
- https://www.bod.fr/prix
- https://www.bod.fr/mybod/livraison-et-paiement.html
- https://www.bod.fr/editeurs
- https://www.mecanismes-dhistoires.fr/imprimer-son-livre-a-la-demande-avec-bod-avis-complet/
- https://www.authormedia.com/printing-the-future-print-on-demand-with-offset-quality-by-bookvault/
- https://karldrinkwater.substack.com/p/weekly-writers-print-on-demand-cost
- https://selfpublishingdirectory.com/print-on-demand/bookvault

Postal APIs and card services
- https://www.merci-facteur.com/tarifs.php
- https://www.merci-facteur.com/pro/lp/api/api-carte-postale.php
- https://www.merci-facteur.com/cartes/r5-fetes-de-fin-d-annee/r6-cartes-de-voeux/r41-carte-voeux-professionnelle/c227-voeux-entreprise-batiment/3822-harmonie-verte-pour-une-ann-eacute-e-prosp-egrave-re/
- https://github.com/MerciFacteur/Merci-facteur-API
- https://github.com/MerciFacteur/Merci-facteur-API/blob/master/README.md
- https://manuscry.com/manuscry-ou-merci-facteur
- https://www.maileva.com/nos-solutions/courrier-en-ligne/
- https://www.maileva.com/app/uploads/2024/06/Plaquette_plateforme_envoi_courrier_Maileva.pdf
- https://www.index-education.com/fr/service-maileva-edt.php
- https://postal-api.com/tarifs
- https://www.pingen.com/en/prices/
- https://www.pingen.fr/en/price-list/
- https://www.pingen.nl/en/post-api/
- https://help.pingen.com/en/submit-and-send-letters/introduction-delivery-products
- https://www.stannp.com/uk
- https://www.stannp.com/uk/direct-mail-api/postcards?lang=php
- https://help.lob.com/print-and-mail/building-a-mail-strategy/international-mail
- https://www.popcarte.com/envoi-direct-destinataire
- https://www.popcarte.com/votre-photo-seule/importez-votre-graphisme-personalise
- https://www.fizzer.com/fr/carte-postale-en-ligne/
- https://www.pvt.fr/code-promo/fizzer-avis-test-carte-postale-code-promo
- https://support.mypostcard.com/en/support/solutions/articles/75000054648-how-much-does-mypostcard-cost-
- https://omr.com/en/reviews/product/mypostcard-print-api

Trade-printer APIs
- https://flyeralarm.pro/
- https://flyeralarm.pro/wp-content/uploads/media/2025-02-10_Client-API-Documentation_Reseller-Migration_v3.pdf
- https://wiki.beprint.app/pl/add-ons/reseller-api-dropshipping/informationen-dienstleister-flyeralarm-pro.html
- https://www.saxoprint.de/
- https://m.saxoprint.at/faq
- https://dropshipping.key2print.com/how-to-start-with-api/
- https://www.onlineprinters.de/c/lp/op-connect
- https://www.helloprint.com/en-us/reseller-solutions
- https://www.printweek.com/content/news/helloprint-adds-new-api
- https://www.print.com/en/api/
- https://www.printapi.io/
- https://www.tradeprint.co.uk/tradeprint-api

Postage (sibling note `market-business.competitors-costs.md`)
- https://www.laposte.fr/tarifs-postaux-courrier-lettres-timbres
- https://www.laposte.fr/tarif-lettre-verte
- https://www.laposte.fr/tarif-lettre-suivie
- https://www.laposte.fr/tarifs-postaux-etranger
- https://www.toutsurmesfinances.com/argent/a/prix-du-timbre
- https://lessentieldeleco.fr/8883-quel-est-le-meilleur-site-de-carte-de-voeux-2027/

ESG
- https://www.climatepartner.com/de/klimaneutral-drucken
- https://www.onlineprinters.ie/c/about-us/certificates-and-awards/environment-and-sustainability
- https://www.pr.com/press-release/555928
- https://www.pressebox.com/pressrelease/onlineprinters-gmbh/ImprimVert-Environmental-certification-renewed-for-the-sixth-time/boxid/1110757
- https://de.onlineprinters.ch/$WS/diedruckerei/shopdata/web/images/landingpages/unternehmen/was-uns-wichtig-ist/op_esg-report-2022_en.pdf
- https://www.flyeralarm.com/images/upload/content/_pdf/de/de/FA_Infoblatt_KlimaneutralesDrucken_12122012_WEB.pdf
- https://www.dieumweltdruckerei.de/assets/content_files/10580/Nachhaltigkeitsbericht_02_23_dieUmweltDruckerei.pdf
- https://www.unea.fr/sites/default/files/doc_ea_4657_copiver_express_.pdf

## Implications for the implementation

1. Offer fulfilment in this fixed default order: (a) print at home, (b) one card printed and mailed to the recipient, (c) batch order of one title. Home print is always available and free; (b) and (c) are disabled until a provider is configured.
2. Put every printer behind one provider interface that declares capabilities: `single_fold_card`, `double_crease_jacket`, `folder_with_spine`, `book_cover_with_spine`, `plain_letter`. Each provider also declares: minimum quantity, quantity tiers, accepted formats, bleed, crease-mark convention, delivery mode (letter to recipient or parcel to buyer), production country, hosting country, DPA reference.
3. Ship a **fake print provider** first. It must cover both flows: a tiered batch quote (HT and TTC by quantity) and a single mailed order with the states accepted, printed, posted, delivered, failed, plus a weight-band calculation. Real adapters come in this order: Merci Facteur, then Gelato. Book POD stays behind a feature flag that is off by default.
4. Generate these layouts from the same card data: `home-a4` (A4 landscape, spread 230 × 178 mm at 100 %, fold ticks outside the trim, 100 mm calibration ruler); `jacket-poche` (trim 230 × 178 mm, creases at 110 and 120 mm from the left trim edge, 3 mm bleed, so 236 × 184 mm); `jacket-a5` (trim 306 × 210 mm, creases at 148 and 158 mm); `card-a5-single-fold` (open 300 × 210 mm, one fold at 150 mm, for Merci Facteur); `card-a6-single-fold` (open 210 × 148 mm, for catalogue web-to-print).
5. On single-fold layouts, print the 10 mm spine band on the front panel immediately beside the fold (150–160 mm on the A5 layout) and fit the cover, uncropped, in the remaining 140 × 210 mm. Print tick marks outside the trim for an optional second, hand-made fold at 160 mm, which turns the card into a jacket with a real spine. This is a design inference to test on a real sample.
6. Export per-provider PDFs: 3 mm bleed by default; no crease or fold line inside the trim for card APIs; artwork supplied unfolded with bleed for Gelato; batch files marked "livré à plat, rainé non plié". The crease-line convention (outside ticks, spot-colour layer, or separate view file) is a provider setting.
7. Compute the mass of every mailed item: area × grammage + envelope (8 g default for C5). Warn when the total exceeds 19 g and refuse the "20 g" postage price when it exceeds 20 g. Default board for mailed poche jackets: 250 g/m² (10.2 g + 8 g = 18.2 g); at 300 g/m² the same jacket reaches 20.3 g and moves to the 100 g band. Default for batches: 300 g/m². A closed poche jacket (120 × 178 mm) fits a C5 envelope (162 × 229 mm).
8. Store postage as dated data, not constants: Lettre verte ≤ 20 g €1.52 (2026) and €1.67 (from 2027-01-01); ≤ 100 g €3.10 and €3.34; tracked €2.02 / €3.60 and €2.17 / €3.84. Select the row by order date.
9. Keep all supplier prices in one data file. Each entry has: provider, variant, quantity tier, amount, currency, HT or TTC or unknown, as-of date, evidence class (Q, L, T, E) and source URL. The UI labels anything that is not class Q as "estimate" and never shows a class E figure as a price. Entries older than 90 days are flagged stale.
10. Seed that file with the low / base / high values already used by `12-market-and-business-inputs.md`: batch single-fold card €0.36 / 0.48 / 0.60 per card at 100; €0.50 / 1.30 / 2.10 at 25–50; €0.07 / 0.10 / 0.13 at 1,000 (class E); true jacket €0.60 / 1.00 / 1.50 at 100–500 (class E, assumption); Merci Facteur folded card €2.99, postage €1.30 (floor) to €3.34 (ceiling), tracking €0.50, plan €19.95 per month and €39.95 above 2,000 cards a month; dummy book €9 / 10.50 / 12 (class E).
11. Quantity routing: 1–24 identical copies go to home print or the single-card provider; batch providers are offered from 25, with 50 as the default minimum for jackets and folders. Offer tiers 25, 50, 100, 250, 500, 1,000.
12. Redemption codes printed by a third party are created **inactive**. Mailed singles: by default the code activates when the provider reports "posted", so that the card works on arrival; the buyer is notified at redemption; a strict option keeps the code inactive until the buyer activates it. Batch codes activate when the seller scans or enters them. Codes are single-use, and the buyer or seller can revoke and re-issue a code at any time. (Design choice derived from the bearer-secret risk; to be confirmed against the redemption-UX document.)
13. Support two batch modes: variable data (a multi-page PDF with one unique code per copy, plus a CSV manifest of code identifiers, never the secrets in clear outside the PDF) and fixed artwork (one shared code with a redemption cap set by the seller).
14. Send a print provider only the PDF and the delivery name and address. Never send the buyer's e-mail, the gift message outside the PDF, or order metadata. Delete the recipient's address 30 days after the "delivered" or "posted" status; keep only provider, production country, timestamps and cost.
15. Refuse any order to a print provider unless a rights basis is recorded for the cover (reseller licence reference or rights-holder declaration). Home print never needs one.
16. Keep a supplier registry with these fields: legal entity, seat country, ultimate owner, production country for French orders, hosting country, DPA URL, sub-processor list date, and an exposure value among `none_known`, `us_listed_group`, `us_parent`, `us_cloud_host`, `unknown`. Unknown is the default and is displayed as unknown.
17. Provide a strict setting `eu_only` (default on for mailed singles) that hides any provider not meeting all three: controlled from the EU/EEA, EU hosting stated in writing, DPA signed. On current evidence no real provider passes: Maileva meets the first two conditions, Merci Facteur only the first as far as is known. Each passes once its signed DPA and hosting answer are recorded in the registry.
18. Environmental and social statements shown to users are built only from registry fields that carry a licence number and a verification date less than 12 months old, using the pattern "Imprimé à {ville} par {imprimeur} — {label} n° {licence}". A test fails the build if translation files contain "neutre en carbone", "climate neutral", "klimaneutral", "éco-responsable", "eco-friendly", "green printing" or a tree-planting promise.
19. The printer brief generated with every batch PDF states: CMYK only, no lamination, no foil, no spot UV, recycled uncoated or matt-coated board, grain parallel to the creases, delivered flat. Lamination is an explicit opt-in; the generator suggests it, or a lighter spine colour, when a dark solid crosses a crease.
20. Implement the ESG grid of section 6.4 as data: twelve criteria with their maxima, a floor and a ceiling score per supplier, and the evidence link and date for every point. The admin view sorts by floor score and shows the gap.
21. Show delivery promises only from provider data: "posted" when the provider reports it; no fixed delivery date for letter post. Lead times to display as estimates: Merci Facteur same-day posting, Flyeralarm 5–7 working days, Popcarte-class services 4–5 working days.
22. Store every price HT with its VAT rate and show consumers TTC. When a supplier's VAT status is unknown (Merci Facteur today), assume the published figure is HT, which is the more expensive reading, until an invoice shows otherwise.
23. Raise an admin alert when mailed singles exceed 300 a month for two consecutive months (time to request a direct mailing quote from an EA or mailing house) and when they exceed 2,000 a month (Merci Facteur plan step).
24. For the book variant, compute spine width from page count with the provider's own formula (Lulu: pages / 444 + 0.06 in) and target 148–175 pages for 10 mm; warn that the object costs about €9–12 delivered and uses about 150 blank pages.
