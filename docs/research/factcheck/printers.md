# Fact-check report — printers

_Produced by an adversarial fact-checker on 2026-11-12. Read this next to the matching document: where that document's status is still "draft", these corrections were NOT applied._

# Adversarial fact-check of docs/research/04-printers.md

## 0. Hard limitation: no fresh web verification was possible

- **Zero searches ran.** My first four WebSearch calls (Cimpress/SAXOPRINT, Merci Facteur tariffs, La Poste 2026 and 2027) were all refused: the turn's shared 200-search budget was already spent. WebFetch is blocked. I did not work around either limit.
- **Step 2 of the brief (independent re-verification with fresh queries) was therefore not done.** Nothing below is confirmed by a new source.
- What I did instead, tagged on every verdict:
  - **[MEM]** my own recollection (cutoff mid-2026), independent of the document;
  - **[CALC]** I recomputed the arithmetic;
  - **[XREF]** I cross-read the three underlying notes and two sibling notes in `/home/user/ebook-cadeau/docs/research/notes/`;
  - **[NONE]** no independent evidence.
- **No new URL is cited.** URLs below are ones already in the document or notes, so they are not independent. Legal texts are cited by identifier only.
- "CONFIRMED [MEM]" means I am confident from memory; it is weaker than a sourced confirmation.
- The document transcribes its notes faithfully (I found no transcription error) and all table sums and derived figures recompute correctly. The problems are in evidence quality, generalisations and unstated assumptions.

## 1. The 20 most load-bearing claims

