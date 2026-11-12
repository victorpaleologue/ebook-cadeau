# Market, competitors, cost inputs and business-model options

_Status: draft — research date 2026-11-12_

**Scope.** Reference document for the "eBook Cadeau" business case: market size, demand evidence, competitors, price benchmarks, cost inputs (machine-readable), business-model options, risks and a go-to-market sequence. France first, then DE / NL / ES / IT.

**How this was built, and its limits.**

- Synthesis of two researcher notes: `docs/research/notes/market-business.market.md` (16 searches) and `docs/research/notes/market-business.competitors-costs.md` (25 searches). Print-cost, legal and e-reader facts are cross-read from sibling notes (`printers.web2print.md`, `printers.print-api.md`, `legal-gifting-commerce.commerce.md`, `ereader-ux.ecosystems.md`) and labelled "sibling note".
- I attempted six further searches (gift share of book purchases, self-publishing volumes, retailer shares, Seebook, 7switch gifting, affiliate rates). **All six were refused: the workflow-wide search budget was exhausted.** No gap was filled and no confidence marker was upgraded. Every gap listed by the researchers is still open (section 10.3).
- No page was read in full by anyone (page fetching is blocked). Every sourced figure comes from a search-result extract. Official tariffs (La Poste, Stripe, Mollie, URSSAF, INPI) must be re-read on the official page before they are hard-coded.
- Confidence markers: **[V2]** = verified in two or more independent sources, **[V1]** = single source (or several outlets relaying one origin, or conflicting sources), **[M]** = from memory, not verified in this session. "Derived" = arithmetic on sourced inputs. "ASSUMPTION" = a planning value with no source.
- Money is in EUR unless stated. Prices are consumer prices including VAT unless marked HT (excluding VAT).

---

## 1. Executive summary

**Is there a business here?** Yes, but a small one. It is a low-fixed-cost niche service or a feature licensed to resellers. On current evidence it is not a business that pays a full-time salary in the base case.

| Question | Answer | Confidence |
|---|---|---|
| How big is the base market? | France: about 17 M paid consumer eBooks a year, EUR 137 M, about EUR 8 each. FR+DE+NL+ES+IT: about 85-90 M eBooks, about EUR 0.7 bn. eBooks are 3.4% of French consumer book value (5.8% of literature) and 5.4-6.3% in DE, NL, ES and IT; flat everywhere | [V1] France retail, [V2] Germany, order of magnitude for the sum |
| How many cards a year? | France TAM 0.5 / 1.0 / 2.0 M, SAM 0.10 / 0.33 / 0.98 M, SOM year 3 about 3,000 / 27,000 / 147,000 (low / base / high). Five countries, SOM year 5: about 10,500 / 85,000 / 504,000 | Derived from ASSUMPTIONS (gift share 3-12%, non-Amazon share 40-60%) |
| How much revenue is that? | At EUR 0.50-4 per card: France base EUR 13,000-106,000 a year; five-country base EUR 43,000-340,000; only the five-country high case reaches EUR 0.25-2.0 M | Derived |
| What does it cost to run? | Lean cash costs of roughly EUR 600-1,300 a year (hosting EUR 10-25 a month, domain, insurance, bank, CFE), excluding the owner's time. This is possible only because PDF, QR and font work run in the browser | Hosting [V1]-[V2]; the rest [M] |
| Where is break-even? | About 16-35 authors paying EUR 49 a year, or one reseller licence of about EUR 100 a month, or about 500-1,050 mailed cards a year at EUR 4.90 printed in-house, or about 2,500-5,500 cards a year at a EUR 0.30 B2B fee | Derived from partly unverified inputs |
| What could year 3 look like in France? | Illustrative mix of reseller fee, mailed upgrades and author sales (section 7.4): revenue about EUR 4,000 / 32,000 / 139,000 and contribution about EUR 1,700 / 15,500 / 67,000 (low / base / high) | Derived from ASSUMPTIONS |
| Can it pay a salary? | One salary of EUR 3,000 net a month costs about EUR 63,300 a year. That needs about 37,000 mailed cards a year at EUR 1.70 contribution, or about 211,000 cards at a EUR 0.30 fee. Both are above the France base SOM (27,000) | Derived |
| Does demand exist? | Unproven. Books are a leading Christmas gift and no eBook gifting flow produces an object, but four title-specific eBook card programmes sold in shops all failed, and no source measures the attach rate of a gift option | [V2] for the failures; no source for attach rate |

**Most promising model** (detail in section 7):

1. **Free in-browser print-at-home generator** as the product demo and acquisition channel. Marginal cost is close to zero.
2. **First revenue from authors and publishers**: a yearly plan for batches of single-use codes plus print-ready jackets for signings and fairs (EUR 29 / 49 / 99 a year to test), with printed packs at cost plus margin. This is the only segment with a proven, continuing payer (BookFunnel, Dropcards, Bookwire).
3. **Scale lever: a reseller licence** (flat monthly fee plus a metered per-card fee above a quota, invoiced by SEPA). The reseller runs the generator in its own checkout; eBook Cadeau never handles the eBook money.
4. **Mailed single card only as a premium option**, at EUR 6.90 or more, fulfilled by a French postal-card API. At EUR 4.90 it loses money when outsourced. Launch it only after a price test shows demand.

**What was not found** (and limits every number above): the share of book purchases made as gifts; any survey on why people do not gift eBooks; retailer shares of French eBook sales; the attach rate of a gift option at checkout; e-reader sales since 2013; self-publishing volumes; whether Fnac/Kobo, 7switch or Cultura can issue a giftable entitlement; affiliate rates; real printer quotes for a double-scored jacket.

---

## 2. Market sizing

### 2.1 Headline numbers by country

National statistics are not comparable with each other: some are publisher net revenue, some are consumer sell-out, some include professional and academic digital.

