# Fact-check report — market-business

_Produced by an adversarial fact-checker on 2026-11-12. Read this next to the matching document: where that document's status is still "draft", these corrections were NOT applied._

# Fact-check report: docs/research/12-market-and-business-inputs.md

## 0. Limit that governs every verdict below

**No fresh web search was possible, so no external claim in this report is independently verified.** My first three WebSearch calls were refused: the workflow-wide budget of 200 searches per turn was already spent. I did not work around it. To finish the job, raise `CLAUDE_CODE_MAX_WEB_SEARCHES_PER_SESSION` or send a follow-up turn, then run the queries in section 5.

What I did instead:

- **[recomputed]**: I redid the document's arithmetic and parsed its JSON block with a script.
- **[repo]**: I cross-read the sibling documents (`04-printers.md`, `09-legal-gifting-and-commerce.md`, `notes/*.md`).
- **[memory]**: my own recollection, with a confidence level. It is not evidence. Treat each one as a lead to check, never as a correction to apply.

Following the brief, any external fact I could not re-verify is UNSUPPORTED. WRONG and PARTLY are used only where the evidence is in the repository or in the arithmetic.

File checked: `/home/user/ebook-cadeau/docs/research/12-market-and-business-inputs.md` (not edited).

## 1. The 20 most load-bearing claims

