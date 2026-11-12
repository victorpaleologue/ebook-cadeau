# Printers for the eBook Cadeau jacket card — environmental and social (ESG) criteria, ranking, EU seat and hosting

Angle: environment and society. Research date: 2026-11-12. Market: France first, then EU.
Companion notes: `printers.web2print.md` (catalogue printers, prices) and `printers.print-api.md` (POD and postal APIs).

## 0. Method and hard limitations (read first)

- **No web search succeeded in this session.** The first three queries (Imprim'Vert criteria, Print'Ethic register, Directive 2024/825) were refused: the shared WebSearch budget (200 calls per turn across all agents) was already spent. WebFetch is blocked by the network allowlist. The 25+ search target was not met; the count is zero.
- Consequently **almost everything below is [M]: recalled from memory (knowledge cutoff mid-2026), not verified today.** It is a structured brief and a verification plan, not verified research. Do not reuse any company-specific statement in public copy before checking it.
- The only linked sources are URLs that appeared in the two sibling notes' search results in this same workflow run. Tags on those: **[V2]** / **[V1]** = verified by a sibling agent in two / one source, carried over, not re-checked by me. **[M]** = memory. **[M-low]** = memory I am not confident in. **[est.]** = my arithmetic.
- No URL was invented. Registers are named, not linked, because I could not confirm their addresses today.
- Section 10 lists the searches to run when the budget is available again.

## 1. Key findings

