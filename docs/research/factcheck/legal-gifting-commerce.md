# Fact-check report — legal-gifting-commerce

_Produced by an adversarial fact-checker on 2026-11-12. Read this next to the matching document: where that document's status is still "draft", these corrections were NOT applied._

# Fact-check of docs/research/09-legal-gifting-and-commerce.md

## 0. Limitation: no web verification ran

- **Zero searches executed.** All four WebSearch calls I sent (CJEU C-366/24, La Poste 2026 tariffs, franchise en base 2026, Danish book VAT) were refused: "this turn's web search budget is used up (limit: 200 WebSearch calls per turn, shared by every agent in it)". I did not work around it.
- **Consequence.** No external claim was re-verified. Under the "default to UNSUPPORTED" rule every external claim below is UNSUPPORTED.
- **What I did instead**, tagged per finding:
  - **[LOCAL]** recomputed or checked against files in this repository. These are real verdicts.
  - **[REPO]** rests on a sibling note's earlier search; the URL is in the repo and I did not reopen it.
  - **[RECALL]** my own recollection (knowledge to mid-2026). It is a lead for the next verification pass, not evidence.
- **URLs.** I add none. Every URL cited below already exists in the repository.

## 1. The 20 most load-bearing claims

| # | Claim (location) | Verdict | Corrected statement / note | Sources |
|---|---|---|---|---|
| 1 | *Tom Kabinet* C-263/18: eBook download is communication to the public, no exhaustion (§1.2, §2.2) | UNSUPPORTED; recall agrees (high) | Grand Chamber, 19 Dec 2019, ECLI:EU:C:2019:1111. I know of no later CJEU decision reopening digital exhaustion for eBooks. Paragraph numbers 69–72 unchecked. | [REPO] https://www.scl.org/10768-cjeu-sale-of-second-hand-e-books-via-a-website-is-a-communication-to-the-public/ |
| 2 | CPI L122-5 2° wording ("source licite", "usage privé du copiste"); loi 2011-1898; *ACI Adam* (§2.5) | UNSUPPORTED; recall agrees (high) | No correction. | [REPO] https://www.baumann-avocats.com/CPI/article-l122-5-du-code-de-la-propriete-intellectuelle.php |
| 3 | CPI L335-2-1 (3 years, EUR 300,000); L335-3-1 (EUR 3,750; 6 months and EUR 30,000); R335-3 4th class (§1.2, §2.4, §2.6) | UNSUPPORTED; recall agrees (high) on amounts | L335-2-1 targets a "logiciel manifestement destiné à la mise à disposition **du public** non autorisée". Applying it to a hosted one-to-one service is the document's extrapolation; label it (analysis), not [M]. | none |
| 4 | *YouTube/Cyando* and DSA recital 20 "remove the hosting exemption" (§1.2 point 4, §2.6) | PARTLY | Recall agrees on date, case numbers, the three indicators and the recital 20 wording. Overstated: the *Cyando* indicators decide whether the operator itself communicates to the public; the exemption was answered separately (active role, knowledge). [LOCAL] §2.6 also says "a one-token transfer is not dissemination to the public", but docs/api.md v0.2 makes a hosted file "re-downloadable by whoever holds the card", 10 sessions, no expiry. The "one-recipient" premise is not what the contract implements. | [LOCAL] docs/api.md lines 81–89 |
| 5 | Cass. 1re civ. 23 Oct 2024 n° 23-13.738 *Valve*; *Dstorage* CA Paris 12 Apr 2023, Cass. Feb 2025 (§2.2, §2.6) | UNSUPPORTED; recall agrees (medium-high) | Recall: the *Dstorage* ruling is Cass. **com.**, 26 Feb 2025; state the chamber. | none |
| 6 | Store gifting: Amazon.fr, Decitre/Vivlio, Thalia/Hugendubel yes; Kobo/Fnac none; "every store that allows a gift sells a fresh entitlement" (§2.3, §3.6, §4.2) | PARTLY | The three positives are [REPO V2], not reopened. The Kobo negative rests on a Dealabs thread and a low-authority blog. Recall (medium): Rakuten Kobo launched title-level eBook and audiobook gifting on kobo.com in late 2024 in several English-speaking markets; France and Fnac unknown. If so, "Fnac/Kobo has to create the entitlement mechanism" is overstated. 7switch: still unknown. | [REPO] https://www.amazon.fr/gp/help/customer/display.html?nodeId=GVWGP284MQ6ZRM59 ; https://www.decitre.fr/offrir-un-ebook ; https://www.dealabs.com/discussions/carte-cadeau-kobo-ou-fnac-2273026 |
| 7 | Kindle: "licensed, not sold"; USB transfer withdrawn Feb 2025; US licence wording since 2025; "reported" 2026 EPUB/PDF downloads (§2.3, Could-not-verify 16) | UNSUPPORTED; recall agrees | Recall (medium-high): from 20 Jan 2026 verified purchasers can download DRM-free Kindle titles as EPUB/PDF. This is no longer just "reported", and it means retail DRM-free EPUBs without Adobe/LCP markers now circulate, so the upload gate's retail-copy detection matters more. | none |
| 8 | Loi 2011-590 arts 2–4 (price binds sellers to buyers in France; primes only if offered by the publisher to all); décret 2012-146 3rd-class fine (§5.1) | UNSUPPORTED; recall agrees (high on arts 2–4, medium on the fine class) | Art. 4 opens "sous réserve des dispositions de l'article L. 121-35 du code de la consommation". Only the general principle is [REPO V2]. | [REPO] https://www.wipo.int/wipolex/fr/legislation/details/18453 ; https://www.livreshebdo.fr/article/la-loi-sur-le-prix-en-vigueur |
| 9 | Loi Lang art. 1 lets a retailer add fees for "prestations supplémentaires exceptionnelles"; "the logic transfers" to a paid card (§5.1, [M-hi]) | PARTLY; recall (high) | The sentence reads "Tout détaillant doit offrir le service gratuit de commande à l'unité. Toutefois, **et dans ce seul cas**, le détaillant peut ajouter…". The surcharge is confined to unit orders. The quotation drops the limiting words, so the analogy is weaker than presented. | none |
| 10 | Darcos: EUR 3 minimum under EUR 35, in force 7 Oct 2023; CJEU C-366/24 outcome unknown (§5.4) | UNSUPPORTED; recall agrees on figures and case number | Recall (low on content): an Advocate General opinion was delivered in 2025, so a judgment very probably exists by Nov 2026. I do not know the outcome. No effect on the card either way. | none |
| 11 | BuchPrG § 2(1) Nr. 3, § 3, § 7(4); BGH I ZR 83/14; Germany more permissive on the card (§5.5) | UNSUPPORTED; recall agrees (high) | No correction. | none |
| 12 | VAT rates: FR 5.5/20, DE 7/19, IE 0, DK 25 and others (§6.1) | PARTLY | Recall agrees for FR, DE (since 18 Dec 2019), IE 0 (1 Jan 2024), BE, NL, ES, IT, AT, LU, PT, PL, SE. **DK 25 is probably outdated**: recall (medium) that Denmark decided to abolish VAT on books in 2026; effective date and eBook coverage need checking. | none |
| 13 | CGI art. 278-0 / 257 ter: highest rate on a composite operation (§1.4, §6.2) | UNSUPPORTED; recall agrees (medium-high) | §1.4's "one bundled price risks the highest rate" is loose. Art. 278-0 applies only when the elements form a single operation under 257 ter and none is ancillary; a single price is an indicator, not the test. | none |
| 14 | CJEU C-68/23 *M-GbR*, 18 Apr 2024: "codes limited to users in Germany are SPVs" (§6.3) | PARTLY; recall (medium-high) | The case concerned prepaid store credit with a country identifier. The Court held the place of supply was known; whether the VAT due was known (the second SPV condition) was left to the referring court. Say "can be SPVs if both conditions are met". Applying it to a one-title code is the document's own analysis. | none |
| 15 | EUR 10,000 threshold, OSS since 1 July 2021; franchise en base "EUR 85,000/37,500 vs 91,900/36,800, conflicting"; EU SME scheme (§6.5) | PARTLY | Recall agrees (high) on EUR 10,000 and OSS. Franchise: 85,000/37,500 (tolerance 93,500/41,250) apply since 1 Jan 2025; **91,900/36,800 are the 2023–2024 figures**, so the "conflict" is old versus new. Recall (medium): the 2025 Finance Act's single EUR 25,000 threshold was suspended, then dropped. Whether the 2026 Finance Act changed anything must be checked. | [REPO] https://entreprisma.fr/micro-entrepreneur |
| 16 | La Poste 2026: EUR 1.52 / 2.25 (20 g), 3.10 / 4.85 (100 g); worked examples A–D (§6.6) | PARTLY | See the notes under this table. | [REPO] https://www.laposte.fr/tarifs-postaux-courrier-lettres-timbres |
| 17 | Withdrawal: Directive 2011/83 arts 9(2)(c), 16(m), 14(4)(b); L221-28 13°; withdrawal function "from 19 June 2026" (§7.1) | UNSUPPORTED; recall agrees (high) | The framing is outdated: on 2026-11-12 art. 11a has been applicable for five months. It is a current requirement, and the French transposition text is still unidentified. | none |
| 18 | Expiry: FR no statutory minimum; DE 3 years (OLG München 29 U 3193/07); NL 2 years; IE 5 years (§7.4) | UNSUPPORTED; recall agrees (medium-high) | No correction. | none |
| 19 | EAA: 28 June 2025; micro-enterprise exemption for services; loi 2023-171, décret 2023-931 (§10) | UNSUPPORTED; recall agrees (high) | No correction. | none |
| 20 | Card mentions and EPR: Triman/Info-tri, IDU, fine up to EUR 15,000; Regulation 2025/40 "general application recalled as 12 Aug 2026"; LUCID with no de minimis (§11) | PARTLY | Recall agrees on the French articles, amounts and VerpackG. Regulation 2025/40 has applied since 12 Aug 2026; treat it as current law, not outlook. The status of the Commission's Triman proceedings and of the German implementing act is unknown. | none |