| # | Claim | Verdict | Corrected statement or lead | Where to re-check |
|---|---|---|---|---|
| 1 | France retail 2025: about 17 M eBooks, EUR 137 M (+4%); print 307 M copies, EUR 3.9 bn (NielsenIQ BookData) | UNSUPPORTED | Single origin (Livres Hebdo; CCFI is a relay). Derived EUR 8.06 and the 3.4% / 5.2% shares recompute correctly. The panel's scope is not stated: if it excludes Amazon/KDP self-published titles, applying assumption A3 (non-Amazon 40-60%) discounts Amazon twice, or the base is understated | https://www.livreshebdo.fr/article/marche-du-livre-baisse-des-ventes-et-polarisation-accrue-en-2025 |
| 2 | SNE 2025: revenue EUR 2,883.3 M; digital EUR 275.3 M (10%, -1.2%); literature eBook share 5.8% (6.2% in 2024) | UNSUPPORTED | 2024 figures (2,901.6 / 278.6) agree with [memory, medium]. Derived values recompute (67.4% gives 185.6 M; 694.6 x 5.8% gives 40.3 M) | https://www.sne.fr/economie/chiffres-cles/ |
| 3 | Germany 2025: 37.2 M eBooks (-4.5%), 6.3% share, EUR 7.01, 3.4 M buyers | UNSUPPORTED | Internally consistent (3.4 M x 10.9 = 37.1 M; 76.46 / 10.9 = 7.01). "2.2 times France" compares a consumer panel with a retail panel of unknown scope | Börsenverein PDF cited in section 2.1 |
| 4 | Baromètre 2026: 14 M digital readers; smartphone 48%, e-reader 34%, tablet 33%; 20% buyers | UNSUPPORTED | Derived 4.76-4.9 M e-reader users recomputes. Population base still unknown, as the document admits | SGDL / SNE pages cited in section 2.2 |
| 5 | Five-country base 87.5 M units, EUR 693 M; share "flat everywhere", "flat or slightly rising" | PARTLY [recomputed] | Sums are right. The trend wording contradicts the document's own data: French literature share fell 6.2% to 5.8%, SNE digital -1.2% and -1.6%, German units -4.5%. Say "flat to declining in units; value held up by price". The Spanish 18.4 M is all-category publisher digital revenue divided by a trade price, so the total is an upper bound | Section 2.1 of the document |
| 6 | La Poste: Lettre verte 20 g EUR 1.52 (2026), 1.67 (2027); 100 g 3.10, 3.34; international 20 g 2.25, 2.45 | UNSUPPORTED | 1.52 and 2.25 agree with [memory, high / medium]. **3.10 is suspect**: the 100 g rate was exactly twice the 20 g rate in 2024, 2025 and again in the 2027 figures, which gives 3.04 (band jump 1.52, tracked 3.54); the quoted online price 2.97 is close to 2 x 1.49. All 2027 values are beyond what I can recall | https://www.laposte.fr/tarif-lettre-verte |
| 7 | Stripe: 1.5% + EUR 0.25; premium EEA 2.8% kept as base; non-EEA 3.15% as base | UNSUPPORTED (likely WRONG on premium) | [memory, high as of 2025] Premium EEA is 1.9% + 0.25 and non-EEA is 3.25% + 0.25; 2.8% looks like a comparator error. Standard rate, UK 2.5%, +2% conversion and EUR 20 dispute agree with memory | Stripe's French pricing page |
| 8 | Mollie 1.2% + 0.25 (commercial 2.9%); GoCardless 1% + 0.20 capped at EUR 2; Stancer 0.7% + 0.15 | UNSUPPORTED (GoCardless cap likely OUTDATED) | [memory, medium] GoCardless cap is EUR 4; Mollie commercial and non-EU cards are 3.25% + 0.25; Mollie's EUR 0.25 fixed part does apply. Stancer unknown | https://www.mollie.com/pricing |
| 9 | Micro-entreprise charges 2026: 12.3% goods, 21.2% BIC services, 25.6% BNC | UNSUPPORTED (BNC likely WRONG) | [memory, medium-high] BNC is 26.1% from 1 January 2026 (23.1%, 24.6%, 26.1% under the 2024 decree). Fix `tax_micro_social_services_bnc`. The BNC rate is not used in the model. Training levy (about 0.1-0.3%) is not included | URSSAF auto-entrepreneur rates page |
| 10 | Ceilings EUR 203,100 / 83,600; VAT franchise EUR 85,000 / 37,500 | UNSUPPORTED | [memory, medium-high] Both pairs are right; 91,900 / 36,800 were the 2023-2024 thresholds, so the "conflict" is old against new. Missing: tolerance thresholds 93,500 / 41,250, and the EUR 25,000 single-threshold episode of 2025 (see 3.4) | impots.gouv / service-public |
| 11 | Flat tax 31.4% in 2026; SASU cost 1.82 x net (EUR 5,275 for 3,000 net); corporate tax 15% to EUR 42,500 | UNSUPPORTED | Agrees with [memory, medium-high for the flat tax; high for corporate tax]. 5,275 x 12 = 63,300 recomputes | URSSAF / impots.gouv |
| 12 | INPI EUR 190 + 40 per class, renewal 290; EUIPO EUR 850 | UNSUPPORTED | Agrees with [memory, high]. The EUIPO "conflict" can probably be closed at 850 (second class 50, then 150 each) | INPI and EUIPO fee pages |
| 13 | Outsourced mailed card lands at EUR 4.29; "works only at EUR 6.90 or more"; high value 4.79 | **PARTLY, WRONG as a base case [repo + recomputed]** | `04-printers.md` (lines 121-124, 645) calls 4.29 a floor. The Merci Facteur card is A4 open, about 19 g on 300 g board before the envelope, so it probably falls in the 100 g band: ceiling EUR 6.09 (2026) to 6.33 (2027). HT or TTC is unknown. See section 2.1 for the effect | https://www.merci-facteur.com/tarifs.php, test order |
| 14 | "Four title-specific card programmes all failed", marked [V2] | PARTLY [repo] | Only Livrada and Enthrill are [V2] in the document's own table. Kindle Cards rests on one source. Walmart's "dead" rests on forum posts about device login and e-reader sales, after about six years of operation; store closure and poor card sales are not established | Section 4.1 of the document |
| 15 | Livrada in 1,500 of 1,700 Target stores; Kindle Cards in 61 Bartell stores; The Martian USD 14.99 against 8.99 | UNSUPPORTED | The outline agrees with [memory, medium]; the exact figures were not re-verified, and the document itself notes two excerpts "not pinned" | Publishers Weekly and GeekWire links in section 4.1 |
| 16 | BookFunnel print codes need a USD 200 or 300 plan (rise on 20 Feb 2026); "no evidence it produces a card or QR: that is the gap eBook Cadeau fills" | UNSUPPORTED | The rise rests on one comparator; my recollection stops at the old USD 20 / 100 / 250 list. The "gap" is an argument from absence on a search extract and should not be stated as positioning fact | https://bookfunnel.com/pricing/ and https://authors.bookfunnel.com/help/print-codes/ |
| 17 | Authors and publishers are "the only segment with a proven, continuing payer (BookFunnel, Dropcards, Bookwire)" | UNSUPPORTED, overstated [repo] | Bookwire's price is "not found"; Dropcards' eBook activity is "not verified"; BookFunnel customers pay for file delivery and list-building, of which print codes are a side feature. Evidence supports "authors pay for delivery tooling", not "authors pay for printed code cards" | Section 4.3 of the document |
| 18 | "Every existing gifting flow is e-mail only. None produces a physical object"; Kobo by Fnac has no gift flow | PARTLY [repo] | A universal claim from five retailers; seven named ones were "not searched". Amazon's flow gives the buyer redemption links, which are printable, and the document's own implication 19 relies on that. Thalia's delivery mechanics are "not confirmed". The Fnac finding rests on a Dealabs thread and one blog | Section 4.2 of the document |
| 19 | Platform map (Fnac = Kobo; Cultura, Decitre, Furet du Nord, E.Leclerc = Vivlio; 7switch = Immatériel); "Kobo by Fnac, the largest French non-Amazon channel"; 7switch as an "open-file store" | UNSUPPORTED | The map agrees with [memory, medium]. "Largest" has no source: the document says retailer shares were not found. `notes/ereader-ux.ecosystems.md` line 66 says 7switch has "some DRM titles", so the claim-a-download flow covers only part of its catalogue | Retailer sites |
| 20 | Fixed-price law art. 4 on "primes"; VAT 5.5% / 20% (FR), 7% / 19% (DE); OSS threshold EUR 10,000 | UNSUPPORTED | All are [M] in the document and in `09-legal-gifting-and-commerce.md` (line 359: "quoted from memory and must be read on Légifrance"). They agree with [memory, medium-high], including the art. 4 wording | Légifrance; https://www.wipo.int/wipolex/fr/legislation/details/18453 |