1. **Labels are not interchangeable.** Only Type I ecolabels on the printed product (Blauer Engel DE-UZ 195, EU Ecolabel, Nordic Swan, Austrian Umweltzeichen UZ 24) and Cradle to Cradle Certified cover paper, inks, emissions and recyclability together. FSC/PEFC cover fibre origin only. Imprim'Vert covers hazardous-waste and chemical housekeeping at the site only. ISO 14001 certifies a management process, EMAS adds a verified public statement [M].
2. **Since 27 September 2026 the Empowering Consumers Directive (EU) 2024/825 applies** [M for the date]: offset-based "climate neutral" product claims and unsubstantiated generic claims ("éco-responsable", "green") towards consumers are blacklisted. ClimatePartner itself warns printers about "klimaneutral drucken" ([ClimatePartner](https://www.climatepartner.com/de/klimaneutral-drucken)) [V1]. Offset schemes (ClimatePartner, PrintReleaf, tree planting) therefore score zero in the grid below, and eBook Cadeau must not relay such claims.
3. **For a single card, logistics dominate the footprint, not the print shop.** A 12–15 g card is worth tens of grams CO2e; a courier parcel is worth several hundred grams [est., M]. Ranking of fulfilment modes: print at home or in store, then local print plus letter post, then one batch parcel, and last a cross-border parcel per card.
4. **Short runs (1–300 copies) belong on digital presses or gang-run offset**; a dedicated offset makeready wastes more sheets than the job contains [M].
5. **Best social profiles in France are SCOP printers and entreprises adaptées / ESAT workshops**; none is an upload-and-pay web shop at quantity 1. All names are [M] except one UNEA-listed shop.
6. **Best documented mainstream options:** Onlineprinters (ESG report, Imprim'Vert, Blue Angel/FSC/PEFC papers) [V1] and Exaprint (in-house plant at Mauguio, France) [V2]. Both have financial owners (Bregal; Cimpress).
7. **Ownership is concentrated in Cimpress plc** (Vistaprint, Pixartprinting, Exaprint, Easyflyer, WIRmachenDRUCK, and since 2 July 2026 SAXOPRINT and viaprinto) ([Nasdaq](https://www.nasdaq.com/press-release/cimpress-agrees-purchase-saxoprint-and-viaprinto-cewe-2026-05-11), [TipRanks](https://www.tipranks.com/news/company-announcements/cimpress-completes-acquisition-of-saxoprint-and-viaprinto)) [V2]. Cimpress is an Irish plc listed on Nasdaq with large US operations and, from memory, AWS infrastructure: the group with the highest third-country exposure among EU-seated candidates.
8. **Hosting location was not found for any printer** except Gelato (AWS) and Maileva (France) [V1, sibling]. This must be asked in writing (DPA, sub-processor list).

## 2. Labels: what each one actually guarantees

All rows [M] unless tagged.

| Label | Object certified | Guarantees | Does not guarantee | Check in | Weight for us |
|---|---|---|---|---|---|
| **FSC** (Mix, Recycled, 100 %) | Fibre chain of custody; the printer needs its own CoC licence to print the logo | Fibre from certified forests, recycled material or "controlled wood"; FSC Mix needs about 70 % certified or recycled input (percentage or credit system) | Printing process, inks, energy, carbon. Paper bought FSC but printed by an uncertified printer cannot carry the logo | FSC public certificate search (licence code FSC-C…) | Medium |
| **PEFC** | Same, umbrella of national schemes; dominant in French forests | About 70 % certified fibre for the label | As FSC. NGOs rate it as less demanding; a 2017 France 2 "Cash Investigation" programme ridiculed French PEFC forest-level controls [M-low] | PEFC certificate search | Medium-low |
| **EU Ecolabel, printed paper products** (Decision (EU) 2020/1803, replacing 2012/481/EU) | The printed product | Substrate is EU Ecolabel paper; restricted substances in inks, toners, varnishes, glues; recyclability and de-inkability; VOC limits; waste and energy management | Carbon footprint; social aspects. Very few licence holders in print | EU Ecolabel catalogue (ECAT) | High, rare |
| **Blauer Engel DE-UZ 195 "Druckerzeugnisse"** (RAL gGmbH, criteria by Umweltbundesamt) | The printed product | Blue Angel recycled paper; mineral-oil-free and restricted inks; VOC limits; de-inkability; energy management at the printer; limits on film lamination | Social aspects; transport | Blauer Engel product search | High — the most demanding print label reachable from France |
| **Nordic Swan, printing companies** | The printing company and its products | Points system on paper, chemicals, VOC, energy, paper waste | Not available at French printers | Nordic Ecolabel licence search | High in the Nordics (LaserTryk, Onlineprinters group, DK) [M-low] |
| **Österreichisches Umweltzeichen UZ 24** | Printed product | Close to Blue Angel | — | Umweltzeichen register | High, AT only |
| **Cradle to Cradle Certified** (gugler*, AT; licensed "PurePrint" partners such as Lokay DE, KLS PurePrint DK, Vögeli CH) | A defined paper + ink + coating system | Material health of every ingredient, circularity, renewable energy, water stewardship, **social fairness**; levels Bronze to Platinum | Only the certified material combinations; small supplier base; gugler*'s current corporate status unverified | C2C certified products registry | Highest scope, niche |
| **Imprim'Vert** (FR, managed by P2i) | The production site | Five criteria: compliant disposal of hazardous waste; secured storage of hazardous liquids; no products labelled toxic/CMR; staff and customer awareness; monitoring of site energy use. Annual renewal | Paper origin, carbon, inks' composition beyond CMR, social. Held by well over a thousand French sites and by some foreign ones, e.g. Onlineprinters ([Pressebox](https://www.pressebox.com/pressrelease/onlineprinters-gmbh/ImprimVert-Environmental-certification-renewed-for-the-sixth-time/boxid/1110757)) [V1] | Imprim'Vert directory | Baseline (hygiene factor), not a differentiator |
| **Print'Ethic** (UNIIC, FR) | The company's CSR approach, ISO 26000-based, assessed by a labelling committee | Structured CSR practice across governance, social, environment, fair practices, territory [M-low on mechanics] | Product properties; tens of holders only | UNIIC / Print'Ethic list | Medium, the only French sector CSR label |
| **ISO 14001** | Environmental management system | Audited process, legal-compliance commitment, continual improvement | Any performance level; no public reporting | Certificate from an accredited body | Medium |
| **EMAS** (Regulation (EC) 1221/2009) | Site | ISO 14001 plus a public environmental statement with core indicators validated by an accredited verifier; legal compliance checked | Product properties | EU EMAS register | High as management evidence; common in DE/AT, rare in FR |
| **ISO 50001** | Energy management | Audited energy monitoring and targets | — | Certificate | Low-medium |
| **ClimatePartner / "klimaneutral"** | An order or a company | Footprint calculated, offsets bought; since 2023 the label is "ClimatePartner certified" with reduction requirements [M] | Reduction of the product's own emissions. Claim now legally fragile [V1] | ClimatePartner ID tracking | **Zero** |
| **PrintReleaf** | Paper volume | Trees planted in proportion to paper used, plantings audited | Lower emissions; it is compensation | PrintReleaf certificate | Zero as a climate claim; fine as a plain fact |
| **B Corp** | Whole company | Scored impact assessment (governance, workers, community, environment, customers), re-certified every three years | Product properties; standards were overhauled in 2025 [M-low] | B Corp directory | Medium-high (social and governance) |
| **EcoVadis** | Company questionnaire rating | Percentile medal on environment, labour, ethics, procurement | Scorecards are private unless shared; medal thresholds change yearly | Ask for the scorecard | Medium, only with the scorecard in hand |
| **Gemeinwohl-Bilanz** (DE/AT) | Whole company | Externally audited common-good balance sheet | — | GWÖ register | Medium |

Process attributes (not labels) [M]:

- **Waterless offset** (silicone plates, no dampening solution): no isopropyl alcohol, lower VOC, less start-up waste. Rare; best-known practitioner is Seacourt (UK, outside the EU).
- **Vegetable-oil inks**: standard in sheetfed offset. In France, loi AGEC art. 112 bans mineral oils in printed matter for the public since 1 January 2025 (thresholds set by an arrêté of 2022), so "encres végétales" is a legal baseline, not a merit. Pigments remain petrochemical.
- **Digital print and de-inking**: dry toner de-inks well; HP Indigo liquid toner and some water-based inkjet historically score poorly on INGEDE Method 11; UV-cured inks and film lamination ("pelliculage") also hinder recycling.
- **Recycled board**: 100 % recycled 250–350 g/m² stocks with Blue Angel or FSC Recycled exist (Flyeralarm lists a recycled 300 g stock ([product page](https://www.flyeralarm.com/fr/p/cartes-de-fidelite-525038)) [V1]; Onlineprinters lists Blue Angel papers ([environment page](https://www.onlineprinters.ie/c/about-us/certificates-and-awards/environment-and-sustainability)) [V1]).

## 3. What may legally be claimed (France, EU, Germany)

All [M]; a lawyer must confirm before launch.

- **Directive (EU) 2024/825 ("EmpCo")**, amending the Unfair Commercial Practices Directive: transposition due 27 March 2026, **application from 27 September 2026**. New blacklist items: generic environmental claims without "recognised excellent environmental performance" (EU Ecolabel, an official ISO 14024 Type I label, or top performance under EU law); claims about a whole product when only one aspect is concerned; **claims that a product is climate-neutral, reduced or positive based on offsetting**; sustainability labels not backed by a certification scheme or a public authority; presenting legal requirements as a distinctive feature. Future-performance claims need a public, independently monitored plan. Scope is business-to-consumer.
- Consequence: "éco-responsable" or "green printing" wording is only defensible with an EU Ecolabel or Blue Angel product. FSC/PEFC are certification schemes and may be shown as such. Whether Imprim'Vert (awarded by committees, not an accredited certifier) meets the directive's "certification scheme" definition is an open question [my analysis, unverified]. "Encres sans huiles minérales" in France would present a legal requirement as a merit.
- **France**: loi Climat et Résilience (Code de l'environnement art. L. 229-68, décret 2022-539) already restricted "neutre en carbone" advertising from 1 January 2023 (published life-cycle balance, reduction path, offset details). Loi AGEC art. 13 (art. L. 541-9-1) forbids "biodégradable", "respectueux de l'environnement" and equivalent wording on products and packaging.
- **Germany**: Bundesgerichtshof, 27 June 2024, I ZR 98/23 ("klimaneutral", Katjes): the advertisement itself must say whether neutrality comes from reduction or compensation.
- **Green Claims Directive** (proposal COM(2023) 166): the Commission announced its intention to withdraw it in June 2025; not adopted as far as I know.
- Safe pattern for our pages: specific, checkable facts only, e.g. "imprimé à [ville] par [imprimeur], licence FSC-C…, papier 100 % recyclé".

## 4. What drives the footprint of one jacket card

Order of magnitude only [est., built on M]. Card: 110 × 178 mm closed, 10 mm spine, open 230 × 178 mm, 300 g/m² = about 12 g finished, about 15 g of board consumed with trim.

| Item | Rough CO2e | Basis |
|---|---|---|
| Board, 15 g | 10–20 g | ADEME Base Empreinte average for paper is about 0.9 kg CO2e/kg [M]; recycled Blue Angel grades are lower |
| Digital printing, both sides | 5–30 g | Energy, toner/ink, consumables; no plates |
| Dedicated offset makeready for a 50-card job | Higher than the product itself | 100–250 waste sheets plus aluminium plates [M]; gang-run printing ("amalgame", "Sammeldruck") shares this across jobs |
| Envelope plus letter post in France | Tens of grams | Postal round already exists; no parcel |
| Courier parcel within Europe, one card | 300–1,000 g | Published carrier and ADEME averages per parcel [M-low] |
| Batch of 100 cards in one parcel | 3–10 g per card | Same parcel divided by 100 |
| Home inkjet, one A4 sheet of 160–200 g paper | 10–30 g | No transport |

Reading:

- For single cards the ranking is set by logistics: **home or in-store print < French print + letter post < batch parcel to the seller < cross-border parcel per card**.
- For batches the printer's paper and process matter; transport becomes minor, so a Blue Angel printer in Germany is defensible for a French batch.
- Offset versus digital crossover is typically several hundred sheets [M]; our volumes per title sit below it.
- Design choices that keep the card recyclable and label-compatible: no film lamination, no spot UV or foil; dispersion varnish at most; matt uncoated or lightly coated recycled board (also better for QR scanning).
- A "dummy book" with about 150 blank pages (see `printers.print-api.md` §5) multiplies paper mass by roughly 15 and is the worst option unless the pages are used.

## 5. Social and governance criteria

### 5.1 Ownership

| Group | Brands relevant to us | Owner / legal form | Tag |
|---|---|---|---|
| Cimpress plc (Dundalk, IE; Nasdaq) | Vistaprint, Pixartprinting, Exaprint, Easyflyer, WIRmachenDRUCK, Printdeal, druck.at, Tradeprint; SAXOPRINT and viaprinto since 2026-07-02 | Listed, leveraged, institutional shareholders | 2026 deal [V2]; Exaprint 2015 ([PIWorld](https://piworld.com/article/cimpress-acquires-french-web-print-company-exagroup)) [V1]; rest [M] |
| Onlineprinters group (Fürth, DE) | Onlineprinters, LaserTryk (DK), Solopress (UK), Copysell (ES) | Private equity: Bregal Unternehmerkapital since 2016 | Sites [V1]; owner [M] |
| Flyeralarm (Würzburg, DE) | Flyeralarm | Founder-owned GmbH | [M] |
| Unitedprint SE (Radebeul, DE) | print24, Easyprint | Founder-owned | [M-low] |
| Helloprint (Rotterdam, NL) | Marketplace over 300+ partner printers ([reseller page](https://www.helloprint.com/en-us/reseller-solutions)) [V1] | Venture/PE-backed | [M-low] |
| Print.com (NL) | Print.com | Independent, started by Drukwerkdeal's founder after selling Drukwerkdeal (now Printdeal) to Cimpress. **This contradicts the sibling note, which lists Print.com as Cimpress-owned** | [M], resolve |
| CEWE (Oldenburg, DE; listed, family anchor shareholder) | Left commercial online print in 2026; keeps photo products (Cheerz, France) | Listed | Exit [V2]; rest [M] |
| Gelato (Oslo, NO) | Partner network in 32 countries [V2, sibling] | Venture-backed AS | [M] |
| Printful (US Inc., Latvian operations) | Plants in Riga and Barcelona [V2, sibling] | Private; merger with Printify announced late 2024 | [M] |
| Lulu Press (US) | Lulu Print API | Private; certified B Corp | [M] |
| La Poste group | Maileva (Docaposte) | State-owned (Caisse des Dépôts, État); "entreprise à mission" since 2021 | [M] |
| French SMEs | Merci Facteur, Popcarte, Fizzer, Printoclock, Realisaprint, Rapid-Flyer | Not established | — |

### 5.2 Inclusive and cooperative printers in France

- **SCOP** (worker cooperatives): Laballery (Clamecy, book printer with digital short runs), La Contemporaine (near Nantes), Média Graphic (Rennes), Imprimerie 34 (Toulouse) [M; the last M-low]. Quote-based.
- **Entreprise adaptée (EA)**: an ordinary employer with at least 55 % disabled employees on normal contracts at or above the SMIC. **ESAT**: medico-social workshop whose workers are users, not employees, paid a guaranteed fraction of the SMIC; rights (union, strike, health cover) were extended by the "plein emploi" law of December 2023 [M]. EA is the stronger social claim.
- Print workshops: Handiprint (EA, Normandy), ANRH (Paris), the APF Entreprises network [M]; Copiver Express is listed by UNEA ([UNEA sheet](https://www.unea.fr/sites/default/files/doc_ea_4657_copiver_express_.pdf)) [V1]. Directories: UNEA, Hosmoz (ex Réseau Gesat), and the State's "Marché de l'inclusion" [M].
- **Selling point for resellers**: a company with 20+ employees can deduct part of the labour cost of EA/ESAT purchases from its disabled-employment contribution (OETH) [M]. A Fnac-size reseller therefore has a financial reason to accept an EA printer.
- Prison print workshops exist; pay far below the SMIC makes them a contested choice [M]. Exclude.

### 5.3 Labour and controversies (recollections only — do not repeat publicly)

- Flyeralarm: German union ver.di has criticised the absence of a collective agreement and, for years, of a works council [M-low].
- Cimpress/Vistaprint: customer care offshored to Tunisia, Jamaica and the Philippines; repeated restructurings [M]. No specific abuse report recalled.
- Marketplaces and networks (Helloprint, WIRmachenDRUCK's partner printers ([WhatTheyThink](https://whattheythink.com/news/76915-cimpress-agrees-acquire-leading-german-web-print-company-wirmachendruck)) [V2], Gelato, Cloudprinter): working conditions at the plant that actually prints are not visible to the customer. Opaqueness is itself a negative score.
- Parcel delivery in France relies on subcontracted drivers and has seen disputes over undocumented workers at La Poste group parcel subsidiaries [M-low]; letter post is delivered by postal employees. This favours envelope formats.
- Exaprint invested in workplace quality ([Journal des Entreprises](https://www.lejournaldesentreprises.com/article/limprimeur-exaprint-investit-dans-la-qualite-de-vie-au-travail-2119064)) [V1].

## 6. Proposed scoring grid (100 points)

| # | Criterion | Max | Full marks for |
|---|---|---|---|
| E1 | Product-level Type I ecolabel or C2C available for our job | 15 | Blauer Engel DE-UZ 195, EU Ecolabel, Nordic Swan, UZ 24, C2C |
| E2 | Audited site system | 10 | EMAS 10; ISO 14001 6; ISO 50001 +2; Imprim'Vert alone 3 |
| E3 | Paper | 10 | 100 % recycled Blue Angel / FSC Recycled board at 250–350 g by default; FSC Mix or PEFC 4 |
| E4 | Process fit | 10 | Digital or gang-run for short runs; alcohol-free or waterless; de-inkable output; no lamination needed |
| E5 | Energy | 5 | Renewable electricity with evidence; own generation |
| E6 | Transport | 15 | Produced in the destination country and delivered by letter post or pick-up; cross-border parcel per card = 0 |
| E7 | Claims hygiene | 5 | Publishes measured data; no offset-based neutrality claim |
| S1 | Ownership and form | 5 | SCOP, foundation or employee ownership 5; family/founder 3; listed or PE 1 |
| S2 | Inclusive employment | 8 | EA 8; ESAT or insertion structure 6; documented partnership 3 |
| S3 | Labour transparency | 7 | Own plants in the EU, collective agreement, named sites; opaque partner network 0–2 |
| S4 | Third-party social rating | 5 | B Corp, Print'Ethic, GWÖ, or EcoVadis scorecard shared |
| G1 | Data and jurisdiction | 5 | EU-owned group, EU hosting, DPA and sub-processor list published |

Environment 70, social 25, data 5. Offsetting earns nothing. Self-declared claims earn half until a register entry is seen.

## 7. Candidates reachable from France — provisional tiers

Bands, not scores: evidence is too thin for numbers.

| Tier | Candidate | Use case | Evidence in favour | Missing or against |
|---|---|---|---|---|
| A (if confirmed) | French SCOP printers (Laballery, La Contemporaine, Média Graphic) | Batches 25–1,000, on quote | Worker-owned, French production, Imprim'Vert and FSC/PEFC likely [M] | No web shop; every label unverified |
| A (if confirmed) | Entreprises adaptées with digital print (Handiprint, ANRH, APF Entreprises, Copiver Express) | Batches and possibly per-order fulfilment with mailing ("routage") | Highest social score; OETH deduction for resellers [M]; Copiver Express exists in UNEA [V1] | Environmental labels and capability for a double-creased card unknown |
| A for environment | Blauer Engel / EMAS printers: Lokay (Reinheim), oeding print (Braunschweig), dieUmweltDruckerei (Hannover, online shop), gugler* (AT) | Batches shipped once to France | Strongest product label and public EMAS data [M]; dieUmweltDruckerei publishes a sustainability report ([PDF, 02/2023](https://www.dieumweltdruckerei.de/assets/content_files/10580/Nachhaltigkeitsbericht_02_23_dieUmweltDruckerei.pdf)) [V1] | German-language ordering; delivery to France unverified; weak for single cards (parcel) |
| B | Onlineprinters | Batches via onlineprinters.fr | ESG report 2022 ([PDF](https://de.onlineprinters.ch/$WS/diedruckerei/shopdata/web/images/landingpages/unternehmen/was-uns-wichtig-ist/op_esg-report-2022_en.pdf)), Imprim'Vert, FSC/PEFC/Blue Angel papers [V1] | Private-equity owner; production in Germany; "carbon-neutral option" marketing ([press release](https://www.pr.com/press-release/555928)) [V1] is the kind of claim EmpCo targets |
| B | Exaprint | Batches, trade accounts | Own plant at Mauguio, ~400 staff ([Journal des Entreprises](https://www.lejournaldesentreprises.com/occitanie/breve/limprimeur-en-ligne-exaprint-investit-15-millions-deuros-2053466), [La Réclame](https://lareclame.fr/exaprint-factory-cartes-visite-232104)) [V2]; shortest transport among the majors | Cimpress group; Imprim'Vert / FSC / PEFC from memory only |
| B | French postal-card services (Merci Facteur, Fizzer, Popcarte); Maileva for plain letters | One card to one recipient | Fizzer prints in France; Maileva prints and hosts in France; Merci Facteur and Maileva post the same day via La Poste [V1 each, sibling]; letter, not parcel | Merci Facteur's and Popcarte's print location not confirmed; no label, paper or labour evidence found for any of them |
| B- | SAXOPRINT | Batches | ISO 14001 and ISO 50001 ([2022 presentation](https://www.heidelberg.com/global/media/global_media/company___press_lounge/press_kits_1/2022_15/saxoprint/presentation/SAXOPRINT_Corporate_presentation-2022.pdf)) [V1] | Now Cimpress [V2]; pre-takeover evidence |
| C | Pixartprinting, Vistaprint, Flyeralarm, Printoclock, Realisaprint, Rapid-Flyer | Batches | Flyeralarm recycled stock [V1]; Printoclock self-declares Imprim'Vert ([brochure](https://www.printoclock.com//media/file/1e/1c/08fddf0a58413472f4b21d371925.pdf)) [V1] | Little or old evidence; Flyeralarm's only climate document found is a 2012 offset leaflet [V1] |
| C | Gelato, Cloudprinter, Helloprint, WIRmachenDRUCK | Singles or batches through networks | Local routing can cut transport; France named as a Gelato production country ([merchtitans](https://merchtitans.com/blog/print-on-demand-europe-guide)) [V1] | Printing plant unknown per order; no label can be promised |
| D | UK or US origin (MOO, Solopress, Prodigi UK items, Stannp, Lob), dummy-book POD | — | — | Customs border, long transport; ~150 blank pages per gift |

Outside the table: **print at home and in-store printing** beat every printer on transport and data exposure. Fnac stores run photo-print counters (operated with CEWE, from memory [M-low]); producing the card there at pick-up would remove shipping altogether.

## 8. Who is in the EU, precisely

| Candidate | Seat and controlling entity | Production relevant to France | Hosting / processors | Third-country exposure |
|---|---|---|---|---|
| Vistaprint | Cimpress plc, Dundalk (IE); EU sales entity in Venlo (NL) [M] | Venlo (NL) [M]; no French plant ([L'Essentiel de l'Éco](https://lessentieldeleco.fr/8883-quel-est-le-meilleur-site-de-carte-de-voeux-2027)) [V1] | AWS, from memory of a public migration [M]; customer care outside the EU [M] | High: Nasdaq-listed group with US management and subsidiaries; US cloud |
| Pixartprinting | Pixartprinting S.p.A., Quarto d'Altino (IT), Cimpress | Quarto d'Altino and Lavis (IT) ([Wikipedia](https://en.wikipedia.org/wiki/Pixartprinting)) [V1] | Not found | Indirect, via Cimpress |
| Exaprint | Exaprint SAS, Mauguio (FR), Cimpress [V2] | Mauguio (FR) [V2] | Not found; Cimpress shared platform likely [M-low] | Indirect, via Cimpress |
| WIRmachenDRUCK | GmbH, Backnang (DE), Cimpress [V2] | Own plants plus partners [V2] | Not found | Indirect |
| SAXOPRINT | GmbH, Dresden (DE), Cimpress since 2026-07-02 [V2] | Dresden ([CEWE page](https://www.cewe-group.com/en/about-us/corporate-group/sites/dresden-site-saxoprint.html)) [V1] | Not found | Indirect, new |
| Onlineprinters | GmbH, Fürth (DE); Bregal Unternehmerkapital (German PE, family capital based in Switzerland/NL) [M] | Neustadt an der Aisch (DE); group sites in PL, DK, SE, ES, UK ([overview](https://onlineprinters.ie/c/about-us/the-company/overview)) [V1] | Not found | No US parent; depends on cloud vendors |
| Flyeralarm | GmbH, Würzburg (DE), founder-owned [M] | Franconia and Saxony plants ([Lifeverde](https://www.lifeverde.de/nachhaltigkeitsmagazin/news-tipps/trendentwicklung-co2-reduziertes-drucken)) [V1] | Not found | No US parent |
| Helloprint | B.V., Rotterdam (NL) [M] | Partner printers, unknown per order | Not found | Unknown |
| Gelato | AS, Oslo (NO, EEA) [M] | Partner in the destination country where available [V1] | AWS with EU residency options; transfers under DPF or SCC ([DPA](https://gelato.com/legal/data-processing-terms), [trust page](https://www.gelato.com/es/trust)) [V1] | US cloud; address data goes to a local partner |
| Printful | US corporation; Latvian subsidiary [M] | Riga (LV), Barcelona (ES) ([Wikipedia](https://en.wikipedia.org/wiki/Printful,_Inc)) [V2] | Not found | High: US parent |
| Lulu | US corporation [M] | Routed to the nearest facility ([API docs](https://api.lulu.com/docs/?_gl=1)) [V1] | Not found | High: US parent |
| BoD | GmbH, Norderstedt (DE) [M] | Norderstedt [M] | Not found | Low |
| Maileva | Docaposte, La Poste group (FR, State-owned) [M] | France | France ([brochure](https://www.maileva.com/app/uploads/2024/06/Plaquette_plateforme_envoi_courrier_Maileva.pdf)) [V1] | Lowest |
| Merci Facteur, Popcarte, Fizzer | French companies; legal entities not retrieved | France (Fizzer [V1]; others assumed) | Not found | Unknown |
| Lokay, oeding print, dieUmweltDruckerei | German owner-managed SMEs [M] | Reinheim, Braunschweig, Hannover [M] | Not found | Low |
| MOO, Solopress, Mixam UK, Prodigi, Stannp | UK | UK ([MOO](https://www.moo.com/blog/uk/inside-moo/made-in-the-uk)) [V1] | Not found | Third country under an adequacy decision [M]; customs border |

Sovereignty notes [M]:

- The US CLOUD Act binds providers of communication and cloud services under US jurisdiction for data in their "possession, custody or control", wherever stored. Its direct lever is the **cloud host** (AWS, Azure, Google Cloud) more than the printer. A US-parented or US-listed printer group is additionally reachable through ordinary US subpoenas for data its subsidiaries control.
- Transfers to US entities certified under the EU–US Data Privacy Framework are lawful today; the General Court dismissed the Latombe challenge in September 2025, an appeal is pending. A reversal would hit Printful, Lulu and Cimpress US functions first.
- GDPR roles in reseller mode: reseller = controller, eBook Cadeau = processor, printer = sub-processor (Art. 28 contract and prior authorisation needed). Data actually sent: recipient name and address, the PDF (cover, gift message, redemption code). Send nothing else.
- Proportionate target for a gift card: EU-owned printer, EU hosting stated in the DPA, redemption code inactive until the buyer or seller activates it (see sibling notes).

## 9. Implications for eBook Cadeau

1. Offer fulfilment in this default order: print at home / in store, then French print + letter post, then batch from a certified printer. State the reason in the UI with facts, not adjectives.
2. Supply a "printer brief" PDF profile that keeps label options open: recycled uncoated board 250–300 g, no lamination, CMYK only, flat delivery with two creases.
3. Ask each shortlisted printer for: Imprim'Vert certificate of the current year, FSC/PEFC licence number, Blue Angel or EU Ecolabel licence if any, EMAS registration number or ISO certificates, EcoVadis scorecard, named production site for our job, DPA with sub-processor list and hosting country.
4. For reseller pitches, lead with an entreprise adaptée option (OETH deduction) and a SCOP option.
5. Never display "neutre en carbone", "éco-responsable" or a tree-planting promise as an environmental benefit.

## 10. Searches to run when the budget is back

1. Imprim'Vert: current criteria and number of sites; directory lookup for Exaprint, Printoclock, Realisaprint, Rapid-Flyer, Merci Facteur's printer, Fizzer, Popcarte.
2. Print'Ethic: list of labelled companies and levels.
3. Blauer Engel DE-UZ 195 holders shipping to France; EU Ecolabel printed-paper licence holders in France.
4. EMAS register entries: Lokay, oeding print, gugler*; gugler* corporate status.
5. B Corp directory: printers in FR/BE/NL/DE; Print.com, Lulu, MOO.
6. EcoVadis medals: Onlineprinters, Flyeralarm, Exaprint, Pixartprinting, CEWE.
7. Labour reporting: "Flyeralarm Betriebsrat ver.di", "Pixartprinting sciopero", "Vistaprint Tunisie conditions de travail", "Onlineprinters Stellenabbau".
8. Privacy policies: "politique de confidentialité" + hébergeur for Exaprint, Printoclock, Merci Facteur, Popcarte, Fizzer; "Datenschutzerklärung" + Auftragsverarbeiter for Onlineprinters, Flyeralarm, Saxoprint.
9. EmpCo transposition in France (text, date) and DGCCRF guidance; status of the Green Claims Directive.
10. ADEME and La Poste figures for a letter versus a parcel; Intergraf or UBA data on digital versus offset.
11. Ownership of Print.com, Helloprint, Printoclock, Realisaprint, Rapid-Flyer.
12. EA/ESAT print shops with web ordering and mailing; SCOP printers with digital short-run lines.

## Sources

No source was retrieved by this agent. The links below appeared in the sibling notes' search results on 2026-11-12 and are reused as carried-over evidence.

- https://www.climatepartner.com/de/klimaneutral-drucken
- https://www.nasdaq.com/press-release/cimpress-agrees-purchase-saxoprint-and-viaprinto-cewe-2026-05-11
- https://www.tipranks.com/news/company-announcements/cimpress-completes-acquisition-of-saxoprint-and-viaprinto
- https://piworld.com/article/cimpress-acquires-french-web-print-company-exagroup
- https://www.lejournaldesentreprises.com/article/limprimeur-exaprint-investit-dans-la-qualite-de-vie-au-travail-2119064
- https://www.lejournaldesentreprises.com/occitanie/breve/limprimeur-en-ligne-exaprint-investit-15-millions-deuros-2053466
- https://lareclame.fr/exaprint-factory-cartes-visite-232104
- https://en.wikipedia.org/wiki/Pixartprinting
- https://whattheythink.com/news/76915-cimpress-agrees-acquire-leading-german-web-print-company-wirmachendruck
- https://www.cewe-group.com/en/about-us/corporate-group/sites/dresden-site-saxoprint.html
- https://www.heidelberg.com/global/media/global_media/company___press_lounge/press_kits_1/2022_15/saxoprint/presentation/SAXOPRINT_Corporate_presentation-2022.pdf
- https://onlineprinters.ie/c/about-us/the-company/overview
- https://www.onlineprinters.ie/c/about-us/certificates-and-awards/environment-and-sustainability
- https://www.pr.com/press-release/555928
- https://www.pressebox.com/pressrelease/onlineprinters-gmbh/ImprimVert-Environmental-certification-renewed-for-the-sixth-time/boxid/1110757
- https://de.onlineprinters.ch/$WS/diedruckerei/shopdata/web/images/landingpages/unternehmen/was-uns-wichtig-ist/op_esg-report-2022_en.pdf
- https://www.lifeverde.de/nachhaltigkeitsmagazin/news-tipps/trendentwicklung-co2-reduziertes-drucken
- https://www.flyeralarm.com/fr/p/cartes-de-fidelite-525038
- https://www.printoclock.com//media/file/1e/1c/08fddf0a58413472f4b21d371925.pdf
- https://www.dieumweltdruckerei.de/assets/content_files/10580/Nachhaltigkeitsbericht_02_23_dieUmweltDruckerei.pdf
- https://www.unea.fr/sites/default/files/doc_ea_4657_copiver_express_.pdf
- https://www.helloprint.com/en-us/reseller-solutions
- https://lessentieldeleco.fr/8883-quel-est-le-meilleur-site-de-carte-de-voeux-2027
- https://merchtitans.com/blog/print-on-demand-europe-guide
- https://gelato.com/legal/data-processing-terms
- https://www.gelato.com/es/trust
- https://en.wikipedia.org/wiki/Printful,_Inc
- https://api.lulu.com/docs/?_gl=1
- https://www.maileva.com/app/uploads/2024/06/Plaquette_plateforme_envoi_courrier_Maileva.pdf
- https://www.moo.com/blog/uk/inside-moo/made-in-the-uk

## Could not verify

Everything tagged [M] or [M-low], in particular:

- **Label mechanics**: exact Imprim'Vert criteria wording, audit frequency and number of labelled sites; Print'Ethic levels, assessor and holders; current versions and expiry of DE-UZ 195 and of Decision (EU) 2020/1803; FSC Mix and PEFC percentage thresholds; whether Imprim'Vert qualifies as a "certification scheme" under Directive 2024/825.
- **Law**: application date of Directive 2024/825 and its French transposition text; status of the Green Claims Directive; AGEC mineral-oil thresholds; references of the French "neutre en carbone" rules; BGH case reference; Data Privacy Framework litigation status; whether printed products remain in scope of the EU Deforestation Regulation after its 2025 revision.
- **Every eco or social printer profile**: Lokay, oeding print, gugler* (including whether it still trades), KLS PurePrint, Vögeli, Seacourt, Laballery, La Contemporaine, Média Graphic, Imprimerie 34, Handiprint, ANRH, APF Entreprises; their labels, minimum runs, prices and delivery to France.
- **B Corp and EcoVadis status** of any printer, including Lulu, Print.com, MOO, Onlineprinters, Flyeralarm, Exaprint, Pixartprinting.
- **Labour controversies**: all items in §5.3 are recollections without a source.
- **Ownership**: Bregal (Onlineprinters), Flyeralarm, Unitedprint, Helloprint, Gelato investors, Printful–Printify merger, BoD, and the Print.com contradiction with the sibling note; owners of Merci Facteur, Popcarte, Fizzer, Printoclock, Realisaprint, Rapid-Flyer.
- **Hosting and processors** for every candidate except Gelato and Maileva; Cimpress on AWS is from memory.
- **Labels of mainstream printers**: Exaprint (Imprim'Vert, FSC, PEFC), Pixartprinting (FSC, sustainability report), SAXOPRINT after the takeover, Vistaprint, Flyeralarm (current reporting), French postal-card services.
- **All carbon figures in §4**: orders of magnitude from memory and arithmetic; no life-cycle study was consulted.
- **Fnac in-store photo printing operator** and whether its kiosks can print a creased 300 g card.
- **OETH deduction rules** for EA/ESAT purchases and the current legal status of ESAT workers.