Notes on claim 16:
- **Arithmetic: CONFIRMED [LOCAL].** I reproduced every total (13.66/1.35; 12.51/2.50; 14.23/0.78; B1 14.27/1.47; B2 14.30/1.44; B3 14.17/1.57; EUR 1.82 pass-through). Example C's 4.19/0.83 is the per-line sum; a single-line calculation gives 4.18/0.84.
- **20 g prices.** 1.52 and 2.25 are [REPO V2] and match my recall.
- **100 g prices are suspect.** La Poste's 21–100 g band has been exactly two stamps, which would give 3.04 and 4.50. Sibling doc 12 (line 297) itself flags 3.10 for re-check.
- **2027 tariff omitted.** Sibling docs 04 and 12 record EUR 1.67 from 1 Jan 2027 (international 2.45), seven weeks away.
- **Weight band.** A jacket of about 260×190 mm at 300 g/m² weighs about 15 g; with a C5 envelope (my estimate, about 7 g) it exceeds 20 g. Example A's EUR 1.52 is the optimistic band.
- **Own cost rule broken.** Example A charges EUR 5.02 TTC (4.19 excl. VAT) for card plus postage, against about EUR 6 single-copy landed cost quoted in §5.3 point 4. The example fails the document's "at or above cost" rule and implementation rule 23.