| # | Claim (section) | Verdict | Corrected statement | Basis / source |
|---|---|---|---|---|
| 1 | Merci Facteur folded card lands at about €4.29 (€2.99 + postage "from €1.30"), "[V2]", "the only verified per-card route" (§1, §3.4, §7) | **UNSUPPORTED, and overstated** | The price rests on one vendor's pages plus a competitor (Manuscry). By the document's own definition that is V1, not V2. €4.29 is an unverified floor and should not be the base case; `12-market-and-business-inputs.md` uses it as both low and base. | [XREF] `printers.print-api.md` line 67 says "V2, same vendor, two pages"; [NONE] on the prices themselves |
| 2 | Same route: the card "probably falls in the 20–100 g band", so the ceiling is €6.10–6.33 (§3.4) | **PARTLY** | The weight is right: 0.30 × 0.21 m × 300 g/m² = 18.9 g before the envelope, so the mailed item is over 20 g. But €1.30 is below the public 20 g rate (€1.52), so Merci Facteur is evidently not charging retail stamps. Whether retail weight bands apply at all is unknown. The same note records Maileva postage as 0–50 g bands at €0.747–1.538 HT. Both "floor" and "ceiling" are guesses until a test invoice. | [CALC]; [XREF] `printers.print-api.md` Maileva row |
| 3 | Merci Facteur can print our layout (implied by "launch choice") | **UNSUPPORTED** | Nothing shows the API accepts custom artwork on all four panels, including a QR code inside and a designed back. My recollection is that its cards take a front visual plus a typed message inside. If so, the launch route fails. §8.1 lists "accepted artwork formats" as unverified, but §1 and §7 do not carry that caveat. | [MEM-low]; [XREF] no evidence in the notes |
| 4 | La Poste 2026: Lettre verte ≤ 20 g €1.52; ≤ 100 g €3.10 (€2.97 online); international €2.25 / €4.85 (§3.6) | **PARTLY** | €1.52 and €2.25 match my memory. The 100 g figure is doubtful: in 2023–2025 the 100 g rate was exactly twice the 20 g rate, which would give €3.04. €2.97 is also not a plausible online discount from €3.10. The sibling note flags the same anomaly. If €3.04 is right, the band jump is €1.52, not €1.58. | [MEM]; [XREF] `market-business.competitors-costs.md` line 92 |
| 5 | La Poste from 1 Jan 2027: €1.67 / €3.34 / €2.17 / €3.84 / €2.45 / €5.25 (§3.6) | **UNSUPPORTED** | Announced after my cutoff (the sibling note says 27 July 2026). Plausible in trend (+9.9 %), but not re-verified. Re-read laposte.fr before hard-coding. | [NONE]; https://www.laposte.fr/tarifs-postaux-courrier-lettres-timbres (document's own) |
| 6 | Cimpress owns SAXOPRINT and viaprinto since 2 July 2026 (§1 finding 2, §5) | **UNSUPPORTED by me; the document's evidence looks solid** | I cannot confirm (at or after my cutoff). The note cites a primary press release and three secondary reports, so keep it, but re-check. | [NONE]; https://www.nasdaq.com/press-release/cimpress-agrees-purchase-saxoprint-and-viaprinto-cewe-2026-05-11 (document's own) |
| 7 | Cimpress also owns Vistaprint, Pixartprinting, Exaprint, Easyflyer, WIRmachenDRUCK, Printdeal, druck.at, Tradeprint; Cimpress is an Irish plc on Nasdaq (§1, §6.7) | **CONFIRMED [MEM]** | Correct to my knowledge. The "[V2]" tag on the whole list is too generous: only the 2026 deal and WIRmachenDRUCK had two sources in the run. | [MEM] |
| 8 | Print.com is independent; the Cimpress brand is Printdeal (§5 contradiction) | **CONFIRMED [MEM]** | The document chose the right version. Drukwerkdeal was sold to Cimpress in 2014 and later renamed Printdeal; its founder then started Print.com. Still unsourced. | [MEM] |
| 9 | "Mass web-to-print does not sell single copies (minimums 10 to 50)" and "no catalogue sells a 10 mm double crease at quantity 1" (§1) | **PARTLY: over-generalised** | The run saw minimums at only six shops. From memory, several upload-and-print shops sell digital products from 1 copy: viaprinto (its whole model), print24/Unitedprint and WIRmachenDRUCK on many items. viaprinto also lists folders with 1/5/10 mm spines, and no minimum was captured for it. A quantity-1 folder with a real 10 mm spine may exist. The narrow claim (no book-trim double-crease card at quantity 1) stands as "not found". | [MEM]; [XREF] `print-requirements.folding.md` line 136; https://www.viaprinto.at/praesentationsmappen (document's own) |
| 10 | "100 folded, creased A6 cards: about €52 HT delivered"; "four printers agree on €0.36–0.60" (§1, §3.2) | **PARTLY: overstated** | Only Onlineprinters priced the actual product (folded A6, €52.41–59.24). Flyeralarm's €52.27 is a business-card-size folded card (8.5 × 5.5 cm). Pixartprinting's €36.15 is a flat A6 read by a competitor. Vistaprint's €0.61 is TTC without shipping. The agreement is across four different products. A poche-size jacket has about 30 % more sheet area than an open A6 and an A5 card twice as much, so the €0.60 / 1.00 / 1.50 jacket assumption has no analogue behind it. | [XREF] web2print note §4; [CALC] |
| 11 | Exaprint is "the only large brand with a confirmed plant in France"; scored E6 = 12 and S3 = 5 (§1, §5, §6.5) | **PARTLY** | The Mauguio plant and Cimpress ownership since 2015 match my memory. But Exaprint was historically a platform subcontracting to partner printers, and the in-house factory covers only part of the catalogue. "A plant exists in France" does not mean our job is printed there, so the two scores assume what the quote must establish. Exaprint also sells to professionals only (SIRET), so it is no route for a standalone individual. Size, staff and revenue figures are not re-verified. | [MEM]; [NONE] on figures |
| 12 | Directive (EU) 2024/825 applies from 27 September 2026 and blacklists offset-based climate-neutral claims and four other practices (§1 finding 4, §6.2) | **CONFIRMED [MEM], with a caveat** | Dates (transposition by 27 March 2026, application 27 September 2026) and the five blacklist items are correct. It binds traders through national transposition. Whether France has transposed is unverified, so "blacklisted since 27 September 2026" is not established for France. Existing French law already covers part of it (row 13). | [MEM]: Directive (EU) 2024/825 |
| 13 | France: Code de l'environnement art. L. 229-68 restricts "neutre en carbone"; loi AGEC art. 13 forbids "respectueux de l'environnement" on products; BGH 27 June 2024, I ZR 98/23 (§6.2) | **CONFIRMED [MEM]** | All three citations are right. AGEC art. 13 is codified at art. L. 541-9-1 and also bans "biodégradable"; it applies to wording on the product or packaging, so on the card itself. | [MEM] |
| 14 | "Vegetable-oil inks are a legal baseline in France (AGEC art. 112, since 1 January 2025)" (§6.1) | **PARTLY: overstated** | The date and article are right. The law bans specified mineral-oil fractions above thresholds (arrêté of 13 April 2022) in printed matter for the public; it does not mandate vegetable inks. It mainly concerns offset; toner and inkjet, the likely process for runs of 1–300, are barely affected. Missed consequence: it applies to what is distributed in France, so a German or Italian batch printer must be asked for compliance. | [MEM] |
| 15 | Data Privacy Framework challenge dismissed by the General Court in September 2025, appeal pending; UK and Swiss adequacy (§5) | **CONFIRMED [MEM] to mid-2026** | Latombe v Commission, T-553/23, 3 September 2025; appeal lodged autumn 2025. Status after mid-2026 unknown. UK adequacy was, to my recollection, renewed in December 2025; Swiss adequacy was confirmed in January 2024. | [MEM] |
| 16 | "Hosted in the EU can be stated today for Maileva only"; Maileva is La Poste group, State-owned (§5) | **PARTLY** | Ownership is right (Docaposte; La Poste held by Caisse des Dépôts and the State). The hosting statement rests on one 2024 vendor brochure, with no DPA or sub-processor list read. The document's own `eu_only` rule requires hosting "stated in writing"; a brochure is not that. | [MEM] ownership; [XREF] single brochure |
| 17 | Gelato: Oslo, production in 32 countries including France, Gelato+ "$14.99 to about $30", public DPA; "EU scale-out choice" (§4) | **PARTLY** | Oslo and 32 countries match my memory. France as a production country for cards rests on one review blog. $14.99 is, from memory, an older Gelato+ price. No unit or shipping price exists. Gelato's G1 floor of 4 breaks the document's own rule (memory counts zero): its EEA seat is tagged [M] and the sub-processor list was not read, so the floor should be about 2. | [MEM]; [XREF] |
| 18 | Onlineprinters: owned by Bregal; "best evidenced mainstream printer" (ESG report 2022, Imprim'Vert, PEFC and Blue Angel papers); G1 = 2 (§1, §6.5) | **PARTLY** | Bregal ownership since 2016 matches my memory; current status not re-verified. The evidence is old company PR: the Imprim'Vert item is a "sixth renewal" press release (probably around 2017; the label is annual), and the offsetting release dates from 2013. G1 = 2 ("controlled from the EU/EEA") contradicts the document's own text, which places Bregal's family capital in Switzerland. | [MEM]; [XREF] web2print note line 232 |
| 19 | Entreprise adaptée = at least 55 % disabled employees; a reseller with 20+ employees can deduct part of EA/ESAT purchases (§1, §7.1) | **CONFIRMED [MEM], with a nuance** | Both rules are right (30 % of the labour cost of the purchase, capped at 50 % or 75 % of the contribution). The deduction goes to the company that buys from the EA. If eBook Cadeau is the buyer of record, the reseller gets nothing; the reseller must contract directly. | [MEM]: Code du travail, OETH rules in force since 2020 |
| 20 | Phase-1 economics: 1.5 % + €0.25 payment fee, micro-entreprise charges 12.3 %, contribution €2.26 then €1.41, plan covered from 15 cards (§7.2) | **PARTLY** | The arithmetic is right. 12.3 % is the rate for sales of goods; if the activity is classed as a service the rate is 21.2 %, which cuts the €1.41 to about €0.80. The cost side also inherits rows 1–2 and an unknown VAT status (no input VAT recovery under the VAT franchise). | [CALC]; [MEM] |

Other points checked, in brief:

- **Label and case citations** (Blauer Engel DE-UZ 195, EU Ecolabel Decision (EU) 2020/1803, EMAS, the five Imprim'Vert criteria): CONFIRMED [MEM].
- **Book variant**: Lulu spine formula (pages / 444 + 0.06 in gives 148 pages for 10 mm) and the KDP range of 158–175 pages: CONFIRMED [CALC + MEM]. Lulu B Corp and the Printful–Printify merger (announced November 2024): CONFIRMED [MEM].
- **Carbon table (§6.3) and the ranking "home print is best for the environment"**: UNSUPPORTED. No life-cycle study was used; the orders of magnitude are plausible, but the ranking is an opinion.
- **All other vendor prices** (Popcarte, Fizzer, MyPostcard, Printful €2.36, Print&Bind, Prodigi, Pingen, Maileva, BoD, Bookvault): UNSUPPORTED [NONE].

## 2. Omissions

All from memory; none could be searched. Treat each as a lead to verify.

1. **Quantity-1 digital upload-and-print shops** were not examined: viaprinto, print24, WIRmachenDRUCK short runs, and French walk-in chains with online ordering (Copy-Top). They are the most likely source of a single-copy folder or card with a real spine (row 9).
2. **French postal and card APIs are missing from the comparison.**
   - MySendingBox (formerly Seeuletter) is a French letter and postcard API.
   - La Poste has its own online letter and postcard services.
   - Dromadaire, Cheerz, Photoweb and Planet Cards mail single printed cards and are useful benchmarks.
   - Popcarte and Fizzer have business offers; "no API" was a non-finding, not a finding.
3. **Industrial mail tariffs.** API providers and mailing houses pay bulk rates with different weight bands, not retail stamps. The 20 g cliff, the "ceiling" in §3.4 and the Phase-2 saving (€1.08 + €1.67 retail stamp, for a different and lighter object than the Merci Facteur card) all assume retail stamps.
4. **VAT on postage.** Universal-service stamps are VAT-exempt, while a printed-and-mailed card sold as one supply carries 20 % VAT on the whole price. This changes the comparison between in-house and outsourced once the VAT franchise is exceeded.
5. **Mandatory marks on the printed object in France.**
   - The printer's name and address on printed matter made public (loi du 29 juillet 1881, art. 3, with an exemption for private stationery).
   - The Triman and sorting information for printed paper given to households (Code de l'environnement art. L. 541-9-3).
   Neither appears in the printer brief; applicability to this object needs checking.
6. **EU Deforestation Regulation.** Paper is in scope. My recollection is that the December 2025 amendment postponed application to end-2026 / mid-2027 and removed printed products from the scope; to verify. It affects what "legal baseline" means for paper sourcing.
7. **Green Claims Directive.** The Commission announced in June 2025 that it intended to withdraw the proposal. The document lists its status only as unverified.
8. **De-inkability blocks the top ecolabel for singles.** HP Indigo and much inkjet output fail de-inking tests, which Blauer Engel DE-UZ 195 requires. Criterion E1 is therefore probably unattainable for any single-card service, and the grid should say so.
9. **GDPR in standalone mode.** The document describes only the reseller chain. When an individual orders a mailed card, eBook Cadeau is the controller for the recipient's address and owes the recipient information (Art. 14). The postal operator is normally an independent controller, not a sub-processor.
10. **EU scale-out postage.** Other national posts price by format as well as weight (a C5 envelope is not a standard letter in Germany), so the 20 g logic is France-specific.
11. **Stale evidence earns floor points.** Onlineprinters (about 2013–2017 PR, 2022 report), Flyeralarm (2012 leaflet), SAXOPRINT (2022 presentation, pre-takeover). The grid has no age limit, while implementation item 18 demands evidence under 12 months old.
12. **Prodigi inconsistency.** The document records that cards are fulfilled from the UK, EU and US, yet scores Prodigi as UK origin (E6 = 0) and calls it weak for customs reasons.

## 3. Statements to downgrade to "unverified"

- Every "[V2]" on Merci Facteur prices and plan (§1, §3.4, §4): downgrade to V1, single vendor.
- "The only verified per-card route to a French letterbox" (§1): becomes "the only one found; artwork capability, price basis, DPA and print site unverified".
- "€4.29" as planning or base cost (§1, §7.1, §7.2, implementation item 10, and doc 12): use a range, with base no lower than the midpoint, until a test invoice exists.
- "Mass web-to-print does not sell single copies" (§1): becomes "the six shops read had minimums of 10–50".
- "Four printers agree on €0.36–0.60" and "about €52 HT delivered" (§1, §3.2): one printer, one product.
- True-jacket assumption €0.60 / 1.00 / 1.50 (§3.3, implementation item 10): no analogue; label as a pure guess.
- Folder minimum "50" (§2, P3): one French printer only.
- La Poste 2026 100 g rate (€3.10 / €2.97) and all 2027 rates (§3.6, implementation item 8).
- Exaprint E6 = 12 and S3 = 5; "re-insourced since 2019" as applying to our product (§5, §6.5).
- Gelato G1 = 4 and Onlineprinters G1 = 2 (§6.5): both conflict with the grid's own rules.
- Onlineprinters E2 and E7 floor points (§6.5): undated or old company PR.
- "Hosted in the EU can be stated today for Maileva" (§5): one brochure.
- "Applicable from 27 September 2026" as binding in France (§1 finding 4, §6.2): add "subject to French transposition, not checked".
- "Vegetable-oil inks are a legal baseline" (§6.1): reword to a mineral-oil threshold ban.
- "Best for the environment: print at home" and the whole of §6.3: an estimate with no study behind it.
- Micro-entreprise rate 12.3 % (§7.2): depends on goods-versus-service classification.
- Gelato+ price range, Pixartprinting's Pittsburgh site, Exaprint size/staff/revenue, Flyeralarm site count: single or third-party sources.

## 4. Searches to run first when the budget returns

1. Merci Facteur: tariff page, PRO/API price list, card artwork parameters in the API README, legal entity, DPA.
2. La Poste 2026 and 2027 grids on laposte.fr (20 g, 100 g, tracked, international).
3. Cimpress / CEWE closing announcement for SAXOPRINT and viaprinto.
4. viaprinto and print24 minimum quantities for folders with a 10 mm spine and for folded cards; delivery to France.
5. French transposition of Directive (EU) 2024/825.
6. Exaprint: which products the Mauguio plant makes; account requirements; current Imprim'Vert attestation.
7. Gelato: folded-card price and shipping to France, production country for cards, current Gelato+ price.
8. Onlineprinters: current owner; current-year Imprim'Vert entry.
9. MySendingBox and La Poste online card services: products, prices, API.
10. Imprint and Triman obligations for a printed gift card; current scope of the EU Deforestation Regulation.

Files read: `/home/user/ebook-cadeau/docs/research/04-printers.md`, and in `/home/user/ebook-cadeau/docs/research/notes/`: `printers.web2print.md`, `printers.print-api.md`, `printers.esg.md`, `print-requirements.folding.md`, `market-business.competitors-costs.md`; also `/home/user/ebook-cadeau/docs/research/12-market-and-business-inputs.md`. The document was not edited.