Also UNSUPPORTED and not re-verified: hosting prices and the Hetzner order restriction (rows 367-377), Popcarte / Fizzer / MyPostcard / Moonpig prices, Dropcards and Distro Print prices, the gift-card market and breakage figures, the December share of book revenue, and the L'ObSoCo "2.6 books bought to give".

## 2. Internal findings (firm: recomputed or read in the repo)

1. **Mailed-card economics are too optimistic** (claim 13). With the micro-entreprise formula the document uses:

   | Landed cost | Case | Price floor | Contribution at EUR 6.90 | At EUR 8.90 |
   |---|---|---|---|---|
   | 4.29 | document's base | 5.27 | +1.41 | +3.13 |
   | 4.89 | 2.99 is HT, VAT not recoverable | 5.96 | +0.81 | — |
   | 6.33 | 100 g band, 2027 | 7.63 | -0.63 | +1.09 |
   | 6.93 | both | 8.33 | — | +0.49 |

   `fulfil_postal_api_landed` should be 4.29 / about 5.3 / 6.33 or more, not 4.29 / 4.29 / 4.79. The verdict on option B2, GTM step 2 and implication 12 depend on it.
2. **Fixed costs.** The planning range EUR 600 / 900 / 1,300 is below the document's own line-item high of EUR 1,830 (section 6.2, item 10), and excludes the print API plan (EUR 239) and a EUR 25 a month hosting budget (risk 20). At EUR 1,830, break-even is about 49 author plans, 7,740 cards at a EUR 0.30 fee, or 1,500 in-house mailed cards. The summary's "16-35 authors" and "2,500-5,500 cards" understate the high end.
3. **Amazon is excluded from the SAM but supported by the product.** Assumption A3 removes Amazon units, while implication 19 and option E wrap Amazon's gift links. One of the two needs to change.
4. **Lesson 4.4 item 3** ("redemption friction killed the earlier products; account creation plus code typing") has no source anywhere in section 4.1. It is an inference.
5. **Seasonality.** The 45-55% share between 20 November and 24 December is far above the cited evidence (December is 17-21% of French book revenue; November plus December is 23.7% in Germany). It drives the load-test targets in implication 5.
6. **GTM step 0** ("live before 20 November 2026") is eight days after the research date, with the build still in progress.
7. Small slip: a EUR 3-4 card adds 37-50% to an EUR 8.06 eBook, not 30-50%.
8. **Checked and correct:**
   - The JSON block matches the table: 162 entries, no mismatch, low <= base <= high everywhere.
   - TAM / SAM / SOM, contributions, price floors (5.27 / 3.48 / 5.42), the section 7.4 scenarios, jacket weights and the postage-increase range all recompute.

## 3. Omissions

No URL is given because none could be retrieved this session; each is a [memory] lead.