### Secondary claims
- **E-invoicing (§6.5):** UNSUPPORTED; recall agrees on the calendar. The duty to receive e-invoices has been live since 1 Sept 2026; it is not a future item.
- **PSD2 art. 3(k), 37(2); PSD3 (§8):** UNSUPPORTED; recall agrees. A provisional PSD3/PSR agreement dates from late Nov 2025; adoption status unknown.
- **Project Gutenberg (§3.5):** the 20 % royalty and the 1930 cut-off match recall (high). "LG Frankfurt 2018 led PG to block German users" is probably OUTDATED: recall (medium-high) that the case settled in 2021 and the blanket German block was lifted. Only the litigated titles stay blocked until their German terms expire.
- **T-553/23, UK adequacy, ODR closure on 20 July 2025, Z-Library blocking (TJ Paris 25 Aug 2022):** UNSUPPORTED; recall agrees.

### Internal errors [LOCAL]
1. **Code format is wrong** (Implications 46–47). The document says `XXXXX-XXXXX` and Crockford base32 look-alike mapping. docs/api.md v0.2 line 20 defines 12 symbols from a 20-consonant alphabet, shown `XXXX-XXXX-XXXX`, with no digits.
2. **"Field and state names follow docs/api.md" is wrong.** Nine names have zero occurrences in docs/api.md: `retail_copy_suspected`, `non_commercial_licence`, `pending_review`, `redemption_territory`, `vat_treatment`, `withdrawal_waiver`, `card_cost_floor`, `allow_format_conversion`, `redeemable_from`. `publisher_premium` is absent from api.md too, but is defined in doc 07 rule 29. api.md has states `exhausted` and `reissued`; the document omits the first and treats the second as a cancellation reason.
3. **"docs/api.md already encodes this" (§3.2) is PARTLY true.** api.md refuses only EPUBs with a non-font `encryption.xml` or `rights.xml`. It does not mention `license.lcpl`, `.acsm`, Kindle containers or encrypted PDFs.
4. **Paths.** Every `notes/…` reference should read `docs/research/notes/…`; there is no top-level `notes/` directory.