| Country | Year | Paid eBook units | eBook value | Share | Avg price paid | What is measured | Source | Conf. |
|---|---|---|---|---|---|---|---|---|
| France (retail) | 2025 | ~17 M | EUR 137 M (+4%) | ~3.4% of value, ~5.2% of units (derived) | ~EUR 8.06 (derived) | NielsenIQ BookData consumer sell-out | [Livres Hebdo](https://www.livreshebdo.fr/article/marche-du-livre-baisse-des-ventes-et-polarisation-accrue-en-2025), [CCFI](https://www.ccfi.asso.fr/marche-du-livre-baisse-des-ventes-et-polarisation-accrue-en-2025/) | [V1] |
| France (publishers) | 2025 | n/a | EUR 275.3 M (-1.2%) | 10% of publishers' book sales | n/a | SNE publisher net revenue; 67.4% of it is professional/academic | [SNE](https://www.sne.fr/economie/chiffres-cles/), [Livres Hebdo](https://www.livreshebdo.fr/article/marche-du-livre-2025-un-chiffre-daffaires-quasi-stable-qui-masque-de-profondes-mutations) | [V2] |
| Germany | 2025 | 37.2 M (-4.5%) | ~EUR 261 M (derived) | 6.3% of Publikumsmarkt | EUR 7.01 (+4.7%) | Börsenverein consumer panel, excl. school and professional books | [Börsenverein](https://www.boersenverein.de/fileadmin/bundesverband/dokumente/presse/digitale_pressemappen/WIPK/Buchmarkt_kompakt_2025_2026_Zahlenuebersicht.pdf), [handelsdaten.de](https://www.handelsdaten.de/branchen/buchhandel), [Statista](https://de.statista.com/statistik/daten/studie/232200/umfrage/durchschnittspreis-von-e-books-in-deutschland/) | [V2] |
| Netherlands | 2025 | ~4 M | ~EUR 42 M | 9% of units, 6% of revenue | ~EUR 10.5 (derived) | KVB Boekwerk general market, preliminary | [KVB Boekwerk](https://kvbboekwerk.nl/monitor/markt/verkoopcijfers-2025), [Stivako](https://www.stivako.nl/nieuws-blog/3327479_papieren-boek-blijft-koning-onderzoek-abn-amro) | [V2] share, [V1] absolutes |
| Spain | 2024 | ~18.4 M (derived, upper bound) | EUR 165.57 M (+14.9%) | 5.5% of publishers' domestic sales | EUR 9.0 (was 9.7) | FGEE publisher revenue, all categories | [FGEE](https://federacioneditores.org/wp-content/uploads/2025/10/Comercio-Interior-2024_Interior.pdf), [Instituto Autor](https://institutoautor.org/espana-la-federacion-de-gremios-de-editores-de-espana-publica-los-datos-de-del-comercio-interior-del-libro-en-2024/) | [V2] |
| Italy | 2025 | ~11 M (ASSUMPTION: EUR 8 avg) | EUR 87 M (+2.4%) | ~5.4% of trade "varia" (derived) | not found | AIE, trade market at cover price | [Orizzonte Scuola](https://www.orizzontescuola.it/il-mercato-editoriale-italiano-chiude-il-2025-con-tre-milioni-di-copie-in-meno-calano-i-libri-cartacei-crescono-ebook-e-audiolibri/), [AgenziaCult](https://www.agenziacult.it/economia/editoria-aie-mercato-2025-a-32-mld-calo-dell12/), [AIE](https://network.aie.it/Portals/_default/Skede/Allegati/Skeda105-10721-2025.10.14/Rapporto_2025_sintesi.pdf?IDUNI=f0tez4j4la20peflur5ghzw1193) | [V2] |
| UK | 2023 | not found | GBP 292 M consumer digital (eBook + audio) | not found | not found | Publishers Association invoiced sales | [The Bookseller](https://thebookseller.com/news/review-of-the-year-hachette-retains-the-digital-crown-as-all-of-the-big-six-post-gains) | [V1] |
| Europe (FEP) | 2024 | n/a | eBooks ~13% of EUR 24.9 bn publisher turnover | print 82.9%, audio 4.2% | n/a | FEP, all publishing incl. academic | [FEP](https://fep-fee.eu/European-Book-Publishing-Statistics-2024), [Bursa](https://www.bursa.ro/the-european-book-market-nearly-24-9-billion-but-inflation-erodes-publishers39-revenues-25136061) | turnover [V2], eBook share [V1] |

Five-country total (FR+DE+NL+ES+IT): 17.0 + 37.2 + 4.0 + 18.4 + ~10.9 = **about 87.5 M paid consumer eBooks a year**; 137 + 261 + 42 + 166 + 87 = **about EUR 693 M**. The sum mixes retail and publisher-side valuations: treat it as an order of magnitude.

Reading across countries: the consumer eBook share sits in a narrow 5-6.5% band of trade value in FR (literature), DE, NL, ES and IT, flat or slightly rising. This is a stable niche, not a growth market.

### 2.2 France in detail

**Publisher side (SNE).**

- Publishers' revenue 2025: EUR 2,883.3 M (-0.63% value, -1.49% volume, 419.6 M copies) [V2] ([Livres Hebdo](https://www.livreshebdo.fr/article/marche-du-livre-2025-un-chiffre-daffaires-quasi-stable-qui-masque-de-profondes-mutations), [ActuaLitté](https://actualitte.com/article/132253/economie/edition-en-2025-le-chiffre-d-affaires-resiste-mieux-que-les-ventes)).
- Digital 2025: EUR 275.3 M = 10% of publishers' book sales of EUR 2,751.9 M, down 1.2% [V2] ([SNE chiffres clés](https://www.sne.fr/economie/chiffres-cles/)). Professional and university publishing is 67.4% of it (about EUR 185.6 M), leaving about EUR 89.7 M for consumer and school digital (derived).
- Digital 2024: EUR 278.6 M, -1.6% on 2023; total publishers' revenue EUR 2,901.6 M [V2] ([idboox](https://www.idboox.com/news-livres/marche-du-livre-en-france-etat-des-lieux-chiffres-cles-2025/), [Livremploi](https://www.livremploi.fr/actualites/decouvrez-la-synthese-2024-2025-des-chiffres-de-ledition)). The Ministry of Culture describes digital as stagnating slightly under 10% of publishers' revenue since 2020 [V1] ([DEPS chiffres clés 2024](https://www.culture.gouv.fr/Media/medias-creation-rapide/Chiffres-cles-2024_DEPS_Livre-et-lecture-publique_Fiche.pdf)).
- Literature: eBooks are **5.8% of publishers' literature sales in 2025, down from 6.2% in 2024** [V2] ([SNE chiffres clés](https://www.sne.fr/economie/chiffres-cles/), [Livres Hebdo](https://www.livreshebdo.fr/article/marche-du-livre-2025-un-chiffre-daffaires-quasi-stable-qui-masque-de-profondes-mutations)). With literature at EUR 694.6 M (+4.7%), digital literature is about EUR 40 M publisher net (derived). Truncated extracts of the SNE syntheses show EUR 40.9 M (2024) and about EUR 40.2 M (2025) [V1] ([SNE synthèse 2025/2026](https://www.alire.asso.fr/wp-content/uploads/2026/07/sne-synthese-statistique-2026-web-v-def.pdf), [idboox](https://www.idboox.com/news-livres/france-marche-du-livre-papier-et-numerique-chiffres-cles-2025/)).
- 2024 detail: digital literature +8.5%, driven by romance; consumer digital outside literature -17.1% [V1] ([SNE synthèse 2024/2025](https://livre-bourgognefranchecomte.fr/sites/livre-bourgognefranchecomte/files/2025-07/SNE-Synthese-Statistique-2025-Web-avec-edito.pdf)).

**Retail side (NielsenIQ BookData).** 2025 digital sales: about 17 M copies for EUR 137 M, +4%. New print books: 307 M copies (-2.5%), EUR 3.9 bn (-1.5%) [V1: Livres Hebdo, relayed by CCFI]. Derived average price about EUR 8.06.

*Reconciliation.* SNE says EUR 275.3 M and NielsenIQ says EUR 137 M. They do not contradict each other. EUR 137 M at retail (including 5.5% VAT and the retailer's margin) is consistent with about EUR 90 M of non-professional publisher digital revenue. **We keep NielsenIQ's 17 M units as the base for the funnel** because it is the only consumer sell-out unit count.

**Readers, buyers, devices** (14th Baromètre Sofia/SNE/SGDL, Médiamétrie, usage year 2025).

- 47 M people read or listened to a book in 2025; **14 M (1 in 4) read at least one digital book**; 10 M listened to a digital audiobook [V2] ([SNE](https://www.sne.fr/actu/resultats-du-barometre-2026-des-usages-dachat-et-de-lecture-des-livres-imprimes-numeriques-et-audio/), [Livres Hebdo](https://www.livreshebdo.fr/article/barometre-sofiasnesgdl-2026-des-usages-en-recomposition-pour-les-livres-imprimes-numeriques)). A secondary summary gives 14.4 M and 10.1 M [V1] ([JOMO Horizon](https://horizon-jomo.eklablog.com/2026/04/barometre-sofia-sne-sgdl-des-usages-d-achat-et-de-lecture-des-livres-imprimes-numeriques-et-audio-2026.html)). *We keep 14 M as the base (two sources) and use 14.4 M as the high value.*
- 20% say they bought at least one digital book, against 71% for print [V1: slide extract of the [barometer PDF](https://www.sgdl.org/images/phocadownload/Baromètres/BarometreSOFIA-SNE-SGDL_2026.VDEF.pdf); also hosted by [La Sofia](https://www.la-sofia.org/wp-content/uploads/2026/04/BarometreSOFIA-SNE-SGDL_2026.VDEFMM.pdf); not read in full].
- Devices used by digital readers: **smartphone 48%, liseuse (e-reader) 34%, tablet 33%** [V1] ([SGDL](https://www.sgdl.org/sgdl-accueil/le-guide-pratique/ressources-documentaires/barometres-et-observatoire/4867-barometre-sgdl-sofia-sne-2026-des-usages-dachat-et-de-lecture-des-livres-imprimes-numeriques-et-audio)). Derived: about 4.8-4.9 M people in France read on an e-reader. Two thirds of digital readers do not use one.
- Where digital books are bought: websites of large specialist chains ("grandes surfaces spécialisées", the Fnac type) 33%, digital platforms 29%. Pay-per-title is still the most cited way to obtain a digital book (21%) [V1] (same SGDL page).
- Piracy: 6% of the French (about 3 M), 17% among digital readers [V1] ([SNE](https://www.sne.fr/actu/resultats-du-barometre-2026-des-usages-dachat-et-de-lecture-des-livres-imprimes-numeriques-et-audio/)).
- *Contradiction on buyer counts.* 20% declared buyers would be 11-12 M people, i.e. only about 1.5 paid eBooks each against NielsenIQ's 17 M units. The German purchase panel finds 10.9 eBooks per buyer. Declared surveys overstate. **We keep the unit count and treat the buyer count as unknown** (ASSUMPTION: 2-4 M regular paying buyers).

**E-readers.** No recent unit sales or installed base found. Historic GfK series: 5,000 (2008), 27,000 (2010), 145,000 (2011), 300,000 (2012), 350,000 (2013), then stagnation [V1] ([Wikipedia FR](https://fr.wikipedia.org/wiki/Liseuse)); a specialist blog judged these underestimated [V1] ([liseuses.net](https://www.liseuses.net/les-chiffres-bidons-des-ventes-de-liseuses-en-france/)). The only brand split found (Amazon 60 / Kobo 30 / Vivlio 10, large-format readers only) is a weak commercial page [V1, low quality] ([Accio](https://fr.accio.com/business/grande-liseuse-top-ventes)). Do not rely on it.

### 2.3 Germany and the other markets

- Germany, eBook share of the Publikumsmarkt: **6.3% in 2025**, 6.1% in 2024 [V2] ([Börsenverein](https://www.boersenverein.de/fileadmin/bundesverband/dokumente/presse/digitale_pressemappen/WIPK/Buchmarkt_kompakt_2025_2026_Zahlenuebersicht.pdf), [literaturcafe.de](https://www.literaturcafe.de/boersenverein-praesentiert-zahlen-zum-buchmarkt-weniger-titel-weniger-umsatz-in-2025/)). 2025: 37.2 M eBooks (-4.5%), 3.4 M buyers, EUR 7.01 average, 10.9 eBooks and EUR 76.46 per buyer [V2] ([handelsdaten.de](https://www.handelsdaten.de/branchen/buchhandel), [Statista](https://de.statista.com/statistik/daten/studie/232200/umfrage/durchschnittspreis-von-e-books-in-deutschland/)). Fiction is 90% of eBook revenue (2024) [V1] ([Börsenverein dossier](https://www.boersenverein.de/fileadmin/bundesverband/dokumente/presse/mediendossiers/verlage/Dossier_Verlage_2024.pdf)).
- *Contradiction.* One extract gives "3.0 M buyers, 41.7 M eBooks" without a clear year, while 37.2 M at -4.5% implies about 39 M for 2024. **We keep 37.2 M for 2025** (two sources, internally consistent with price and spend per buyer) and do not use the 41.7 M figure.
- E-reader ownership in Germany: 17% of the population ("more than eleven million people") per GfK as quoted by tolino (about 2023); tolino claims 44% of eBooks sold [V1, vendor claim] ([mytolino](https://mytolino.de/news/zehn-jahre-tolino-die-ereading-allianz-der-deutschen-buchhaendler-forciert-weiteres-wachstum/)).
- Netherlands 2025: 44 M books, EUR 697 M; eBooks 9% of units and 6% of revenue, preliminary [V2] ([KVB Boekwerk](https://kvbboekwerk.nl/monitor/markt/verkoopcijfers-2025)); 2024: over 3 M eBooks [V1] ([Stichting Lezen](https://www.lezen.nl/onderzoek/aandeel-e-boek-blijft-bescheiden/)).
- Spain 2024, Italy 2025, UK 2023, FEP 2024: see the table in 2.1. Italy's audiobooks: EUR 34 M (+13.3%) in 2025 [V2].

Germany is the larger prize: 2.2 times France's units, with a bookseller alliance (tolino) that claims 44% of units.

### 2.4 Funnel assumptions

Unit: one gifted eBook delivered with a jacket card (printed at home or professionally), per year. Every row except A1 is an ASSUMPTION.

| # | Assumption | Low | Base | High | Basis |
|---|---|---|---|---|---|
| A1 | Paid consumer eBook units, France | 17.0 M | 17.0 M | 17.0 M | NielsenIQ 2025 [V1] |
| A1' | Same, FR+DE+NL+ES+IT | 87.5 M | 87.5 M | 87.5 M | Section 2.1; ES is an upper bound, IT assumed |
| A2 | Share of eBook units that could be gifts once a physical object exists | 3% | 6% | 12% | Print gift share believed to be 15-25% [M]; eBooks need a compatible recipient, so well below print |
| A3 | Share of those units sold outside Amazon (Kindle is closed; Amazon will not integrate) | 40% | 50% | 60% | Not found; ASSUMPTION |
| A4 | Share of gift occasions where the giver wants a physical token | 50% | 65% | 80% | Directional only (section 3) |
| A5 | Capture of SAM by year 3 (France) | 3% | 8% | 15% | Low = standalone only; base = one mid-size reseller; high = one Fnac/Kobo-class reseller |
| A5' | Capture of SAM by year 5 (five countries) | 2% | 5% | 10% | Sequential roll-out, localisation |

### 2.5 TAM / SAM / SOM (cards per year)

| Level | Formula | France low | France base | France high | 5 countries low | 5 countries base | 5 countries high |
|---|---|---|---|---|---|---|---|
| TAM | A1 x A2 | 0.51 M | 1.02 M | 2.04 M | 2.6 M | 5.3 M | 10.5 M |
| SAM | TAM x A3 x A4 | 0.10 M | 0.33 M | 0.98 M | 0.53 M | 1.7 M | 5.0 M |
| SOM | SAM x A5 | ~3,000 (year 3) | ~27,000 (year 3) | ~147,000 (year 3) | ~10,500 (year 5) | ~85,000 (year 5) | ~504,000 (year 5) |
| Revenue pool at SOM | SOM x EUR 0.50 to 4.00 | EUR 1,500-12,000 | EUR 13,000-106,000 | EUR 73,000-588,000 | EUR 5,000-42,000 | EUR 43,000-340,000 | EUR 0.25-2.0 M |

Not in the funnel: the **author / signing segment**, sized at 10,000-50,000 cards a year for France (ASSUMPTION: 10,000-20,000 self-published authors active at signings and "salons du livre", 2-5% adopt, 50 cards each a year). Nothing could be searched for this segment; self-publishing as 15-20% of French legal deposit is [M].

Cross-checks (derived):

- France base TAM of 1.0 M is 0.21 gifted eBooks a year per e-reader user (4.9 M), or 0.07 per digital reader. Plausible, not aggressive.
- Against print: if 15-25% of 307 M new print copies are gifts [M], that is 46-77 M gifted print books; the base TAM is 1.3-2.2% of it.
- As an attach rate: the base SOM of 27,000 cards is 0.3% of the non-Amazon French units (8.5 M). It equals one reseller holding about 8% of French eBook units with a 2% attach rate. The base SAM (0.33 M) would need every non-Amazon reseller to integrate and a 3.9% attach rate.
- Value of the gifted eBooks themselves at base TAM: about EUR 10 M a year in France (ASSUMPTION: EUR 10 per title, gifts skew to new releases). The service revenue pool is far smaller.

**Seasonality overlay** (ASSUMPTION built on section 3): 45-55% of yearly card volume falls between 20 November and 24 December, with a redemption spike on 25 December. Professional print has a cut-off around mid-December; print-at-home covers 20-24 December.

---

## 3. Demand evidence and the gifting gap

### 3.1 What supports demand

| Evidence | Figure | Source | Conf. |
|---|---|---|---|
| December share of French book revenue | 17% in 2017 and 2018 (weeks 50 and 51 alone: 11%); 21% in 2020 | [NielsenIQ 2018](https://nielseniq.com/global/fr/insights/report/2019/marche-du-livre-2018-une-industrie-de-4mds-e/), [NielsenIQ 2017](https://nielseniq.com/global/fr/news-center/2018/les-francais-ont-achete-plus-de-350-millions-de-livres-en-2017/), [NielsenIQ 2021](https://nielseniq.com/global/fr/insights/report/2021/tendances-du-marche-livre-en-france-formats-physiques-et-numeriques/) | [V2] |
| Last week of 2024 weighed unusually heavily | qualitative | [Livres Hebdo](https://www.livreshebdo.fr/article/comment-noel-sauve-le-marche-du-livre-2024-selon-gfk) | [V1] |
| The book is the main physical entertainment product bought at Christmas in FR, IT, ES (GfK) | qualitative | [ActuaLitté](https://actualitte.com/article/103491/economie/le-livre-roi-des-achats-a-noel-sur-fond-de-croissance-du-marche-jeunesse) | [V1] |
| Books bought to give, per book buyer, France | 2.6 a year | [L'ObSoCo for the SLF, 2026](https://lobsoco.com/wp-content/uploads/2026/06/LObSoCo_SLF-2026_Rapport-danalyse-6-juin.pdf) | [V1] |
| Buyers of new books who buy only for themselves | 8% (a quarter of second-hand buyers never buy to give) | [Chasse aux livres](https://www.chasse-aux-livres.fr/pourquoi-acheter-des-livres-d-occasion) | [V1, secondary] |
| French Christmas gift budget 2025 | EUR 243.60 on average, -7% | [The Media Leader, Havas Market](https://fr.themedialeader.com/?p=205798) | [V1] |
| Germany: November + December share of book trade revenue | 23.7% in 2024, 26.6% in 2021, 24% in 2019 | [Börsenblatt](https://www.boersenblatt.net/home/hde-rechnet-mit-umsatzplus-im-weihnachtsgeschaeft-397445), [handelsdaten.de](https://www.handelsdaten.de/anteil-des-weihnachtsgeschafts-am-jahresumsatz-einzelner-branchen-im-deutschen-einzelhandel-im-jahr) | [V2] |
| Germany: books as Christmas gifts | First place with 49% (FOM study, 2024); printed books 31% in 2025, third behind vouchers/money (48%) and food/sweets (36%) | [Handelsblatt](https://www.handelsblatt.com/unternehmen/handel-konsumgueter/weihnachtsgeschenke-2024-wie-viel-geben-die-deutschen-fuer-geschenke-aus/100004143.html), [Statista](https://de.statista.com/statistik/daten/studie/71364/umfrage/beliebteste-kategorien-fuer-weihnachtsgeschenke/) | [V1] each |
| French gift-card market | About EUR 8.6 bn in 2025, projected about EUR 10.8 bn in 2029; other estimates EUR 5-8 bn depending on scope | [Buybox](https://www.buybox.net/en/blog/the-gift-card-market), [Ecommerce Nation](https://www.ecommerce-nation.fr/futur-cartes-cadeaux-france/), [Place des Cartes via Social CSE](https://socialcse.fr/place-des-cartes-redistribue-1-mde-aux-francais/) | [V2] for the order of magnitude |
| Unused share of French gift cards | 20-25% (Sia Partners), 20% (Place des Cartes), up to a third with gift boxes; EUR 1-1.8 bn lost a year | [Europe 1](https://www.europe1.fr/societe/cartes-et-cheques-cadeaux-comment-loubli-de-leur-utilisation-constitue-un-business-juteux-pour-les-enseignes-4223693), [Sleeping Money](https://www.sleepingmoney.com/fr-fr/etudes/achat-revente-cartes-cadeaux-france/), [Sud Radio](https://www.sudradio.fr/quest-ce-qui-cloche-2/cartes-et-coffrets-cadeaux-que-faire-quand-la-date-est-depassee) | [V2], industry sources only |

The December share for 2025, and the share of book purchases made as gifts in any country, were not found. Working value for the gift share of print units: 15-25% [M].

### 3.2 The gap

- The diagnosis in the French press is old and unchanged. ActuaLitté's "L'impossible cadeau du livre numérique": a book is an object one gives; an eBook is easy to buy for oneself and awkward to give [V1] ([ActuaLitté](https://actualitte.com/article/69820/usages-numeriques/l-039-impossible-cadeau-du-livre-numerique)). The Conversation: an eBook is a file and a usage right, "il ne fait pas un cadeau très remarquable" [V1] ([The Conversation](https://theconversation.com/pourquoi-les-livres-papier-nont-ils-pas-disparu-118617)).
- Every existing gifting flow is e-mail only (section 4.2). None produces a physical object. Kobo by Fnac, the largest French non-Amazon channel, shows no title-level gift flow at all [V1; two low-authority sources agree: [Dealabs thread](https://www.dealabs.com/discussions/carte-cadeau-kobo-ou-fnac-2273026), [ebook-enligne.fr](https://ebook-enligne.fr/blog/offrir-ebook-guide-cadeau-numerique/)].
- Closest survey evidence (US, retailer-commissioned): 95% of book gifters would give a printed title rather than a digital one, and 89% prefer to receive print (Talker Research for ThriftBooks, 2025, n=2,000) [V1] ([ThriftBooks](https://www.thriftbooks.com/blog/which-books-are-the-most-popular-holiday-gifts-this-year/)). This shows the size of the barrier. It does not show that a printed jacket removes it.

### 3.3 What the evidence does not show

1. No survey asks why people do not gift eBooks. "You cannot wrap an eBook" is press commentary.
2. No source gives the attach rate of a gift option at an eBook checkout. The 0.5% / 2% / 5% planning values are ASSUMPTIONS.
3. Evidence against: four programmes that sold title-specific eBook cards in shops all closed (section 4.1). The trade press wrote that nobody wanted to buy the card, "much less redeem" it [V1] ([The Digital Reader](https://the-digital-reader.com/ebook-startup-livrada-shut/)).
4. Breakage: expect 10-20% of eBook gift codes never to be redeemed (ASSUMPTION, from the gift-card figures above). A title-specific card with a one-scan redemption should do better than a value card, but nothing measures it. Do not count breakage as revenue: who keeps that money is a legal and reputational question.

Conclusion: the need is plausible and the gap is real, but willingness to pay is unproven. The plan in section 9 therefore starts with tests that cost almost nothing.

---

## 4. Competitors and precedents

The brief's "Livrio" is **Livrada** (US) in the sources.

### 4.1 Title-specific eBook cards sold in physical retail: all dead

| Venture | Status (2026-11) | Model | Scale and price | Lesson |
|---|---|---|---|---|
| **Livrada** (US, 2012-c.2016) | **Dead.** Target pilot ended "a short while later"; pivot to own site, then bulk sales to authors and publishers ([The Digital Reader](https://the-digital-reader.com/ebook-gift-card-startup-livrada-completes-funding-round-plans-to-shift/)); site dead and e-mails unanswered by May 2017 ([The Digital Reader](https://the-digital-reader.com/ebook-startup-livrada-shut/)) [V2] | Card for one specific eBook on a rack; redeemed online (Kindle and Nook supported [M]) | Pilot in 1,500 of 1,700 Target stores ([Publishers Weekly, 2012](https://publishersweekly.com/pw/print/20120827/53710-livrada-tries-its-luck-with-e-book-gift-cards.html)); funded Feb 2013 ([TechCrunch](https://techcrunch.com/2013/02/11/livrada-funding/amp/)) [V2] | One retailer was the whole business. It reportedly ran out of money after Target stopped ([BookWorks](https://www.bookworks.com/2018/08/walmart-ebook-gift-card-launch-doomed/); second-hand) [V1] |
| **Enthrill** (Canada, 2012-2016) | **Dead.** Gift-card and retail programmes "decommissioned in early 2016"; Firebrand bought the DRM/delivery IP in Jan 2017, not the card business ([The Digital Reader](https://the-digital-reader.com/2017/01/12/firebrand-aquires-enthrills-ebook-gift-card-tech/), [Publishing Perspectives](https://publishingperspectives.com/2017/01/harpercollins-crowley-firebrand-enthrill/)) [V2] | Title-specific cards plus C$10/25/50 Walmart-branded eBook value cards; own device-agnostic delivery and social DRM | Safeway from 2012; Walmart Canada from 2014 ([Publishers Weekly, 2014](https://www.publishersweekly.com/pw/print/20140922/64046-enthrill-partners-with-walmart-canadian-publishing-2014.html)); "over 3,000 locations" in 2015 ([The eBook Reader Blog](https://blog.the-ebook-reader.com/2016/01/22/kindle-cards-new-book-specific-kindle-gift-cards); page not pinned) [V1]; raised "$1.3M" ([BetaKit](https://betakit.com/ebook-product-and-distribution-startup-enthrill-raises-1-3m-from-private-investors/)) [V1] | 3,000 points of sale did not make the card viable. The delivery technology had resale value; the card business had none |
| **Amazon "Kindle Cards"** (US, Jan 2016) | **Dead.** Never announced formally, never rolled out ([BookWorks, 2018](https://www.bookworks.com/2018/08/walmart-ebook-gift-card-launch-doomed/)) [V1] | Physical card showing one of 20 Kindle bestsellers or Kindle Unlimited; technically a fixed-value Amazon gift card | 61 Bartell Drugs stores ([Shelf Awareness](https://shelf-awareness.com/theshelf/2016-01-26/amazon_testing_title-specific_kindle_gift_cards_.html), [GeekWire](https://www.geekwire.com/2016/amazons-other-physical-retail-test-testing-the-kindle-e-book-kiosk-at-the-local-drug-store/)) [V2]. Card for *The Martian* USD 14.99 while the Kindle edition sold at USD 8.99 (GeekWire / [Consumerist](https://consumerist.com/2016/01/25/amazon-puts-mini-e-book-stores-in-drugstores-in-washington-state); excerpt not pinned) [V1] | A fixed printed price cannot follow a dynamic eBook price |
| **Walmart eBooks by Rakuten Kobo** (US, Aug 2018-c.2024) | **Dead.** Partnership ended; Walmart login on Kobo devices discontinued 31 Oct 2024 ([MobileRead](https://www.mobileread.com/forums/showthread.php?t=363478), [MobileRead](https://www.mobileread.com/forums/showthread.php?t=360234), [Good e-Reader](https://goodereader.com/blog/kobo-ereader-news/walmart-no-longer-selling-kobo-e-readers-in-their-stores)) [V1, forum-grade] | About 40 titles on "digital book cards" | 3,500 Walmart stores at launch ([TechCrunch](https://techcrunch.com/?p=1697131), [The Digital Reader](https://the-digital-reader.com/walmarts-ebookstore-is-launching-today/); excerpt not pinned) [V1] | Tiny assortment; store staff unaware of the cards (forum reports) [V1] |
| **Symtio** (Zondervan, US, c.2009) | **Defunct** [M] | Cards for digital books sold in bookshops | Only the headline was retrieved ([Publishers Weekly](https://publishersweekly.com/article/397075-Digital_Books_to_Stores_via_Symtio.php)) | n/a |
| **Seebook** (Spain, c.2013) | Not researched [M] | QR cards for eBooks sold in bookshops | n/a | Worth a dedicated search: closest European precedent |

*Contradiction.* One outlet says the Kindle Cards test ran in 41 stores, two say 61. **We keep 61** (two sources).

Background reading retrieved by the researcher but not quoted above: [The Digital Reader, 2014](https://the-digital-reader.com/2014/12/15/ebook-gift-cards-rise-decline/), [The Book Designer](https://www.thebookdesigner.com/e-books-for-real-e-book-gift-cards-from-livrada-and-enthrill/), [Build Book Buzz](http://buildbookbuzz.com/e-book-gift-cards/).

Reasons given in the trade press for the failures: nobody wanted to buy or redeem the card [V1]; "individualized ebook gift cards" had not taken off ([The eBook Reader Blog](https://blog.the-ebook-reader.com/2016/01/22/kindle-cards-new-book-specific-kindle-gift-cards)) [V1]; price mismatch [V1]; an assortment of 20-40 titles against an online catalogue of millions [V2]; dependence on one retailer [V1]; no merchandising [V1]. No sales or redemption figures exist publicly for any of these programmes.

### 4.2 Gifting inside the retailer's own checkout: alive, e-mail only

| Retailer | Status | Model | Price | Lesson / constraint | Conf. |
|---|---|---|---|---|---|
| **Amazon.fr (Kindle)** | Alive | "Acheter pour d'autres personnes" on the eBook page; prepaid redemption links or recipient e-mail ([Amazon.fr help](https://www.amazon.fr/gp/help/customer/display.html?nodeId=GVWGP284MQ6ZRM59), [KDP help](https://kdp.amazon.com/fr_FR/help/topic/G200652260), [IDBoox](https://www.idboox.com/news-livres/amazon-on-peut-enfin-offrir-un-livre-numerique-kindle/)) | No gift surcharge mentioned | Recipient must live in the same country; unredeemed copies can be reassigned or refunded. An older [ActuaLitté](https://actualitte.com/article/15659/usages-numeriques/offrir-un-ebook-amazon-facilite-les-cadeaux-de-derniere-minute) piece saying France was excluded is outdated | [V2] |
| **Decitre / Furet du Nord (Vivlio, ex-TEA)** | Alive | "Offrir un ebook ... en 2 clics": gift e-mail on a chosen date with an optional message ([Decitre](https://www.decitre.fr/offrir-un-ebook), [Decitre FAQ](https://www.decitre.fr/faq/numerique/offrir-un-ebook), [Liseuses.net](https://www.liseuses.net/offrir-des-ebooks-avec-tea/)) | No gift surcharge mentioned | A gift entitlement already exists; a printable object could be attached to it. Vivlio gift cards are valid for physical products only, not eBooks ([Vivlio help](https://help.vivlio.com/hc/fr/articles/11334184838428-Je-souhaite-utiliser-un-code-promotionnel-ou-une-carte-cadeau)) | [V2] |
| **Thalia / Hugendubel (tolino alliance, DE/AT)** | Alive | "Verschenken" button on eBook pages ([Hugendubel](https://www.hugendubel.de/de/shortcut/ebooksverschenken), [heise](https://www.heise.de/tipps-tricks/E-Books-verschenken-so-klappt-s-7476189.html), [test.de](https://www.test.de/E-Books-Nuetzliche-Tipps-und-Tricks-5391265-0/)) | Not found | Delivery mechanics (e-mail or code) not confirmed. A recollection that one alliance shop offered a print-it-yourself voucher is [M, low] (sibling note) | [V2] |
| **Kobo by Fnac (FR)** | No gift flow found | Product pages tie the eBook to the buyer's account; users fall back on value gift cards | n/a | A Fnac-type integration must create the prepaid entitlement as well as the card. Negative finding only; no official statement | [V1] |
| **7switch, Cultura, Apple Books, Google Play Books, bol.com** | Not searched | Apple and Google offer e-mail gifting of a book in some countries, same-country redemption [M]; bol.com had "ebook cadeau geven" [M, low]; 7switch and Cultura unknown | n/a | France is fragmented by platform, not by brand: Fnac = Kobo; Cultura / Decitre / Furet du Nord / E.Leclerc = Vivlio; independents = ePagine and others; 7switch = Immatériel [M] (sibling note). One platform integration reaches several storefronts | [M] |

### 4.3 Author and publisher code tools: alive, and the payer is the author or publisher

| Tool | Status | Model | Price (2026) | Lesson | Conf. |
|---|---|---|---|---|---|
| **BookFunnel print codes** | Alive | Single-use download codes an author prints on anything; one code can deliver several books | No per-code charge; needs the Mid-List (USD 200/yr) or Bestseller (USD 300/yr or USD 30/month) plan; redemptions count against the monthly quota ([BookFunnel help](https://authors.bookfunnel.com/help/print-codes/), [BookFunnel pricing](https://bookfunnel.com/pricing/)). Prices rose on 20 Feb 2026 from USD 20/100/250 ([CartMango](https://cartmango.com/bookfunnel-pricing/)) [V1]. Older cap of 500 print codes a month ([Indie Author Magazine](https://indieauthormagazine.com/demystifying-bookfunnel/)) [V1] | Authors pay a flat yearly fee for code issuance. No evidence that BookFunnel produces the printed card or a QR itself: that is the gap eBook Cadeau fills | [V2] |
| **Dropcards** (US) | Alive (whether it still actively serves eBooks is not verified) | Printed cards with unique codes plus a hosted redemption page ([The Book Designer](https://www.thebookdesigner.com/dropcards-for-e-books-digital-downloads-with-a-difference/)) | Plastic: USD 0.45/card at 500, USD 0.38 at 1,000; cardstock: USD 0.39 / 0.33; USD 49 set-up under 5,000 pieces ([Dropcards pricing](https://www.dropcards.com/pricing/)); 2018: USD 0.34 at 1,000, USD 0.24 at 5,000 ([Indies Unlimited](https://indiesunlimited.com/2018/10/02/ebook-gift-cards-dropcards/)) | Closest living comparable to "packs of cards for fairs and signings". Unique codes are printable at under USD 0.50 a card | [V1] |
| **Distro Print** (AU) | Alive | Full-colour unique-code download cards | AUD 239 / 295 / 345 incl. GST for 250 / 500 / 1,000 ([Distro Print](https://www.distroprint.com.au/download-cards-drop-cards)) | Second data point: about 0.35-1.00 per card depending on run length | [V1] |
| **Bookwire download codes** (DE) | Alive | Codes issued by publishers, redeemed on a Bookwire page; download link valid 28 days; the publisher sets the number of redemptions and the validity ([Bookwire FAQ](https://bookwire.net/faq-download-codes)) | Not found | The dominant DACH distributor already offers the entitlement half of the product | [V1] |

### 4.4 Lessons

1. **Stock-based retail racks failed four times.** The surviving forms are on-demand: e-mail gifting at checkout and author-printed codes. Keep the product zero-inventory and any-title. Do not put pre-printed stock for shop racks on the roadmap.
2. **Never print a price or a stored value on the object.** The card stands for one specific prepaid order, not for money. This avoids the USD 14.99 against 8.99 problem and keeps the product away from stored-value regulation.
3. **Redemption friction killed the earlier products.** They needed account creation plus code typing on a different device. The QR and the short code must land on the retailer's "add to my library" step in one or two actions.
4. **The payer who persists is the author or publisher.** Consumer willingness to pay for a title-specific card is unproven.
5. **The earlier object was weak**: a plastic card on a peg is not a present. A jacket with cover and spine that stands on a shelf is an untested differentiator, not a validated one.
6. **Dependence on one partner is fatal** (Livrada/Target, Enthrill/Walmart). The standalone mode must be viable alone.

Not reached by any search: StoryOrigin, Leanpub coupons, DigitalDownloadCards; subscription gift cards (Nextory/Youboox, BookBeat, Storytel, Kobo Plus, Audible); French precedents (Emoticourt, Short Édition, Glose); German "eBook-Karten"/epubli (one search, no hits), Papego, "Buchkarte"; corporate and works-council gifting.

---

## 5. Price benchmarks

### 5.1 What consumers pay for one personalised card (as of 2026-11)

| Service (market) | Product | Price | Postage included? | Source | Conf. |
|---|---|---|---|---|---|
| **Popcarte** (FR) | Personalised postcard, envelope included | EUR 2.89 single; EUR 2.75/unit for 8 | No; a review cites EUR 1.95 | [Popcarte](https://www.popcarte.com/carte-postale/personnalisee-destination-photos), [tariff FAQ](https://support.popcarte.com/support/solutions/articles/152068-quels-sont-nos-tarifs-), [L'Oiseau Rose](https://oiseaurose.com/popcarte-cartes-postales-personnalisees/) | [V1] |
| Popcarte | Folded 14 x 14 cm card, sent direct to the recipient | EUR 3.99 + EUR 2.95 "envoi direct" fee (1-49 cards) = **EUR 6.94**; fee falls to EUR 1.75 at 1,000+; tracking +EUR 1.50 | Yes, after the fee | [envoi direct](https://www.popcarte.com/envoi-direct-destinataire) (sibling note; page may be about 1.5 years old) | [V1] |
| Popcarte | Virtual card (e-mail) | from EUR 1.99 | n/a | [Popcarte](https://www.popcarte.com/carte-virtuelle) | [V1] |
| **Fizzer** (FR) | Postcard in envelope = 1 credit | "from EUR 2.99 all-in"; packs: 10 credits EUR 21.90, 20 EUR 41.90, 40 EUR 77.90, 80 EUR 149.90, 150 EUR 249.90 (EUR 2.19 down to EUR 1.67 per card) | Yes | [Fizzer](https://www.fizzer.com/fr/carte-postale-en-ligne/), [app page](https://www.fizzer.com/fr/application-carte-postale/), [pvt.fr](https://www.pvt.fr/code-promo/fizzer-avis-test-carte-postale-code-promo) | [V1], sources differ on the per-credit price |
| **MyPostcard** (DE, sells in FR) | Postcard | EUR 3.49 incl. production, worldwide postage, VAT | Yes | [MyPostcard FAQ](https://support.mypostcard.com/fr/support/solutions/articles/75000054648-combien-co%C3%BBte-une-carte-postale-ou-une-carte-de-voeux-), [codes-parrain](https://codes-parrain.com/voyage/mypostcard) | [V2] |
| MyPostcard | Folded greeting card with envelope | EUR 4.29 (FR FAQ), from EUR 3.99 ([App Store listing](https://apps.apple.com/sn/app/mypostcard-cartes-postales/id858686797?l=fr)), EUR 3.69-4.29 ([EN FAQ](https://support.mypostcard.com/en/support/solutions/articles/75000054648-how-much-does-mypostcard-cost-), sibling note) | Yes | same FAQ | [V1], conflicting |
| **Merci Facteur** (FR) | Folded card 15 x 21 cm closed, envelope included, sent by La Poste | EUR 2.99 + postage from EUR 1.30 = about **EUR 4.29**; tracking +EUR 0.50 | Yes, after postage | [tarifs](https://www.merci-facteur.com/tarifs.php) (sibling note) | [V2, same vendor, two pages] |
| **Moonpig** (UK) | Standard personalised card | GBP 3.99 + GBP 1.90 first class = GBP 5.89; tracked from GBP 2.79; Moonpig Plus GBP 10.99/yr | No | [TCH Works, checked 2026-10-21](https://www.tchworks.co.uk/compare/), [FindPrices](https://www.findprices.com/uk/stores/moonpig-prices), [Household Money Saving](https://www.householdmoneysaving.com/why-moonpig-plus-isnt-worth-the-money/) | [V2] |
| Online print runs (FR) | Personalised greeting cards, entry format, incl. VAT, excl. shipping | EUR 0.82-3.96/unit at 30 copies; EUR 0.61-2.40 at 100 | No | [L'Essentiel de l'Éco](https://lessentieldeleco.fr/8883-quel-est-le-meilleur-site-de-carte-de-voeux-2027/) | [V1] |
| Shop greeting card (FR) | Single card in a shop | EUR 2-5 | n/a | no national average found | [M] |

*Contradictions resolved.*

- **Ceiling for a mailed card.** The competitors note gives EUR 3.0-4.3 all-in. That range mixes flat postcards and folded cards. Sorting by product: a **flat postcard** mailed in France costs EUR 3.0-3.5 (Fizzer, MyPostcard) and up to about EUR 4.84 at Popcarte (EUR 2.89 + EUR 1.95); a **folded card** costs EUR 3.7-4.3 (MyPostcard, Merci Facteur) and EUR 6.94 at Popcarte. The jacket is a folded card. **We keep EUR 3.7 / 4.3 / 6.9 (low / base / high) as the benchmark for the product** and EUR 3.0-3.5 as the floor that mass-market apps have taught consumers.
- **Fizzer.** "From EUR 2.99" is the single-card price; EUR 1.67-2.19 is the per-card price inside prepaid packs. Both are kept.
- **MyPostcard folded card.** Three figures (EUR 3.69, 3.99, 4.29) from the vendor's own pages. We keep the range.

Those operators survive on volume, prepaid credits (Fizzer) and subscriptions (Popcarte+, Moonpig Plus). What booksellers charge for gift wrap or gift cards was not searched. No excerpt mentions a surcharge for the e-mail gift option at any eBook retailer [V1].

### 5.2 What authors and publishers pay

| Anchor | Price | Conf. |
|---|---|---|
| Unique-code card, printed, 500-1,000 pieces (Dropcards) | USD 0.33-0.45 per card + USD 49 set-up | [V1] |
| Unique-code card, 250-1,000 pieces (Distro Print) | AUD 0.35-0.96 per card | [V1] |
| Flat plan for issuing printable codes (BookFunnel) | USD 200-300 a year | [V2] |
| Folded A6-class card, 300-350 g, 100 identical copies, delivered in France (web-to-print; sibling note) | EUR 0.36-0.60 HT per card | [V1] per printer, consistent across four |

### 5.3 The eBook price as the anchor

Average price paid: France EUR 8.06 (derived) [V1], Germany EUR 7.01 [V2], Netherlands about EUR 10.5 (derived) [V1], Spain EUR 9.0 [V2]. A EUR 3-4 card adds 30-50% to the average French eBook; a EUR 6.90 card adds about 85%. The card must therefore be free (print at home), bundled by the reseller as a conversion feature, or sold as a premium object on new releases.

### 5.4 Price points to test (ASSUMPTIONS, not findings)

| Offer | Low | Base | High | Anchor |
|---|---|---|---|---|
| Print-at-home PDF | free | free | EUR 1.99 | Popcarte virtual card from EUR 1.99; card fees take 14% of EUR 1.99 |
| Mailed single card, France | EUR 4.90 | EUR 6.90 | EUR 8.90 | Folded-card benchmark EUR 3.7 / 4.3 / 6.9 |
| Author plan (codes + print-ready jackets) | EUR 29/yr | EUR 49/yr | EUR 99/yr | BookFunnel USD 200-300/yr, which also hosts and delivers the files |
| Printed jacket in a pack of 100, one title | EUR 1.50 | EUR 2.00 | EUR 2.50 | Dropcards USD 0.33-0.45 for a much smaller card |
| Reseller fee per card | EUR 0.20 | EUR 0.30 | EUR 0.50 | No public comparable found |
| Reseller flat licence | EUR 99/month | EUR 200/month | EUR 499/month | No public comparable found |

---

## 6. Cost inputs

All values are as of 2026-11 unless a date is given. They come from search-result extracts, not from the official pages read in full. **Re-check every tariff on the official page before hard-coding it.**

### 6.1 Conventions

- One row per input. The `item` cell starts with the identifier used in the JSON block.
- `value` is `low / base / high`. A single number means low = base = high.
- Rates are fractions (0.015 = 1.5%). Money is in EUR unless the unit says otherwise. Consumer prices include VAT; "HT" means excluding VAT.
- `confidence`: V2, V1, M as defined at the top; `derived` = arithmetic on other rows; `assumption` = planning value with no source. A row marked "conflicting" has sources that disagree; low and high then carry the conflicting figures.
- Where a base value is a midpoint of a range, the note says so. A midpoint is not a finding.

### 6.2 Contradictions resolved

1. **Print cost of one card.** The competitors note used a EUR 1.00 placeholder for "print". The printers notes show that mass web-to-print does not sell single copies; a single print-on-demand card costs EUR 2.4-2.8 before shipping and about EUR 6 landed; Merci Facteur delivers a folded card to a French letterbox for about EUR 4.29. **We keep EUR 1.00 (0.50-1.50) only for in-house printing, EUR 4.29 for outsourced single-card fulfilment, and EUR 0.36-0.60 HT for batches of 100 identical copies.**
2. **Stripe premium EEA cards.** 2.8% + EUR 0.25 in October 2026 comparators; 1.9% on an older Wise page. We keep 2.8% as base and high (more recent, conservative) and 1.9% as low.
3. **Mollie fixed fee.** The official excerpt showed 1.20% without a fixed part; three comparators show + EUR 0.25. We keep EUR 0.25 as base, 0 as low.
4. **Stancer.** 0.7% + EUR 0.07 or + EUR 0.15 depending on the comparator. We keep EUR 0.15 as base (conservative).
5. **VAT franchise thresholds (micro-entreprise).** EUR 85,000 / 37,500 per most sources; EUR 91,900 / 36,800 per others. We keep the first pair: it has more sources, and the second pair matches the thresholds that applied before 2025 [M].
6. **EUIPO filing fee.** EUR 850 for one class on one site, a different base on another. We keep EUR 850, which also matches the fee schedule as I remember it [M]; the marker stays [V1].
7. **OVHcloud VPS-1.** EUR 4.49 HT or EUR 7.79 after the 1 April 2026 rise. We keep EUR 7.79 as base (latest dated figure, conservative).
8. **Brevo Starter.** About EUR 7 a month (USD 9) or EUR 14. We keep EUR 7 as base because it matches the dollar list price; EUR 14 is the high value.
9. **La Poste 100 g rate for 2026.** EUR 3.10 in the extracts, which is not exactly twice the 20 g rate, whereas the 2027 rate is. Kept, flagged for re-check.
10. **Yearly fixed cash costs.** The competitors note says EUR 600-1,300. Summing its own line items gives about EUR 290-1,830 (hosting 120-300, domains 17-80, insurance 150-500, bank 0-180, CFE 0-600, e-mail 0-170). We keep 600 / 900 / 1,300 as planning scenarios and expose every line item so the model can recompute.
11. **Break-even at a EUR 0.30 B2B fee.** The note's 2,000-4,300 cards a year ignores micro-entreprise social charges (21.2% of turnover for services). With them the range is about 2,500-5,500. We keep the latter.
12. **Hetzner.** Ordering of new servers is reported as restricted since 26 June 2026, with conflicting reports on current availability [V1]. Do not make the launch depend on Hetzner.

### 6.3 Table

| item | value | unit | date | source | confidence |
|---|---|---|---|---|---|
| `postage_fr_lettre_verte_20g_2026` Lettre verte up to 20 g, 2026 (low = stamp printed online) | 1.49 / 1.52 / 1.52 | EUR per letter | 2026 | [La Poste tarifs](https://www.laposte.fr/tarifs-postaux-courrier-lettres-timbres), [Lettre verte](https://www.laposte.fr/tarif-lettre-verte), [Tout sur mes finances](https://www.toutsurmesfinances.com/argent/a/prix-du-timbre) | [V2] |
| `postage_fr_lettre_verte_20g_2027` Lettre verte up to 20 g, announced 27 July 2026 | 1.67 | EUR per letter | 2027-01-01 | [La Poste tarifs](https://www.laposte.fr/tarifs-postaux-courrier-lettres-timbres), [Lettre verte](https://www.laposte.fr/tarif-lettre-verte), [Tout sur mes finances](https://www.toutsurmesfinances.com/argent/a/prix-du-timbre) | [V2] |
| `postage_fr_lettre_verte_100g_2026` Lettre verte up to 100 g, 2026 (low = online); re-check | 2.97 / 3.10 / 3.10 | EUR per letter | 2026 | [La Poste tarifs](https://www.laposte.fr/tarifs-postaux-courrier-lettres-timbres), [Lettre verte](https://www.laposte.fr/tarif-lettre-verte), [Tout sur mes finances](https://www.toutsurmesfinances.com/argent/a/prix-du-timbre) | [V2]; not exactly twice the 20 g rate in the extracts |
| `postage_fr_lettre_verte_100g_2027` Lettre verte up to 100 g | 3.34 | EUR per letter | 2027-01-01 | [La Poste tarifs](https://www.laposte.fr/tarifs-postaux-courrier-lettres-timbres), [Lettre verte](https://www.laposte.fr/tarif-lettre-verte), [Tout sur mes finances](https://www.toutsurmesfinances.com/argent/a/prix-du-timbre) | [V2] |
| `postage_fr_lettre_suivie_20g_2026` Lettre verte suivie up to 20 g, 2026 | 2.02 | EUR per letter | 2026 | [La Poste Lettre suivie](https://www.laposte.fr/tarif-lettre-suivie) | [V2] |
| `postage_fr_lettre_suivie_20g_2027` Lettre verte suivie up to 20 g | 2.17 | EUR per letter | 2027-01-01 | [La Poste Lettre suivie](https://www.laposte.fr/tarif-lettre-suivie) | [V2] |
| `postage_fr_lettre_suivie_100g_2026` Lettre verte suivie up to 100 g, 2026 | 3.60 | EUR per letter | 2026 | [La Poste Lettre suivie](https://www.laposte.fr/tarif-lettre-suivie) | [V2] |
| `postage_fr_lettre_suivie_100g_2027` Lettre verte suivie up to 100 g | 3.84 | EUR per letter | 2027-01-01 | [La Poste Lettre suivie](https://www.laposte.fr/tarif-lettre-suivie) | [V2] |
| `postage_intl_20g_2026` Lettre internationale up to 20 g, 2026 (low = online) | 2.19 / 2.25 / 2.25 | EUR per letter | 2026 | [La Poste étranger](https://www.laposte.fr/tarifs-postaux-etranger), [tarifs-postaux.org](https://tarifs-postaux.org/tarif-lettre-internationale) | [V2] |
| `postage_intl_20g_2027` Lettre internationale up to 20 g | 2.45 | EUR per letter | 2027-01-01 | [La Poste étranger](https://www.laposte.fr/tarifs-postaux-etranger), [tarifs-postaux.org](https://tarifs-postaux.org/tarif-lettre-internationale) | [V2] |
| `postage_intl_100g_2026` Lettre internationale up to 100 g, 2026 (low = online) | 4.70 / 4.85 / 4.85 | EUR per letter | 2026 | [La Poste étranger](https://www.laposte.fr/tarifs-postaux-etranger), [tarifs-postaux.org](https://tarifs-postaux.org/tarif-lettre-internationale) | [V2] |
| `postage_intl_100g_2027` Lettre internationale up to 100 g | 5.25 | EUR per letter | 2027-01-01 | [La Poste étranger](https://www.laposte.fr/tarifs-postaux-etranger), [tarifs-postaux.org](https://tarifs-postaux.org/tarif-lettre-internationale) | [V2] |
| `postage_intl_suivie_20g_2026` Lettre suivie internationale up to 20 g, 2026 (2027 not retrieved) | 5.05 | EUR per letter | 2026 | [La Poste étranger](https://www.laposte.fr/tarifs-postaux-etranger), [tarifs-postaux.org](https://tarifs-postaux.org/tarif-lettre-internationale) | [V1] |
| `postage_intl_suivie_100g_2026` Lettre suivie internationale up to 100 g, 2026 | 7.65 | EUR per letter | 2026 | [La Poste étranger](https://www.laposte.fr/tarifs-postaux-etranger), [tarifs-postaux.org](https://tarifs-postaux.org/tarif-lettre-internationale) | [V1] |
| `postage_band_jump_2026` Extra cost of the 100 g band over the 20 g band, 2026 | 1.58 | EUR per letter | 2026 | derived (3.10 - 1.52) | derived |
| `postage_band_jump_2027` Extra cost of the 100 g band over the 20 g band, 2027 | 1.67 | EUR per letter | 2027-01-01 | derived (3.34 - 1.67) | derived |
| `postage_yearly_increase` Yearly postage increase; observed 2026 to 2027 changes range from 6.7% to 9.9%; base = rounded median | 0.067 / 0.08 / 0.099 | fraction per year | 2026 to 2027 | derived from the rows above | derived; one year of evidence only |
| `weight_letter_band_limit` Upper limit of the first letter band | 20 | g | 2026 | [La Poste tarifs](https://www.laposte.fr/tarifs-postaux-courrier-lettres-timbres), [Lettre verte](https://www.laposte.fr/tarif-lettre-verte), [Tout sur mes finances](https://www.toutsurmesfinances.com/argent/a/prix-du-timbre) | [V2] |
| `weight_jacket_area_pocket` Pocket jacket sheet, (110 + 10 + 110) x 178 mm | 0.041 | m2 | 2026-11 | derived | derived |
| `weight_jacket_area_large` Large-format jacket sheet, (135 + 10 + 135) x 215 mm | 0.06 | m2 | 2026-11 | derived | derived |
| `weight_jacket_pocket` Pocket jacket at 250 / 300 / 350 g/m2 | 10.2 / 12.3 / 14.3 | g | 2026-11 | derived; weigh real samples | derived |
| `weight_jacket_large` Large-format jacket at 250 / 300 / 350 g/m2 | 15.1 / 18.1 / 21.1 | g | 2026-11 | derived; weigh real samples | derived |
| `weight_envelope_c5` C5 envelope | 7 / 7.5 / 8 | g | 2026-11 | competitors note, section 3.1 (not sourced) | [M] |
| `print_inhouse_per_card` One jacket printed in-house (paper + toner); placeholder | 0.50 / 1.00 / 1.50 | EUR per card | 2026-11 | competitors note placeholder EUR 1.00; range is mine | assumption; needs measurement |
| `print_envelope` Envelope C5/C6, bought by 500 | 0.03 / 0.08 / 0.10 | EUR per envelope | 2026-11 | competitors note, table 3.7 | [M]; base = value used in the unit economics |
| `print_batch100_folded_card` Folded A6-class card, 300-350 g, 4/4, 100 identical copies, delivered in France; base = midpoint | 0.36 / 0.48 / 0.60 | EUR HT per card | 2026-11 | [Onlineprinters](https://www.onlineprinters.fr/p/cartes-pliables-en-papier-eco-naturel-paysage-a6), [Flyeralarm](https://www.flyeralarm.com/fr/p/cartes-de-fidelite-525038), [Manuscry on Pixartprinting](https://manuscry.com/manuscry-ou-pixartprinting) (sibling note) | [V1]; identical copies only |
| `print_batch25_50_folded_card` Same, 25-50 copies; base = midpoint | 0.50 / 1.30 / 2.10 | EUR HT per card | 2026-11 | [Realisaprint](https://www.realisaprint.com/carte-correspondance-personnalisee-r2284.html), [Rapid-Flyer](https://www.rapid-flyer.com/p/carte-de-voeux-double), [Printshot](https://www.printshot.fr/1185-flyer-2-volets) (sibling note) | [V1] |
| `print_batch1000_folded_card` Same, 1,000 copies; estimate from a flyer price floor | 0.07 / 0.10 / 0.13 | EUR HT per card | 2026-11 | [comparatif-imprimerie.fr](https://comparatif-imprimerie.fr/avis/exaprint/) (sibling note) | [M]; estimate only |
| `print_jacket_double_score_100_500` True jacket with two scores 10 mm apart, runs of 100-500; needs quotes | 0.60 / 1.00 / 1.50 | EUR per jacket | 2026-11 | competitors note, section 4 (derived band) | assumption; no quote obtained; no variable data |
| `print_pod_single_card` Single print-on-demand greeting card, before shipping | 2.36 / 2.60 / 2.82 | EUR per card | 2026-11 | [Printful](https://www.printful.com/custom/collections/made-in-eu/greeting-card), [Print&Bind](https://www.printenbind.nl/en/card/greeting-card) (sibling note) | [V1] |
| `fulfil_pod_single_card_landed` Single print-on-demand card delivered (Printful 2.36 + 3.49 shipping) | 5.85 / 6.00 / 6.00 | EUR per card | 2026-11 | [Printful](https://www.printful.com/custom/collections/made-in-eu/greeting-card), [Print&Bind](https://www.printenbind.nl/en/card/greeting-card) (sibling note) | [V1]; estimate |
| `fulfil_postal_api_folded_card` Merci Facteur folded card 15 x 21 cm, envelope included | 2.99 | EUR per card | 2026-11 | [Merci Facteur tarifs](https://www.merci-facteur.com/tarifs.php) (sibling note) | [V2]; same vendor, two pages |
| `fulfil_postal_api_postage_fr` Merci Facteur postage to France, 'from' | 1.30 | EUR per card | 2026-11 | [Merci Facteur tarifs](https://www.merci-facteur.com/tarifs.php) (sibling note) | [V2] |
| `fulfil_postal_api_tracking` Merci Facteur tracking option | 0.50 | EUR per card | 2026-11 | [Merci Facteur tarifs](https://www.merci-facteur.com/tarifs.php) (sibling note) | [V2] |
| `fulfil_postal_api_landed` Folded card printed and posted to a French address (high = with tracking) | 4.29 / 4.29 / 4.79 | EUR per card | 2026-11 | [Merci Facteur tarifs](https://www.merci-facteur.com/tarifs.php) (sibling note) | derived; VAT status unknown |
| `fulfil_postal_api_plan` Merci Facteur API plan (high = above about 2,000 cards a month) | 19.95 / 19.95 / 39.95 | EUR per month | 2026-11 | [Merci Facteur API](https://www.merci-facteur.com/pro/lp/api/api-carte-postale.php), [Manuscry comparison](https://manuscry.com/manuscry-ou-merci-facteur) (sibling note) | [V2] |
| `fulfil_postal_api_plan_threshold` Monthly volume above which the higher API plan applies | 2000 | cards per month | 2026-11 | [Merci Facteur API](https://www.merci-facteur.com/pro/lp/api/api-carte-postale.php), [Manuscry comparison](https://manuscry.com/manuscry-ou-merci-facteur) (sibling note) | [V1]; approximate |
| `fulfil_maileva_letter_landed` Plain A4 colour letter printed and posted (fallback, not a gift object); base = midpoint | 1.00 / 1.45 / 1.90 | EUR HT per letter | 2026-01-01 | [Maileva](https://www.maileva.com/nos-solutions/courrier-en-ligne/) (sibling note) | [V1]; estimate |
| `fulfil_dummy_book_landed` Print-on-demand paperback with a 10 mm spine, one copy to France; base = midpoint | 9.00 / 10.50 / 12.00 | EUR per copy | 2026-11 | [bod.fr](https://www.bod.fr/prix), [Lulu help](https://help.api.lulu.com/en/support/solutions/articles/64000254631-are-there-fees-to-use-lulu-s-print-api-) (sibling note) | [V1]; estimate |
| `pay_stripe_eea_pct` Stripe, standard EEA consumer card, variable part | 0.015 | fraction | 2026-10 | [MargeDaily](https://margedaily.com/blog/frais-stripe/), [Vincent Hego](https://www.vincenthego.com/calculateur-frais-stripe/), [FindMyCompany](https://findmycompany.ai/en/guides/stripe-fees) | [V2] |
| `pay_stripe_fixed` Stripe, fixed part per transaction | 0.25 | EUR per transaction | 2026-10 | [MargeDaily](https://margedaily.com/blog/frais-stripe/), [Vincent Hego](https://www.vincenthego.com/calculateur-frais-stripe/), [FindMyCompany](https://findmycompany.ai/en/guides/stripe-fees) | [V2] |
| `pay_stripe_eea_premium_pct` Stripe, premium or corporate EEA card (conflicting: 1.9% vs 2.8%) | 0.019 / 0.028 / 0.028 | fraction | 2026-10 | [MargeDaily](https://margedaily.com/blog/frais-stripe/), [Vincent Hego](https://www.vincenthego.com/calculateur-frais-stripe/), [FindMyCompany](https://findmycompany.ai/en/guides/stripe-fees), [Wise](https://wise.com/fr/blog/stripe-carte-bancaire-paiement) | [V1]; conflicting |
| `pay_stripe_uk_pct` Stripe, UK card | 0.025 | fraction | 2026-10 | [MargeDaily](https://margedaily.com/blog/frais-stripe/), [Vincent Hego](https://www.vincenthego.com/calculateur-frais-stripe/), [FindMyCompany](https://findmycompany.ai/en/guides/stripe-fees) | [V1] |
| `pay_stripe_non_eea_pct` Stripe, non-EEA card (conflicting: 3.15% vs 3.25%) | 0.0315 / 0.0315 / 0.0325 | fraction | 2026-10 | [MargeDaily](https://margedaily.com/blog/frais-stripe/), [Vincent Hego](https://www.vincenthego.com/calculateur-frais-stripe/), [FindMyCompany](https://findmycompany.ai/en/guides/stripe-fees) | [V1]; conflicting |
| `pay_stripe_fx_pct` Stripe, currency conversion surcharge | 0.02 | fraction | 2026-10 | [MargeDaily](https://margedaily.com/blog/frais-stripe/), [Vincent Hego](https://www.vincenthego.com/calculateur-frais-stripe/), [FindMyCompany](https://findmycompany.ai/en/guides/stripe-fees) | [V1] |
| `pay_stripe_dispute_fee` Stripe, fee per dispute | 20 | EUR per dispute | 2026-10 | [MargeDaily](https://margedaily.com/blog/frais-stripe/), [Vincent Hego](https://www.vincenthego.com/calculateur-frais-stripe/), [FindMyCompany](https://findmycompany.ai/en/guides/stripe-fees) | [V1] |
| `pay_mollie_fr_pct` Mollie, domestic consumer card (Cartes Bancaires) | 0.012 | fraction | 2026-11 | [Mollie](https://www.mollie.com/pricing), [Ecomstream](https://www.ecomstream.eu/mollie-fees/), [Captain Freelance](https://www.captainfreelance.com/avis/mollie), [Quasa](https://quasa.io/fr/media/stripe-ou-mollie-en-france-le-moyen-de-paiement-change-la-facture) | [V2] |
| `pay_mollie_fixed` Mollie, fixed part (official extract showed none) | 0.00 / 0.25 / 0.25 | EUR per transaction | 2026-11 | [Mollie](https://www.mollie.com/pricing), [Ecomstream](https://www.ecomstream.eu/mollie-fees/), [Captain Freelance](https://www.captainfreelance.com/avis/mollie), [Quasa](https://quasa.io/fr/media/stripe-ou-mollie-en-france-le-moyen-de-paiement-change-la-facture) | [V1]; conflicting |
| `pay_mollie_eu_pct` Mollie, EU consumer card | 0.018 | fraction | 2026-11 | [Mollie](https://www.mollie.com/pricing), [Ecomstream](https://www.ecomstream.eu/mollie-fees/), [Captain Freelance](https://www.captainfreelance.com/avis/mollie), [Quasa](https://quasa.io/fr/media/stripe-ou-mollie-en-france-le-moyen-de-paiement-change-la-facture) | [V1] |
| `pay_mollie_commercial_pct` Mollie, commercial card | 0.029 | fraction | 2026-11 | [Mollie](https://www.mollie.com/pricing), [Ecomstream](https://www.ecomstream.eu/mollie-fees/), [Captain Freelance](https://www.captainfreelance.com/avis/mollie), [Quasa](https://quasa.io/fr/media/stripe-ou-mollie-en-france-le-moyen-de-paiement-change-la-facture) | [V1] |
| `pay_mollie_non_eu_pct` Mollie, non-EU card (base = high, conservative) | 0.028 / 0.0325 / 0.0325 | fraction | 2026-11 | [Mollie](https://www.mollie.com/pricing), [Ecomstream](https://www.ecomstream.eu/mollie-fees/), [Captain Freelance](https://www.captainfreelance.com/avis/mollie), [Quasa](https://quasa.io/fr/media/stripe-ou-mollie-en-france-le-moyen-de-paiement-change-la-facture) | [V1] |
| `pay_mollie_plan` Mollie optional plan for volumes above EUR 1,900 a month | 20 | EUR per month | 2026-11 | [Mollie](https://www.mollie.com/pricing), [Ecomstream](https://www.ecomstream.eu/mollie-fees/), [Captain Freelance](https://www.captainfreelance.com/avis/mollie), [Quasa](https://quasa.io/fr/media/stripe-ou-mollie-en-france-le-moyen-de-paiement-change-la-facture) | [V1]; optional |
| `pay_sumup_online_pct` SumUp online, percentage only | 0.025 | fraction | 2026-11 | [Meilleur e-commerce](https://meilleur-ecommerce.com/frais-de-paiement-en-ligne/), [MobileTransaction](https://fr.mobiletransaction.org/paiement-en-ligne-meilleures-solutions/) | [V1] |
| `pay_sumup_inperson_pct` SumUp in person (fairs, signings) | 0.0175 | fraction | 2026-11 | [Meilleur e-commerce](https://meilleur-ecommerce.com/frais-de-paiement-en-ligne/), [MobileTransaction](https://fr.mobiletransaction.org/paiement-en-ligne-meilleures-solutions/) | [V1] |
| `pay_stancer_pct` Stancer, variable part | 0.007 | fraction | 2026-11 | [Digitiz](https://digitiz.fr/solutions-paiement-en-ligne/), [Independant.io](https://independant.io/solution-paiement-en-ligne/), [Comparateur-pro](https://www.comparateur-pro.fr/comparatifs/meilleure-solution-paiement-en-ligne) | [V1] |
| `pay_stancer_fixed` Stancer, fixed part (conflicting: 0.07 vs 0.15) | 0.07 / 0.15 / 0.15 | EUR per transaction | 2026-11 | [Digitiz](https://digitiz.fr/solutions-paiement-en-ligne/), [Independant.io](https://independant.io/solution-paiement-en-ligne/), [Comparateur-pro](https://www.comparateur-pro.fr/comparatifs/meilleure-solution-paiement-en-ligne) | [V1]; conflicting |
| `pay_gocardless_pct` GoCardless SEPA direct debit, variable part | 0.01 | fraction | 2026-11 | [Digitiz](https://digitiz.fr/solutions-paiement-en-ligne/), [Independant.io](https://independant.io/solution-paiement-en-ligne/), [Comparateur-pro](https://www.comparateur-pro.fr/comparatifs/meilleure-solution-paiement-en-ligne) | [V1] |
| `pay_gocardless_fixed` GoCardless, fixed part | 0.20 | EUR per transaction | 2026-11 | [Digitiz](https://digitiz.fr/solutions-paiement-en-ligne/), [Independant.io](https://independant.io/solution-paiement-en-ligne/), [Comparateur-pro](https://www.comparateur-pro.fr/comparatifs/meilleure-solution-paiement-en-ligne) | [V1] |
| `pay_gocardless_cap` GoCardless, fee cap | 2.00 | EUR per transaction | 2026-11 | [Digitiz](https://digitiz.fr/solutions-paiement-en-ligne/), [Independant.io](https://independant.io/solution-paiement-en-ligne/), [Comparateur-pro](https://www.comparateur-pro.fr/comparatifs/meilleure-solution-paiement-en-ligne) | [V1] |
| `pay_model_card_pct` Model default, card variable part (Mollie domestic / Stripe standard / Stripe premium) | 0.012 / 0.015 / 0.028 | fraction | 2026-11 | derived from the rows above | derived |
| `pay_model_card_fixed` Model default, card fixed part (low = Stancer) | 0.07 / 0.25 / 0.25 | EUR per transaction | 2026-11 | derived from the rows above | derived |
| `host_scaleway_dev1s_instance` Scaleway DEV1-S (2 vCPU, 2 GB), instance only, after the 1 June 2026 increase | 6.55 | EUR per month | 2026-06-01 | [Scaleway](https://www.scaleway.com/en/pricing/virtual-instances/), [EU Cloud Cost](https://www.eucloudcost.com/providers/scaleway/), [AgentX](https://agentxcloud.com/news/scaleway-june-2026-pricing-update) | [V2] |
| `host_scaleway_dev1s_hour` Scaleway DEV1-S hourly price | 0.00898 | EUR per hour | 2026-06-01 | [Scaleway](https://www.scaleway.com/en/pricing/virtual-instances/), [EU Cloud Cost](https://www.eucloudcost.com/providers/scaleway/), [AgentX](https://agentxcloud.com/news/scaleway-june-2026-pricing-update) | [V2] |
| `host_scaleway_dev1s_all_in` Scaleway DEV1-S with 20 GB disk and a public IPv4 | 10.90 | EUR per month | 2026-06-01 | [Scaleway](https://www.scaleway.com/en/pricing/virtual-instances/), [EU Cloud Cost](https://www.eucloudcost.com/providers/scaleway/), [AgentX](https://agentxcloud.com/news/scaleway-june-2026-pricing-update) | [V2] |
| `host_hetzner_cx23` Hetzner CX23, excl. IPv4, after two 2026 increases; availability uncertain | 5.49 | EUR per month | 2026-06-15 | [Better Stack](https://betterstack.com/community/guides/web-servers/hetzner-cloud-review/), [Bitdoze](https://www.bitdoze.com/hetzner-cloud-cost-optimized-plans/), [CloudTally](https://cloudtally.eu/hetzner-cloud-prices), [AgentDeals](https://agentdeals.dev/hetzner-pricing-2026) | [V2]; new orders reported restricted since 2026-06-26 |
| `host_hetzner_object_storage_base` Hetzner Object Storage base fee | 6.49 | EUR per month | 2026-04 | [Better Stack](https://betterstack.com/community/guides/web-servers/hetzner-cloud-review/), [Bitdoze](https://www.bitdoze.com/hetzner-cloud-cost-optimized-plans/), [CloudTally](https://cloudtally.eu/hetzner-cloud-prices), [AgentDeals](https://agentdeals.dev/hetzner-pricing-2026) | [V2] |
| `host_hetzner_egress_per_tb` Hetzner extra egress | 1.00 | EUR per TB | 2026-11 | [Better Stack](https://betterstack.com/community/guides/web-servers/hetzner-cloud-review/), [Bitdoze](https://www.bitdoze.com/hetzner-cloud-cost-optimized-plans/), [CloudTally](https://cloudtally.eu/hetzner-cloud-prices), [AgentDeals](https://agentdeals.dev/hetzner-pricing-2026) | [V2]; approximate |
| `host_ovh_vps1` OVHcloud VPS-1 (conflicting: 4.49 HT vs 7.79 after 1 April 2026) | 4.49 / 7.79 / 7.79 | EUR per month | 2026-04-01 | [Valebyte](https://valebyte.com/en/providers/ovh/), [Repha](https://repha.fr/tarification-vps-2026/), [hebergement-vps.fr](https://www.hebergement-vps.fr/test/ovhcloud), [VPSBenchmarks](https://www.vpsbenchmarks.com/hosters/ovhcloud/plans/vps-1) | [V1]; conflicting |
| `host_object_storage_per_gb` Scaleway Object Storage | 0.01 | EUR per GB per month | 2026-11 | [Cloud Price Calculators](https://cloudpricecalculators.com/scaleway-calculator/) | [V1]; one outlier source disagrees |
| `host_egress_free` Scaleway free egress allowance | 75 | GB per month | 2026-11 | [Cloud Price Calculators](https://cloudpricecalculators.com/scaleway-calculator/) | [V1] |
| `host_egress_per_gb` Scaleway egress beyond the allowance | 0.01 | EUR per GB | 2026-11 | [Cloud Price Calculators](https://cloudpricecalculators.com/scaleway-calculator/) | [V1] |
| `host_total_launch` Small API + object storage at launch, any of the three hosts | 10 / 15 / 25 | EUR per month | 2026-11 | derived from the rows above | derived; budget the high value |
| `mail_scaleway_tem_free` Scaleway TEM free allowance | 300 | e-mails | 2026-11 | [Scaleway managed services](https://www.scaleway.com/en/pricing/managed-services/), [StackRates](https://stackrates.ai/hosting/scaleway/) | [V1] |
| `mail_scaleway_tem_per_1000` Scaleway TEM beyond the allowance | 0.25 | EUR per 1,000 e-mails | 2026-11 | [Scaleway managed services](https://www.scaleway.com/en/pricing/managed-services/), [StackRates](https://stackrates.ai/hosting/scaleway/) | [V1] |
| `mail_scaleway_tem_scale_plan` Scaleway TEM 'Scale' plan, 100,000 e-mails, dedicated IP | 80 | EUR per month | 2026-11 | [Scaleway managed services](https://www.scaleway.com/en/pricing/managed-services/), [StackRates](https://stackrates.ai/hosting/scaleway/) | [V1] |
| `mail_brevo_free_per_day` Brevo free plan cap | 300 | e-mails per day | 2026-11 | [Kanal](https://getkanal.com/blog/brevo-pricing), [Agence SW](https://www.agencesw.com/actu/marketingdigital/tarifs-brevo/), [MSM Web Solutions](https://www.msm-web-solutions.com/brevo-email-marketing/) | [V2] |
| `mail_brevo_starter` Brevo Starter, 5,000 e-mails (conflicting: 7 vs 14) | 7 / 7 / 14 | EUR per month | 2026-11 | [Kanal](https://getkanal.com/blog/brevo-pricing), [Agence SW](https://www.agencesw.com/actu/marketingdigital/tarifs-brevo/), [MSM Web Solutions](https://www.msm-web-solutions.com/brevo-email-marketing/) | [V1]; conflicting |
| `mail_emails_per_paid_order` Transactional e-mails per paid order (none in print-at-home mode) | 0 / 2 / 3 | e-mails per order | 2026-11 | none | assumption |
| `tax_micro_social_goods` Micro-entreprise social charges, sale of goods | 0.123 | fraction of turnover | 2026 | [L'Expert-comptable](https://www.l-expert-comptable.com/a/52319-le-regime-de-la-micro-entreprise-pour-l-entreprise-individuelle.html), [Abby](https://abby.fr/guide/micro-entreprise/cotisations-sociales-auto-entrepreneur), [Astrolabe](https://www.astrolabe-conseil.fr/micro-entreprise-taux-2026/) | [V2]; which lines count as goods is [M] |
| `tax_micro_social_services_bic` Micro-entreprise social charges, commercial services (BIC) | 0.212 | fraction of turnover | 2026 | [L'Expert-comptable](https://www.l-expert-comptable.com/a/52319-le-regime-de-la-micro-entreprise-pour-l-entreprise-individuelle.html), [Abby](https://abby.fr/guide/micro-entreprise/cotisations-sociales-auto-entrepreneur), [Astrolabe](https://www.astrolabe-conseil.fr/micro-entreprise-taux-2026/) | [V2] |
| `tax_micro_social_services_bnc` Micro-entreprise social charges, liberal services (BNC; 24.6% in 2025) | 0.256 | fraction of turnover | 2026 | [L'Expert-comptable](https://www.l-expert-comptable.com/a/52319-le-regime-de-la-micro-entreprise-pour-l-entreprise-individuelle.html), [Abby](https://abby.fr/guide/micro-entreprise/cotisations-sociales-auto-entrepreneur), [Astrolabe](https://www.astrolabe-conseil.fr/micro-entreprise-taux-2026/) | [V2] |
| `tax_micro_ceiling_goods` Micro-entreprise turnover ceiling, goods | 203,100 | EUR per year | 2026 | [Entreprisma](https://entreprisma.fr/micro-entrepreneur), [Developp](https://developp.fr/micro-entreprise-plafonds/) | [V1]; one outlier source |
| `tax_micro_ceiling_services` Micro-entreprise turnover ceiling, services | 83,600 | EUR per year | 2026 | [Entreprisma](https://entreprisma.fr/micro-entrepreneur), [Developp](https://developp.fr/micro-entreprise-plafonds/) | [V1]; one outlier source |
| `tax_vat_franchise_goods` VAT franchise threshold, goods (conflicting: 85,000 vs 91,900) | 85,000 / 85,000 / 91,900 | EUR per year | 2026 | [Entreprisma](https://entreprisma.fr/micro-entrepreneur), [Developp](https://developp.fr/micro-entreprise-plafonds/) | [V1]; conflicting |
| `tax_vat_franchise_services` VAT franchise threshold, services (conflicting: 37,500 vs 36,800) | 36,800 / 37,500 / 37,500 | EUR per year | 2026 | [Entreprisma](https://entreprisma.fr/micro-entrepreneur), [Developp](https://developp.fr/micro-entreprise-plafonds/) | [V1]; conflicting |
| `tax_vat_card_fr` VAT on the printed card and postage charged, France | 0.20 | fraction | 2026 | sibling legal note | [M] |
| `tax_vat_ebook_fr` VAT on the eBook, France | 0.055 | fraction | 2026 | sibling legal note | [M] |
| `tax_vat_card_de` VAT on the printed card, Germany | 0.19 | fraction | 2026 | sibling legal note | [M] |
| `tax_vat_ebook_de` VAT on the eBook, Germany | 0.07 | fraction | 2026 | sibling legal note | [M] |
| `tax_eu_oss_threshold` EU-wide yearly threshold for cross-border B2C sales before destination VAT (OSS) | 10,000 | EUR per year | 2026 | sibling legal note | [M] |
| `tax_sasu_salary_cost_multiplier` SASU president: total cost divided by net salary | 1.82 | ratio | 2026 | [LS Compta](https://www.lscompta.fr/blog/urssaf-sasu/), [Wisestart](https://wisestart.fr/guides/fiscalite/cotisations-sociales-president-sasu/), [NGT Conseil](https://www.ngt-conseil.fr/salaire-ou-dividendes---quelle-remuneration-pour-un-dirigeant-de-sasu-en-2026--_ad41.html) | [V2] |
| `tax_salary_total_cost_3000_net` Total monthly cost of EUR 3,000 net for a SASU president | 5275 | EUR per month | 2026 | [LS Compta](https://www.lscompta.fr/blog/urssaf-sasu/), [Wisestart](https://wisestart.fr/guides/fiscalite/cotisations-sociales-president-sasu/), [NGT Conseil](https://www.ngt-conseil.fr/salaire-ou-dividendes---quelle-remuneration-pour-un-dirigeant-de-sasu-en-2026--_ad41.html) | [V2] |
| `tax_flat_tax_dividends` Flat tax on dividends, 2026 (12.8% + 18.6%; was 30%) | 0.314 | fraction | 2026 | [LS Compta](https://www.lscompta.fr/blog/urssaf-sasu/), [Wisestart](https://wisestart.fr/guides/fiscalite/cotisations-sociales-president-sasu/), [NGT Conseil](https://www.ngt-conseil.fr/salaire-ou-dividendes---quelle-remuneration-pour-un-dirigeant-de-sasu-en-2026--_ad41.html) | [V2] |
| `tax_corporate_reduced_rate` Corporate tax, reduced rate on the first EUR 42,500 of profit | 0.15 | fraction | 2026 | none | [M] |
| `tax_corporate_standard_rate` Corporate tax, standard rate | 0.25 | fraction | 2026 | none | [M] |
| `ip_inpi_filing_one_class` INPI trade mark filing, one class, electronic | 190 | EUR one-off | 2026 | [LegalPlace](https://www.legalplace.fr/guides/prix-depot-marque-inpi/), [Legalstart](https://www.legalstart.fr/fiches-pratiques/depot-marque/cout-depot-marque/), [Kohen Avocats](https://kohenavocats.com/depot-marque-inpi-prix-cout-taxes-classes-recours-2026/) | [V2] |
| `ip_inpi_extra_class` INPI, each additional class | 40 | EUR per class | 2026 | [LegalPlace](https://www.legalplace.fr/guides/prix-depot-marque-inpi/), [Legalstart](https://www.legalstart.fr/fiches-pratiques/depot-marque/cout-depot-marque/), [Kohen Avocats](https://kohenavocats.com/depot-marque-inpi-prix-cout-taxes-classes-recours-2026/) | [V2] |
| `ip_inpi_renewal_one_class` INPI renewal, one class (every 10 years) | 290 | EUR per renewal | 2026 | [LegalPlace](https://www.legalplace.fr/guides/prix-depot-marque-inpi/), [Legalstart](https://www.legalstart.fr/fiches-pratiques/depot-marque/cout-depot-marque/), [Kohen Avocats](https://kohenavocats.com/depot-marque-inpi-prix-cout-taxes-classes-recours-2026/) | [V2] |
| `ip_euipo_filing_one_class` EUIPO filing, one class (another site gives a different base) | 850 | EUR one-off | 2026 | [Elly Assurance](https://www.elly-assurance.fr/actualites/inpi/), [ValorUp](https://valorup.fr/blog/deposer-une-marque-prix/) | [V1]; conflicting |
| `legal_docs_lawyer` CGV and privacy documents drafted by a lawyer; base = midpoint | 500 / 1000 / 1500 | EUR one-off | 2026-11 | competitors note, table 3.7 | [M] |
| `legal_docs_templates` Same, from templates; base = midpoint | 0 / 100 / 200 | EUR one-off | 2026-11 | competitors note, table 3.7 | [M] |
| `fix_domain` .fr or .eu domain; base = rounded midpoint | 7 / 12 / 20 | EUR per year | 2026-11 | competitors note, table 3.7 | [M] |
| `fix_short_domain` Short domain for the typed code (ordinary name); base = rounded midpoint | 10 / 30 / 60 | EUR per year | 2026-11 | competitors note, table 3.7 | [M]; premium 2-3 letter names cost far more |
| `fix_rc_pro` Professional liability insurance, digital micro-business; base = rounded midpoint | 150 / 300 / 500 | EUR per year | 2026-11 | competitors note, table 3.7 | [M] |
| `fix_accountant_company` Online accountant for a SASU (not needed for a micro-entreprise); base = midpoint | 1000 / 1400 / 1800 | EUR HT per year | 2026-11 | competitors note, table 3.7 | [M] |
| `fix_bank` Business bank account; base = rounded midpoint | 0 / 8 / 15 | EUR per month | 2026-11 | competitors note, table 3.7 | [M] |
| `fix_cfe_year1` CFE in the year of creation | 0 | EUR per year | 2026-11 | competitors note, table 3.7 | [M] |
| `fix_cfe_from_year2` CFE minimum from the second year, by commune; base = midpoint | 200 / 400 / 600 | EUR per year | 2026-11 | competitors note, table 3.7 | [M] |
| `fix_fonts_ofl` Fonts under the SIL Open Font License | 0 | EUR | 2026-11 | competitors note, table 3.7 | [M]; commercial font licences are quoted case by case |
| `fix_lean_cash_costs` Yearly cash costs excluding the owner's time (planning scenarios; see 6.2 item 10) | 600 / 900 / 1300 | EUR per year | 2026-11 | competitors note, section 4; base is mine | derived; line items sum to about 290-1,830 |
| `labour_dev_day_rate` Freelance developer day rate, France; base = midpoint | 400 / 500 / 600 | EUR per day | 2026-11 | competitors note, table 3.7 | [M] |
| `labour_support_minutes_per_incident` Support time per order with an incident; base = midpoint | 3 / 4 / 5 | minutes | 2026-11 | competitors note, table 3.7 | assumption |
| `labour_incident_rate` Share of orders with an incident; base = midpoint | 0.05 / 0.075 / 0.10 | fraction of orders | 2026-11 | competitors note, table 3.7 | assumption |
| `labour_inhouse_minutes_per_card` Handling time per mailed card printed in-house | 3 / 5 / 8 | minutes per card | 2026-11 | none | assumption |
| `bench_ebook_price_fr` Average price paid for an eBook, France 2025 | 8.06 | EUR | 2025 | section 2.1 | [V1]; derived from EUR 137 M / 17 M |
| `bench_ebook_price_de` Average price paid for an eBook, Germany 2025 | 7.01 | EUR | 2025 | section 2.1 | [V2] |
| `bench_ebook_price_nl` Average price paid for an eBook, Netherlands 2025 | 10.50 | EUR | 2025 | section 2.1 | [V1]; derived |
| `bench_ebook_price_es` Average price of an eBook, Spain 2024 | 9.00 | EUR | 2024 | section 2.1 | [V2] |
| `bench_ebook_price_it` Average price of an eBook, Italy | 8.00 | EUR | 2025 | none | assumption |
| `bench_mailed_postcard_fr` Flat personalised postcard mailed in France (Fizzer / MyPostcard / Popcarte 2.89 + 1.95) | 2.99 / 3.49 / 4.84 | EUR all-in | 2026-11 | section 5.1 | [V1] |
| `bench_mailed_folded_card_fr` Folded personalised card mailed in France (MyPostcard / Merci Facteur / Popcarte) | 3.69 / 4.29 / 6.94 | EUR all-in | 2026-11 | section 5.1 | [V1] |
| `bench_fizzer_pack_per_card` Fizzer per-card price inside prepaid packs of 150 down to 10 | 1.67 / 2.19 / 2.19 | EUR per card | 2026-11 | section 5.1 | [V1] |
| `bench_virtual_card` Popcarte virtual card, 'from' | 1.99 | EUR | 2026-11 | section 5.1 | [V1] |
| `bench_moonpig_uk` Moonpig standard card + first-class postage | 5.89 | GBP | 2026-10-21 | section 5.1 | [V2] |
| `bench_print_run_30_ttc` Personalised greeting cards, 30 copies, incl. VAT, excl. shipping; base = midpoint | 0.82 / 2.39 / 3.96 | EUR per card | 2026-11 | section 5.1 | [V1] |
| `bench_print_run_100_ttc` Same, 100 copies; base = midpoint | 0.61 / 1.50 / 2.40 | EUR per card | 2026-11 | section 5.1 | [V1] |
| `bench_dropcards_per_card` Dropcards unique-code card, cardstock at 1,000 / cardstock at 500 / plastic at 500 | 0.33 / 0.39 / 0.45 | USD per card | 2026-11 | section 5.2 | [V1] |
| `bench_dropcards_setup` Dropcards set-up fee under 5,000 pieces | 49 | USD one-off | 2026-11 | section 5.2 | [V1] |
| `bench_distroprint_per_card` Distro Print unique-code card at 1,000 / 500 / 250, incl. GST | 0.345 / 0.59 / 0.956 | AUD per card | 2026-11 | section 5.2 | [V1] |
| `bench_bookfunnel_plan` BookFunnel plans that allow print codes (Mid-List / Bestseller) | 200 / 200 / 300 | USD per year | 2026-02-20 | section 5.2 | [V2] |
| `price_mailed_card` Mailed single card, France, incl. VAT | 4.90 / 6.90 / 8.90 | EUR per card | 2026-11 | section 5.4 | assumption; to test |
| `price_paid_pdf` Print-at-home PDF (base = free) | 0.00 / 0.00 / 1.99 | EUR per card | 2026-11 | section 5.4 | assumption; to test |
| `price_author_plan` Author plan: code batches + print-ready jackets | 29 / 49 / 99 | EUR per year | 2026-11 | section 5.4 | assumption; to test |
| `price_author_jacket_pack100` Printed jacket in a pack of 100, one title | 1.50 / 2.00 / 2.50 | EUR per jacket | 2026-11 | section 5.4 | assumption; to test |
| `price_reseller_fee_per_card` Reseller fee per card generated | 0.20 / 0.30 / 0.50 | EUR per card | 2026-11 | section 5.4 | assumption; to test |
| `price_reseller_licence` Reseller flat licence | 99 / 200 / 499 | EUR per month | 2026-11 | section 5.4 | assumption; to test |
| `price_affiliate_per_gifted_ebook` Affiliate commission on a EUR 10 eBook; base = midpoint | 0.30 / 0.40 / 0.50 | EUR per eBook | 2026-11 | competitors note, section 4 | [M]; not searched |
| `demand_ebook_units_fr` Paid consumer eBook units, France 2025 (A1) | 17,000,000 | units per year | 2025 | section 2 | [V1] |
| `demand_ebook_value_fr` Consumer eBook sales, France 2025 | 137,000,000 | EUR per year | 2025 | section 2 | [V1] |
| `demand_ebook_units_5c` Paid consumer eBook units, FR+DE+NL+ES+IT (A1') | 87,500,000 | units per year | 2024-2025 | section 2 | derived; order of magnitude |
| `demand_gift_share` Share of eBook units that could be gifts (A2) | 0.03 / 0.06 / 0.12 | fraction | 2026-11 | section 2 | assumption |
| `demand_non_amazon_share` Share of units sold outside Amazon (A3) | 0.40 / 0.50 / 0.60 | fraction | 2026-11 | section 2 | assumption |
| `demand_physical_token_share` Share of gift occasions wanting a physical token (A4) | 0.50 / 0.65 / 0.80 | fraction | 2026-11 | section 2 | assumption |
| `demand_capture_fr_year3` Capture of SAM by year 3, France (A5) | 0.03 / 0.08 / 0.15 | fraction | 2026-11 | section 2 | assumption |
| `demand_capture_5c_year5` Capture of SAM by year 5, five countries (A5') | 0.02 / 0.05 / 0.10 | fraction | 2026-11 | section 2 | assumption |
| `demand_tam_fr` TAM France | 510,000 / 1,020,000 / 2,040,000 | cards per year | 2026-11 | section 2 | derived |
| `demand_sam_fr` SAM France | 102,000 / 331,500 / 979,200 | cards per year | 2026-11 | section 2 | derived |
| `demand_som_fr_year3` SOM France, year 3 | 3060 / 26,520 / 146,880 | cards per year | 2026-11 | section 2 | derived |
| `demand_tam_5c` TAM five countries | 2,625,000 / 5,250,000 / 10,500,000 | cards per year | 2026-11 | section 2 | derived |
| `demand_sam_5c` SAM five countries | 525,000 / 1,706,250 / 5,040,000 | cards per year | 2026-11 | section 2 | derived |
| `demand_som_5c_year5` SOM five countries, year 5 | 10,500 / 85,313 / 504,000 | cards per year | 2026-11 | section 2 | derived |
| `demand_author_cards_fr` Author / signing segment, France; base = rounded midpoint | 10,000 / 25,000 / 50,000 | cards per year | 2026-11 | section 2 | assumption; outside the funnel |
| `demand_attach_rate` Gift option attach rate at an eBook checkout | 0.005 / 0.02 / 0.05 | fraction of eBook orders | 2026-11 | none found | assumption |
| `demand_mailed_upgrade_share` Share of generated cards upgraded to the mailed option | 0.02 / 0.05 / 0.10 | fraction of cards | 2026-11 | none | assumption; used in section 7.4 at 0.05 |
| `demand_unredeemed_share` Share of gift codes never redeemed; base = midpoint | 0.10 / 0.15 / 0.20 | fraction of codes | 2026-11 | section 3.3 | assumption |
| `demand_peak_share` Share of yearly cards between 20 November and 24 December | 0.45 / 0.50 / 0.55 | fraction | 2026-11 | section 2.5 | assumption |
| `demand_digital_readers_fr` People who read a digital book in 2025, France | 14,000,000 / 14,000,000 / 14,400,000 | people | 2025 | section 2 | [V2]; high = secondary summary |
| `demand_ereader_users_fr` People reading on an e-reader, France (34% of digital readers) | 4,760,000 / 4,900,000 / 4,900,000 | people | 2025 | section 2 | derived |
| `demand_paying_buyers_fr` Regular paying eBook buyers, France | 2,000,000 / 3,000,000 / 4,000,000 | people | 2026-11 | section 2 | assumption |
| `demand_device_share_smartphone` Digital readers using a smartphone | 0.48 | fraction | 2025 | section 2 | [V1] |
| `demand_device_share_ereader` Digital readers using an e-reader | 0.34 | fraction | 2025 | section 2 | [V1] |
| `demand_device_share_tablet` Digital readers using a tablet | 0.33 | fraction | 2025 | section 2 | [V1] |

### 6.4 Machine-readable block

One entry per row of the table above, same identifiers. `low <= base <= high` holds for every entry.

```json
{
  "meta": {
    "document": "docs/research/12-market-and-business-inputs.md",
    "research_date": "2026-11-12",
    "currency": "EUR unless the unit says otherwise",
    "rates": "fractions, 0.015 means 1.5%",
    "confidence_values": ["V2", "V1", "M", "derived", "assumption"],
    "warning": "values come from search-result extracts; re-check official tariffs before use"
  },
  "cost_inputs": {
    "postage_fr_lettre_verte_20g_2026": {"low": 1.49, "base": 1.52, "high": 1.52, "unit": "EUR per letter", "as_of": "2026", "confidence": "V2", "group": "postage", "label": "Lettre verte up to 20 g, 2026 (low = stamp printed online)"},
    "postage_fr_lettre_verte_20g_2027": {"low": 1.67, "base": 1.67, "high": 1.67, "unit": "EUR per letter", "as_of": "2027-01-01", "confidence": "V2", "group": "postage", "label": "Lettre verte up to 20 g, announced 27 July 2026"},
    "postage_fr_lettre_verte_100g_2026": {"low": 2.97, "base": 3.1, "high": 3.1, "unit": "EUR per letter", "as_of": "2026", "confidence": "V2", "group": "postage", "label": "Lettre verte up to 100 g, 2026 (low = online); re-check", "note": "not exactly twice the 20 g rate in the extracts"},
    "postage_fr_lettre_verte_100g_2027": {"low": 3.34, "base": 3.34, "high": 3.34, "unit": "EUR per letter", "as_of": "2027-01-01", "confidence": "V2", "group": "postage", "label": "Lettre verte up to 100 g"},
    "postage_fr_lettre_suivie_20g_2026": {"low": 2.02, "base": 2.02, "high": 2.02, "unit": "EUR per letter", "as_of": "2026", "confidence": "V2", "group": "postage", "label": "Lettre verte suivie up to 20 g, 2026"},
    "postage_fr_lettre_suivie_20g_2027": {"low": 2.17, "base": 2.17, "high": 2.17, "unit": "EUR per letter", "as_of": "2027-01-01", "confidence": "V2", "group": "postage", "label": "Lettre verte suivie up to 20 g"},
    "postage_fr_lettre_suivie_100g_2026": {"low": 3.6, "base": 3.6, "high": 3.6, "unit": "EUR per letter", "as_of": "2026", "confidence": "V2", "group": "postage", "label": "Lettre verte suivie up to 100 g, 2026"},
    "postage_fr_lettre_suivie_100g_2027": {"low": 3.84, "base": 3.84, "high": 3.84, "unit": "EUR per letter", "as_of": "2027-01-01", "confidence": "V2", "group": "postage", "label": "Lettre verte suivie up to 100 g"},
    "postage_intl_20g_2026": {"low": 2.19, "base": 2.25, "high": 2.25, "unit": "EUR per letter", "as_of": "2026", "confidence": "V2", "group": "postage", "label": "Lettre internationale up to 20 g, 2026 (low = online)"},
    "postage_intl_20g_2027": {"low": 2.45, "base": 2.45, "high": 2.45, "unit": "EUR per letter", "as_of": "2027-01-01", "confidence": "V2", "group": "postage", "label": "Lettre internationale up to 20 g"},
    "postage_intl_100g_2026": {"low": 4.7, "base": 4.85, "high": 4.85, "unit": "EUR per letter", "as_of": "2026", "confidence": "V2", "group": "postage", "label": "Lettre internationale up to 100 g, 2026 (low = online)"},
    "postage_intl_100g_2027": {"low": 5.25, "base": 5.25, "high": 5.25, "unit": "EUR per letter", "as_of": "2027-01-01", "confidence": "V2", "group": "postage", "label": "Lettre internationale up to 100 g"},
    "postage_intl_suivie_20g_2026": {"low": 5.05, "base": 5.05, "high": 5.05, "unit": "EUR per letter", "as_of": "2026", "confidence": "V1", "group": "postage", "label": "Lettre suivie internationale up to 20 g, 2026 (2027 not retrieved)"},
    "postage_intl_suivie_100g_2026": {"low": 7.65, "base": 7.65, "high": 7.65, "unit": "EUR per letter", "as_of": "2026", "confidence": "V1", "group": "postage", "label": "Lettre suivie internationale up to 100 g, 2026"},
    "postage_band_jump_2026": {"low": 1.58, "base": 1.58, "high": 1.58, "unit": "EUR per letter", "as_of": "2026", "confidence": "derived", "group": "postage", "label": "Extra cost of the 100 g band over the 20 g band, 2026"},
    "postage_band_jump_2027": {"low": 1.67, "base": 1.67, "high": 1.67, "unit": "EUR per letter", "as_of": "2027-01-01", "confidence": "derived", "group": "postage", "label": "Extra cost of the 100 g band over the 20 g band, 2027"},
    "postage_yearly_increase": {"low": 0.067, "base": 0.08, "high": 0.099, "unit": "fraction per year", "as_of": "2026 to 2027", "confidence": "derived", "group": "postage", "label": "Yearly postage increase; observed 2026 to 2027 changes range from 6.7% to 9.9%; base = rounded median", "note": "one year of evidence only"},
    "weight_letter_band_limit": {"low": 20, "base": 20, "high": 20, "unit": "g", "as_of": "2026", "confidence": "V2", "group": "weight", "label": "Upper limit of the first letter band"},
    "weight_jacket_area_pocket": {"low": 0.041, "base": 0.041, "high": 0.041, "unit": "m2", "as_of": "2026-11", "confidence": "derived", "group": "weight", "label": "Pocket jacket sheet, (110 + 10 + 110) x 178 mm"},
    "weight_jacket_area_large": {"low": 0.06, "base": 0.06, "high": 0.06, "unit": "m2", "as_of": "2026-11", "confidence": "derived", "group": "weight", "label": "Large-format jacket sheet, (135 + 10 + 135) x 215 mm"},
    "weight_jacket_pocket": {"low": 10.2, "base": 12.3, "high": 14.3, "unit": "g", "as_of": "2026-11", "confidence": "derived", "group": "weight", "label": "Pocket jacket at 250 / 300 / 350 g/m2"},
    "weight_jacket_large": {"low": 15.1, "base": 18.1, "high": 21.1, "unit": "g", "as_of": "2026-11", "confidence": "derived", "group": "weight", "label": "Large-format jacket at 250 / 300 / 350 g/m2"},
    "weight_envelope_c5": {"low": 7, "base": 7.5, "high": 8, "unit": "g", "as_of": "2026-11", "confidence": "M", "group": "weight", "label": "C5 envelope"},
    "print_inhouse_per_card": {"low": 0.5, "base": 1.0, "high": 1.5, "unit": "EUR per card", "as_of": "2026-11", "confidence": "assumption", "group": "print", "label": "One jacket printed in-house (paper + toner); placeholder", "note": "needs measurement"},
    "print_envelope": {"low": 0.03, "base": 0.08, "high": 0.1, "unit": "EUR per envelope", "as_of": "2026-11", "confidence": "M", "group": "print", "label": "Envelope C5/C6, bought by 500", "note": "base = value used in the unit economics"},
    "print_batch100_folded_card": {"low": 0.36, "base": 0.48, "high": 0.6, "unit": "EUR HT per card", "as_of": "2026-11", "confidence": "V1", "group": "print", "label": "Folded A6-class card, 300-350 g, 4/4, 100 identical copies, delivered in France; base = midpoint", "note": "identical copies only"},
    "print_batch25_50_folded_card": {"low": 0.5, "base": 1.3, "high": 2.1, "unit": "EUR HT per card", "as_of": "2026-11", "confidence": "V1", "group": "print", "label": "Same, 25-50 copies; base = midpoint"},
    "print_batch1000_folded_card": {"low": 0.07, "base": 0.1, "high": 0.13, "unit": "EUR HT per card", "as_of": "2026-11", "confidence": "M", "group": "print", "label": "Same, 1,000 copies; estimate from a flyer price floor", "note": "estimate only"},
    "print_jacket_double_score_100_500": {"low": 0.6, "base": 1.0, "high": 1.5, "unit": "EUR per jacket", "as_of": "2026-11", "confidence": "assumption", "group": "print", "label": "True jacket with two scores 10 mm apart, runs of 100-500; needs quotes", "note": "no quote obtained; no variable data"},
    "print_pod_single_card": {"low": 2.36, "base": 2.6, "high": 2.82, "unit": "EUR per card", "as_of": "2026-11", "confidence": "V1", "group": "print", "label": "Single print-on-demand greeting card, before shipping"},
    "fulfil_pod_single_card_landed": {"low": 5.85, "base": 6.0, "high": 6.0, "unit": "EUR per card", "as_of": "2026-11", "confidence": "V1", "group": "fulfilment", "label": "Single print-on-demand card delivered (Printful 2.36 + 3.49 shipping)", "note": "estimate"},
    "fulfil_postal_api_folded_card": {"low": 2.99, "base": 2.99, "high": 2.99, "unit": "EUR per card", "as_of": "2026-11", "confidence": "V2", "group": "fulfilment", "label": "Merci Facteur folded card 15 x 21 cm, envelope included", "note": "same vendor, two pages"},
    "fulfil_postal_api_postage_fr": {"low": 1.3, "base": 1.3, "high": 1.3, "unit": "EUR per card", "as_of": "2026-11", "confidence": "V2", "group": "fulfilment", "label": "Merci Facteur postage to France, 'from'"},
    "fulfil_postal_api_tracking": {"low": 0.5, "base": 0.5, "high": 0.5, "unit": "EUR per card", "as_of": "2026-11", "confidence": "V2", "group": "fulfilment", "label": "Merci Facteur tracking option"},
    "fulfil_postal_api_landed": {"low": 4.29, "base": 4.29, "high": 4.79, "unit": "EUR per card", "as_of": "2026-11", "confidence": "derived", "group": "fulfilment", "label": "Folded card printed and posted to a French address (high = with tracking)", "note": "VAT status unknown"},
    "fulfil_postal_api_plan": {"low": 19.95, "base": 19.95, "high": 39.95, "unit": "EUR per month", "as_of": "2026-11", "confidence": "V2", "group": "fulfilment", "label": "Merci Facteur API plan (high = above about 2,000 cards a month)"},
    "fulfil_postal_api_plan_threshold": {"low": 2000, "base": 2000, "high": 2000, "unit": "cards per month", "as_of": "2026-11", "confidence": "V1", "group": "fulfilment", "label": "Monthly volume above which the higher API plan applies", "note": "approximate"},
    "fulfil_maileva_letter_landed": {"low": 1.0, "base": 1.45, "high": 1.9, "unit": "EUR HT per letter", "as_of": "2026-01-01", "confidence": "V1", "group": "fulfilment", "label": "Plain A4 colour letter printed and posted (fallback, not a gift object); base = midpoint", "note": "estimate"},
    "fulfil_dummy_book_landed": {"low": 9.0, "base": 10.5, "high": 12.0, "unit": "EUR per copy", "as_of": "2026-11", "confidence": "V1", "group": "fulfilment", "label": "Print-on-demand paperback with a 10 mm spine, one copy to France; base = midpoint", "note": "estimate"},
    "pay_stripe_eea_pct": {"low": 0.015, "base": 0.015, "high": 0.015, "unit": "fraction", "as_of": "2026-10", "confidence": "V2", "group": "payment", "label": "Stripe, standard EEA consumer card, variable part"},
    "pay_stripe_fixed": {"low": 0.25, "base": 0.25, "high": 0.25, "unit": "EUR per transaction", "as_of": "2026-10", "confidence": "V2", "group": "payment", "label": "Stripe, fixed part per transaction"},
    "pay_stripe_eea_premium_pct": {"low": 0.019, "base": 0.028, "high": 0.028, "unit": "fraction", "as_of": "2026-10", "confidence": "V1", "group": "payment", "label": "Stripe, premium or corporate EEA card (conflicting: 1.9% vs 2.8%)", "note": "conflicting"},
    "pay_stripe_uk_pct": {"low": 0.025, "base": 0.025, "high": 0.025, "unit": "fraction", "as_of": "2026-10", "confidence": "V1", "group": "payment", "label": "Stripe, UK card"},
    "pay_stripe_non_eea_pct": {"low": 0.0315, "base": 0.0315, "high": 0.0325, "unit": "fraction", "as_of": "2026-10", "confidence": "V1", "group": "payment", "label": "Stripe, non-EEA card (conflicting: 3.15% vs 3.25%)", "note": "conflicting"},
    "pay_stripe_fx_pct": {"low": 0.02, "base": 0.02, "high": 0.02, "unit": "fraction", "as_of": "2026-10", "confidence": "V1", "group": "payment", "label": "Stripe, currency conversion surcharge"},
    "pay_stripe_dispute_fee": {"low": 20, "base": 20, "high": 20, "unit": "EUR per dispute", "as_of": "2026-10", "confidence": "V1", "group": "payment", "label": "Stripe, fee per dispute"},
    "pay_mollie_fr_pct": {"low": 0.012, "base": 0.012, "high": 0.012, "unit": "fraction", "as_of": "2026-11", "confidence": "V2", "group": "payment", "label": "Mollie, domestic consumer card (Cartes Bancaires)"},
    "pay_mollie_fixed": {"low": 0.0, "base": 0.25, "high": 0.25, "unit": "EUR per transaction", "as_of": "2026-11", "confidence": "V1", "group": "payment", "label": "Mollie, fixed part (official extract showed none)", "note": "conflicting"},
    "pay_mollie_eu_pct": {"low": 0.018, "base": 0.018, "high": 0.018, "unit": "fraction", "as_of": "2026-11", "confidence": "V1", "group": "payment", "label": "Mollie, EU consumer card"},
    "pay_mollie_commercial_pct": {"low": 0.029, "base": 0.029, "high": 0.029, "unit": "fraction", "as_of": "2026-11", "confidence": "V1", "group": "payment", "label": "Mollie, commercial card"},
    "pay_mollie_non_eu_pct": {"low": 0.028, "base": 0.0325, "high": 0.0325, "unit": "fraction", "as_of": "2026-11", "confidence": "V1", "group": "payment", "label": "Mollie, non-EU card (base = high, conservative)"},
    "pay_mollie_plan": {"low": 20, "base": 20, "high": 20, "unit": "EUR per month", "as_of": "2026-11", "confidence": "V1", "group": "payment", "label": "Mollie optional plan for volumes above EUR 1,900 a month", "note": "optional"},
    "pay_sumup_online_pct": {"low": 0.025, "base": 0.025, "high": 0.025, "unit": "fraction", "as_of": "2026-11", "confidence": "V1", "group": "payment", "label": "SumUp online, percentage only"},
    "pay_sumup_inperson_pct": {"low": 0.0175, "base": 0.0175, "high": 0.0175, "unit": "fraction", "as_of": "2026-11", "confidence": "V1", "group": "payment", "label": "SumUp in person (fairs, signings)"},
    "pay_stancer_pct": {"low": 0.007, "base": 0.007, "high": 0.007, "unit": "fraction", "as_of": "2026-11", "confidence": "V1", "group": "payment", "label": "Stancer, variable part"},
    "pay_stancer_fixed": {"low": 0.07, "base": 0.15, "high": 0.15, "unit": "EUR per transaction", "as_of": "2026-11", "confidence": "V1", "group": "payment", "label": "Stancer, fixed part (conflicting: 0.07 vs 0.15)", "note": "conflicting"},
    "pay_gocardless_pct": {"low": 0.01, "base": 0.01, "high": 0.01, "unit": "fraction", "as_of": "2026-11", "confidence": "V1", "group": "payment", "label": "GoCardless SEPA direct debit, variable part"},
    "pay_gocardless_fixed": {"low": 0.2, "base": 0.2, "high": 0.2, "unit": "EUR per transaction", "as_of": "2026-11", "confidence": "V1", "group": "payment", "label": "GoCardless, fixed part"},
    "pay_gocardless_cap": {"low": 2.0, "base": 2.0, "high": 2.0, "unit": "EUR per transaction", "as_of": "2026-11", "confidence": "V1", "group": "payment", "label": "GoCardless, fee cap"},
    "pay_model_card_pct": {"low": 0.012, "base": 0.015, "high": 0.028, "unit": "fraction", "as_of": "2026-11", "confidence": "derived", "group": "payment", "label": "Model default, card variable part (Mollie domestic / Stripe standard / Stripe premium)"},
    "pay_model_card_fixed": {"low": 0.07, "base": 0.25, "high": 0.25, "unit": "EUR per transaction", "as_of": "2026-11", "confidence": "derived", "group": "payment", "label": "Model default, card fixed part (low = Stancer)"},
    "host_scaleway_dev1s_instance": {"low": 6.55, "base": 6.55, "high": 6.55, "unit": "EUR per month", "as_of": "2026-06-01", "confidence": "V2", "group": "hosting", "label": "Scaleway DEV1-S (2 vCPU, 2 GB), instance only, after the 1 June 2026 increase"},
    "host_scaleway_dev1s_hour": {"low": 0.00898, "base": 0.00898, "high": 0.00898, "unit": "EUR per hour", "as_of": "2026-06-01", "confidence": "V2", "group": "hosting", "label": "Scaleway DEV1-S hourly price"},
    "host_scaleway_dev1s_all_in": {"low": 10.9, "base": 10.9, "high": 10.9, "unit": "EUR per month", "as_of": "2026-06-01", "confidence": "V2", "group": "hosting", "label": "Scaleway DEV1-S with 20 GB disk and a public IPv4"},
    "host_hetzner_cx23": {"low": 5.49, "base": 5.49, "high": 5.49, "unit": "EUR per month", "as_of": "2026-06-15", "confidence": "V2", "group": "hosting", "label": "Hetzner CX23, excl. IPv4, after two 2026 increases; availability uncertain", "note": "new orders reported restricted since 2026-06-26"},
    "host_hetzner_object_storage_base": {"low": 6.49, "base": 6.49, "high": 6.49, "unit": "EUR per month", "as_of": "2026-04", "confidence": "V2", "group": "hosting", "label": "Hetzner Object Storage base fee"},
    "host_hetzner_egress_per_tb": {"low": 1.0, "base": 1.0, "high": 1.0, "unit": "EUR per TB", "as_of": "2026-11", "confidence": "V2", "group": "hosting", "label": "Hetzner extra egress", "note": "approximate"},
    "host_ovh_vps1": {"low": 4.49, "base": 7.79, "high": 7.79, "unit": "EUR per month", "as_of": "2026-04-01", "confidence": "V1", "group": "hosting", "label": "OVHcloud VPS-1 (conflicting: 4.49 HT vs 7.79 after 1 April 2026)", "note": "conflicting"},
    "host_object_storage_per_gb": {"low": 0.01, "base": 0.01, "high": 0.01, "unit": "EUR per GB per month", "as_of": "2026-11", "confidence": "V1", "group": "hosting", "label": "Scaleway Object Storage", "note": "one outlier source disagrees"},
    "host_egress_free": {"low": 75, "base": 75, "high": 75, "unit": "GB per month", "as_of": "2026-11", "confidence": "V1", "group": "hosting", "label": "Scaleway free egress allowance"},
    "host_egress_per_gb": {"low": 0.01, "base": 0.01, "high": 0.01, "unit": "EUR per GB", "as_of": "2026-11", "confidence": "V1", "group": "hosting", "label": "Scaleway egress beyond the allowance"},
    "host_total_launch": {"low": 10, "base": 15, "high": 25, "unit": "EUR per month", "as_of": "2026-11", "confidence": "derived", "group": "hosting", "label": "Small API + object storage at launch, any of the three hosts", "note": "budget the high value"},
    "mail_scaleway_tem_free": {"low": 300, "base": 300, "high": 300, "unit": "e-mails", "as_of": "2026-11", "confidence": "V1", "group": "email", "label": "Scaleway TEM free allowance"},
    "mail_scaleway_tem_per_1000": {"low": 0.25, "base": 0.25, "high": 0.25, "unit": "EUR per 1,000 e-mails", "as_of": "2026-11", "confidence": "V1", "group": "email", "label": "Scaleway TEM beyond the allowance"},
    "mail_scaleway_tem_scale_plan": {"low": 80, "base": 80, "high": 80, "unit": "EUR per month", "as_of": "2026-11", "confidence": "V1", "group": "email", "label": "Scaleway TEM 'Scale' plan, 100,000 e-mails, dedicated IP"},
    "mail_brevo_free_per_day": {"low": 300, "base": 300, "high": 300, "unit": "e-mails per day", "as_of": "2026-11", "confidence": "V2", "group": "email", "label": "Brevo free plan cap"},
    "mail_brevo_starter": {"low": 7, "base": 7, "high": 14, "unit": "EUR per month", "as_of": "2026-11", "confidence": "V1", "group": "email", "label": "Brevo Starter, 5,000 e-mails (conflicting: 7 vs 14)", "note": "conflicting"},
    "mail_emails_per_paid_order": {"low": 0, "base": 2, "high": 3, "unit": "e-mails per order", "as_of": "2026-11", "confidence": "assumption", "group": "email", "label": "Transactional e-mails per paid order (none in print-at-home mode)"},
    "tax_micro_social_goods": {"low": 0.123, "base": 0.123, "high": 0.123, "unit": "fraction of turnover", "as_of": "2026", "confidence": "V2", "group": "tax", "label": "Micro-entreprise social charges, sale of goods", "note": "which lines count as goods is [M]"},
    "tax_micro_social_services_bic": {"low": 0.212, "base": 0.212, "high": 0.212, "unit": "fraction of turnover", "as_of": "2026", "confidence": "V2", "group": "tax", "label": "Micro-entreprise social charges, commercial services (BIC)"},
    "tax_micro_social_services_bnc": {"low": 0.256, "base": 0.256, "high": 0.256, "unit": "fraction of turnover", "as_of": "2026", "confidence": "V2", "group": "tax", "label": "Micro-entreprise social charges, liberal services (BNC; 24.6% in 2025)"},
    "tax_micro_ceiling_goods": {"low": 203100, "base": 203100, "high": 203100, "unit": "EUR per year", "as_of": "2026", "confidence": "V1", "group": "tax", "label": "Micro-entreprise turnover ceiling, goods", "note": "one outlier source"},
    "tax_micro_ceiling_services": {"low": 83600, "base": 83600, "high": 83600, "unit": "EUR per year", "as_of": "2026", "confidence": "V1", "group": "tax", "label": "Micro-entreprise turnover ceiling, services", "note": "one outlier source"},
    "tax_vat_franchise_goods": {"low": 85000, "base": 85000, "high": 91900, "unit": "EUR per year", "as_of": "2026", "confidence": "V1", "group": "tax", "label": "VAT franchise threshold, goods (conflicting: 85,000 vs 91,900)", "note": "conflicting"},
    "tax_vat_franchise_services": {"low": 36800, "base": 37500, "high": 37500, "unit": "EUR per year", "as_of": "2026", "confidence": "V1", "group": "tax", "label": "VAT franchise threshold, services (conflicting: 37,500 vs 36,800)", "note": "conflicting"},
    "tax_vat_card_fr": {"low": 0.2, "base": 0.2, "high": 0.2, "unit": "fraction", "as_of": "2026", "confidence": "M", "group": "tax", "label": "VAT on the printed card and postage charged, France"},
    "tax_vat_ebook_fr": {"low": 0.055, "base": 0.055, "high": 0.055, "unit": "fraction", "as_of": "2026", "confidence": "M", "group": "tax", "label": "VAT on the eBook, France"},
    "tax_vat_card_de": {"low": 0.19, "base": 0.19, "high": 0.19, "unit": "fraction", "as_of": "2026", "confidence": "M", "group": "tax", "label": "VAT on the printed card, Germany"},
    "tax_vat_ebook_de": {"low": 0.07, "base": 0.07, "high": 0.07, "unit": "fraction", "as_of": "2026", "confidence": "M", "group": "tax", "label": "VAT on the eBook, Germany"},
    "tax_eu_oss_threshold": {"low": 10000, "base": 10000, "high": 10000, "unit": "EUR per year", "as_of": "2026", "confidence": "M", "group": "tax", "label": "EU-wide yearly threshold for cross-border B2C sales before destination VAT (OSS)"},
    "tax_sasu_salary_cost_multiplier": {"low": 1.82, "base": 1.82, "high": 1.82, "unit": "ratio", "as_of": "2026", "confidence": "V2", "group": "tax", "label": "SASU president: total cost divided by net salary"},
    "tax_salary_total_cost_3000_net": {"low": 5275, "base": 5275, "high": 5275, "unit": "EUR per month", "as_of": "2026", "confidence": "V2", "group": "tax", "label": "Total monthly cost of EUR 3,000 net for a SASU president"},
    "tax_flat_tax_dividends": {"low": 0.314, "base": 0.314, "high": 0.314, "unit": "fraction", "as_of": "2026", "confidence": "V2", "group": "tax", "label": "Flat tax on dividends, 2026 (12.8% + 18.6%; was 30%)"},
    "tax_corporate_reduced_rate": {"low": 0.15, "base": 0.15, "high": 0.15, "unit": "fraction", "as_of": "2026", "confidence": "M", "group": "tax", "label": "Corporate tax, reduced rate on the first EUR 42,500 of profit"},
    "tax_corporate_standard_rate": {"low": 0.25, "base": 0.25, "high": 0.25, "unit": "fraction", "as_of": "2026", "confidence": "M", "group": "tax", "label": "Corporate tax, standard rate"},
    "ip_inpi_filing_one_class": {"low": 190, "base": 190, "high": 190, "unit": "EUR one-off", "as_of": "2026", "confidence": "V2", "group": "ip_legal", "label": "INPI trade mark filing, one class, electronic"},
    "ip_inpi_extra_class": {"low": 40, "base": 40, "high": 40, "unit": "EUR per class", "as_of": "2026", "confidence": "V2", "group": "ip_legal", "label": "INPI, each additional class"},
    "ip_inpi_renewal_one_class": {"low": 290, "base": 290, "high": 290, "unit": "EUR per renewal", "as_of": "2026", "confidence": "V2", "group": "ip_legal", "label": "INPI renewal, one class (every 10 years)"},
    "ip_euipo_filing_one_class": {"low": 850, "base": 850, "high": 850, "unit": "EUR one-off", "as_of": "2026", "confidence": "V1", "group": "ip_legal", "label": "EUIPO filing, one class (another site gives a different base)", "note": "conflicting"},
    "legal_docs_lawyer": {"low": 500, "base": 1000, "high": 1500, "unit": "EUR one-off", "as_of": "2026-11", "confidence": "M", "group": "ip_legal", "label": "CGV and privacy documents drafted by a lawyer; base = midpoint"},
    "legal_docs_templates": {"low": 0, "base": 100, "high": 200, "unit": "EUR one-off", "as_of": "2026-11", "confidence": "M", "group": "ip_legal", "label": "Same, from templates; base = midpoint"},
    "fix_domain": {"low": 7, "base": 12, "high": 20, "unit": "EUR per year", "as_of": "2026-11", "confidence": "M", "group": "fixed", "label": ".fr or .eu domain; base = rounded midpoint"},
    "fix_short_domain": {"low": 10, "base": 30, "high": 60, "unit": "EUR per year", "as_of": "2026-11", "confidence": "M", "group": "fixed", "label": "Short domain for the typed code (ordinary name); base = rounded midpoint", "note": "premium 2-3 letter names cost far more"},
    "fix_rc_pro": {"low": 150, "base": 300, "high": 500, "unit": "EUR per year", "as_of": "2026-11", "confidence": "M", "group": "fixed", "label": "Professional liability insurance, digital micro-business; base = rounded midpoint"},
    "fix_accountant_company": {"low": 1000, "base": 1400, "high": 1800, "unit": "EUR HT per year", "as_of": "2026-11", "confidence": "M", "group": "fixed", "label": "Online accountant for a SASU (not needed for a micro-entreprise); base = midpoint"},
    "fix_bank": {"low": 0, "base": 8, "high": 15, "unit": "EUR per month", "as_of": "2026-11", "confidence": "M", "group": "fixed", "label": "Business bank account; base = rounded midpoint"},
    "fix_cfe_year1": {"low": 0, "base": 0, "high": 0, "unit": "EUR per year", "as_of": "2026-11", "confidence": "M", "group": "fixed", "label": "CFE in the year of creation"},
    "fix_cfe_from_year2": {"low": 200, "base": 400, "high": 600, "unit": "EUR per year", "as_of": "2026-11", "confidence": "M", "group": "fixed", "label": "CFE minimum from the second year, by commune; base = midpoint"},
    "fix_fonts_ofl": {"low": 0, "base": 0, "high": 0, "unit": "EUR", "as_of": "2026-11", "confidence": "M", "group": "fixed", "label": "Fonts under the SIL Open Font License", "note": "commercial font licences are quoted case by case"},
    "fix_lean_cash_costs": {"low": 600, "base": 900, "high": 1300, "unit": "EUR per year", "as_of": "2026-11", "confidence": "derived", "group": "fixed", "label": "Yearly cash costs excluding the owner's time (planning scenarios; see 6.2 item 10)", "note": "line items sum to about 290-1,830"},
    "labour_dev_day_rate": {"low": 400, "base": 500, "high": 600, "unit": "EUR per day", "as_of": "2026-11", "confidence": "M", "group": "labour", "label": "Freelance developer day rate, France; base = midpoint"},
    "labour_support_minutes_per_incident": {"low": 3, "base": 4, "high": 5, "unit": "minutes", "as_of": "2026-11", "confidence": "assumption", "group": "labour", "label": "Support time per order with an incident; base = midpoint"},
    "labour_incident_rate": {"low": 0.05, "base": 0.075, "high": 0.1, "unit": "fraction of orders", "as_of": "2026-11", "confidence": "assumption", "group": "labour", "label": "Share of orders with an incident; base = midpoint"},
    "labour_inhouse_minutes_per_card": {"low": 3, "base": 5, "high": 8, "unit": "minutes per card", "as_of": "2026-11", "confidence": "assumption", "group": "labour", "label": "Handling time per mailed card printed in-house"},
    "bench_ebook_price_fr": {"low": 8.06, "base": 8.06, "high": 8.06, "unit": "EUR", "as_of": "2025", "confidence": "V1", "group": "benchmark", "label": "Average price paid for an eBook, France 2025", "note": "derived from EUR 137 M / 17 M"},
    "bench_ebook_price_de": {"low": 7.01, "base": 7.01, "high": 7.01, "unit": "EUR", "as_of": "2025", "confidence": "V2", "group": "benchmark", "label": "Average price paid for an eBook, Germany 2025"},
    "bench_ebook_price_nl": {"low": 10.5, "base": 10.5, "high": 10.5, "unit": "EUR", "as_of": "2025", "confidence": "V1", "group": "benchmark", "label": "Average price paid for an eBook, Netherlands 2025", "note": "derived"},
    "bench_ebook_price_es": {"low": 9.0, "base": 9.0, "high": 9.0, "unit": "EUR", "as_of": "2024", "confidence": "V2", "group": "benchmark", "label": "Average price of an eBook, Spain 2024"},
    "bench_ebook_price_it": {"low": 8.0, "base": 8.0, "high": 8.0, "unit": "EUR", "as_of": "2025", "confidence": "assumption", "group": "benchmark", "label": "Average price of an eBook, Italy"},
    "bench_mailed_postcard_fr": {"low": 2.99, "base": 3.49, "high": 4.84, "unit": "EUR all-in", "as_of": "2026-11", "confidence": "V1", "group": "benchmark", "label": "Flat personalised postcard mailed in France (Fizzer / MyPostcard / Popcarte 2.89 + 1.95)"},
    "bench_mailed_folded_card_fr": {"low": 3.69, "base": 4.29, "high": 6.94, "unit": "EUR all-in", "as_of": "2026-11", "confidence": "V1", "group": "benchmark", "label": "Folded personalised card mailed in France (MyPostcard / Merci Facteur / Popcarte)"},
    "bench_fizzer_pack_per_card": {"low": 1.67, "base": 2.19, "high": 2.19, "unit": "EUR per card", "as_of": "2026-11", "confidence": "V1", "group": "benchmark", "label": "Fizzer per-card price inside prepaid packs of 150 down to 10"},
    "bench_virtual_card": {"low": 1.99, "base": 1.99, "high": 1.99, "unit": "EUR", "as_of": "2026-11", "confidence": "V1", "group": "benchmark", "label": "Popcarte virtual card, 'from'"},
    "bench_moonpig_uk": {"low": 5.89, "base": 5.89, "high": 5.89, "unit": "GBP", "as_of": "2026-10-21", "confidence": "V2", "group": "benchmark", "label": "Moonpig standard card + first-class postage"},
    "bench_print_run_30_ttc": {"low": 0.82, "base": 2.39, "high": 3.96, "unit": "EUR per card", "as_of": "2026-11", "confidence": "V1", "group": "benchmark", "label": "Personalised greeting cards, 30 copies, incl. VAT, excl. shipping; base = midpoint"},
    "bench_print_run_100_ttc": {"low": 0.61, "base": 1.5, "high": 2.4, "unit": "EUR per card", "as_of": "2026-11", "confidence": "V1", "group": "benchmark", "label": "Same, 100 copies; base = midpoint"},
    "bench_dropcards_per_card": {"low": 0.33, "base": 0.39, "high": 0.45, "unit": "USD per card", "as_of": "2026-11", "confidence": "V1", "group": "benchmark", "label": "Dropcards unique-code card, cardstock at 1,000 / cardstock at 500 / plastic at 500"},
    "bench_dropcards_setup": {"low": 49, "base": 49, "high": 49, "unit": "USD one-off", "as_of": "2026-11", "confidence": "V1", "group": "benchmark", "label": "Dropcards set-up fee under 5,000 pieces"},
    "bench_distroprint_per_card": {"low": 0.345, "base": 0.59, "high": 0.956, "unit": "AUD per card", "as_of": "2026-11", "confidence": "V1", "group": "benchmark", "label": "Distro Print unique-code card at 1,000 / 500 / 250, incl. GST"},
    "bench_bookfunnel_plan": {"low": 200, "base": 200, "high": 300, "unit": "USD per year", "as_of": "2026-02-20", "confidence": "V2", "group": "benchmark", "label": "BookFunnel plans that allow print codes (Mid-List / Bestseller)"},
    "price_mailed_card": {"low": 4.9, "base": 6.9, "high": 8.9, "unit": "EUR per card", "as_of": "2026-11", "confidence": "assumption", "group": "price_point", "label": "Mailed single card, France, incl. VAT", "note": "to test"},
    "price_paid_pdf": {"low": 0.0, "base": 0.0, "high": 1.99, "unit": "EUR per card", "as_of": "2026-11", "confidence": "assumption", "group": "price_point", "label": "Print-at-home PDF (base = free)", "note": "to test"},
    "price_author_plan": {"low": 29, "base": 49, "high": 99, "unit": "EUR per year", "as_of": "2026-11", "confidence": "assumption", "group": "price_point", "label": "Author plan: code batches + print-ready jackets", "note": "to test"},
    "price_author_jacket_pack100": {"low": 1.5, "base": 2.0, "high": 2.5, "unit": "EUR per jacket", "as_of": "2026-11", "confidence": "assumption", "group": "price_point", "label": "Printed jacket in a pack of 100, one title", "note": "to test"},
    "price_reseller_fee_per_card": {"low": 0.2, "base": 0.3, "high": 0.5, "unit": "EUR per card", "as_of": "2026-11", "confidence": "assumption", "group": "price_point", "label": "Reseller fee per card generated", "note": "to test"},
    "price_reseller_licence": {"low": 99, "base": 200, "high": 499, "unit": "EUR per month", "as_of": "2026-11", "confidence": "assumption", "group": "price_point", "label": "Reseller flat licence", "note": "to test"},
    "price_affiliate_per_gifted_ebook": {"low": 0.3, "base": 0.4, "high": 0.5, "unit": "EUR per eBook", "as_of": "2026-11", "confidence": "M", "group": "price_point", "label": "Affiliate commission on a EUR 10 eBook; base = midpoint", "note": "not searched"},
    "demand_ebook_units_fr": {"low": 17000000, "base": 17000000, "high": 17000000, "unit": "units per year", "as_of": "2025", "confidence": "V1", "group": "demand", "label": "Paid consumer eBook units, France 2025 (A1)"},
    "demand_ebook_value_fr": {"low": 137000000, "base": 137000000, "high": 137000000, "unit": "EUR per year", "as_of": "2025", "confidence": "V1", "group": "demand", "label": "Consumer eBook sales, France 2025"},
    "demand_ebook_units_5c": {"low": 87500000, "base": 87500000, "high": 87500000, "unit": "units per year", "as_of": "2024-2025", "confidence": "derived", "group": "demand", "label": "Paid consumer eBook units, FR+DE+NL+ES+IT (A1')", "note": "order of magnitude"},
    "demand_gift_share": {"low": 0.03, "base": 0.06, "high": 0.12, "unit": "fraction", "as_of": "2026-11", "confidence": "assumption", "group": "demand", "label": "Share of eBook units that could be gifts (A2)"},
    "demand_non_amazon_share": {"low": 0.4, "base": 0.5, "high": 0.6, "unit": "fraction", "as_of": "2026-11", "confidence": "assumption", "group": "demand", "label": "Share of units sold outside Amazon (A3)"},
    "demand_physical_token_share": {"low": 0.5, "base": 0.65, "high": 0.8, "unit": "fraction", "as_of": "2026-11", "confidence": "assumption", "group": "demand", "label": "Share of gift occasions wanting a physical token (A4)"},
    "demand_capture_fr_year3": {"low": 0.03, "base": 0.08, "high": 0.15, "unit": "fraction", "as_of": "2026-11", "confidence": "assumption", "group": "demand", "label": "Capture of SAM by year 3, France (A5)"},
    "demand_capture_5c_year5": {"low": 0.02, "base": 0.05, "high": 0.1, "unit": "fraction", "as_of": "2026-11", "confidence": "assumption", "group": "demand", "label": "Capture of SAM by year 5, five countries (A5')"},
    "demand_tam_fr": {"low": 510000, "base": 1020000, "high": 2040000, "unit": "cards per year", "as_of": "2026-11", "confidence": "derived", "group": "demand", "label": "TAM France"},
    "demand_sam_fr": {"low": 102000, "base": 331500, "high": 979200, "unit": "cards per year", "as_of": "2026-11", "confidence": "derived", "group": "demand", "label": "SAM France"},
    "demand_som_fr_year3": {"low": 3060, "base": 26520, "high": 146880, "unit": "cards per year", "as_of": "2026-11", "confidence": "derived", "group": "demand", "label": "SOM France, year 3"},
    "demand_tam_5c": {"low": 2625000, "base": 5250000, "high": 10500000, "unit": "cards per year", "as_of": "2026-11", "confidence": "derived", "group": "demand", "label": "TAM five countries"},
    "demand_sam_5c": {"low": 525000, "base": 1706250, "high": 5040000, "unit": "cards per year", "as_of": "2026-11", "confidence": "derived", "group": "demand", "label": "SAM five countries"},
    "demand_som_5c_year5": {"low": 10500, "base": 85313, "high": 504000, "unit": "cards per year", "as_of": "2026-11", "confidence": "derived", "group": "demand", "label": "SOM five countries, year 5"},
    "demand_author_cards_fr": {"low": 10000, "base": 25000, "high": 50000, "unit": "cards per year", "as_of": "2026-11", "confidence": "assumption", "group": "demand", "label": "Author / signing segment, France; base = rounded midpoint", "note": "outside the funnel"},
    "demand_attach_rate": {"low": 0.005, "base": 0.02, "high": 0.05, "unit": "fraction of eBook orders", "as_of": "2026-11", "confidence": "assumption", "group": "demand", "label": "Gift option attach rate at an eBook checkout"},
    "demand_mailed_upgrade_share": {"low": 0.02, "base": 0.05, "high": 0.1, "unit": "fraction of cards", "as_of": "2026-11", "confidence": "assumption", "group": "demand", "label": "Share of generated cards upgraded to the mailed option", "note": "used in section 7.4 at 0.05"},
    "demand_unredeemed_share": {"low": 0.1, "base": 0.15, "high": 0.2, "unit": "fraction of codes", "as_of": "2026-11", "confidence": "assumption", "group": "demand", "label": "Share of gift codes never redeemed; base = midpoint"},
    "demand_peak_share": {"low": 0.45, "base": 0.5, "high": 0.55, "unit": "fraction", "as_of": "2026-11", "confidence": "assumption", "group": "demand", "label": "Share of yearly cards between 20 November and 24 December"},
    "demand_digital_readers_fr": {"low": 14000000, "base": 14000000, "high": 14400000, "unit": "people", "as_of": "2025", "confidence": "V2", "group": "demand", "label": "People who read a digital book in 2025, France", "note": "high = secondary summary"},
    "demand_ereader_users_fr": {"low": 4760000, "base": 4900000, "high": 4900000, "unit": "people", "as_of": "2025", "confidence": "derived", "group": "demand", "label": "People reading on an e-reader, France (34% of digital readers)"},
    "demand_paying_buyers_fr": {"low": 2000000, "base": 3000000, "high": 4000000, "unit": "people", "as_of": "2026-11", "confidence": "assumption", "group": "demand", "label": "Regular paying eBook buyers, France"},
    "demand_device_share_smartphone": {"low": 0.48, "base": 0.48, "high": 0.48, "unit": "fraction", "as_of": "2025", "confidence": "V1", "group": "demand", "label": "Digital readers using a smartphone"},
    "demand_device_share_ereader": {"low": 0.34, "base": 0.34, "high": 0.34, "unit": "fraction", "as_of": "2025", "confidence": "V1", "group": "demand", "label": "Digital readers using an e-reader"},
    "demand_device_share_tablet": {"low": 0.33, "base": 0.33, "high": 0.33, "unit": "fraction", "as_of": "2025", "confidence": "V1", "group": "demand", "label": "Digital readers using a tablet"}
  }
}
```

---

## 7. Business-model options compared

All contributions below are per unit, before the owner's time, for a micro-entreprise without VAT (social charges: 12.3% of turnover on goods, 21.2% on services [V2]; which rate applies to which line is my reading [M]). Card payments cost 1.5% + EUR 0.25 [V2]. "Fixed" means the yearly cash costs of section 6: EUR 600 / 900 / 1,300.

### 7.1 Options side by side

| Option | Payer | Price tested | Variable cost per unit | Contribution per unit | Units a year to cover fixed costs (600 / 900 / 1,300) | Evidence of willingness to pay | Main blocker |
|---|---|---|---|---|---|---|---|
| **A. Free print-at-home PDF** | nobody | EUR 0 | about 0 (generated in the browser) | 0 | n/a: acquisition channel | n/a | No revenue by design |
| **A'. Paid PDF, sold singly** | consumer | EUR 1.99 | payment 0.28 (14%) + charges 0.42 | about 1.29 (about 1.13 inside a EUR 14.90 pack of 10) | 465 / 700 / 1,010 | Weak: Popcarte virtual card from EUR 1.99 [V1]; retailers' e-mail gifting is free [V1] | Free alternatives; conversion unknown |
| **B1. Mailed card, printed in-house**, pocket format at most 20 g, 2027 stamp | consumer | EUR 4.90 | stamp 1.67 + print and envelope 1.08 (placeholder) + payment 0.32 + charges 0.60 | about 1.2, before 3-8 minutes of handling per card | 490 / 730 / 1,060 | Medium: mailed cards sell at EUR 3-7 (section 5.1) | Manual work; does not scale; the 20 g limit |
| **B2. Mailed card, outsourced** to a French postal-card API | consumer | EUR 6.90 | fulfilment 4.29 + payment 0.35 + charges 0.85; API plan EUR 239 a year | about 1.4 | 595 / 810 / 1,090 | Medium to low: top of the folded-card range (Popcarte EUR 6.94) | Adds about 85% to an EUR 8 eBook; APIs sell a single centre fold, not a scored 10 mm spine |
| **C1. Printed pack for an author or publisher**, 100 jackets of one title | author / publisher | EUR 2.00 per jacket | print 0.60-1.50 (base 1.00, needs quotes) + charges 0.25 + payment 0.03 | about 0.7 (0.2 to 1.1) | 830 / 1,250 / 1,800 jackets, i.e. 8 / 13 / 18 packs | Strong: Dropcards, Distro Print [V1] | Unique codes need variable-data printing, not priced; no real quote yet |
| **C2. Author plan**: code batches + print-ready jackets, author prints where they like | author / publisher | EUR 49 a year | payment 0.99 + charges 10.39 | about 37.6 per subscriber | 16 / 24 / 35 subscribers | Strong: BookFunnel USD 200-300 a year [V2] | Reaching authors; BookFunnel or Bookwire could add a card generator |
| **D1. Reseller fee per card** | reseller | EUR 0.30 | charges 0.064; SEPA fee negligible on a monthly invoice | about 0.24 | 2,540 / 3,810 / 5,500 cards | None found | Attach rate unknown; reseller's roadmap; no entitlement at Fnac/Kobo |
| **D2. Reseller flat licence** | reseller | EUR 200 a month | charges 509 a year | about EUR 1,890 a year per reseller | 1 reseller (EUR 99 a month nets about EUR 940 a year) | None found | Long sales cycle; dependence on one partner |
| **E. Affiliate links** to retailers' gift flows | retailer | EUR 0.30-0.50 per gifted eBook [M] | 0 | about 0.24-0.39 | same order as D1 | Not searched | Rates unverified; works only where a title-level gift exists (Amazon, Decitre) |
| **F. Corporate / works-council gifting** | employer, CSE | unknown | unknown | unknown | unknown | Not searched | Not researched at all |

### 7.2 The mailed card in detail

In-house printing, EUR 4.90, pocket format at most 20 g, 2027 stamp (derived; print EUR 1.00 and envelope EUR 0.08 are placeholders):

| Line | Micro-entreprise, no VAT | Company, 20% VAT |
|---|---|---|
| Revenue kept | 4.90 | 4.08 excl. VAT |
| Stamp (Lettre verte 20 g, 2027) | -1.67 | -1.67 |
| Print + envelope | -1.08 | -1.08 |
| Payment (1.5% + 0.25) | -0.32 | -0.32 |
| Social charges 12.3% of turnover | -0.60 | n/a |
| **Contribution per card** | **about 1.2** | **about 1.0** (before overheads and corporate tax) |

- Over 20 g the stamp is EUR 3.34 and the same EUR 4.90 price loses about EUR 0.45 a card; about EUR 6.90 is needed to get back to about EUR 1.3.
- Price floor (zero contribution, micro-entreprise): EUR 3.48 at 20 g or less, EUR 5.42 above 20 g.
- Handling time is not in the table. At 5 minutes a card and EUR 20 an hour, labour is EUR 1.67 a card, which is more than the contribution. In-house printing is a validation method, not an operating model.

Outsourced fulfilment at about EUR 4.29 landed (Merci Facteur folded card EUR 2.99 + postage from EUR 1.30 [V2, same vendor]; whether VAT is recoverable on it is not known):

| Consumer price | Micro-entreprise contribution | Company contribution (20% VAT) |
|---|---|---|
| EUR 4.90 | -0.32 | 0.0 to 0.2 |
| EUR 5.90 | 0.55 | 0.8 to 1.0 |
| EUR 6.90 | 1.41 | 1.6 to 1.8 |
| EUR 7.90 | 2.27 | 2.4 to 2.6 |
| EUR 8.90 | 3.13 | 3.2 to 3.5 |

Price floor with outsourced fulfilment: EUR 5.27. **The outsourced mailed card works only at EUR 6.90 or more.** That is the top of what French consumers pay for a folded card and close to the price of the eBook itself.

### 7.3 Structural points that decide the model

1. **Micro-payments are taxed by the fixed fee.** With Stripe a EUR 1.99 sale costs EUR 0.28 (14%), EUR 4.90 costs EUR 0.32 (6.6%), EUR 9.90 costs EUR 0.40 (4.0%). A percentage-only tariff (2.5%) is cheaper than 1.5% + EUR 0.25 for any basket under EUR 25. Prepaid credit packs (the Fizzer pattern) or monthly B2B invoices by SEPA (GoCardless 1% + EUR 0.20, capped at EUR 2 [V1]) avoid it.
2. **Micro-entreprise charges apply to turnover, including re-billed postage and print.** On a mailed card where 55-60% of the price is pass-through cost, 12.3% of turnover is about 30% of the value added. The micro regime suits validation and the plan and licence lines. A mailed-card line at volume belongs in a company that deducts its costs (president's salary costs about 1.82 times net; dividends carry a 31.4% flat tax in 2026 [V2]).
3. **French price law shapes who can pay for the card** [M, sibling legal note, unverified]. The publisher fixes the eBook price ([loi n° 2011-590](https://www.wipo.int/wipolex/fr/legislation/details/18453)), so a reseller cannot fund the card by discounting the eBook. A free, professionally printed and mailed card tied to the sale risks being a "prime" (art. 4). A free print-at-home PDF is low risk. A paid optional card on its own line, at or above cost, is low risk. In reseller mode the PDF is therefore free and the printed card is a separately priced line (20% VAT; the eBook stays at 5.5%).
4. **eBook Cadeau must not collect the price of the eBook.** It would then be bound by the fixed-price law and would drift towards payment services [M]. The reseller, author or publisher sells the eBook; eBook Cadeau sells the object, the code tooling or the licence.
5. **A per-card fee competes with the reseller's margin.** If a retailer keeps about 30% of the ex-VAT price [M], that is about EUR 2.3 on an EUR 8.06 eBook, and a EUR 0.30 fee takes 13% of it. A flat licence is easier to accept until the gift option is shown to create extra orders.
6. **The code is BSD-3-Clause.** A reseller can integrate the generator without paying. Paid value must sit outside the repository: the hosted redemption service, metering and reports, print fulfilment, integration work and support.
7. **Batch prices are for identical copies.** The EUR 0.36-0.60 per card at 100 copies assumes one design. Unique codes per card need variable-data printing (Dropcards sells that at USD 0.33-0.45 a card [V1]) or one shared code with a redemption cap (the Bookwire pattern [V1]). No French quote for variable data was obtained.

### 7.4 Illustrative year-3 scenarios, France

Every volume here is an ASSUMPTION. The point is the order of magnitude.

| Line | Low | Base | High |
|---|---|---|---|
| Cards generated (SOM) | 3,060, standalone only | 26,520, one mid-size reseller | 146,880, one Fnac/Kobo-class reseller |
| Reseller fee, EUR 0.30 a card | 0 | 7,956 revenue; 6,270 after charges | 44,064 revenue; 34,720 after charges |
| Mailed upgrade, 5% of cards at EUR 6.90 (contribution 1.41) | 153 cards; 216 | 1,326 cards; 1,870 | 7,344 cards; 10,355 |
| Author plans at EUR 49 (20 / 100 / 300 authors) | 753 | 3,763 | 11,289 |
| Author packs of 100 at EUR 2.00 a jacket (10 / 50 / 150 packs) | 720 | 3,600 | 10,800 |
| **Revenue** | **about EUR 4,000** | **about EUR 32,000** | **about EUR 139,000** |
| **Contribution before fixed costs** | **about EUR 1,700** | **about EUR 15,500** | **about EUR 67,000** |
| After fixed costs (600-1,300, plus EUR 239 for the print API) | about EUR 150 to 850 | about EUR 14,000 | about EUR 66,000, an upper bound: the VAT franchise thresholds are exceeded in this case, and VAT on the consumer lines would remove up to about EUR 16,000 before input VAT is recovered |

Reading: the low case pays its own costs. The base case pays for a part-time activity. Only the high case reaches the cost of one salary (EUR 63,300 a year), and it depends on one large reseller.

### 7.5 Verdict

- **Keep**: A (free generator), C2 then C1 (authors and publishers), D2 with a metered component (reseller licence).
- **Test before building**: B2 (mailed card at EUR 6.90 or more).
- **Use only for validation**: B1 (in-house mailing).
- **Do not pursue now**: A' as a single card payment, E and F until researched, and any stock-based retail card.

---

## 8. Risks and unknowns

| # | Risk or unknown | Why it matters | Evidence status | Mitigation or test |
|---|---|---|---|---|
| 1 | Consumers may not want to pay for, or redeem, a title-specific card | Four retail programmes failed for that stated reason | Failures [V2]; reasons [V1]; no sales figures exist | Price test on the mailed option before building it; measure scans and redemptions from day one |
| 2 | Attach rate of a gift option at checkout is unknown | It drives every reseller number; 0.5% / 2% / 5% are assumptions | No source found | Free pilot with one reseller in exchange for the data |
| 3 | Gift share of eBooks (A2: 3-12%) and non-Amazon share (A3: 40-60%) are assumptions | TAM and SAM scale linearly with both | Not found | Re-run the searches; ask resellers under NDA |
| 4 | Fnac/Kobo has no title-level gift entitlement; a third party cannot add a title to a Kindle or Kobo library [M] | The largest French non-Amazon channel needs more than a card | Negative finding [V1]; platform behaviour [M] | Start with open-file stores (the 7switch type) and the Vivlio network, where Decitre already has an entitlement [V2] |
| 5 | Dependence on one reseller | Killed Livrada and Enthrill | [V2] | Keep the standalone and author modes self-sufficient; flat licence with several small resellers before one large one |
| 6 | Free printed card treated as a "prime"; fixed-price law; withdrawal right on unredeemed codes; VAT split | Could make the reseller's offer unlawful or mis-taxed | All [M] in the sibling legal note; no precedent known | Paid optional card on its own line; revocable codes; lawyer review before a reseller launch |
| 7 | No print API offers a double-scored 10 mm spine at quantity 1 | The mailed single card cannot be the true jacket through an API | [V1] (sibling note: Gelato sells standard folds only, per its [product UID article](https://apisupport.gelato.com/hc/en-us/articles/360010095259-What-is-a-product-UID-); Merci Facteur's card is 15 x 21 cm closed, per its [API README](https://github.com/MerciFacteur/Merci-facteur-API/blob/master/README.md)); "not found" is not "impossible" | Two layouts: true jacket for batches by quote, single-fold card for mailed singles |
| 8 | Real print quotes are missing; unique codes need variable-data printing | C1 margin ranges from EUR 0.2 to 1.1 a jacket | Placeholder | Three quotes from French trade printers before pricing packs |
| 9 | The 20 g postage threshold | One band up costs EUR 1.58 (2026) or EUR 1.67 (2027) a card | Tariffs [V2]; weights are my arithmetic | Weigh real samples; default to 250 g/m² for anything mailed |
| 10 | Postage inflation | Lettre verte 20 g rises 9.9% on 1 January 2027; the other letter rates rise 6.7-8.9% | [V2] | Tariffs as dated data; re-price each January |
| 11 | Seasonality | 45-55% of volume in five weeks (assumption); a missed December is a missed year | December share of book revenue [V2]; our share is an assumption | Be live before 20 November; print cut-off with a print-at-home fallback |
| 12 | Card fees on small tickets | 14% of a EUR 1.99 sale | [V2] | Credit packs; SEPA invoices for B2B |
| 13 | Micro-entreprise limits | Charges on pass-through turnover; VAT thresholds conflict between sources | Rates [V2]; thresholds [V1, conflicting] | Confirm with URSSAF or an accountant before the first mailed order |
| 14 | A thin moat | BookFunnel, Bookwire or a retailer can add a printable card; the code is BSD-3-Clause | Analysis | Compete on the object (jacket with spine), on redemption speed and on the hosted service |
| 15 | Breakage and reputation | 10-20% of codes may never be redeemed (assumption); unclear who keeps the money | Gift-card figures [V2], industry sources | Reminders to the giver; no expiry; never book breakage as revenue |
| 16 | The printed QR is a bearer secret that the printer and the postal chain can read | Theft before delivery | Analysis (sibling note) | Single-use codes, revocation and re-issue |
| 17 | Unpriced compliance costs | EPR registration and Triman marking for printed cards sent to French households; accessibility duties passed down by resellers [M] | [M] (sibling legal note) | Price them before the mailed option launches |
| 18 | The name "eBook cadeau" is descriptive in French | A word mark may be refused or weak [M] | [M] | Choose a distinctive name before paying the EUR 190 INPI fee |
| 19 | Data quality | No page read in full; several tariffs conflict | Stated throughout | Re-check official pages before modelling; raise the search budget and re-run the gap list |
| 20 | Hosting price rises | All three EU hosts raised prices in 2026; Hetzner restricted new orders [V1] | [V2] prices | Budget EUR 25 a month, not EUR 10; keep the server portable |

---

## 9. Recommended go-to-market sequence

Cheapest validation first. The thresholds in the "gate" column are proposals, not findings.

| Step | When | What | Cash cost | What to measure | Gate to the next step |
|---|---|---|---|---|---|
| **0. Free tool in public** | Now; live before 20 November 2026 to catch this Christmas, otherwise aim at Valentine's Day and Mother's Day 2027 | Print-at-home generator in the browser. Anonymous counters. A "receive it by post" button that only records interest, shown at EUR 4.90, 6.90 or 8.90 | Hosting EUR 10-25 a month; domain EUR 7-20 a year [M] | Cards generated; share scanned; share redeemed; clicks on the mailed option at each price | At least 200 cards generated in a season and at least 5% of users clicking the mailed option at EUR 6.90 or more: run step 2 |
| **1. Authors and publishers** | December 2026 to March 2027 | Recruit 10-20 self-published authors (fairs, author groups). Free code batches. Two or three pilot runs of 50-100 jackets from a trade printer by quote | EUR 50-250 HT for two or three runs of standard single-fold cards (EUR 25-80 per run of 25-50 copies; EUR 36-60 per run of 100 [V1]); a double-scored jacket by quote will cost more | Redemption rate of cards handed out at signings; how many authors pay EUR 49 a year or buy a pack | 3 payers out of 10: open the paid plan (break-even at 16-35 subscribers) |
| **2. Mailed card, by hand** | February to June 2027 (Valentine's Day, Mother's Day, Father's Day) | Up to 100 orders printed in-house: pocket format, 250 g/m², at most 20 g, Lettre verte EUR 1.67. Price EUR 6.90 | Self-funding (contribution about EUR 2.9 a card in-house at EUR 6.90, before labour) | Orders, delivery time, complaints, share of orders over 20 g | 50 paid orders or more over the test period: connect a postal-card API (EUR 19.95 a month) before Christmas 2027 |
| **3. Reseller pilot** | Talks from January 2027; live by October 2027 for Christmas 2027 | Order of approach: an open-file store of the 7switch type (card = claim on a download), then the Vivlio network (Decitre already has "Offrir"), then Fnac/Kobo (needs a prepaid entitlement). Offer three months free in exchange for attach-rate data. Demo with the fake stores in this repository | Time only | Attach rate, redemption rate, support tickets per 1,000 cards | Attach rate of 0.5% or more: propose a licence of EUR 99-499 a month plus a metered fee. Under 0.2%: keep it as a free feature and stop selling |
| **4. Formalise and extend** | When the first invoice is due | Micro-entreprise for the plan and licence lines. Company for the mailed-card line once its turnover makes charges on pass-through costs material or a VAT threshold is near. INPI filing (EUR 190) only for a distinctive name. Then Germany (2.2 times France's units; tolino alliance), then NL, ES, IT | INPI EUR 190 [V2]; accountant EUR 1,000-1,800 a year for a company [M] | Contribution per line; share of revenue from the largest customer | Largest customer under 50% of revenue before hiring or fixed commitments |

What not to do: pre-printed stock; cards on shop racks; any stored value or printed price; single card payments under EUR 5; building mailed fulfilment before the price test; a launch that depends on one reseller.

---

## 10. Sources and gaps

### 10.1 Sources from the market note

**France**

- <https://www.sne.fr/economie/chiffres-cles/>
- <https://www.livreshebdo.fr/article/marche-du-livre-2025-un-chiffre-daffaires-quasi-stable-qui-masque-de-profondes-mutations>
- <https://actualitte.com/article/132253/economie/edition-en-2025-le-chiffre-d-affaires-resiste-mieux-que-les-ventes>
- <https://www.alire.asso.fr/wp-content/uploads/2026/07/sne-synthese-statistique-2026-web-v-def.pdf>
- <https://livre-bourgognefranchecomte.fr/sites/livre-bourgognefranchecomte/files/2025-07/SNE-Synthese-Statistique-2025-Web-avec-edito.pdf>
- <https://www.idboox.com/news-livres/france-marche-du-livre-papier-et-numerique-chiffres-cles-2025/>
- <https://www.idboox.com/news-livres/marche-du-livre-en-france-etat-des-lieux-chiffres-cles-2025/>
- <https://www.livremploi.fr/actualites/decouvrez-la-synthese-2024-2025-des-chiffres-de-ledition>
- <https://www.culture.gouv.fr/Media/medias-creation-rapide/Chiffres-cles-2024_DEPS_Livre-et-lecture-publique_Fiche.pdf>
- <https://www.livreshebdo.fr/article/marche-du-livre-baisse-des-ventes-et-polarisation-accrue-en-2025>
- <https://www.ccfi.asso.fr/marche-du-livre-baisse-des-ventes-et-polarisation-accrue-en-2025/>
- <https://www.sne.fr/actu/resultats-du-barometre-2026-des-usages-dachat-et-de-lecture-des-livres-imprimes-numeriques-et-audio/>
- <https://www.sgdl.org/sgdl-accueil/le-guide-pratique/ressources-documentaires/barometres-et-observatoire/4867-barometre-sgdl-sofia-sne-2026-des-usages-dachat-et-de-lecture-des-livres-imprimes-numeriques-et-audio>
- <https://www.sgdl.org/images/phocadownload/Baromètres/BarometreSOFIA-SNE-SGDL_2026.VDEF.pdf>
- <https://www.la-sofia.org/wp-content/uploads/2026/04/BarometreSOFIA-SNE-SGDL_2026.VDEFMM.pdf>
- <https://www.livreshebdo.fr/article/barometre-sofiasnesgdl-2026-des-usages-en-recomposition-pour-les-livres-imprimes-numeriques>
- <https://horizon-jomo.eklablog.com/2026/04/barometre-sofia-sne-sgdl-des-usages-d-achat-et-de-lecture-des-livres-imprimes-numeriques-et-audio-2026.html>
- <https://fr.wikipedia.org/wiki/Liseuse>
- <https://www.liseuses.net/les-chiffres-bidons-des-ventes-de-liseuses-en-france/>
- <https://fr.accio.com/business/grande-liseuse-top-ventes>
- <https://nielseniq.com/global/fr/insights/report/2019/marche-du-livre-2018-une-industrie-de-4mds-e/>
- <https://nielseniq.com/global/fr/news-center/2018/les-francais-ont-achete-plus-de-350-millions-de-livres-en-2017/>
- <https://nielseniq.com/global/fr/insights/report/2021/tendances-du-marche-livre-en-france-formats-physiques-et-numeriques/>
- <https://www.livreshebdo.fr/article/comment-noel-sauve-le-marche-du-livre-2024-selon-gfk>
- <https://actualitte.com/article/103491/economie/le-livre-roi-des-achats-a-noel-sur-fond-de-croissance-du-marche-jeunesse>
- <https://lobsoco.com/wp-content/uploads/2026/06/LObSoCo_SLF-2026_Rapport-danalyse-6-juin.pdf>
- <https://www.chasse-aux-livres.fr/pourquoi-acheter-des-livres-d-occasion>
- <https://fr.themedialeader.com/?p=205798>
- <https://actualitte.com/article/69820/usages-numeriques/l-039-impossible-cadeau-du-livre-numerique>
- <https://theconversation.com/pourquoi-les-livres-papier-nont-ils-pas-disparu-118617>
- <https://www.idboox.com/news-livres/amazon-on-peut-enfin-offrir-un-livre-numerique-kindle/>
- <https://www.decitre.fr/faq/numerique/offrir-un-ebook>
- <https://ebook-enligne.fr/blog/offrir-ebook-guide-cadeau-numerique/>
- <https://www.buybox.net/en/blog/the-gift-card-market>
- <https://www.ecommerce-nation.fr/futur-cartes-cadeaux-france/>
- <https://socialcse.fr/place-des-cartes-redistribue-1-mde-aux-francais/>
- <https://www.europe1.fr/societe/cartes-et-cheques-cadeaux-comment-loubli-de-leur-utilisation-constitue-un-business-juteux-pour-les-enseignes-4223693>
- <https://www.sleepingmoney.com/fr-fr/etudes/achat-revente-cartes-cadeaux-france/>
- <https://www.sudradio.fr/quest-ce-qui-cloche-2/cartes-et-coffrets-cadeaux-que-faire-quand-la-date-est-depassee>

**Germany**

- <https://www.boersenverein.de/fileadmin/bundesverband/dokumente/presse/digitale_pressemappen/WIPK/Buchmarkt_kompakt_2025_2026_Zahlenuebersicht.pdf>
- <https://www.literaturcafe.de/boersenverein-praesentiert-zahlen-zum-buchmarkt-weniger-titel-weniger-umsatz-in-2025/>
- <https://www.handelsdaten.de/branchen/buchhandel>
- <https://de.statista.com/statistik/daten/studie/232200/umfrage/durchschnittspreis-von-e-books-in-deutschland/>
- <https://www.boersenverein.de/fileadmin/bundesverband/dokumente/presse/mediendossiers/verlage/Dossier_Verlage_2024.pdf>
- <https://mytolino.de/news/zehn-jahre-tolino-die-ereading-allianz-der-deutschen-buchhaendler-forciert-weiteres-wachstum/>
- <https://www.boersenblatt.net/home/hde-rechnet-mit-umsatzplus-im-weihnachtsgeschaeft-397445>
- <https://www.handelsdaten.de/anteil-des-weihnachtsgeschafts-am-jahresumsatz-einzelner-branchen-im-deutschen-einzelhandel-im-jahr>
- <https://www.handelsblatt.com/unternehmen/handel-konsumgueter/weihnachtsgeschenke-2024-wie-viel-geben-die-deutschen-fuer-geschenke-aus/100004143.html>
- <https://de.statista.com/statistik/daten/studie/71364/umfrage/beliebteste-kategorien-fuer-weihnachtsgeschenke/>

**Netherlands, Spain, Italy, UK, Europe, US**

- <https://kvbboekwerk.nl/monitor/markt/verkoopcijfers-2025>
- <https://www.stivako.nl/nieuws-blog/3327479_papieren-boek-blijft-koning-onderzoek-abn-amro>
- <https://www.lezen.nl/onderzoek/aandeel-e-boek-blijft-bescheiden/>
- <https://federacioneditores.org/wp-content/uploads/2025/10/Comercio-Interior-2024_Interior.pdf>
- <https://institutoautor.org/espana-la-federacion-de-gremios-de-editores-de-espana-publica-los-datos-de-del-comercio-interior-del-libro-en-2024/>
- <https://www.orizzontescuola.it/il-mercato-editoriale-italiano-chiude-il-2025-con-tre-milioni-di-copie-in-meno-calano-i-libri-cartacei-crescono-ebook-e-audiolibri/>
- <https://www.agenziacult.it/economia/editoria-aie-mercato-2025-a-32-mld-calo-dell12/>
- <https://network.aie.it/Portals/_default/Skede/Allegati/Skeda105-10721-2025.10.14/Rapporto_2025_sintesi.pdf?IDUNI=f0tez4j4la20peflur5ghzw1193>
- <https://thebookseller.com/news/review-of-the-year-hachette-retains-the-digital-crown-as-all-of-the-big-six-post-gains>
- <https://fep-fee.eu/European-Book-Publishing-Statistics-2024>
- <https://www.bursa.ro/the-european-book-market-nearly-24-9-billion-but-inflation-erodes-publishers39-revenues-25136061>
- <https://www.thriftbooks.com/blog/which-books-are-the-most-popular-holiday-gifts-this-year/>

### 10.2 Sources from the competitors and costs note

**Competitors and precedents**

- <https://publishersweekly.com/pw/print/20120827/53710-livrada-tries-its-luck-with-e-book-gift-cards.html>
- <https://techcrunch.com/2013/02/11/livrada-funding/amp/>
- <https://the-digital-reader.com/ebook-gift-card-startup-livrada-completes-funding-round-plans-to-shift/>
- <https://the-digital-reader.com/ebook-startup-livrada-shut/>
- <https://the-digital-reader.com/2014/12/15/ebook-gift-cards-rise-decline/>
- <https://www.thebookdesigner.com/e-books-for-real-e-book-gift-cards-from-livrada-and-enthrill/>
- <https://www.bookworks.com/2018/08/walmart-ebook-gift-card-launch-doomed/>
- <https://www.publishersweekly.com/pw/print/20140922/64046-enthrill-partners-with-walmart-canadian-publishing-2014.html>
- <https://betakit.com/ebook-product-and-distribution-startup-enthrill-raises-1-3m-from-private-investors/>
- <https://the-digital-reader.com/2017/01/12/firebrand-aquires-enthrills-ebook-gift-card-tech/>
- <https://publishingperspectives.com/2017/01/harpercollins-crowley-firebrand-enthrill/>
- <http://buildbookbuzz.com/e-book-gift-cards/>
- <https://blog.the-ebook-reader.com/2016/01/22/kindle-cards-new-book-specific-kindle-gift-cards>
- <https://shelf-awareness.com/theshelf/2016-01-26/amazon_testing_title-specific_kindle_gift_cards_.html>
- <https://www.geekwire.com/2016/amazons-other-physical-retail-test-testing-the-kindle-e-book-kiosk-at-the-local-drug-store/>
- <https://consumerist.com/2016/01/25/amazon-puts-mini-e-book-stores-in-drugstores-in-washington-state>
- <https://the-digital-reader.com/walmarts-ebookstore-is-launching-today/>
- <https://techcrunch.com/?p=1697131>
- <https://www.mobileread.com/forums/showthread.php?t=363478>
- <https://www.mobileread.com/forums/showthread.php?t=360234>
- <https://goodereader.com/blog/kobo-ereader-news/walmart-no-longer-selling-kobo-e-readers-in-their-stores>
- <https://publishersweekly.com/article/397075-Digital_Books_to_Stores_via_Symtio.php>
- <https://www.amazon.fr/gp/help/customer/display.html?nodeId=GVWGP284MQ6ZRM59>
- <https://kdp.amazon.com/fr_FR/help/topic/G200652260>
- <https://www.idboox.com/news-livres/amazon-on-peut-enfin-offrir-un-livre-numerique-kindle/>
- <https://actualitte.com/article/15659/usages-numeriques/offrir-un-ebook-amazon-facilite-les-cadeaux-de-derniere-minute>
- <https://www.decitre.fr/offrir-un-ebook>
- <https://www.liseuses.net/offrir-des-ebooks-avec-tea/>
- <https://help.vivlio.com/hc/fr/articles/11334184838428-Je-souhaite-utiliser-un-code-promotionnel-ou-une-carte-cadeau>
- <https://www.dealabs.com/discussions/carte-cadeau-kobo-ou-fnac-2273026>
- <https://www.hugendubel.de/de/shortcut/ebooksverschenken>
- <https://www.heise.de/tipps-tricks/E-Books-verschenken-so-klappt-s-7476189.html>
- <https://www.test.de/E-Books-Nuetzliche-Tipps-und-Tricks-5391265-0/>
- <https://bookwire.net/faq-download-codes>
- <https://authors.bookfunnel.com/help/print-codes/>
- <https://bookfunnel.com/pricing/>
- <https://cartmango.com/bookfunnel-pricing/>
- <https://indieauthormagazine.com/demystifying-bookfunnel/>
- <https://www.dropcards.com/pricing/>
- <https://indiesunlimited.com/2018/10/02/ebook-gift-cards-dropcards/>
- <https://www.thebookdesigner.com/dropcards-for-e-books-digital-downloads-with-a-difference/>
- <https://www.distroprint.com.au/download-cards-drop-cards>

**Price benchmarks**

- <https://www.popcarte.com/carte-postale/personnalisee-destination-photos>
- <https://support.popcarte.com/support/solutions/articles/152068-quels-sont-nos-tarifs->
- <https://www.popcarte.com/carte-virtuelle>
- <https://oiseaurose.com/popcarte-cartes-postales-personnalisees/>
- <https://www.fizzer.com/fr/carte-postale-en-ligne/>
- <https://www.fizzer.com/fr/application-carte-postale/>
- <https://www.pvt.fr/code-promo/fizzer-avis-test-carte-postale-code-promo>
- <https://support.mypostcard.com/fr/support/solutions/articles/75000054648-combien-co%C3%BBte-une-carte-postale-ou-une-carte-de-voeux->
- <https://codes-parrain.com/voyage/mypostcard>
- <https://apps.apple.com/sn/app/mypostcard-cartes-postales/id858686797?l=fr>
- <https://www.tchworks.co.uk/compare/>
- <https://www.findprices.com/uk/stores/moonpig-prices>
- <https://www.householdmoneysaving.com/why-moonpig-plus-isnt-worth-the-money/>
- <https://lessentieldeleco.fr/8883-quel-est-le-meilleur-site-de-carte-de-voeux-2027/>

**Postage**

- <https://www.laposte.fr/tarifs-postaux-courrier-lettres-timbres>
- <https://www.laposte.fr/tarif-lettre-verte>
- <https://www.laposte.fr/tarif-lettre-suivie>
- <https://www.laposte.fr/tarifs-postaux-etranger>
- <https://www.toutsurmesfinances.com/argent/a/prix-du-timbre>
- <https://tarifs-postaux.org/tarif-lettre-internationale>

**Payments**

- <https://margedaily.com/blog/frais-stripe/>
- <https://www.vincenthego.com/calculateur-frais-stripe/>
- <https://findmycompany.ai/en/guides/stripe-fees>
- <https://wise.com/fr/blog/stripe-carte-bancaire-paiement>
- <https://www.mollie.com/pricing>
- <https://www.ecomstream.eu/mollie-fees/>
- <https://www.captainfreelance.com/avis/mollie>
- <https://quasa.io/fr/media/stripe-ou-mollie-en-france-le-moyen-de-paiement-change-la-facture>
- <https://meilleur-ecommerce.com/frais-de-paiement-en-ligne/>
- <https://fr.mobiletransaction.org/paiement-en-ligne-meilleures-solutions/>
- <https://digitiz.fr/solutions-paiement-en-ligne/>
- <https://independant.io/solution-paiement-en-ligne/>
- <https://www.comparateur-pro.fr/comparatifs/meilleure-solution-paiement-en-ligne>

**Hosting and e-mail**

- <https://www.scaleway.com/en/pricing/virtual-instances/>
- <https://www.scaleway.com/en/pricing/managed-services/>
- <https://www.eucloudcost.com/providers/scaleway/>
- <https://agentxcloud.com/news/scaleway-june-2026-pricing-update>
- <https://cloudpricecalculators.com/scaleway-calculator/>
- <https://stackrates.ai/hosting/scaleway/>
- <https://betterstack.com/community/guides/web-servers/hetzner-cloud-review/>
- <https://www.bitdoze.com/hetzner-cloud-cost-optimized-plans/>
- <https://cloudtally.eu/hetzner-cloud-prices>
- <https://agentdeals.dev/hetzner-pricing-2026>
- <https://valebyte.com/en/providers/ovh/>
- <https://repha.fr/tarification-vps-2026/>
- <https://www.hebergement-vps.fr/test/ovhcloud>
- <https://www.vpsbenchmarks.com/hosters/ovhcloud/plans/vps-1>
- <https://getkanal.com/blog/brevo-pricing>
- <https://www.agencesw.com/actu/marketingdigital/tarifs-brevo/>
- <https://www.msm-web-solutions.com/brevo-email-marketing/>

**Company and trademark**

- <https://www.l-expert-comptable.com/a/52319-le-regime-de-la-micro-entreprise-pour-l-entreprise-individuelle.html>
- <https://abby.fr/guide/micro-entreprise/cotisations-sociales-auto-entrepreneur>
- <https://www.astrolabe-conseil.fr/micro-entreprise-taux-2026/>
- <https://entreprisma.fr/micro-entrepreneur>
- <https://developp.fr/micro-entreprise-plafonds/>
- <https://www.lscompta.fr/blog/urssaf-sasu/>
- <https://wisestart.fr/guides/fiscalite/cotisations-sociales-president-sasu/>
- <https://www.ngt-conseil.fr/salaire-ou-dividendes---quelle-remuneration-pour-un-dirigeant-de-sasu-en-2026--_ad41.html>
- <https://www.legalplace.fr/guides/prix-depot-marque-inpi/>
- <https://www.legalstart.fr/fiches-pratiques/depot-marque/cout-depot-marque/>
- <https://kohenavocats.com/depot-marque-inpi-prix-cout-taxes-classes-recours-2026/>
- <https://www.elly-assurance.fr/actualites/inpi/>
- <https://valorup.fr/blog/deposer-une-marque-prix/>

### 10.3 Sources taken from sibling notes

Used for print costs, fulfilment and legal context. Markers are those of the sibling notes.

- <https://www.merci-facteur.com/tarifs.php> (`printers.print-api.md`)
- <https://www.merci-facteur.com/pro/lp/api/api-carte-postale.php> (`printers.print-api.md`)
- <https://manuscry.com/manuscry-ou-merci-facteur> (`printers.print-api.md`)
- <https://github.com/MerciFacteur/Merci-facteur-API/blob/master/README.md> (`printers.print-api.md`)
- <https://apisupport.gelato.com/hc/en-us/articles/360010095259-What-is-a-product-UID-> (`printers.print-api.md`)
- <https://www.popcarte.com/envoi-direct-destinataire> (`printers.print-api.md`)
- <https://support.mypostcard.com/en/support/solutions/articles/75000054648-how-much-does-mypostcard-cost-> (`printers.print-api.md`)
- <https://www.printful.com/custom/collections/made-in-eu/greeting-card> (`printers.print-api.md`)
- <https://www.maileva.com/nos-solutions/courrier-en-ligne/> (`printers.print-api.md`)
- <https://www.bod.fr/prix> (`printers.print-api.md`)
- <https://help.api.lulu.com/en/support/solutions/articles/64000254631-are-there-fees-to-use-lulu-s-print-api-> (`printers.print-api.md`)
- <https://www.onlineprinters.fr/p/cartes-pliables-en-papier-eco-naturel-paysage-a6> (`printers.web2print.md`)
- <https://www.flyeralarm.com/fr/p/cartes-de-fidelite-525038> (`printers.web2print.md`)
- <https://manuscry.com/manuscry-ou-pixartprinting> (`printers.web2print.md`)
- <https://www.realisaprint.com/carte-correspondance-personnalisee-r2284.html> (`printers.web2print.md`)
- <https://www.rapid-flyer.com/p/carte-de-voeux-double> (`printers.web2print.md`)
- <https://www.printshot.fr/1185-flyer-2-volets> (`printers.web2print.md`)
- <https://www.printenbind.nl/en/card/greeting-card> (`printers.web2print.md`)
- <https://comparatif-imprimerie.fr/avis/exaprint/> (`printers.web2print.md`)
- <https://www.wipo.int/wipolex/fr/legislation/details/18453> (`legal-gifting-commerce.commerce.md`)

### 10.4 Could not verify

Market and demand:

- Share of book purchases made as gifts in France (Kantar / CNL "Les Français et la lecture"), Germany (Geschenkanteil) and the UK. The 15-25% used here is from memory.
- Any survey of the reasons people do not gift eBooks; any measure of how many eBooks are gifted today.
- Retailer shares of eBook sales in France (Amazon, Kobo/Fnac, Apple, Vivlio network, 7switch). Assumption A3 is unsupported.
- E-reader unit sales and installed base for France after 2013 and for Germany after about 2023; device shares per country.
- Self-publishing volumes (KDP, Librinova, BoD), number of active self-published authors, number of book fairs and signings in France.
- Number of eBook buyers in France from a purchase panel; whether the 20% and the device percentages relate to the 6+ or the 15+ population.
- Germany: eBook units for 2024. Netherlands: final 2025 figures. Italy: eBook units and average price. Spain: trade-only units and 2025 data. UK: consumer eBook revenue for 2024 and 2025. FEP: the 13% eBook share.
- France: December share of book revenue for 2024 and 2025; fourth-quarter share for eBooks in any country.
- Gift-card breakage from a public or regulatory source; EU-wide gift-card market size.
- SNE digital literature revenue in euros (seen only in truncated extracts).

Competitors, prices and costs:

- Why exactly the Target, Bartell and Walmart programmes ended; any sales or redemption figure for any eBook gift card programme.
- Whether Enthrill went through a formal insolvency; whether Dropcards still actively serves eBooks.
- Whether Kobo by Fnac, 7switch or Cultura offer any gift flow for a specific eBook; Apple Books and Google Play Books gifting in France.
- Whether BookFunnel print codes can be produced as QR codes or ready-to-print cards.
- Not searched at all: StoryOrigin, Leanpub coupons, DigitalDownloadCards; subscription gift cards (Nextory/Youboox, BookBeat, Storytel, Kobo Plus, Audible); French precedents (Emoticourt, Short Édition, Glose); German "eBook-Karten", Papego, "Buchkarte"; Seebook (Spain); bookseller gift-wrap fees; affiliate rates (Kobo, Fnac, Amazon); gift-option attach rates; corporate gifting; Clever Cloud; Lydia.
- Stripe's current premium-EEA and non-EEA rates; whether Mollie's domestic rate carries the EUR 0.25 fixed fee; Stancer's tariff; whether SumUp online is percentage-only.
- 2026 micro-entreprise VAT-franchise thresholds and turnover ceilings; which contribution rate applies to each revenue line.
- EUIPO filing fee; Brevo Starter price in euros; OVHcloud VPS-1 price; Hetzner CX23 availability; Scaleway Object Storage unit price.
- La Poste 2026 rate for 100 g; international tracked letter rate for 2027.
- Every [M] line of section 6: envelopes, domains, insurance, accountant, bank, CFE, legal documents, developer rates, support time.
- Per-piece print quotes for a double-scored jacket, with and without variable data; real weights of printed samples.
- All legal points quoted from the sibling legal note (every one is [M] there).

To close these gaps: raise the search budget and re-run the six queries listed at the top of this document, then read the official tariff pages.

---

## Implications for the implementation

1. **On demand only.** Any title, quantity 1, no stock. Do not build a feature for pre-printed inventory or retail racks.
2. **No price and no monetary value on the card.** The card model has no price field. A card stands for one prepaid order identified by an opaque single-use token. No stored value, no balance.
3. **Browser-side generation is a cost requirement.** PDF composition, QR code, cover processing and font identification run as WebAssembly in the browser. The server issues and redeems tokens, keeps counters and serves static files. The default path has no server-side PDF rendering. Target: the whole service runs on one 2 vCPU / 2 GB instance plus object storage (EUR 10-25 a month) up to about 150,000 cards a year.
4. **No personal data on the server in print-at-home mode.** Recipient name and message stay in the browser. Only the mailed option sends a name and a postal address, and only to the print provider.
5. **Size for the Christmas peak.** Assume 45-55% of yearly cards between 20 November and 24 December. Load-test for about 340-420 cards a day (France base), 1,900-2,300 a day (France high) and 6,500-7,900 a day (five countries high). Assume 30-40% of those cards are redeemed within 24 hours on 25 December; target 10 redemptions a second sustained, with static assets cached.
6. **Phone first.** Digital readers use a smartphone (48%), an e-reader (34%) or a tablet (33%). The QR must open a page that works in a phone browser and reaches "add to my library" or the download in at most two actions. Always print a short URL and a typed code beside the QR for e-reader browsers and for accessibility.
7. **Measure the funnel from day one.** Per batch or per API key, count cards generated, scanned and redeemed, with no personal data. Show the redemption rate to the issuer. Reports assume 10-20% unredeemed until real data exists.
8. **Codes are revocable and re-issuable, with no expiry by default.** This covers a lost card, a card read in transit and the buyer's withdrawal right on an unredeemed code.
9. **Weight calculator in the generator.** weight (g) = sheet area (m²) x paper weight (g/m²) + envelope (default 7.5 g). Warn above 20 g and show the price of the next postage band. Defaults: pocket jacket 110 + 10 + 110 mm wide by 178 mm high (0.041 m²); 250 g/m² for anything mailed (about 10.2 g, about 17.7 g with envelope); 300-350 g/m² only for hand delivery. A large-format jacket (135 x 215 mm covers) is flagged as falling in the 100 g band.
10. **Tariffs are dated data, not constants.** Postage (2026: EUR 1.52 at 20 g, EUR 3.10 at 100 g; from 2027-01-01: EUR 1.67 and EUR 3.34; international 20 g EUR 2.25 then EUR 2.45), payment fees and VAT rates live in a versioned data file. Each entry has a validity date and a confidence marker; the loader rejects entries without them and selects the tariff by order date.
11. **Pricing engine with five modes**: free, price per card, prepaid credit packs (5 / 10 / 20), yearly plan, and metered B2B invoice. Payment fee = rate x amount + fixed part, per provider, from the data file. Do not offer a single card payment under EUR 5 by default (the fee is 14% of EUR 1.99 and 6.6% of EUR 4.90).
12. **Mailed option priced from cost.** Floor price = (fulfilment cost + fixed payment fee) / (1 - payment rate - social rate): EUR 5.27 with outsourced fulfilment at EUR 4.29, EUR 3.48 in-house at 20 g or less, EUR 5.42 in-house above 20 g. Default price EUR 6.90. The engine refuses a price under the floor.
13. **Order cut-off.** Show the last order date for delivery before 24 December (configurable; default 7 calendar days before) and switch to print-at-home after it.
14. **Interest-only switch for the mailed option.** A configuration flag shows the mailed option at EUR 4.90, 6.90 or 8.90 and records one anonymous counter per price, without taking an order. It is needed for step 0 of section 9.
15. **Two print layouts behind one print-provider interface.** (a) True jacket with two scores 10 mm apart, for batches from a trade printer by quote. (b) Single centre-fold card, 15 x 21 cm closed, as sold by postal-card APIs, with the spine band printed and fold marks but no score. The generator outputs both from the same card data.
16. **Batch mode for authors and publishers.** N cards for one title in one job; default sizes 25 / 50 / 100 / 250. Two code modes: one single-use code per card (default; output is an N-page PDF plus a CSV of codes for variable-data printing) and one shared code with a redemption cap and optional validity (identical copies, cheapest to print). Include a free-text dedication area for signings.
17. **Reseller integration without handling money.** The generator is embeddable in a reseller's checkout (WebAssembly module with a small JavaScript API) and callable as a server API. Input: an opaque redemption URL or code issued by the reseller, plus ISBN and title metadata. Support both cases: the reseller already has a gift entitlement to wrap, or it issues a prepaid single-title code. eBook Cadeau never collects the eBook price.
18. **Metering for licences.** One anonymous counter per API key for cards generated and redeemed, with a monthly export for invoicing (flat fee plus a metered fee above a quota). Self-hosting must work without the hosted service; hosted features sit behind the API key.
19. **Standalone mode complete on its own.** It works with no reseller: author or publisher files, and a pasted redemption link issued by a retailer (Amazon "Acheter pour d'autres", Decitre "Offrir"), with a notice that the retailer's terms were not verified.
20. **Invoices keep the card apart from the eBook.** The card and its postage are a separate line at the standard VAT rate (20% in France); they are never bundled with the eBook (5.5%). The redemption territory is configurable and defaults to France.
21. **Markets and languages.** French first, German second, then Dutch, Spanish and Italian. EUR only. Postage tables are per country and optional outside France.
22. **Demo stores expose the attach rate.** The fake 7switch (open-file store) and the fake Fnac (no entitlement; the store issues a prepaid code) log orders and uses of the gift option, so the attach rate (planning values 0.5% / 2% / 5%) is a visible metric in integration tests.
23. **Financial model contract.** The model reads the `cost_inputs` block of section 6, runs low / base / high, and reports per option: contribution per unit, units needed to cover fixed costs of EUR 600 / 900 / 1,300, and volume needed for one salary (EUR 63,300 a year). It marks every result that depends on an input whose confidence is M or assumption.