1. **French e-invoicing reform** [high]: every business, micro-entrepreneurs included, must be able to receive electronic invoices from 1 September 2026; small firms must issue them and e-report B2C sales from 1 September 2027. This affects the reseller licence invoices, the "monthly export for invoicing" and fixed costs. Not in the document.
2. **European Accessibility Act** [high]: Directive 2019/882 has applied since 28 June 2025 to eBooks and e-commerce, with a micro-enterprise exemption for services. Risk 17 mentions it only as a vague [M].
3. **Counter-examples to "physical tokens for digital content fail"** [medium-high]: Tonies and Yoto cards (children's audio), "code in a box" video games, and free print-at-home gift cards at Amazon.fr and Fnac. The last one supports pricing the PDF at zero. The document's sample of precedents is only failed eBook cards.
4. **VAT franchise history** [medium]: a single EUR 25,000 threshold was voted for March 2025, suspended, then repealed. This explains the conflicting sources and is a regulatory risk for the micro-entreprise plan.
5. **CFE** [high]: businesses with turnover of EUR 5,000 or less are exempt from the minimum CFE, so the low scenario pays none. The 200 / 400 / 600 line overstates it for small turnover.
6. **Works-council route (option F)** [medium]: cultural vouchers given by a works council are exempt from social contributions without a ceiling; the general gift-voucher limit is 5% of the monthly social-security ceiling. This is the obvious lever for option F, which the document leaves unresearched.
7. **Title-level gifting elsewhere** [medium-low]: Polish eBook stores offer "kup na prezent"; Apple Books and Audible gift specific titles; Libro.fm offers printable gifts. Each would test the "e-mail only, no object" claim.
8. **Partner stability** [low; check before GTM step 3]: the financial situation of the Furet du Nord / Decitre group, which is behind Vivlio, in 2025-2026, and the reported 2026 takeover offer for Fnac Darty.
9. **Subscriptions** (Kobo Plus, Kindle Unlimited) are absent from the "stable niche" reading, although they move reading away from unit purchases.
10. **Postal rules**: whether a sold printed card counts as goods in an international letter, and whether Merci Facteur's "from EUR 1.30" is a bulk rate with different weight bands.

## 4. Statements to downgrade to "unverified"

- `fulfil_postal_api_landed` and every sentence saying the mailed card "works at EUR 6.90".
- "Four programmes all failed [V2]": [V2] for two, [V1] for Kindle Cards, forum-grade for Walmart.
- "The only segment with a proven, continuing payer" and "that is the gap eBook Cadeau fills".
- "Every existing gifting flow is e-mail only. None produces a physical object."
- "Kobo by Fnac, the largest French non-Amazon channel."
- "Redemption friction killed the earlier products."
- "Flat everywhere" / "flat or slightly rising".
- `postage_fr_lettre_verte_100g_2026` (3.10), the two rows derived from it, and all 2027 postage rows until read on laposte.fr.
- `pay_stripe_eea_premium_pct` base 2.8%, `pay_stripe_non_eea_pct` base 3.15%, `pay_gocardless_cap` 2.00, `pay_mollie_commercial_pct` 2.9%, `tax_micro_social_services_bnc` 25.6%.
- BookFunnel USD 200 / 300 and the 20 February 2026 rise.
- Hetzner availability and all three hosts' 2026 prices.
- The lean fixed-cost range 600-1,300 and the break-even ranges built on it.
- The 45-55% peak share and the load figures derived from it.
- [V2] markers that rest on one vendor's two pages (Merci Facteur, Decitre) should read [V1].

## 5. Queries to run when the search budget is restored (priority order)

1. Merci Facteur folded-card tariff: HT or TTC, postage by weight band.
2. La Poste 2026 Lettre verte 100 g rate, and the 2027 tariff table.
3. URSSAF micro-entrepreneur BNC rate for 2026; VAT franchise thresholds for 2026.
4. Stripe France premium-EEA and non-EEA card rates; GoCardless fee cap; Mollie France card rates.
5. BookFunnel 2026 pricing; whether print codes come with a QR or printable card.
6. NielsenIQ BookData scope for French eBooks (Amazon, self-published titles).
7. Fnac and Kobo "offrir un ebook"; Thalia gift delivery mechanics; Polish "kup na prezent".
8. Walmart eBooks closure date; status of Furet du Nord / Decitre / Vivlio.
9. E-invoicing calendar for micro-entrepreneurs.
10. Share of French book purchases made as gifts (CNL / Ipsos, Kantar).
11. Hetzner, Scaleway and OVHcloud 2026 prices and Hetzner availability.