## 2. Omissions

All [RECALL] unless marked; none web-verified.
1. **General Product Safety Regulation (EU) 2023/988**, applicable since 13 Dec 2024. A printed card sold to consumers is a consumer product: manufacturer name with postal and electronic address, a traceability identifier, and the same information in the online offer. Missing from §11.1 and template rule 49. (high)
2. **Directive (EU) 2024/825** (consumers and the green transition), applicable from 27 Sept 2026. It bans generic environmental claims and offset-based "climate neutral" claims. §11.1 and lint rule 52 cite only AGEC art. 13. Sibling doc 04 lists its application date as unverified. (high)
3. **AGEC art. 112 mineral-oil ban** in printed matter for the public since 1 Jan 2025. [REPO, doc 04 line 253, itself from memory] It is absent from §11 and from the printer-contract checklist.
4. **Kobo title-level gifting** (late 2024). It changes §2.3, §3.6, §4.2 and the fake-Fnac demo premise. (medium)
5. **Kindle DRM-free EPUB/PDF downloads** from Jan 2026. (medium-high)
6. **Project Gutenberg / S. Fischer settlement** (2021). (medium-high)
7. **C-366/24**: opinion and probable judgment. (low on content)
8. **2025 franchise en base episode** (EUR 25,000 threshold enacted, suspended, dropped). It explains the conflicting figures. (medium)
9. **ViDA package** (adopted March 2025): staged changes to platform deemed-supplier rules, OSS scope and EU e-invoicing up to 2030. Relevant to §6.5. (medium-high)
10. **Loi n° 2014-779 of 8 July 2014**, the first ban on free shipping combined with the 5 % discount; the Darcos law builds on it. (high)
11. **La Poste 2027 tariffs**, already in sibling docs 04 and 12. [LOCAL]
12. **Danish book VAT change.** (medium)
13. **§5.6 fixed-price regimes** omit Portugal, Greece, Slovenia and Norway. The Flemish decree probably covers eBooks too. (medium)
14. **Contrary authority.** I know of no court decision allowing a private buyer to pass on an eBook file. The document's central conclusion is not contradicted by anything I know.

## 3. Statements to downgrade to "unverified"

- Every [V2]/[V1] marker: relabel "V-sib, not reopened". The document's own header admits this; the summary tables in §1 do not.
- Kobo/Fnac "no title-level gift" [V1], and the generalisations built on it: "every store…", "none lets a customer move a copy", "Decitre/Vivlio has presumably cleared it".
- La Poste 100 g rates 3.10 / 4.85 [V2].
- Franchise thresholds: delete the 91,900/36,800 pair and mark 85,000/37,500 "to confirm for 2026".
- DK 25 % and the whole "Others" VAT row.
- C-68/23 "are SPVs" → "can be".
- Loi Lang art. 1 quotation [M-hi] → incomplete quotation.
- "LG Frankfurt led PG to block German users".
- §2.6 "one-token transfer is not dissemination to the public; only basic hosting duties apply" [M] → (analysis), and it conflicts with the bearer design in docs/api.md.
- §1.2 point 4 "*Cyando* removes the hosting exemption".
- Withdrawal function, Regulation 2025/40 and e-invoicing reception: rewrite as obligations already applicable, with transposition details unverified.
- The cost figures "EUR 2–7" (§5.2) and "about EUR 6 landed" (§5.3).
- Implementation rule 27: unit tests pinned to unverified rates and tariffs should be marked illustrative.

## 4. Priority queries for a rerun with search budget

1. CJEU C-366/24 judgment.
2. Kobo eBook gifting countries; 7switch "offrir".
3. La Poste 2026/2027 tariff grid at 100 g.
4. Franchise en base thresholds under the 2026 Finance Act.
5. Danish book VAT.
6. French transposition of Directive 2023/2673.
7. Loi 2011-590 art. 4 and loi 81-766 art. 1 on Légifrance.
8. C-68/23 operative part.
9. GPSR and Directive 2024/825 as applied to printed cards.
10. Commission v France on the Triman.

File reviewed: /home/user/ebook-cadeau/docs/research/09-legal-gifting-and-commerce.md (not edited).
