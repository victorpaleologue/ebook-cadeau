# Print-on-demand and print/postal APIs for one-off personalised cards shipped to the recipient

Research notes, angle "print API / POD". Date of research: 2026-11-12. Market: France first, then EU.

Confidence markers: **[V2]** verified in >= 2 independent sources this session, **[V1]** single source this session,
**[M]** from memory, not verified this session, **[est.]** my own arithmetic on top of sourced or remembered figures.

## 0. Method and limits (read first)

- 30 web searches were executed (EN/FR/DE). Five more (Lulu page minimum and EU plants, La Poste 2026 tariff, Gelato notebooks,
  Bookvault EU printing, Gelato sustainability) were **refused because the session's shared web-search budget was exhausted**.
  The ESG section and the book-POD section are therefore weaker than intended and lean on [M].
- WebFetch is blocked, so nothing here comes from a live price calculator. Almost every POD/trade API (Gelato, Cloudprinter,
  Prodigi, Lulu, Helloprint, Flyeralarm) only exposes exact prices through an authenticated quote endpoint or dashboard.
  **No per-unit figure below should be put in a business model without a live quote.**
- Prices are quoted in the currency of the source. HT/TTC status is given only where the source states it.

## 1. Bottom line

1. **No API found offers a custom double-scored 10 mm spine on a single card.** Card products are single-fold (or equal-panel
   tri/roll fold). Gelato's fold options are `fold-ver`, `fold-hor`, `8pp-roll-fold-ver`, `three-fold-hor`
   ([Gelato product UID article](https://apisupport.gelato.com/hc/en-us/articles/360010095259-What-is-a-product-UID-)) [V1];
   Merci Facteur's API card is `folded` = 15 x 21 cm closed / 30 x 21 cm open
   ([README](https://github.com/MerciFacteur/Merci-facteur-API/blob/master/README.md)) [V1]. "Not found" is not "impossible":
   a trade printer can do it on quote, but not at quantity 1 through a public API.
2. **A real 10 mm spine at quantity 1 through an API means book POD** (perfect-bound "dummy book"/notebook, roughly 150-200 pages [M]),
   at roughly 2-3x the delivered cost of a card (section 5).
3. **Cheapest verified path to "a folded card in a French letterbox, via API": Merci Facteur** - folded card 2.99 EUR + postage
   from 1.30 EUR, same-day print and post via La Poste, API on a 19.95 EUR/month plan (section 3) [V2 on price, own pages].
4. **Best fit for EU scale-out: Gelato** (folded cards in catalogue, order/price/catalogue APIs, production in 32 countries
   including France per reviews, published DPA) - but I could not obtain an official per-unit price.
5. Consumer card apps (Popcarte, Fizzer, MyPostcard, Touchnote, Cewe) are price benchmarks, not integration targets: no usable
   public API was found for any of them except MyPostcard's unpriced "Print API".

## 2. POD networks and print APIs (cards)

| Provider (HQ) | Card product and formats | Published unit price | Shipping to FR | Custom fold / spine | API | Production relevant to FR | Notes |
|---|---|---|---|---|---|---|---|
| **Gelato** (Norway [M]) | Flat cards, folded cards, foil, rounded corners; blank envelopes listed separately ([catalogue](https://www-api.gelato.com/en-US/products)) [V1]. Format in the UID is the finished (folded) size; files are supplied unfolded with bleed ([formats](https://support.gelato.com/en/articles/8996362-what-product-formats-and-dimensions-do-you-support)) [V1] | Not found. A third-party roundup gives "from $6.16 for a pack of 10" ([merchize](https://merchize.com/top-etsy-print-on-demand-cards-companies/)) [V1, low - attribution to Gelato uncertain] | Same roundup: "from $6 to the EU" [V1, low] | Standard folds only [V1] | Mature: catalogue, price, order v3 endpoints ([get product](https://dashboard.gelato.com/docs/products/product/get/), [create order](https://dashboard.gelato.com/docs/orders/v3/create/)) [V1] | 32 countries ([mega.management](https://mega.management/gelato-print-on-demand/)); partner counts quoted between "140+ partners" and "250+ facilities" across reviews ([stylefactory](https://www.stylefactoryproductions.com/blog/gelato-review), [syncost](https://www.syncost.com/blogs/gelato-print-on-demand-review)) [V2 for 32 countries]; France named as a local production country ([merchtitans](https://merchtitans.com/blog/print-on-demand-europe-guide)) [V1] | FR->FR delivery quoted at 2-4 business days for a poster in a review [V1]. Optional Gelato+ subscription, quoted between $14.99 and ~$30/month depending on source ([ecommerce-platforms](https://ecommerce-platforms.com/articles/gelato-print-on-demand-pricing)) [V1] |
| **Cloudprinter** (Netherlands [M]) | Commercial print "up to books on demand" ([DE page](https://www.cloudprinter.com/countries/local-printing-in-germany-with-global-print-api)) [V1]; card SKUs not confirmed | Not published; per-region quote via CloudCore price lookup, EUR default ([docs](https://docs.cloudprinter.com/client/how-to-check-prices)) [V1] | Quote only | Not found | Mature (CloudCore REST, PHP SDK on [GitHub](https://github.com/cloudprintercom/cloudapps-php-sdk)) [V1] | "170 print partners", "104 countries" (own claims) [V1] | States it adds no margin on print price and charges a small job fee ([Shopify listing](https://apps.shopify.com/cloudprinter-com)) [V1] |
| **Prodigi** (UK) | Fine-art greetings cards; classic postcards ([cards](https://www.prodigi.com/products/cards-and-stationery/greetings-cards/)) | "From GBP 0.75" excl. tax and shipping ([fine-art cards](https://www.prodigi.com/products/cards-and-stationery/greetings-cards/fine-art-greetings-cards/)) [V1] | Not found for FR. EU "5-7 working days after production" (third-party) [V1] | Not found | Mature, no setup fee ([API reference](https://www.prodigi.com/print-api/docs/reference/)) [V1] | Greetings cards "fulfilled from UK, EU, US"; classic postcards UK only [V1] | UK origin for some SKUs means customs/VAT friction into FR |
| **Printful** (US/Latvia) | "Greeting card" in the Made-in-EU collection | 1.62 EUR shown (discounted) ([Printful](https://www.printful.com/custom/collections/made-in-eu/greeting-card)) [V1]; "$2.25" in a roundup [V1] | Card rate not found; apparel reference ~4.79 EUR first item within Europe, rates changed twice in 2026 ([podvector](https://podvector.ai/articles/printful/shipping/printful-global-shipping-times-costs-and-what-to-expect)) [V1] | No | Mature ([docs](https://developers.printful.com/docs/)) | Riga (LV) and Barcelona (ES) ([Wikipedia](https://en.wikipedia.org/wiki/Printful,_Inc), [merchtitans](https://merchtitans.com/blog/print-on-demand-europe-guide)) [V2]; no French site found | |
| **Printify** (US/Latvia [M]) | Flat and folded cards via third-party providers | "$0.33 flat, $1 folded" headline ([Printify](https://printify.com/custom-greeting-card-printing/)) [V1] | Only provider surfaced was Canadian (from $2.97, shipping from $16.79) [V1] | No | Mature ([docs](https://developers.printify.com/)) | No EU card provider found | Not viable for FR single cards on current evidence |
| **Peecho** (Amsterdam) | Books, magazines, wall art; **no greeting cards found** ([products](https://www.peecho.com/products)) [V1] | Magazines from 2.90 EUR [V1] | Not found | n/a | Free API, pay per order ([Print API](https://www.peecho.com/solutions/print-api)) [V1] | "Dozens of countries" (own claim) [V1] | A thin saddle-stitched "magazine" is a possible carrier, but has no spine |
| **Blurb** (US) | Books, magazines | Not found; reviews call it premium | Self-service API is US shipping only; international via custom API on the RPI Print network ([Blurb](https://www.blurb.com/print-api-software)) [V1] | n/a | Active as of 2026-11-10 ([press release](https://www.einpresswire.com/article/955606544/blurb-launches-instant-store-a-fast-way-to-sell-direct-to-consumer-maximize-earnings)) [V1] | Not found | Low priority |
| **Mixam** (UK, DE site) | Booklets, zines, books; PrintLink POD + Print API ([mixam.de](https://mixam.de/print-on-demand), [PrintLink guide](https://mixam.com/support/printlink-guide.pdf)) [V1] | Not found | Germany listed with regional shipping only [V1] | Not found | API exists; endpoints not documented publicly in results | DE production location not confirmed | France not confirmed as a PrintLink country |

### Trade printers with reseller APIs (not quantity-1 products)

| Provider | What was found | Suitability for one-off cards |
|---|---|---|
| **Flyeralarm PRO** (DE) | Reseller API, REST at `rest.flyeralarm-esolutions.com/de/v2`, old SOAP deprecated; neutral shipping; requires reseller activation ([flyeralarm.pro](https://flyeralarm.pro/), [migration doc](https://flyeralarm.pro/wp-content/uploads/media/2025-02-10_Client-API-Documentation_Reseller-Migration_v3.pdf), [be.print wiki](https://wiki.beprint.app/pl/add-ons/reseller-api-dropshipping/informationen-dienstleister-flyeralarm-pro.html)) [V2] | Products are sold in quantity tiers (100, 250, ...) [V1]; no evidence of quantity 1 |
| **Saxoprint** (DE) | Reseller interface, API key + secret issued by customer service; free neutral shipping with the reseller's sender address ([saxoprint.de](https://www.saxoprint.de/), [FAQ](https://m.saxoprint.at/faq), [key2print](https://dropshipping.key2print.com/how-to-start-with-api/)) [V2] | Same: run-length products |
| **Onlineprinters** (DE) | `op.connect`: free API for resellers and large accounts, documentation on request ([op.connect](https://www.onlineprinters.de/c/lp/op-connect)) [V1] | Product coverage and minimum run not confirmed |
| **Helloprint** (NL) | Direct API to "10,000+ products", white-label parcels, 300+ suppliers in 30+ countries; prices behind reseller login ([reseller page](https://www.helloprint.com/en-us/reseller-solutions), [Printweek](https://www.printweek.com/content/news/helloprint-adds-new-api)) [V2] | Possible for small runs (25-50) shipped to the reseller, not per-recipient |
| **Print.com** (NL) | API for resellers ([print.com/api](https://www.print.com/en/api/)) [V1] | Details not found |
| **Pixartprinting** (IT) | **No public API evidence found.** Only company facts (2023 revenue 226 M EUR, [Wikipedia](https://en.wikipedia.org/wiki/Pixartprinting)) [V1] | Unverified |
| Also surfaced | [printapi.io](https://www.printapi.io/) (NL), [Tradeprint Connect](https://www.tradeprint.co.uk/tradeprint-api) (UK) - existence only [V1] | Not assessed |

Use of trade printers: batch-print jacket blanks or pre-scored shells (custom double score, 10 mm spine, on quote), then
personalise per order elsewhere. They do not solve "one personalised card to one recipient".

## 3. Postal-mail APIs (print + envelope + stamp)

| Provider (HQ) | Product | Price evidence | To France | API | Production / data | Fit |
|---|---|---|---|---|---|---|
| **Merci Facteur** (FR) | Postcard, flat card, **folded card 15 x 21 cm closed (A5), 30 x 21 open**, giant card; envelope included; sent by La Poste ([README](https://github.com/MerciFacteur/Merci-facteur-API/blob/master/README.md), [product page](https://www.merci-facteur.com/cartes/r5-fetes-de-fin-d-annee/r6-cartes-de-voeux/r41-carte-voeux-professionnelle/c227-voeux-entreprise-batiment/3822-harmonie-verte-pour-une-ann-eacute-e-prosp-egrave-re/)) [V1] | Postcard 1.00, flat card 1.99, **folded 2.99**, giant 3.99 EUR; postage to France **from 1.30**; tracking +0.50; discount from 11 cards ([tarifs](https://www.merci-facteur.com/tarifs.php)) [V2, same vendor, two pages]. API mail is billed at the PRO interface price [V1] | **~4.29 EUR per folded card [est.]**, +0.50 with tracking | REST, public docs on [GitHub](https://github.com/MerciFacteur/Merci-facteur-API); production API from the 19.95 EUR/month plan, 39.95 above ~2,000 cards/month ([API page](https://www.merci-facteur.com/pro/lp/api/api-carte-postale.php), [Manuscry comparison](https://manuscry.com/manuscry-ou-merci-facteur) - a competitor) [V2] | Printed and posted same day [V1]. GDPR/hosting terms **not found** | **Best verified fit for FR**. A4 open sheet gives room for front + 10 mm spine band + back, but only one centre fold |
| **Maileva** (La Poste / Docaposte, FR) | Letters (A4 in envelope), not card stock | Print from 0.219-0.224 EUR HT per B&W envelope excl. postage, rates as of 2026-01-01; postage 0-50 g: G2 1.538, G3 0.84, G4 0.747 EUR HT ([Maileva](https://www.maileva.com/nos-solutions/courrier-en-ligne/)) [V1]. Colour first page 0.28 EUR at a reseller ([Index Education](https://www.index-education.com/fr/service-maileva-edt.php)) [V1] | ~1.0-1.9 EUR HT [est.] | Yes, plus intermediaries such as [postal-api.com](https://postal-api.com/tarifs) (colour page 1 at 1.477 EUR HT incl. DL envelope) [V1] | Production and data hosting in France; same-day handover to La Poste before 14:00 in 98 % of cases ([brochure](https://www.maileva.com/app/uploads/2024/06/Plaquette_plateforme_envoi_courrier_Maileva.pdf)) [V1] | Cheap fallback: a plain A4 "print-at-home" sheet mailed to the recipient. Not a gift object |
| **Pingen** (CH) | Letters | France delivery $1.10 per letter excl. VAT ([prices](https://www.pingen.com/en/prices/)); print CHF 0.40 per letter under 500/month ([price list dated 2026-01-03](https://www.pingen.fr/en/price-list/)) [V1] | ~1.4-1.6 EUR [est.] | Mature REST | Letters to FR, DE, BE, ES, UK, AT, CH, NL printed locally ([post API](https://www.pingen.nl/en/post-api/)) [V1]; French products "Lettre verte", "Recommande" ([help](https://help.pingen.com/en/submit-and-send-letters/introduction-delivery-products)) [V1] | Letters only |
| **Stannp** (UK) | A6/A5 postcards, enveloped A5, A5 greeting cards; API sizes `A6`, `A5`, `A5-ENV`, `A5-PORT` ([API](https://www.stannp.com/uk/direct-mail-api/postcards?lang=php)) [V1] | UK volume prices: A6 from GBP 0.36, A5 greetings cards from GBP 0.50 ([Stannp UK](https://www.stannp.com/uk)) [V1, snapshot age unclear] | Not found | Mature; free test mode | UK/US/CA; no EU print site found | Weak for FR |
| **Lob** (US) | Postcards, letters | International = USPS First-Class only, 4x6" postcards only, no reliable tracking ([Lob help](https://help.lob.com/print-and-mail/building-a-mail-strategy/international-mail)) [V1] | n/a | Mature | US | **Exclude** |
| **PostGrid, Docsaway** | - | Not researched (budget) | - | - | - | See "Could not verify" |

## 4. Consumer card services (benchmarks)

| Service | Product | Price to a French address | API | Notes |
|---|---|---|---|---|
| **Popcarte** (FR) | Folded 14 x 14 cm card, satin paper, white envelope included; fully custom artwork upload exists ([custom upload](https://www.popcarte.com/votre-photo-seule/importez-votre-graphisme-personalise)) | 3.99 EUR card + **2.95 EUR "envoi direct" fee per folded card (1-49 cards)** = **6.94 EUR**; fee falls to 1.75 at 1,000+; 3.95 outside metropolitan France; tracking +1.50; 4-5 working days ([envoi direct](https://www.popcarte.com/envoi-direct-destinataire)) [V1; product pages may be ~1.5 years old] | None found | Upper benchmark for retail willingness to pay |
| **Fizzer** (FR) | 10 x 15 cm card in envelope, printed in France | 1 credit per card, worldwide postage included; "from" 1.70-2.99 EUR depending on page and pack ([Fizzer](https://www.fizzer.com/fr/carte-postale-en-ligne/), [review](https://www.pvt.fr/code-promo/fizzer-avis-test-carte-postale-code-promo)) [V2] | None found | Flat card only |
| **MyPostcard** (DE [M]) | Postcards, folded greeting cards | Greeting card 3.69-4.29 EUR incl. worldwide postage (own FAQ pages disagree) ([FAQ](https://support.mypostcard.com/en/support/solutions/articles/75000054648-how-much-does-mypostcard-cost-)) [V1] | "MyPostcard Print API" exists, no public price ([OMR](https://omr.com/en/reviews/product/mypostcard-print-api)) [V1] | Worth a sales enquiry |
| **Touchnote, Cewe** | - | Not found (Touchnote search returned nothing relevant; Cewe not searched) | - | - |

## 5. "Dummy book / notebook with ~1 cm spine" via book POD (one copy to France)

Spine arithmetic [M]: on 80-90 g/m2 uncoated book paper a 10 mm spine needs roughly 150-200 pages (Lulu's published formula,
recalled from memory, gives ~148 pages on 60# paper). A 32-page booklet has a spine of about 2 mm and cannot carry a title.

| Provider | Verified facts | Estimate for a ~160-page B&W paperback, colour cover, 1 copy to FR |
|---|---|---|
| **BoD** (DE, bod.fr) | Print from 2.05 EUR per copy ([bod.fr/prix](https://www.bod.fr/prix)) [V1]; a 300-page paperback on 90 g paper cited at 9.50 EUR ([blog](https://www.mecanismes-dhistoires.fr/imprimer-son-livre-a-la-demande-avec-bod-avis-complet/)) [V1]; shipping to France 4.50 EUR excl. VAT up to 499 g, 6-8 days ([livraison](https://www.bod.fr/mybod/livraison-et-paiement.html)) [V1]. No public order API found; publisher services exist ([editeurs](https://www.bod.fr/editeurs)) | ~5-6 EUR print + 4.50 = **~10-11 EUR HT [est., linear interpolation]** |
| **Lulu Print API** (US) | No subscription or setup fee; pay print + shipping + fulfilment fee + tax ([Lulu help](https://help.api.lulu.com/en/support/solutions/articles/64000254631-are-there-fees-to-use-lulu-s-print-api-)) [V1]; routes to the facility closest to the destination; cost endpoint needs a full address for VAT; EUR supported ([API docs](https://api.lulu.com/docs/?_gl=1)) [V1]; 200-page B&W paperback ~ $5.34 in a third-party blog ([chapter.pub](https://blog.chapter.pub/lulu-pricing/)) [V1] | ~$4.5 print + ~4-7 EUR mail [M] = **~9-12 EUR [est.]** |
| **Bookvault** (UK) | Per-binding + per-page pricing ([Author Media](https://www.authormedia.com/printing-the-future-print-on-demand-with-offset-quality-by-bookvault/)) [V1]; 256-page paperback GBP 3.70 ([cost comparison](https://karldrinkwater.substack.com/p/weekly-writers-print-on-demand-cost)) [V1]; minimum 28 pages on one paper stock [V1]; plan and title-setup fees quoted inconsistently (GBP 19.95 per title on the free plan; Pro GBP 89.95/month) ([directory](https://selfpublishingdirectory.com/print-on-demand/bookvault)) [V1] | ~GBP 3 print + UK->FR post and customs handling = **unknown, probably >= 10 EUR [est.]**; per-title setup fee is a blocker when every order is a new "title" |
| **Gelato / Printful notebooks, Cloudprinter books** | Not verified this session | - |

Reading: the dummy book costs about 9-12 EUR delivered against ~4.3-7 EUR for a folded card, takes longer (BoD 6-8 days
shipping alone) and wastes ~150 blank pages - unless the pages are useful (a reading journal/notebook). Per-title setup
models (Bookvault, BoD) fit a small catalogue of pre-made shells poorly suited to per-order personalisation; Lulu and
Cloudprinter treat every order as a file pair and fit better.

## 6. GDPR and data location

Sending a card to the recipient makes the printer a processor of the recipient's name and postal address (GDPR Art. 28), so a DPA
and a sub-processor list are required from whoever is chosen.

| Provider | Establishment | What was found |
|---|---|---|
| Gelato | Norway (EEA) [M] | Public "Data Processing Terms", last updated 2024-11-29 ([terms](https://gelato.com/legal/data-processing-terms)); sub-processor list as Schedule 2; transfers under EU-US DPF and/or SCC ([GelatoConnect DPA](https://gelato.com/legal/data-processing-agreement-for-gelatoconnect)); infrastructure on AWS with EU data-residency options ([trust page](https://www.gelato.com/es/trust)) [V1]. Contents of the sub-processor list not retrieved. Address data necessarily goes to the local print partner |
| Maileva | France | Production and hosting in France ([brochure](https://www.maileva.com/app/uploads/2024/06/Plaquette_plateforme_envoi_courrier_Maileva.pdf)) [V1] |
| Merci Facteur, Popcarte, Fizzer | France | No DPA or hosting statement found. Fizzer prints in France [V1] |
| Pingen | Switzerland (adequacy decision [M]) | Prints French letters locally [V1] |
| Cloudprinter, Peecho, Helloprint, Print.com | Netherlands [M / V1 for Peecho] | No DPA text retrieved |
| Prodigi, Stannp, Bookvault, Mixam | UK (EU adequacy decision in force [M]) | UK production for part of the range adds customs, not GDPR, friction |
| Printful, Printify, Lulu, Blurb, Lob | US parent (DPF or SCC needed [M]) | Printful EU sites in LV and ES [V2] |
| BoD, Flyeralarm, Saxoprint, Onlineprinters | Germany [M] | Not researched |

Design consequence, independent of provider: **the printed inside carries the redemption QR/code, i.e. a bearer secret that the
printer, its sub-contractor and the postal chain can read.** Mitigate in the product (single-use codes bound at first redemption,
buyer-side revocation and re-issue, optional second factor chosen by the buyer, or printing only a claim link that the
buyer must activate after delivery), and send the printer nothing beyond the PDF and the delivery address.

## 7. ESG - provisional ranking (weak evidence, see section 0)

No sustainability report, certification register or labour audit could be checked this session; the dedicated searches were refused.
The ranking below rests on structural facts that were verified (where things are printed, how far they travel, legal
jurisdiction) plus [M].

| Rank | Option | Verified basis | Unverified (to check before any public claim) |
|---|---|---|---|
| 1 | French postal-card services printing in France and handing to La Poste (Merci Facteur, Fizzer, Popcarte; Maileva for letters) | Printed in France (Fizzer [V1], Maileva [V1]); same-day handover to La Poste (Merci Facteur, Maileva [V1]); one small envelope, no parcel | Imprim'Vert, FSC/PEFC paper, employment model, carbon accounting [M at best] |
| 2 | Gelato (local production routing) | Production in 32 countries incl. France per reviews [V2/V1]; published DPA | Share of orders actually produced in-country, partner audit regime, paper certification, any EcoVadis/B Corp status |
| 3 | Cloudprinter / Helloprint (EU partner networks) | Partner networks in Europe [V1/V2] | Which partner prints for FR, certifications |
| 4 | Printful (Riga, Barcelona) | Own EU plants [V2]; 1,000-2,000 km road/air leg to France | Published sustainability data |
| 5 | BoD / Lulu / Bookvault dummy book | - | ~150 blank pages per gift is the dominant impact; Lulu B Corp status [M]; BoD FSC/climate claims [M] |
| 6 | UK- or US-origin single items (Prodigi UK SKUs, Stannp, Lob, Printify via Canada) | Cross-border shipping for a single card [V1] | - |

Social criteria (wages, ESAT/insertion partnerships, B Corp, EcoVadis) could not be assessed for any provider.

## 8. Implications for the project

- Model the printer as a pluggable `PrintProvider` with two capabilities: `folded_card` (single centre fold, envelope, postal
  delivery) and `book` (cover PDF with computed spine + interior PDF). Start with a fake provider in tests, then Merci Facteur
  (FR) and Gelato (EU); keep Lulu/Cloudprinter for the book variant.
- Generate **per-provider PDF profiles**: Merci Facteur A5-closed card (30 x 21 cm open); Gelato folded-card UID with its own
  bleed and "unfolded with bleed" artwork rule [V1]; book cover with spine width computed from page count and paper.
- With single-fold cards the "spine" can only be a printed 10 mm band next to (or straddling) the fold. A true standing spine
  needs either (a) print-at-home with fold marks, (b) a pre-scored shell ordered in a run from a trade printer, or (c) book POD.
- Budget figures to use until live quotes exist: card via French postal API ~4.3-4.8 EUR + ~20 EUR/month; consumer-grade
  folded card ~7 EUR; dummy book ~9-12 EUR. All excluding the service's own margin; VAT treatment unverified.

## Sources

Gelato: [product catalogue](https://www-api.gelato.com/en-US/products) - [product UID](https://apisupport.gelato.com/hc/en-us/articles/360010095259-What-is-a-product-UID-) - [formats](https://support.gelato.com/en/articles/8996362-what-product-formats-and-dimensions-do-you-support) - [get product API](https://dashboard.gelato.com/docs/products/product/get/) - [create order API](https://dashboard.gelato.com/docs/orders/v3/create/) - [data processing terms](https://gelato.com/legal/data-processing-terms) - [GelatoConnect DPA](https://gelato.com/legal/data-processing-agreement-for-gelatoconnect) - [trust page](https://www.gelato.com/es/trust) - reviews: [mega.management](https://mega.management/gelato-print-on-demand/), [stylefactory](https://www.stylefactoryproductions.com/blog/gelato-review), [syncost](https://www.syncost.com/blogs/gelato-print-on-demand-review), [ecommerce-platforms](https://ecommerce-platforms.com/articles/gelato-print-on-demand-pricing), [merchize](https://merchize.com/top-etsy-print-on-demand-cards-companies/), [merchtitans](https://merchtitans.com/blog/print-on-demand-europe-guide)

Cloudprinter: [price check docs](https://docs.cloudprinter.com/client/how-to-check-prices) - [Germany page](https://www.cloudprinter.com/countries/local-printing-in-germany-with-global-print-api) - [Shopify listing](https://apps.shopify.com/cloudprinter-com) - [PHP SDK](https://github.com/cloudprintercom/cloudapps-php-sdk)

Prodigi: [greetings cards](https://www.prodigi.com/products/cards-and-stationery/greetings-cards/) - [fine-art greetings cards](https://www.prodigi.com/products/cards-and-stationery/greetings-cards/fine-art-greetings-cards/) - [API reference](https://www.prodigi.com/print-api/docs/reference/)

Printful / Printify: [Printful greeting card](https://www.printful.com/custom/collections/made-in-eu/greeting-card) - [Printful, Wikipedia](https://en.wikipedia.org/wiki/Printful,_Inc) - [Printful shipping, podvector](https://podvector.ai/articles/printful/shipping/printful-global-shipping-times-costs-and-what-to-expect) - [Printful API docs](https://developers.printful.com/docs/) - [Printify greeting cards](https://printify.com/custom-greeting-card-printing/) - [Printify API](https://developers.printify.com/)

Peecho / Blurb / Mixam: [Peecho Print API](https://www.peecho.com/solutions/print-api) - [Peecho products](https://www.peecho.com/products) - [Blurb API](https://www.blurb.com/print-api-software) - [Blurb press release 2026-11-10](https://www.einpresswire.com/article/955606544/blurb-launches-instant-store-a-fast-way-to-sell-direct-to-consumer-maximize-earnings) - [Mixam DE](https://mixam.de/print-on-demand) - [Mixam PrintLink guide](https://mixam.com/support/printlink-guide.pdf)

Book POD: [Lulu API fees](https://help.api.lulu.com/en/support/solutions/articles/64000254631-are-there-fees-to-use-lulu-s-print-api-) - [Lulu API docs](https://api.lulu.com/docs/?_gl=1) - [Lulu pricing, chapter.pub](https://blog.chapter.pub/lulu-pricing/) - [BoD prix](https://www.bod.fr/prix) - [BoD livraison](https://www.bod.fr/mybod/livraison-et-paiement.html) - [BoD editeurs](https://www.bod.fr/editeurs) - [BoD review](https://www.mecanismes-dhistoires.fr/imprimer-son-livre-a-la-demande-avec-bod-avis-complet/) - [Bookvault, Author Media](https://www.authormedia.com/printing-the-future-print-on-demand-with-offset-quality-by-bookvault/) - [Bookvault cost comparison](https://karldrinkwater.substack.com/p/weekly-writers-print-on-demand-cost) - [Bookvault directory entry](https://selfpublishingdirectory.com/print-on-demand/bookvault)

Postal APIs: [Merci Facteur tarifs](https://www.merci-facteur.com/tarifs.php) - [Merci Facteur API carte postale](https://www.merci-facteur.com/pro/lp/api/api-carte-postale.php) - [Merci Facteur API GitHub](https://github.com/MerciFacteur/Merci-facteur-API) - [README](https://github.com/MerciFacteur/Merci-facteur-API/blob/master/README.md) - [Manuscry vs Merci Facteur](https://manuscry.com/manuscry-ou-merci-facteur) - [Maileva courrier en ligne](https://www.maileva.com/nos-solutions/courrier-en-ligne/) - [Maileva brochure](https://www.maileva.com/app/uploads/2024/06/Plaquette_plateforme_envoi_courrier_Maileva.pdf) - [Index Education / Maileva](https://www.index-education.com/fr/service-maileva-edt.php) - [postal-api.com tarifs](https://postal-api.com/tarifs) - [Pingen prices](https://www.pingen.com/en/prices/) - [Pingen price list](https://www.pingen.fr/en/price-list/) - [Pingen post API](https://www.pingen.nl/en/post-api/) - [Pingen delivery products](https://help.pingen.com/en/submit-and-send-letters/introduction-delivery-products) - [Stannp UK](https://www.stannp.com/uk) - [Stannp postcards API](https://www.stannp.com/uk/direct-mail-api/postcards?lang=php) - [Lob international mail](https://help.lob.com/print-and-mail/building-a-mail-strategy/international-mail)

Card services: [Popcarte envoi direct](https://www.popcarte.com/envoi-direct-destinataire) - [Popcarte custom upload](https://www.popcarte.com/votre-photo-seule/importez-votre-graphisme-personalise) - [Fizzer](https://www.fizzer.com/fr/carte-postale-en-ligne/) - [Fizzer review, pvt.fr](https://www.pvt.fr/code-promo/fizzer-avis-test-carte-postale-code-promo) - [MyPostcard FAQ](https://support.mypostcard.com/en/support/solutions/articles/75000054648-how-much-does-mypostcard-cost-) - [MyPostcard Print API, OMR](https://omr.com/en/reviews/product/mypostcard-print-api)

Trade printers: [Flyeralarm PRO](https://flyeralarm.pro/) - [Flyeralarm API migration doc](https://flyeralarm.pro/wp-content/uploads/media/2025-02-10_Client-API-Documentation_Reseller-Migration_v3.pdf) - [be.print wiki on Flyeralarm PRO](https://wiki.beprint.app/pl/add-ons/reseller-api-dropshipping/informationen-dienstleister-flyeralarm-pro.html) - [Saxoprint](https://www.saxoprint.de/) - [Saxoprint FAQ](https://m.saxoprint.at/faq) - [key2print on Saxoprint API](https://dropshipping.key2print.com/how-to-start-with-api/) - [Onlineprinters op.connect](https://www.onlineprinters.de/c/lp/op-connect) - [Helloprint reseller solutions](https://www.helloprint.com/en-us/reseller-solutions) - [Printweek on Helloprint API](https://www.printweek.com/content/news/helloprint-adds-new-api) - [Print.com API](https://www.print.com/en/api/) - [Pixartprinting, Wikipedia](https://en.wikipedia.org/wiki/Pixartprinting) - [printapi.io](https://www.printapi.io/) - [Tradeprint Connect](https://www.tradeprint.co.uk/tradeprint-api)

## Could not verify

- **Any official per-unit price + shipping to France for Gelato, Cloudprinter, Prodigi, Printful (card shipping), Lulu, Helloprint,
  Flyeralarm, Saxoprint, Onlineprinters, Print.com, Mixam.** All require a live quote or a logged-in dashboard.
- Whether the "$6.16 per pack of 10 / $6 to the EU" figure in the merchize roundup really refers to Gelato.
- Merci Facteur: whether 2.99 / 1.30 EUR are TTC or HT, the exact API-side price of a folded card, the postal product used,
  accepted artwork formats for custom folded cards, DPA and hosting location. The "from 1.30 EUR" postage looks low against
  the 2026 "lettre verte" tariff, which I could not check (search refused).
- La Poste 2026 public tariffs (lettre verte 20 g / 100 g).
- Lulu: minimum page count, spine formula, EU print locations, shipping rate to France (search refused; figures above are [M]).
- BoD: minimum page count, existence of an order API, author-copy shipping rules for a single copy.
- Bookvault: EU production or EU shipping rates after Brexit; current plan fees (sources conflict); API terms.
- Gelato notebooks / photo books and Printful notebooks as spine carriers (search refused).
- Pixartprinting "PRO API": no evidence found that a public API exists.
- PostGrid, Docsaway, Touchnote, Cewe: not researched or no relevant result.
- Production locations of Cloudprinter, Peecho, Helloprint and Gelato partners that actually serve French orders.
- **All ESG certifications and labour practices** (Imprim'Vert, FSC/PEFC, EcoVadis, B Corp, ClimatePartner, ESAT partnerships)
  for every provider; section 7 is a structural ranking only.
- Sub-processor lists and data-residency defaults for every provider (Gelato's Schedule 2 not retrieved).
- Headquarters marked [M] (Gelato Norway, Cloudprinter Netherlands, MyPostcard Germany, UK/CH adequacy status).
