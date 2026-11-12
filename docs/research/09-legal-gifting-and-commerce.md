# Legal: gifting an eBook, the upload-and-QR scheme, price law, VAT, consumer and data protection

_Status: fact-checked — research date 2026-11-12_

**Research to brief a lawyer and an accountant. Not legal or tax advice.**

## How to read this document

- **Scope.** The gifting mechanism itself (copyright, hosting liability), then the commerce layer (fixed book price, VAT, consumer law, payments, GDPR, accessibility, labelling). The cover printed on the card is in `07-legal-covers-and-reseller-rights.md`; fonts and generated spines are in `08-legal-fonts-and-genai.md`.
- **Inputs.** `docs/research/notes/legal-gifting-commerce.copyright-distribution.md` (copyright note) and `docs/research/notes/legal-gifting-commerce.commerce.md` (commerce note), plus facts taken from sibling notes and documents where stated, and the project contract `docs/api.md` (v0.2).
- **Verification status — important.** No external claim in this document has been verified on the web at any stage.
  - Neither input note could run a web search: the shared search budget was already spent and the legal hosts (EUR-Lex, Légifrance) are outside the sandbox allowlist.
  - For the synthesis five more searches were attempted on 2026-11-12 (wording of art. 4 of loi 2011-590; Tom Kabinet; the CJEU case on the French book-shipping floor; CJEU C-68/23; Cass. 23 Oct 2024 *Valve*). All five were refused.
  - An adversarial fact-check then reviewed the 20 most load-bearing claims. Its four searches were refused, and so were four arbitration searches by the editor (CJEU C-366/24, Kobo gifting, Danish book VAT, 2026 *franchise en base* thresholds). The fact-check could do two things: recompute the arithmetic and compare the text with the files of this repository, and set the text against the checker's own recollection. Its corrections are applied in place and summarised in "Verification log".
  - No URL was added that is not already in this repository.
- **Confidence markers.**
  - **[V-sib2]** found in two or more sources, **[V-sib1]** in one, by the search session of a *sibling* note (the two input notes tag them [S2]/[S1]). The URLs appeared in that agent's results. They were not reopened for this document or for the fact-check, so these markers mean "a sibling agent saw it", not "verified here".
  - **[M]** from memory, not verified. Where the commerce note graded its recall, the grade is kept: **[M-hi]** (text known well), **[M-med]**, **[M-lo]** (verify before relying on it).
  - **[FC]** added or corrected by the fact-check from the checker's recollection (knowledge to mid-2026), with its confidence in words. A lead for the next verification pass, not evidence.
  - **[LOCAL]** recomputed, or checked against files in this repository.
  - **(analysis)** our own reasoning; no authority found or known.
- **Citations.** Statutes and cases are cited by official number so that they can be looked up on Légifrance, EUR-Lex and Curia. They carry no URL because none appeared in a search result.
- **Currency.** The law is described as recalled up to mid-2026. Rules that became applicable during 2026 are written as current obligations, with their details unverified: the online withdrawal function (19 June 2026), the Packaging Regulation (12 August 2026), e-invoice reception in France (1 September 2026) and the green-claims rules of Directive (EU) 2024/825 (27 September 2026). Anything decided after mid-2026 (the CJEU ruling on the shipping floor, PSD3, the 2026 Finance Act, French transposition texts, the 2027 postal tariff grid) is unknown and listed in "Could not verify".

---

## 1. Executive summary

### 1.1 Is the current "file upload → QR code → download" scheme lawful?

It depends entirely on who uploads which file.

| Who uploads | What | Verdict | Main reason |
|---|---|---|---|
| Private buyer | eBook bought in a store, with DRM | **Not lawful** | Unauthorised reproductions (upload, download), breach of a personal licence, plus circumvention of a technical measure |
| Private buyer | eBook bought in a store, watermarked or DRM-free | **Not lawful** | Same, minus circumvention. No exhaustion for eBooks; private copy does not cover a copy made for someone else |
| Recipient of such a file | — | **Not lawful** | The download is a copy from an unlawful source |
| Author who holds the digital rights | Own work | Lawful | Subject to exclusivity clauses (publishing contract, KDP Select) |
| Publisher with digital rights for the territory | Own catalogue | Lawful | Subject to rights in translations, illustrations, embedded fonts |
| Anyone | Work in the public domain **in France**, or under CC0 / CC BY / CC BY-SA | Lawful with checks | Territorial public-domain test; attribution; no DRM added |
| Anyone | CC NonCommercial work | Free print-at-home only | A paid card or paid printing may be "commercial" |
| Anyone | US-only public domain (Project Gutenberg, Standard Ebooks texts) | Not for FR/EU recipients | Public domain is territorial |
| eBook reseller | Gift entitlement redeemed into the recipient's account | **Lawful — the target model** | First, authorised making-available; needs to fit the publisher/distributor contract |
| Anyone | Card carrying the store's own gift link or code | Lawful as to copyright | No file, no act on the work; store terms to be checked |

The first two rows are the use case the present scheme invites ("upload your eBook, get a QR"). On the analysis in §2 that use case must be removed from the product, not merely discouraged. The fact-check knows of no court decision that lets a private buyer pass on an eBook file, so nothing it recalls contradicts this conclusion [FC]. The conclusion is still unverified by search.

### 1.2 Why, in five sentences

1. Supplying an eBook by download is communication to the public, not distribution, so the right is never exhausted by the first sale (CJEU *Tom Kabinet*, C-263/18) [V2].
2. French private copy covers copies "strictement réservées à l'usage privé du copiste" made from a "source licite" [V2 for the text]; a copy made for a friend, and a download from a server copy nobody authorised, meet neither condition [M].
3. Store licences are personal and non-transferable [M]. Every gifting function identified by this research works by selling a new entitlement redeemed into the recipient's account [V2 for Amazon.fr, Decitre, Thalia/Hugendubel]. No store is known to let a customer move a copy already bought, but the survey is incomplete: Kobo, 7switch, Apple and Google in France are unverified.
4. A service whose stated purpose is to pass on purchased eBooks is not a neutral host. Under CJEU *YouTube/Cyando* an operator that provides tools specifically intended for illegal sharing, or knowingly promotes it, may itself be communicating to the public. The hosting exemption is a separate test: it is lost through an active role that gives knowledge of or control over the content, and DSA recital 20 denies it to a provider that deliberately collaborates with infringing users [M]. CPI art. L335-2-1 (3 years, EUR 300,000) targets software "manifestement destiné à la mise à disposition du public non autorisée"; applying it to a hosted, token-gated service is our extrapolation (analysis).
5. The service knows which title each file is, because it builds the card from the title and ISBN; the "no knowledge" defence is therefore weak for in-commerce titles (analysis).

Two facts weaken any "private, one-to-one" defence further:

- **Delivery is to the bearer, not to one named recipient.** `docs/api.md` v0.2 makes a hosted file re-downloadable by whoever holds the card, for up to 10 download sessions, with no expiry by default [LOCAL].
- **A DRM check does not recognise a store-bought copy.** Since 20 January 2026, as recalled, verified purchasers can download DRM-free Kindle titles as EPUB or PDF [FC, medium-high]. Retail files without any DRM container, and possibly without a visible mark, are in circulation.

### 1.3 The compliant target model

| Mode | Lane | What the card's QR / code resolves to | Is a book file on our servers? | Who licenses the recipient |
|---|---|---|---|---|
| Reseller-integrated | R1 `reseller_redirect` | The reseller's redemption page; the title is added to the recipient's account | No | The reseller, under its publisher/distributor contract |
| Reseller-integrated | R2 `reseller_licence` | A licence file (`.lcpl`, `.acsm`) issued by the reseller for the recipient | No (licence token only) | The reseller |
| Reseller-integrated | R3 `hosted_file` with basis `licensed_distributor` | A file the reseller is licensed to deliver, marked per gift by the reseller | Yes, as the reseller's processor | The reseller |
| Standalone | S1 gift link | The store's own gift link or code, pasted by the giver | No | The store |
| Standalone | S2 rights-holder file | A DRM-free file uploaded by its author or publisher | Yes | The uploader |
| Standalone | S3 public-domain / open-licence file | A file that is public domain in France or openly licensed | Yes | Nobody (public domain) or the open licence |

In standalone mode the service sells the **card service only** — never the eBook. In reseller mode the reseller is the seller of record for both the eBook and the card; eBook Cadeau invoices the reseller.

The hosted-file lanes (R3, S2, S3) are **bearer delivery** under `docs/api.md` v0.2: whoever holds the card can download, up to the session cap, and the recipient is not identified [LOCAL]. The uploader's rights (S2) or the reseller's licence (R3) must cover that. A reseller whose licence requires delivery to one identified account uses R1 or R2.

### 1.4 Commerce rules that shape the product

| Topic | Rule to build to | Conf. |
|---|---|---|
| Fixed price (FR) | The buyer pays the publisher's public price; gifting changes nothing. A **free printed and mailed card** attached to the sale looks like a *prime* (loi 2011-590 art. 4) or an indirect discount. A free print-at-home PDF is low risk | [M-med] |
| Card pricing | Printed card = optional, unticked, separately priced line, at or above its real landed cost. The gift option itself must stay available without it. The example prices of §6.6 pass this test only for a batch-printed or in-house card; they fail it at single-copy print-on-demand cost | [M-med] / (analysis) / [LOCAL] |
| VAT | eBook at the reduced rate (FR 5.5 %, DE 7 %); card and postage at the standard rate (FR 20 %, DE 19 %). A compulsory pack that forms a single operation with no ancillary element takes the highest rate on everything (CGI arts 257 ter, 278-0); a single bundled price is an indicator of that, not the test | [M-med] |
| Voucher | A code for one named eBook can be a single-purpose voucher (VAT at sale) only if both the place of supply and the VAT due are known at issue. Restricting redemption to one Member State fixes the place (CJEU C-68/23, as recalled); for one title at one rate the VAT is then known too (analysis). Default: France only | [M-med] / (analysis) |
| Withdrawal | The buyer keeps the 14-day right until the eBook is downloaded, unless the art. 16(m) waiver was collected. The system needs a "revoke unredeemed code" state. An online withdrawal function (Directive 2011/83 art. 11a) has been compulsory since 19 June 2026; the French transposition text is not identified | [M-hi] rule / [M-lo] third-party redemption |
| Expiry | No expiry, or at least 5 years, for a fully paid named title | [M-med] |
| Payments | A title-specific code is not e-money. eBook Cadeau must not collect eBook money for publishers or authors | [M-hi] |
| GDPR | The paper card needs no recipient e-mail and no account. Postal fulfilment adds the recipient's address, an Art. 14 notice and a printer DPA | [M-hi] |
| Accessibility | Redemption site to WCAG 2.1 AA; a typed code and short URL beside every QR | [M-hi] |
| Printed card (FR) | Triman/Info-tri, EPR registration, printer's name, company identifiers, French language. Added by the fact-check: manufacturer's name with postal and electronic address plus a batch identifier (General Product Safety Regulation); no generic or offset-based environmental claims (Directive 2024/825, since 27 Sept 2026); inks without mineral oils (AGEC art. 112). Envelopes to Germany: LUCID registration, no minimum | [M-med] / [FC] |

### 1.5 Risk ranking

| # | Risk | Severity | Likelihood if unaddressed | Fix |
|---|---|---|---|---|
| 1 | Standalone mode used (or described) as a way to pass on purchased eBooks | Criminal and civil exposure; loss of hosting exemption; ends any reseller or publisher relationship | High | Remove the use case; gift-link lane; upload gate (§3) |
| 2 | Reseller gift entitlement outside the publisher/distributor contract | Breach of contract, *contrefaçon* | Medium | Authorisation checklist (§4) |
| 3 | Free printed card, or a card priced below its real landed cost, treated as a *prime* or disguised discount | Fine per offence, action by competitors, trade bodies or the *médiateur du livre* | Medium | Paid optional line with a cost floor set from real quotes (§5, §6.6) |
| 4 | VAT: bundle taxed at 20 %; EU-wide codes; missed OSS registration | Tax reassessment | Medium | Separate lines; FR-only default; written VAT position (§6) |
| 5 | Refund after redemption (defective waiver), no way to revoke a code, or no online withdrawal function | Double loss per order; consumer-law sanction | Medium | Waiver + state machine + withdrawal function (§7) |
| 6 | Printed cards shipped without Triman, EPR identifier, manufacturer and batch identification, or LUCID registration; or carrying a banned environmental claim | Administrative fines, sales ban (DE) | Medium once volume exists | §11 |
| 7 | Building on figures that were never verified (VAT rates, thresholds, postal tariffs, store gifting functions) | Wrong prices, wrong tests, wrong demo premise | High until a verification pass runs | Dated, flagged configuration; "Could not verify" list |

---

## 2. Copyright analysis of gifting and of the upload scheme; hosting liability

### 2.1 The scheme, act by act

| # | Act | Who | Right engaged | Defence? | Conf. |
|---|---|---|---|---|---|
| 1 | Remove DRM to obtain a transferable file (if any) | Buyer | Protection of technical measures: InfoSoc art. 6; CPI L331-5, L335-3-1 | None for this purpose | [M] |
| 2 | Upload: a copy is fixed on the service's server | Buyer triggers; the service runs the equipment | Reproduction: InfoSoc art. 2; CPI L122-3, L122-4 | Private copy fails: not "usage privé du copiste"; the service is not a natural person (§2.5) | [M] |
| 3 | File stored and offered behind the QR / code | Service | Reproduction; making available (InfoSoc art. 3; CPI L122-2) if a "public" exists (§2.6) | DSA art. 6 hosting exemption — conditional, and an exemption from liability only | [M] |
| 4 | Recipient downloads: new copy on the device | Recipient | Reproduction | Private copy fails: source not lawful | [M] |
| 5 | Buyer deletes own copy ("forward and delete") | Buyer | — | Irrelevant: there is no exhaustion to trigger (§2.2) | [V2] for the principle |
| 6 | Breach of the store licence | Buyer | Contract; a licence breach touching IP rights can also be sued as *contrefaçon* (CJEU *IT Development v Free Mobile*, C-666/18, 18 Dec 2019; Cass. 1re civ., 5 Oct 2022, *Entr'ouvert v Orange*) | — | [M] |

*Contrefaçon* is both civil and criminal: CPI L335-2 / L335-3, up to 3 years and EUR 300,000 [M].

### 2.2 No digital exhaustion

**Tom Kabinet** — CJEU (Grand Chamber), 19 Dec 2019, C-263/18, *Nederlands Uitgeversverbond and Groep Algemene Uitgevers v Tom Kabinet Internet BV*, ECLI:EU:C:2019:1111.

- Holding: supplying an eBook to the public by download, for permanent use, is **communication to the public** ("making available") under art. 3(1) of Directive 2001/29, not distribution under art. 4(1). The exhaustion rule of art. 4(2) does not apply — [SCL](https://www.scl.org/10768-cjeu-sale-of-second-hand-e-books-via-a-website-is-a-communication-to-the-public/), [Charles Russell Speechlys](https://www.charlesrussellspeechlys.com/en/insights/expert-insights/tmt/2020/can-you-resell-an-e-book-online/) [V2].
- Reasoning [M]: (i) the WIPO Copyright Treaty confines "distribution" to tangible copies; (ii) recitals 28–29 of the Directive: exhaustion "does not arise in the case of services and on-line services in particular", and art. 3(3): the communication right is never exhausted; (iii) a digital copy does not deteriorate, so a "second-hand" copy is a perfect substitute for a new one; (iv) **new public**: the licence that accompanies an eBook authorises reading only by the user who downloaded it, on that user's equipment, so anyone else is a public the right-holder did not take into account (paras 69–72 as recalled; the paragraph numbers are unchecked).
- The defendant's "reading club" and its request that sellers delete their copy changed nothing [M].
- No later CJEU decision reopening digital exhaustion for eBooks is known to the fact-check [FC, high].

**UsedSoft v Oracle** — CJEU (Grand Chamber), 3 July 2012, C-128/11 [M]. Exhaustion applies to a downloaded copy of a **computer program** sold with a perpetual licence, provided the seller makes its own copy unusable. *Tom Kabinet* distinguishes it: Directive 2009/24 is *lex specialis* for software; an eBook is not a computer program.

Corroboration [M, all rows]:

| Jurisdiction | Decision | Point |
|---|---|---|
| France | Cass. 1re civ., 23 Oct 2024, n° 23-13.738, *UFC-Que Choisir v Valve* (confirming CA Paris, 21 Oct 2022; TGI Paris, 17 Sept 2019 had held the opposite) | No exhaustion for dematerialised video games; the non-resale clause was not struck down. France's highest court follows the *Tom Kabinet* line for non-software content |
| France | CSPLA report on "la seconde vie des biens culturels numériques" (2015) | Official position against a second-hand digital market |
| Germany | OLG Hamm, 15 May 2014, 22 U 60/13 (audiobooks); OLG Hamburg, 10 U 5/11 (eBooks) | Store terms forbidding resale or transfer of downloads upheld |
| USA | *Capitol Records v ReDigi*, 2d Cir. 2018 | Transfer of a digital file is a reproduction; first sale is no defence |
| EU (lending) | *VOB v Stichting Leenrecht*, C-174/15, 10 Nov 2016 | "One copy, one user" e-lending is open only to public libraries under the lending derogation, from a lawful source |

Consequence: giving, lending, reselling and swapping a purchased eBook file are legally the same act; each needs the right-holder's authorisation. A gift can only be *built* by a party entitled to grant a licence.

### 2.3 What store licences say about transfer and gifting

Current terms could not be opened. The wording is as remembered; each must be re-read before it is relied on.

| Store | Licence wording on transfer | Own gifting function | Conf. |
|---|---|---|---|
| Amazon Kindle | Content is "licensed, not sold"; the user may not "sell, rent, lease, distribute, broadcast, sublicense, or otherwise assign any rights" nor circumvent DRM. Since 2025 the US buy button speaks of purchasing a licence. "Download & transfer via USB" was withdrawn in Feb 2025. Since 20 Jan 2026, as recalled, verified purchasers can download DRM-free Kindle titles as EPUB or PDF [FC, medium-high] | Yes: "Acheter pour d'autres personnes" — the buyer receives prepaid redemption links or enters the recipient's e-mail; the recipient must be in the same country; unredeemed copies can be reassigned or refunded — [Amazon.fr help](https://www.amazon.fr/gp/help/customer/display.html?nodeId=GVWGP284MQ6ZRM59), [KDP help](https://kdp.amazon.com/fr_FR/help/topic/G200652260), [IDBoox](https://www.idboox.com/news-livres/amazon-on-peut-enfin-offrir-un-livre-numerique-kindle/) | Terms [M]; gifting [V2] |
| Kobo / Kobo by Fnac | Limited, non-exclusive, non-transferable, revocable licence for personal non-commercial use | **Unverified.** The sibling search found no title-level gift in France; users fall back on gift cards — [Dealabs](https://www.dealabs.com/discussions/carte-cadeau-kobo-ou-fnac-2273026), [ebook-enligne.fr](https://ebook-enligne.fr/blog/offrir-ebook-guide-cadeau-numerique/). That is a weak negative: a forum thread and a low-authority blog. The fact-check recalls that Rakuten Kobo launched title-level eBook and audiobook gifting on kobo.com in late 2024 in several English-speaking markets [FC, medium]; France and Fnac are unknown | Terms [M]; gifting [V1] weak negative, contradicted in part by [FC] |
| Decitre / Furet du Nord (Vivlio) | Not retrieved | Yes: "Offrir un ebook", gift e-mail on a chosen date — [Decitre](https://www.decitre.fr/offrir-un-ebook), [Liseuses.net](https://www.liseuses.net/offrir-des-ebooks-avec-tea/) | [V2] |
| Thalia / Hugendubel (tolino) | "Einfaches, nicht übertragbares Nutzungsrecht" is the usual German wording | Yes: "Verschenken" — [Hugendubel](https://www.hugendubel.de/de/shortcut/ebooksverschenken), [heise](https://www.heise.de/tipps-tricks/E-Books-verschenken-so-klappt-s-7476189.html) | Terms [M]; gifting [V2] |
| Apple Books, Google Play Books | Personal, non-commercial use rules | E-mail gifting of a specific book in some countries, redeemable only in the country of purchase; availability in France not known | [M] |
| 7switch (Immatériel) | Not retrieved. Sells many titles with watermark or without protection, shown per title | Not verified (unconfirmed recollection of an "offrir" option) | [M] |

Pattern: every gifting function identified by this research works by **selling a fresh entitlement redeemed into the recipient's account**. None is known to let a customer move a copy already in their library. The survey covers three confirmed stores and is not exhaustive, so "every store" would overstate it. Sanctioned sharing is known only as household features (Amazon Family Library, Apple Family Sharing) [M].

Consequence of the January 2026 Kindle change [FC]: a retail copy can now be a plain EPUB or PDF with no Adobe or LCP container. The upload gate of §3.3 cannot rely on DRM detection to recognise store purchases; the ISBN and publisher check carries the weight.

### 2.4 DRM and watermark

Legal protection of technical measures [M — to be re-read on Légifrance; the 2021 ARCOM reform renumbered part of this chapter]:

| Rule | Content |
|---|---|
| InfoSoc art. 6 | Member States must protect effective technological measures against circumvention and against trafficking in circumvention tools. Art. 6(4), fourth subparagraph: the duty to let users benefit from exceptions does **not** apply to works made available on demand on agreed contractual terms — i.e. to eBook stores |
| CPI L331-5 | Effective technical measures are protected; they must not block interoperability |
| CPI L335-3-1 | I: knowingly circumventing a technical measure, otherwise than for research: fine of **EUR 3,750**. II: procuring or offering means designed or specially adapted to circumvent: **6 months and EUR 30,000** |
| CPI R335-3 (décret 2006-1763) | Holding or using a circumvention tool for personal use: 4th-class *contravention* (max EUR 750) |
| InfoSoc art. 7; CPI L331-11, L335-3-2 | Removing or altering electronic rights-management information, or distributing works from which it was removed: EUR 3,750; supplying means: 6 months and EUR 30,000 |

Implications:

- A DRM-protected file (Adobe ADEPT, Kindle, Kobo, Apple FairPlay, Readium LCP) is useless to a recipient unless the DRM is removed or account credentials are shared. The service must never host, link to or describe DRM-removal tools: that is the L335-3-1 II offence and "providing tools specifically intended for illegal sharing" in the *Cyando* sense (§2.6) [M].
- An Adobe `.acsm` or an LCP `.lcpl` is a licence token bound at fulfilment. **Issued by the reseller for the recipient**, it is a lawful delivery vehicle (`reseller_licence` in `docs/api.md`). Passed on by a buyer without the store's consent, it involves no circumvention but still breaches the personal licence [M].
- A **watermark (*tatouage numérique*)** is not an access control, so copying a watermarked file is not circumvention. But (i) the file carries the original buyer's identifiers; if the recipient's copy leaks, the trail ends at the **buyer**; (ii) stripping or altering the mark can fall under L335-3-2 where it carries licence or rights information — whether a purely buyer-identifying mark is "rights-management information" is debated, no authority known; (iii) the file contains the buyer's personal data (GDPR) [M].

### 2.5 Private copy and the "cercle de famille"

- **Two different exceptions.** CPI L122-5 **1°** covers "représentations privées et gratuites effectuées exclusivement dans un **cercle de famille**" — performances, not copies. L122-5 **2°** covers copies "réalisées à partir d'une **source licite** et strictement réservées à l'**usage privé du copiste** et non destinées à une utilisation collective" — text per [Baumann](https://www.baumann-avocats.com/CPI/article-l122-5-du-code-de-la-propriete-intellectuelle.php), [Doctrine](https://www.doctrine.fr/l/texts/codes/LEGITEXT000006069414/sections/LEGISCTA000006179045) [V2]. The "cercle de famille" is not the test for copies.
- **The family circle is narrow** in any case: relatives or very close friends "unis de façon habituelle par des liens familiaux ou d'intimité" (T. corr. Paris, 24 Jan 1984, commonly quoted) [M].
- **A copy made for someone else is not "l'usage privé du copiste".** In the *Aurélien D.* litigation (CA Montpellier, 10 Mar 2005; Cass. crim., 30 May 2006; on remand CA Aix-en-Provence, 5 Sept 2007) the final conviction relied, among other things, on copies lent to friends exceeding private use [M]. No decision is known that treats a copy handed to a friend as covered.
- **Lawful source.** Added to L122-5 2° by loi n° 2011-1898 of 20 Dec 2011 [M]; at EU level *ACI Adam* (C-435/12, 10 Apr 2014) excludes copies from unlawful sources — [Next](https://next.ink/news/86942-la-justice-europeenne-impose-decouplage-copie-privee-et-source-illicite.htm) [V1]. The recipient's download is made from a server copy that nobody authorised.
- **Who is the copyist?** Under the *Rannou-Graphie* line the party operating the copying equipment for customers is the copyist — [thesis summary](https://www.memoireonline.com/12/06/297/m_l-exception-de-copie-privee-face-aux-dispositifs-techniques-de-protection-des-oeuvres10.html) [V1]; the online recorder Wizzgo could not shelter behind its users — [Village de la justice](https://www.village-justice.com/articles/affaire-wizzgo-prend-tournant,4999.html) [V1]. CJEU *VCAST* (C-265/16): a service may assist private copying only if it does not itself give access to the work — [Kluwer Copyright Blog](https://legalblogs.wolterskluwer.com/copyright-blog/up-in-the-cloud-some-reflections-on-the-cjeu-judgment-in-vcast/), [IPKat](https://ipkitten.blogspot.com/2017/11/the-vcast-decision-how-to-turn-private.html) [V2].
- **Not a right.** *Mulholland Drive* (Cass. 1re civ., 28 Feb 2006 and 19 June 2008): private copy is an exception that cannot be asserted against DRM, and it is subject to the three-step test [M]. Giving away a substitute for a sale conflicts with normal exploitation (analysis).
- Germany is more generous on paper (§ 53(1) UrhG allows copies for the close personal circle), but § 53(4)(b) requires consent for an essentially complete copy of a book, and contract plus § 95a (DRM) apply; whether § 53(4)(b) covers eBooks is not settled as far as known [M].

Answer to "does giving a copy to a friend qualify?": **no in France** (wrong beneficiary, unlawful source for the recipient, three-step test, contract), and not safely anywhere in the EU.

Relation to document 07: that document accepts a `user_private_copy` basis for **one home-printed jacket bearing the cover** of a book the user bought. That is a copy for the copyist's own use of a different work (the cover image). It gives no support for passing the eBook file to a third person.

### 2.6 Status of the service: host, communicator, or accomplice

**Does the service itself communicate to the public?** *YouTube / Cyando* — CJEU (Grand Chamber), 22 June 2021, joined cases C-682/18 and C-683/18 [M]. Cyando ran the file-locker "Uploaded". The operator of a sharing or file-hosting platform does **not** itself communicate to the public unless it contributes, beyond merely providing the platform, to giving access in breach of copyright. Indicators:

1. specific knowledge of an infringing file and no expeditious removal;
2. knowing "in a general sense" that users infringe, and not deploying the technological measures expected of a reasonably diligent operator;
3. taking part in selecting content, **providing tools specifically intended for illegal sharing**, or **knowingly promoting such sharing** — which a financial model encouraging it may show.

These indicators answer one question only: whether the operator itself performs an act of communication to the public. The hosting exemption was a second, separate question in the same judgment. The Court answered it with the older test: the exemption is lost when the operator plays an active role that gives it knowledge of or control over the content, or when it knows of a specific infringement and does not act [M; the fact-check agrees on the date, the case numbers and the three indicators]. An earlier draft of this document said that *Cyando* "removes the hosting exemption"; that merged the two questions and is corrected here.

Applied to a service marketed as "give an eBook you bought": indicator 3 is met by the product description itself and indicator 2 by the absence of a rights check (analysis). The earlier line points the same way: *Ziggo / The Pirate Bay* (C-610/15), *Filmspeler* (C-527/15), *GS Media* (C-160/15) [M]; reposting a freely accessible work is a new communication (*Renckhoff*, C-161/17 — [IRIS Merlin](https://merlin.obs.coe.int/article/8373) [V2]).

**The "public" argument.** One buyer sending to one named friend is not an indeterminate, fairly large group; this is the one real argument for a one-recipient design. It is weak for the *service*: *Tom Kabinet* counts persons who access the same work successively, the service is open to anyone, and the reproduction right (server copy, recipient copy) has no "public" threshold at all. Untested; not to be built on (analysis).

The product as specified does not even have a one-recipient design [LOCAL]. `docs/api.md` v0.2 (claim semantics) makes a hosted-file gift "re-downloadable by whoever holds the card" until a limit of 10 download sessions, with no expiry by default; holding the card is the credential and no account is needed. That is bearer delivery to an unidentified holder. Any argument in this document that rests on "one token, one recipient" must be read with that in mind, and it is marked (analysis) below.

**Hosting exemption (DSA) and LCEN.**

| Rule | Content | Conf. |
|---|---|---|
| DSA (Regulation (EU) 2022/2065) art. 6 | No liability for stored content without actual knowledge, or if removed expeditiously once aware; lost where the provider plays an active role giving knowledge or control; injunctions remain possible — [Freshfields](https://www.freshfields.com/en/our-thinking/blogs/technology-quotient/the-digital-services-act-new-liability-rules-102iiyf), [Taylor Wessing](https://www.taylorwessing.com/fr/interface/2022/the-eus-digital-services-act/online-intermediaries-and-illegal-content-under-the-digital-services-act) | [V2] |
| DSA recital 20 | No exemption where the provider "deliberately collaborates" with users, e.g. offers the service "with the main purpose of facilitating illegal activities", including by making that purpose explicit | [M] |
| DSA arts 7, 8 | Voluntary own-initiative checks do not forfeit the exemption; no general monitoring obligation | [M] |
| DSA art. 16 | Every host needs an electronic notice mechanism; a precise, substantiated notice gives actual knowledge; decisions must be timely, diligent and objective — [Burges Salmon](https://burges-salmon.com/news-and-insight/legal-updates/data-protection/the-eu-digital-services-act-and-the-article-6-hosting-exemption), [art. 16 text](https://www.springlex.eu/fr/packages/dsa/dsa-regulation/article-16/) | [V2] |
| DSA arts 11–15, 17 | Contact points, terms describing moderation, statement of reasons to the affected user; the transparency report is not required of micro and small enterprises | [M] |
| DSA art. 3(i), 3(k), recital 14; art. 19 | An "online platform" is a host that **disseminates to the public** (a potentially unlimited number of third parties). Micro and small enterprises are exempt from the platform section | [M] |
| — applied here | A token-gated file with no catalogue is arguably not dissemination to the public, so only the basic hosting duties would apply. This is our reading, not an authority, and the bearer design of `docs/api.md` (any card holder, up to 10 sessions) makes it less clear-cut than a strict one-recipient transfer | (analysis) |
| DSM Directive 2019/790 art. 2(6), art. 17 (CPI L137-1 ff.) | Direct liability of services whose main purpose is to give **the public** access to large amounts of user-uploaded works, organised and promoted for profit | [M] |
| — applied here | With no public access to the stored files, the service is arguably not such a service | (analysis) |
| LCEN (loi 2004-575) art. 6 | Aligned on the DSA by the 2024 SREN law; duty to keep data identifying content providers (décret 2021-1362; retention periods to be confirmed); "manifestement illicite" standard from Cons. const. 2004-496 DC | [M] |

File-locker case law [M]: in France, *Nintendo v Dstorage (1fichier)* — CA Paris, 12 Apr 2023, upheld by the Cour de cassation, recalled as the *chambre commerciale* on 26 Feb 2025 [FC, medium-high]: a host that leaves notified content online loses the exemption; a proper notice suffices, no court order is needed. In Germany, BGH *Alone in the Dark* (2012) and *File-Hosting-Dienst* (2013) (heightened duties when the business model invites infringement), then the *uploaded* judgments of 2 June 2022 applying *Cyando*.

**Why the exemption is fragile here (analysis).** The service builds a card from the title, author and ISBN, so it has specific knowledge of *which work* each stored file is. For an in-commerce title uploaded by someone who is not the publisher, it "ought to know" what is going on. That is an active role with knowledge, not passive storage. Document 07 reaches the same conclusion for covers: hosting status can cover a file stored at a user's request, not what the service selects, renders or prints under its own brand.

**French criminal add-ons** [M for the texts; the fact-check agrees on the amounts]:

- CPI **L335-2-1**: 3 years and EUR 300,000 for knowingly publishing or making available a "logiciel manifestement destiné à la mise à disposition **du public** non autorisée d'œuvres", or inciting its use, advertising included.
  - The text is about software for making works available *to the public*. Applying it to a hosted, token-gated transfer service is this document's extrapolation (analysis); whether a one-to-one or bearer transfer reaches a "public" is question 2 for counsel.
  - The cautious reading is kept for the open-source repository: purpose statements, README and marketing copy would be the evidence of what the software is "manifestly intended" for.
- Complicity by supplying means (Code pénal art. 121-7); civil measures against such software (CPI L336-1); injunctions against any intermediary, exempt or not (CPI L336-2).
- Moving the transfer into the browser (WASM / WebRTC, no server copy) does not cure the scheme: the recipient's copy is still unauthorised, and these provisions target the tool and its publisher, not only a host (analysis).

### 2.7 Enforcement context

- HADOPI's graduated response targeted subscribers on **peer-to-peer** networks; HADOPI merged into **ARCOM** on 1 Jan 2022 (loi n° 2021-1382). The graduated response does not reach direct download or a private transfer service; an individual gifter is not exposed through it [M].
- ARCOM regulates technical protection measures, lists infringing services, handles mirror-site blocking, and is France's Digital Services Coordinator under the DSA (loi n° 2024-449) [M].
- The book trade enforces collectively: the SNE obtained ISP blocking of some 200 Z-Library domains (TJ Paris, 25 Aug 2022) under CPI L336-2 [M]. A service perceived as an eBook-passing tool should expect the same route.
- Realistic exposure (analysis): for the individual, a low probability but traceability through the watermark; for the service, notices, injunctions and — decisive commercially — refusal by publishers, e-distributors and resellers to work with it. **The reputation of the standalone mode conditions the reseller-integrated mode.**

---

## 3. Lawful configurations and required safeguards for the standalone mode

### 3.1 The four lawful configurations

| # | Configuration | Lane (see §1.3) | Copyright act by eBook Cadeau on the book |
|---|---|---|---|
| 1 | The reseller sells a gift entitlement and licenses the recipient on redemption | R1, R2, R3 | None in R1/R2; hosting as processor in R3 |
| 2 | Rights-holders distribute their own files | S2 | Hosting and transmission under the uploader's licence |
| 3 | Public-domain (in France) or openly licensed works | S3 | Hosting; none restricted if the work is public domain |
| 4 | The card carries a store code or gift link, not a file | S1 | None |

Configuration 1 is §4. The rest of this section covers standalone mode.

### 3.2 Upload policy matrix

| What the user has | File hosting | Offer instead |
|---|---|---|
| eBook bought in a store, with DRM | **Refuse** (detect and block) | Gift-link card (S1) |
| eBook bought in a store, watermarked or DRM-free | **Refuse** | Gift-link card (S1) |
| Own work (author, publisher with digital rights) | Accept with declaration | — |
| Public domain in France / CC0 / CC BY / CC BY-SA | Accept with licence selection | — |
| CC NonCommercial | Free print-at-home only, or manual review | — |
| US-only public domain | Refuse for FR/EU recipients | Link to the source site |

`docs/api.md` v0.2 encodes this only in part [LOCAL]:

- **In the contract:** the mandatory header `X-Rights-Basis: own_work | public_domain | open_licence | licensed_distributor`, with deliberately no value for a purchased copy; a limit to EPUB and PDF; and the refusal `drm_protected` for an EPUB carrying `META-INF/encryption.xml` with a non-font-obfuscation algorithm, or `META-INF/rights.xml`.
- **Not in the contract:** refusal of `license.lcpl` and of encrypted PDFs; detection of retailer marks; the ISBN check; the review state; the NonCommercial restriction. These are this document's proposals (see "Implications for the implementation") and have to be added to `docs/api.md`.

### 3.3 Required safeguards (recommendations of the copyright note; not sourced)

1. **Purpose and wording.** The service is never described (UI, README, store listing, SEO) as a way to pass on, share, lend or resell purchased eBooks. The upload dialog says the opposite: store purchases are personal licences; use the store's gift function.
2. **Declaration** before each upload: a rights basis from a closed list, identity and verified e-mail of the uploader, warranty and indemnity, and a licence to the service limited to hosting and delivery to the designated recipient. Stored with timestamp and file hash.
3. **Automated licence check** (protected by DSA art. 7 [M]):
   - reject DRM containers (`META-INF/encryption.xml` beyond font obfuscation, `rights.xml`, `license.lcpl`, Kindle/KFX DRM, `.acsm`);
   - flag retailer watermarks, ex-libris pages and retailer identifiers in metadata;
   - read `dc:rights` and the ISBN; an ISBN belonging to an in-commerce title whose publisher is not the verified uploader blocks the upload or sends it to review;
   - for "public domain", require the author's year of death; apply life + 70 and the French extensions; check translator and edition.
   - Absence of DRM clears nothing. Watermark-only stores, and since January 2026 the EPUB/PDF download of DRM-free Kindle titles [FC, medium-high], produce retail files with no DRM container and sometimes no visible mark. The ISBN and publisher check and the declaration are the real gate; DRM and watermark detection only catch the obvious cases.
4. **DRM-free only; unchanged delivery.** No conversion that would drop rights information; no DRM added to Creative Commons works; no DRM-removal help anywhere.
5. **Bearer token, no public access.** No catalogue, no search, no public profile, no "share again" button, `noindex`; download URLs carry a 128-bit random token; the short typed code is protected by rate limits (`docs/api.md`, `docs/research/notes/ereader-ux.ux.md`). This is not a one-recipient design: under `docs/api.md` whoever holds the card can download, up to the session cap [LOCAL]. It must not be described as one, and the uploader's declaration has to cover delivery to the bearer.
6. **Notice-and-action** (DSA art. 16): public form and e-mail; acknowledgement; fast decision (target 24 h); token disabled and file deleted; statement of reasons to the uploader with redress; log. The file hash and ISBN go on a blocklist so the same work does not reappear — the measure *Cyando* expects.
7. **Repeat-infringer policy**: suspension after two upheld notices or one false declaration, ban on recurrence. This is a legal duty only for online platforms (DSA art. 23) and a condition of the US DMCA safe harbour (§ 512(i)) [M], but it is the evidence of diligence a court looks for.
8. **Business model**: charge for the card and printing, never per download; no reward to uploaders (the *Cyando* "financial model" indicator).
9. **Records and contacts**: single contact point, terms describing these rules, identification data kept as LCEN requires, EU hosting, privacy notice covering uploader, recipient and any buyer data inside files.
10. **Repository hygiene** (CPI L335-2-1): demo fixtures and tests use public-domain or own-made files; the fake stores demonstrate the *reseller entitlement* flow, not customer-to-customer transfer.

### 3.4 Rights-holders distributing their own files (lane S2)

- Self-published authors: lawful, unless bound by exclusivity (KDP Select forbids distributing the digital book elsewhere during enrolment [M]) or by a publishing contract — an author who has assigned digital rights (CPI L132-17-1 ff.) cannot hand out files without the publisher [M].
- Publishers: need digital rights for the territory, plus rights in translations, illustrations and embedded fonts [M].
- The uploader grants the service a narrow hosting licence (reproduce and transmit to the designated recipient).
- If the author or publisher *sells* the code, the fixed-price law and the reduced VAT rate apply to that sale (§5, §6). The sale is theirs, not eBook Cadeau's.
- Existing practice: BookFunnel print codes — [BookFunnel](https://authors.bookfunnel.com/help/print-codes/) [V2]; Bookwire download codes — [Bookwire](https://bookwire.net/faq-download-codes) [V1].

### 3.5 Public domain and open licences (lane S3)

All conditions below are [M].

| Source / licence | Usable? | Conditions and traps |
|---|---|---|
| Public domain **in France** | Yes | Life + 70 (CPI L123-1); +30 years for authors "morts pour la France" (L123-10); translators, annotators and illustrators have their own terms; moral rights are perpetual (name, integrity); 25-year right in posthumous publications (L123-4). The cover is a separate work |
| Project Gutenberg | With care | Texts are public domain **in the USA** (works published through 1930, as of 2026), not necessarily in the EU. LG Frankfurt, 9 Feb 2018 (*S. Fischer v PGLAF*) led PG to block all German users at the time; the fact-check recalls that the case settled in 2021 and that the blanket block was lifted, only the litigated titles staying blocked in Germany until their German terms expire [FC, medium-high]. Either way the case shows that a US-public-domain file can infringe in an EU country. "Project Gutenberg" is a registered trade mark: either remove every PG reference from the file, or follow the PG licence — full licence attached and, if any fee is charged, a **royalty of 20 % of gross profits** (licence § 1.E.8) |
| Standard Ebooks | Yes for the edition | Its own work (markup, typography, cover) is dedicated under **CC0 1.0**; the underlying texts are US public domain only — same territorial check |
| Wikisource, Gallica, ebooksgratuits.com, BEQ | Case by case | Site terms may restrict commercial reuse (BnF for Gallica scans; ebooksgratuits non-commercial); BEQ follows Canadian terms |
| CC0, CC BY, CC BY-SA | Yes | Attribution (title, author, source, licence) on the card or the redemption page; **no DRM or other technical restriction** may be added |
| CC BY-NC(-SA/-ND) | Free print-at-home only | A paid card or paid professional printing risks being "commercial"; German courts have read NC both narrowly and broadly (LG / OLG Köln 2014). Manual review or refusal of the paid tier |
| Internet Archive "lending" copies | No | Controlled digital lending held unlawful in *Hachette v Internet Archive* (2d Cir. 2024); not a lawful source |

### 3.6 Codes or links for a store, not files (lane S1)

- **Gift link.** The giver buys the title through the store's own gift function and pastes the redemption link or code; the card's QR encodes it. No file, no copyright act. The link is a bearer secret and is handled client-side.
- **Where it works, as recorded by sibling notes (not re-checked on 2026-11-12):** Amazon.fr [V2], Decitre / Furet du Nord (Vivlio) [V2], Thalia / Hugendubel in Germany and Austria [V2].
- **Unverified:** Kobo / Fnac. The sibling search found no title-level gift in France [V1, weak negative]; the fact-check recalls a Kobo gifting launch in late 2024 in English-speaking markets [FC, medium]. 7switch, Cultura, Apple Books and Google Play Books in France: unknown.
- **Store credit + recommendation.** Where no title-level gift exists, the card can carry a store gift-card code bought by the giver and name the suggested title. Vivlio gift cards do not cover eBooks — [Vivlio help](https://help.vivlio.com/hc/fr/articles/11334184838428-Je-souhaite-utiliser-un-code-promotionnel-ou-une-carte-cadeau) [V2].
- **Limits.** eBook Cadeau must not itself sell stored value redeemable at third parties (§8). A 100 % coupon funded at the full public price respects the fixed-price law; a discount coupon does not [M].
- **Open points.** Whether a store's terms allow its gift link to be re-wrapped in a third-party card, and whether Amazon.fr exposes a copyable link as Amazon.com does, are not verified. Store names should appear on the card as plain text, without logos (analysis; trade-mark use is outside this document).
- **Link hygiene (analysis).** A generator that turns any URL into a branded "gift" QR is a phishing and piracy-link vector, and a for-profit operator posting links is presumed to know when they lead to infringing content (*GS Media*, C-160/15 [M]). The lane therefore accepts only an allow-list of store redemption hosts.

### 3.7 Contradictions between the inputs, and how they are resolved

| # | Input A | Input B | Kept, and why |
|---|---|---|---|
| 1 | Copyright note §9: token with an expiry; file deleted after redemption | Commerce note §3.4: no expiry, or ≥ 5 years; `docs/api.md`: no default expiry | **No default expiry.** A paid card whose file disappears is a consumer-law problem (§7.4). The copyright note's aim — limiting exposure — is met by restricting *who may upload*, not by short retention. Files are deleted on cancellation, takedown, expiry (if set, ≥ 5 years) or when the download cap is reached |
| 2 | Copyright note: single redemption with a small retry allowance | `docs/api.md` and the UX note: re-downloadable by the card holder, 10 download sessions | **Up to 10 sessions for `hosted_file`** (devices fail; the conformity duty favours re-download), counter visible on the gift page, cap lowerable by the uploader. `reseller_redirect` stays strictly single-recipient. Cost of this choice: hosted-file delivery is bearer delivery, so the "one recipient" argument of §2.6 is not available, and the lane is defensible only because the gate admits files the uploader may distribute |
| 3 | Copyright note: deliver files byte-identical, never convert | UX notes: per-device formats (KEPUB for Kobo, AZW3 for Kindle) | **Byte-identical by default.** Conversion only when the rights-holder uploader opts in, and only if rights statements, licence text and attribution survive unchanged. Creative Commons 4.0 licences allow technical format changes [M]; a retailer file never reaches this path |
| 4 | Commerce note §5: DSA art. 6 shield "if passive" | Copyright note §7: shield fragile because the service knows the title | **The copyright note.** The exemption is a backstop for lanes S2/S3 after the upload gate, not a basis for accepting store-bought files |
| 5 | Copyright note §10: a code for one eBook *is* a single-purpose voucher | Commerce note §2.3: only if redemption is confined to one Member State | **The commerce note** — it applies the statutory test (place of supply and VAT known at issue). CJEU C-68/23, as recalled, supports the first half of that test only (§6.3) |
| 6 | Copyright note: identity and verified e-mail of every uploader | Commerce note: print-at-home can process no personal data server-side | Both, for different lanes: S1 (gift link) needs no server-side personal data; S2/S3 (hosted file) require the uploader's identity |
| 7 | Document 07 rule 29: a card line priced at zero is rejected | Commerce note: a free print-at-home PDF is low risk | No conflict once stated precisely: the print-at-home PDF is not a line item; rule 29 applies to printed and mailed cards |

---

## 4. Reseller gift-licence model and publisher authorisation

### 4.1 Structure

- The buyer pays the reseller. The reseller issues a single-use entitlement (code / link). On redemption the content is licensed to the **recipient's** account, with DRM or watermark personalised to the recipient. The buyer never holds a copy. This is the first, authorised making-available; no exhaustion question arises (analysis).
- Contract reading (analysis): a sale with a *stipulation pour autrui* (Code civil arts 1205 ff. [M]). The buyer obtains the reseller's promise to license a beneficiary; the promise is revocable until the beneficiary accepts — which matches Amazon's "reassign or refund if unredeemed" [V2 for the practice]. The recipient accepts the store terms at redemption.
- Every existing gift flow follows the pattern "opaque token → sign in or create an account → entitlement added"; none delivers a file straight from the token (`notes/ereader-ux.ecosystems.md`, [M]). Keeping to it makes the watermark and the licence personal to the recipient.

### 4.2 Where the reseller's authority comes from

| Point | Finding | Conf. |
|---|---|---|
| The publisher fixes the public price; it binds everyone selling to buyers in France | [Livres Hebdo](https://www.livreshebdo.fr/article/la-loi-sur-le-prix-en-vigueur), text on [WIPO Lex](https://www.wipo.int/wipolex/en/legislation/details/13861) | [V2] |
| Before the 2011 law, publishers controlled price through **contrats de mandat** (agency) with e-retailers; mandate retailers stayed under those contracts | [Livres Hebdo](https://www.livreshebdo.fr/article/prix-du-livre-numerique-une-loi-de-securite), [Livres Hebdo](https://www.livreshebdo.fr/article/loi-sur-le-prix-du-livre-numerique-une-victoire-la-pyrrhus) (via `notes/legal-covers-reseller.reseller.md`) | [V1] |
| Under a *mandat* the retailer sells in the publisher's name and for its account; "le mandataire ne peut rien faire au-delà de ce qui est porté dans son mandat" (Code civil art. 1989). An agent has less room for own-initiative formats than a buyer-reseller | | [M] |
| Reseller-facing terms of the French e-distributors (Eden Livres, Immatériel, Numilog, Interforum, Hachette) | Not public; not obtained by any note | — |
| Fnac's eBook catalogue is operated with Kobo; the publisher licences may sit with Rakuten Kobo, not Fnac | | [M] |

Consequence: whether a reseller may sell a gift entitlement is a **contract question**, to be answered title group by title group (per distributor), not a statutory right. A reseller that already runs a gift flow (Decitre/Vivlio [V2]) has presumably cleared it; one that does not (Fnac/Kobo [V1]) has to create both the entitlement mechanism and the authorisation.

### 4.3 Publisher / distributor authorisation checklist

All items are recommendations (analysis) unless marked.

| # | Point to settle | Why it matters | Proposed position |
|---|---|---|---|
| 1 | Who the licensed "end user" is | Retail licences are personal to "the purchaser" [M]; in a gift the reader is someone else | The licensee is the person who redeems; the reseller's terms say so |
| 2 | When the sale is reported and royalties fall due | Purchase or redemption? Amazon's rule is not verified | Report at **purchase**, at the public price in force that day; an unredeemed code is still a sale |
| 3 | Price changes between purchase and redemption | The public price can change (loi 2011-590 art. 2 [M-hi]) | The purchase-date price is final; no top-up, no refund of the difference |
| 4 | Territory | Publisher rights are territorial; VAT treatment depends on it (§6.3) | Buyer and redeemer both in the licensed territory; default France only |
| 5 | Title withdrawn before redemption | A sold code would become undeliverable | Right to deliver sold-but-unredeemed copies after withdrawal for the life of the code; otherwise refund or substitute |
| 6 | Protection | The file must be personalised to the recipient, not the buyer | DRM licence or watermark generated at claim time with the recipient's identity |
| 7 | Withdrawal and refunds | A withdrawn sale must reverse the royalty report | Reversal allowed only while the code is unredeemed |
| 8 | Bulk and corporate gifts | "Offres groupées" have their own price under art. 2 [M-hi]; collective-use licences are a separate regime | One code = one unit sale at the unit public price, unless the publisher has set a grouped price |
| 9 | The card | Cover and text on a printed object (document 07); the *prime* rule (§5) | Cover licence per document 07; card as a paid line |
| 10 | Promotion clauses | Gift flows may count as a "promotional operation" needing prior approval | Written approval from each distributor before launch |

Under loi n° 2011-590 the gift is sold at the fixed price and the card is invoiced separately (§5).

### 4.4 eBook Cadeau's role and its contract with the reseller

- **Role.** Issue and print the code, route redemption (`reseller_redirect`, `reseller_licence`). In those two lanes eBook Cadeau never stores the book and performs no copyright-relevant act on it (analysis). In `hosted_file` with `licensed_distributor` it hosts on the reseller's instructions.
- **Not in the sales chain.** eBook Cadeau never sells or transfers the code in its own name; the reseller does. This keeps it outside the VAT voucher chain (§6.3), the fixed-price law, and payment regulation (§8).
- **Contract clauses to draft** (analysis): reseller's warranty that it is authorised to sell gift entitlements for each title sent to the API; cover licence and flags (document 07); availability commitment for redemption, because downtime at eBook Cadeau is the reseller's failure to supply under Directive 2019/770 (§7.2); indemnities both ways; Art. 28 GDPR processing terms and sub-processor list (§9); revocation and reissue through the API; log retention and evidence on request; **run-off**: on termination, outstanding codes keep working for their full life or are exported to the reseller.
- **Precedent.** Enthrill's title-specific cards ran on publisher agreements — [Publishers Weekly](https://www.publishersweekly.com/pw/print/20140922/64046-enthrill-partners-with-walmart-canadian-publishing-2014.html) [V2].

---

## 5. Fixed book price laws and the card fee

### 5.1 France — loi n° 2011-590 du 26 mai 2011, décret n° 2011-1499 du 10 novembre 2011

Text of the law: WIPO Lex [original](https://www.wipo.int/wipolex/en/legislation/details/13861) and [consolidated to 2018](https://www.wipo.int/wipolex/fr/legislation/details/18453); legislative file at the [Sénat](https://www.senat.fr/dossierleg/ppl09-695.html); trade summary in [Livres Hebdo](https://www.livreshebdo.fr/article/la-loi-sur-le-prix-en-vigueur). These URLs come from a sibling note's searches; the articles below are quoted from memory and must be read on Légifrance.

| Provision | Content (from memory) | Conf. |
|---|---|---|
| Art. 1 | Covers a *livre numérique* that is a work of the mind, sold in digital form and either published in print or printable by its content and composition, "à l'exception des éléments accessoires propres à l'édition numérique" | [M-hi] |
| Art. 2 | Any person established in France who publishes an eBook for commercial distribution in France must set a public sale price "pour tout type d'offre à l'unité ou groupée"; the price "peut différer en fonction du contenu de l'offre et de ses modalités d'accès ou d'usage". Exception for collective/professional licences (research, higher education) | [M-hi] |
| Art. 3 | That price "s'impose aux personnes proposant des offres de livres numériques aux acheteurs situés en France" — extraterritorial, keyed to the **buyer's** location | [M-hi] |
| Art. 4 | "Les ventes à primes de livres numériques ne sont autorisées (…) que si elles sont proposées par l'éditeur (…) simultanément et dans les mêmes conditions à l'ensemble des personnes mentionnées à l'article 3". The article is recalled as opening with a reservation for the general consumer-law rule on primes | [M-med] |
| Art. 5 | The publisher's trade discount must reflect the retailer's qualitative services | [M-hi] |
| Décret 2011-1499 | Defines "contenu de l'offre", "modalités d'accès" (download, streaming, removable medium) and "modalités d'usage" (private/collective, duration, printing/copying/transfer rights) | [M-med] |
| Décret n° 2012-146 du 30 janvier 2012 | Breach = *contravention de 3e classe* (max EUR 450, ×5 for legal persons), per offence | [M-lo] |
| Loi n° 2014-344 (Hamon) | Created the *médiateur du livre*; sworn agents can record breaches. In February 2015 the mediator held that unlimited-subscription offers priced by the platform breached the 2011 law — novel offers are scrutinised | [M-med] |

Differences from print (loi n° 81-766, "loi Lang"):

- **No 5 % retailer discount** for eBooks: the price is a floor and a ceiling [M-med].
- Loi Lang art. 1 lets a retailer add "les frais ou rémunérations correspondant à des prestations supplémentaires exceptionnelles expressément réclamées par l'acheteur et dont le coût a fait l'objet d'un accord préalable" [M-hi]. No equivalent sentence is recalled in the 2011 law [M-lo], but the logic transfers: a service the buyer expressly asks and pays for is not part of the book price (analysis).
- Loi Lang art. 6 bans primes unless offered by the publisher to all retailers [M-hi] — the model for art. 4 of the 2011 law.

**What is a *prime*.** A product or service given **free** with a purchase (former Code de la consommation art. L121-35, now L121-19) [M-med]. General consumer law has allowed primes since 2011 unless they are an unfair practice; the book-specific ban is a stricter special rule [M-med]. Before that liberalisation, the consumer code's regulations excluded from the notion of prime the product's usual packaging and free services "dépourvues de valeur marchande" that are not ordinarily sold [M-lo] — the reasoning by which free gift wrap was never treated as a prime, and the best analogy for a free print-at-home PDF. Whether those exclusions still exist in any form was not checked. The compatibility of the book-specific ban with Directive 2005/29 is arguable after CJEU *Mediaprint* (C-540/08, 2010) [M-med]; that argument should not be built on.

No ministry, mediator or court position on gift cards, gift wrap or shipping accompanying an **eBook** is known to either note. This is the first question for counsel.

### 5.2 Scenarios for the card (France)

All readings are (analysis) on the [M-med] text of art. 4.

| Scenario | Reading | Risk |
|---|---|---|
| Buyer pays the public price; recipient downloads | Price paid in full; the law does not care who reads | None |
| Free **print-at-home PDF** | A presentation of the delivered code, of negligible value; comparable to free gift wrap | Low |
| Free **printed and mailed card** (cost EUR 2–7) | Free good with market value tied to the sale → *prime* under art. 4, unless the *publisher* offers it to all retailers on the same terms; alternatively an indirect discount | Medium–high |
| **Paid optional card**, own price line, ≥ cost | Separate service expressly requested; not a prime (primes are free) | Low |
| Paid card far below cost | Disguised-discount argument by a competitor, the SNE/SLF or the mediator | Medium |
| **Compulsory** paid card to obtain the gift option | The buyer pays more than the public price to get the eBook as a gift; the price is also a ceiling | Medium — keep a free route |
| Publisher defines a "gift offer" with its own public price (art. 2 lets prices vary by access modality) | Lawful if that price binds every retailer | Low; needs publisher action |
| Free or cheap **shipping of the card** | Not caught by the book shipping floor (§5.4) | Low |
| Author or publisher **gives away** codes (festival, press) | No sale, so no price to respect. A publisher *selling* direct must charge its own public price | Low |
| Self-published author sells card + eBook at a fair | The author is the publisher if established in France; the eBook part must equal the price shown at all retailers; the receipt shows the split | Low |
| Buyer abroad, recipient in France | Art. 3 is keyed to the buyer; the French price is arguably not binding. Publisher territorial rights still apply | Unclear |
| Standalone mode: eBook Cadeau sells a card for an eBook bought elsewhere | eBook Cadeau sells no eBook; nothing it gives or charges is tied to *its* sale of a book | None for eBook Cadeau |

### 5.3 Card pricing rules that follow

1. The eBook line always equals the publisher's public price. The gift option never changes it.
2. The gift option is always available without a paid card (free print-at-home PDF, or the reseller's e-mail).
3. The printed card is an optional line, never pre-ticked (also required by Directive 2011/83 art. 22 [M-hi]), with its own price.
4. That price is at or above the fully loaded cost (print, envelope, postage where not charged separately, payment fee), compared excluding VAT. For orientation, the sibling printing note puts single-copy production at about EUR 2.4–2.8 per card and about EUR 6 landed with postage (`notes/printers.web2print.md`, not re-checked); the EUR 3.50 used in §6 is this brief's example, not a costed price.
5. A zero-price printed card is acceptable only as a publisher-backed offer made to all retailers on the same terms (art. 4), recorded as such (`publisher_premium = true` in document 07, rule 29).
6. Discounts, coupons and loyalty credits never fund the eBook line. A promotion may apply to the card line, not below cost.

There is a tension noted in document 07: charging separately helps on price law but weakens the argument that the card is a mere accessory of the sale for the *cover* licence. The price-law risk is the more concrete of the two, so the separate line is kept.

### 5.4 Loi "Darcos" — loi n° 2021-1901 du 30 décembre 2021

Delivery of a **printed** book may not be free unless collected in a bookshop, and must be charged at a minimum set by order. Arrêté du 4 avril 2023: **EUR 3 minimum** for orders containing less than EUR 35 of new books, EUR 0.01 above; in force 7 October 2023 [M-hi]. Amazon's challenge went from the Conseil d'État to the CJEU in 2024 (case number recalled as C-366/24) [M-lo]; the outcome is not known, and the search for it was refused.

Effect on this project: **none on the eBook or on the card alone**. The card is not a *livre*. The floor applies only if a parcel also contains new printed books; then the retailer's ordinary book-shipping rule governs the parcel (analysis). No amendment of the 2011 eBook law by the Darcos law is recalled [M-lo].

### 5.5 Germany — Buchpreisbindungsgesetz (BuchPrG)

| Point | Content | Conf. |
|---|---|---|
| § 2(1) Nr. 3 (since 1 Sept 2016) | Covers products that reproduce or substitute books, "wie zum Beispiel zum dauerhaften Zugriff angebotene elektronische Bücher" | [M-hi] |
| § 2(2) | Foreign-language books fall under the law only if predominantly intended for sale in Germany — so French-language eBooks sold to readers in Germany are normally outside it | [M-med] |
| § 3 | Whoever sells commercially to end buyers **in Germany** must charge the fixed price — applies to foreign sellers since 2016 | [M-hi] |
| § 5 | The publisher sets the price and may set different prices per edition or format | [M-hi] |
| § 7(4) | No breach if the retailer, on the occasion of a book sale, gives goods of low value, bears shipping or special procurement costs, or provides "andere handelsübliche Nebenleistungen" | [M-med] |
| Case law | BGH 23 July 2015, I ZR 83/14 (*Gutscheinaktion beim Buchankauf*): vouchers that fund price-bound books breach § 3. Trade practice treats about 2 % of the book price as "low value" | [M-med] / [M-lo] |

Germany is **more permissive on the card** than France: gift wrapping is the textbook *handelsübliche Nebenleistung*, and a low-value insert is expressly allowed. A printed, mailed card worth more than the eBook is not "low value"; it is charged for.

### 5.6 Other EU regimes (all [M-lo] unless noted)

| Country | Fixed price for print | eBooks covered? |
|---|---|---|
| Austria | Yes (BPrBG, recast 2023) | Yes, since 2014; max 5 % retailer discount |
| Spain | Yes (Ley 10/2007, max 5 % discount) | Yes in principle — the statutory definition of *libro* includes digital |
| Italy | Yes (Legge 128/2011 "Levi", Legge 15/2020: max 5 %) | No |
| Netherlands | Yes — **not abolished** (Wet op de vaste boekenprijs, 2005) [M-med] | No |
| Belgium | Flanders (decree of 2016, in force 2017) and Wallonia-Brussels (décret du 19 octobre 2017) | Wallonia-Brussels: yes; Flanders: not sure |
| No fixed price | UK, IE, SE, FI, DK, PL, CZ, CH | — |

---

## 6. VAT and vouchers

### 6.1 Rates (as recalled; to be verified for 2026)

| Supply | FR | DE | Others |
|---|---|---|---|
| eBook (download) | 5.5 % — CGI art. 278-0 bis A, books "y compris ceux fournis par téléchargement" [M-med] | 7 % since 18 Dec 2019, § 12(2) Nr. 14 UStG [M-hi] | BE 6, NL 9, ES 4, IT 4, AT 10, LU 3, PT 6, PL 5, SE 6, IE 0 (since 2024), DK 25 [M-med/lo] |
| Printed card, envelope, postage charged to the customer | 20 % | 19 % | BE 21, NL 21, ES 21, IT 22, AT 20, LU 17 [M-hi] |
| Fee for generating a print-at-home PDF | Electronically supplied service, standard rate | same | same |

Basis for reduced eBook rates: Directive (EU) 2018/1713, after CJEU C-479/13 *Commission v France* (2015) had condemned the earlier French 5.5 % [M-hi]. Postage charged by the supplier is part of the taxable amount of the goods (VAT Directive art. 78) [M-hi]; it is not a pass-through disbursement, because the seller ships its own supply in its own name (analysis on CGI art. 267 [M-med]). A card is not a "book" for VAT anywhere known.

### 6.2 One supply or two?

- CJEU *Card Protection Plan* (C-349/96, 1999): each supply is normally distinct; an element is ancillary when it is not an aim in itself for the customer but a means of better enjoying the principal supply. *Levob* (C-41/04, 2005): elements so closely linked that splitting them is artificial form one supply. *Stadion Amsterdam* (C-463/16, 2018): one supply takes one rate [M-hi].
- France codified this in CGI art. 257 ter and **art. 278-0**: a single operation made of non-ancillary elements at different rates bears the **highest rate** [M-med].
- Consequences (analysis): (i) a free print-at-home PDF is ancillary — the whole price stays at 5.5 %; (ii) an *optional, separately priced* printed card is a distinct supply at 20 %; optional purchase and separate pricing are strong indicators, not conclusive; (iii) a compulsory "gift pack" at one price risks 20 % on the eBook too. A taxpayer could argue that a paid card is ancillary to the eBook (5.5 % on the card); the conservative position is 20 %.

### 6.3 Vouchers — Directive (EU) 2016/1065 (VAT Directive arts 30a, 30b, 73a; vouchers issued from 1 Jan 2019)

| Term | Definition / rule | Conf. |
|---|---|---|
| Voucher | Instrument carrying an obligation to accept it as consideration for a supply, where the goods/services or the potential suppliers are indicated on it or in its terms | [M-hi] |
| Single-purpose (SPV) | Place of supply **and** VAT due are known at issue. Each transfer by a taxable person acting in its own name is treated as the supply itself; redemption is not a taxable event | [M-hi] |
| Multi-purpose (MPV) | Any other voucher. VAT only at redemption; taxable amount = price paid for the voucher (art. 73a) | [M-hi] |
| Transposition | FR: CGI art. 256 ter (*bon à usage unique / à usages multiples*). DE: § 3(13)–(15) UStG (*Einzweck-/Mehrzweck-Gutschein*) | [M-med] |
| CJEU C-68/23 *M-GbR* (18 April 2024) | Prepaid codes for digital content limited to users in Germany are SPVs: the "place of supply" test looks at the final supply to the end user, and a territorial restriction in the terms makes it known | [M-med] |

A code for one named eBook has a known rate **only if the redeemer's country is fixed**, because electronically supplied services to consumers are taxed where the customer is (VAT Directive art. 58) [M-hi]. So:

| Design | VAT treatment | Operational effect |
|---|---|---|
| Code redeemable only by users located in France (terms + account-country check) | SPV → FR 5.5 % due at sale | Simple. VAT is due even on codes never redeemed |
| Code redeemable EU-wide | Arguably MPV → VAT at redemption in the redeemer's Member State, via OSS | Needs location evidence at redemption; unredeemed codes untaxed; the price received is fixed while the rate varies |
| Sale treated as an immediate supply of the eBook to the **buyer**, with deferred download | VAT at payment, buyer's country | Same result as the SPV for domestic gifts; a defensible alternative reading |

The geo-blocking Regulation (EU) 2018/302 does not prevent a territorial restriction: art. 4(1)(b) excludes services whose main feature is access to copyright works, eBooks included [M-hi]. Amazon itself restricts gifts to recipients in the buyer's country [V2].

The copyright note states without qualification that such a code is an SPV; the commerce note's conditional reading is kept (§3.7, row 5).

### 6.4 Which customer — buyer or recipient?

Outside the voucher rules, the VAT "customer" is the party to the legal relationship who pays: the **buyer** [M-med]. Location evidence follows Implementing Regulation (EU) 282/2011 arts 24a–24f: two non-contradictory items (billing address, IP, bank country…), or one item if such cross-border supplies stay under EUR 100,000 a year [M-med]. Under an MPV reading it is the **redeemer**. This buyer/recipient split is the main open tax question for cross-border gifts; a written position from a VAT adviser is needed before selling outside France.

### 6.5 Cross-border mechanics

- **EUR 10,000** EU-wide annual threshold for cross-border B2C digital services plus intra-EU distance sales of goods (art. 59c); below it a supplier established in one Member State may charge home VAT; above it, destination VAT through the **Union OSS** (since 1 July 2021), records kept 10 years [M-hi].
- A printed card mailed to a consumer in another Member State is an intra-EU distance sale of goods, taxed where transport ends once the threshold is passed — whoever ordered it [M-hi].
- French *franchise en base* (micro-entreprise): no VAT charged below national thresholds. The 2026 figures are disputed in the sibling market note: EUR 85,000 (goods) / EUR 37,500 (services) per most sources, EUR 91,900 / EUR 36,800 per others — [Entreprisma](https://entreprisma.fr/micro-entrepreneur), [Developp](https://developp.fr/micro-entreprise-plafonds/) [V1, conflicting]. The franchise does not shelter supplies located in other Member States beyond the EUR 10,000 threshold, subject to the EU SME scheme applicable since 1 Jan 2025 [M-lo].
- **Art. 9a of Regulation 282/2011**: whoever "takes part" in a supply of electronic services through a portal is presumed to be the supplier unless the real provider is explicitly shown as such and the intermediary neither authorises the charge or delivery nor sets the terms [M-hi]. If eBook Cadeau sold eBooks for authors it would become the deemed VAT supplier. **Standalone mode sells only the card service.**
- Service fee invoiced to a reseller: ordinary B2B service (FR 20 %, or reverse charge to a business customer in another Member State, art. 44) [M-hi].
- French e-invoicing reform: as recalled, all businesses must be able to *receive* electronic invoices from 1 Sept 2026, and small businesses must *issue* them and report B2C transactions from 1 Sept 2027 [M-med]. The invoicing design should be checked against it.

### 6.6 Worked examples

**Assumptions.** Consumer prices include VAT (TTC). eBook at the publisher's price EUR 9.99; optional printed card EUR 3.50; postage re-billed at the 2026 La Poste stamp price for a letter up to 20 g: EUR 1.52 within France, EUR 2.25 international — [La Poste](https://www.laposte.fr/tarifs-postaux-courrier-lettres-timbres), [Tout sur mes finances](https://www.toutsurmesfinances.com/argent/a/prix-du-timbre) (via `notes/market-business.competitors-costs.md`) [V2]. A card plus envelope above 20 g moves to the 100 g band (EUR 3.10 / EUR 4.85 in the same note). The seller is a French reseller on the normal VAT regime; the buyer is a consumer in France. Rates as in §6.1. Amounts are rounded to the cent per line (excl. VAT = TTC ÷ (1 + rate), VAT = TTC − excl. VAT). All treatments are [M-med] at best.

**Example A — recipient in France; code redeemable in France only (SPV, VAT due at sale).**

| Line | Price TTC | Rate | Excl. VAT | VAT |
|---|---|---|---|---|
| eBook (code) | 9.99 | FR 5.5 % | 9.47 | 0.52 |
| Printed card | 3.50 | FR 20 % | 2.92 | 0.58 |
| Postage | 1.52 | FR 20 % | 1.27 | 0.25 |
| **Total** | **15.01** | | **13.66** | **1.35** |

Variants on the same EUR 15.01:

| Treatment | Excl. VAT | VAT | Difference for the seller |
|---|---|---|---|
| Three separate lines (table above) — recommended | 13.66 | 1.35 | — |
| One bundled "gift pack" price taxed at 20 % (CGI art. 278-0 risk) | 12.51 | 2.50 | −1.15 per order |
| Card and postage argued ancillary to the eBook, all at 5.5 % (aggressive) | 14.23 | 0.78 | +0.57 per order, with reassessment risk |

Stamps bought from La Poste carry no deductible VAT [M-med], so EUR 1.52 charged to the customer yields EUR 1.27 against a EUR 1.52 cost. Passing postage through at cost requires charging EUR 1.82 TTC (analysis).

**Example B — card posted to a recipient in Germany.** Under the default France-only code the recipient needs an account in the French store; the order should warn the buyer. If the code is made redeemable in Germany, the publisher's rights must cover Germany and the code is arguably an MPV. Three treatments on the same EUR 15.74 (9.99 + 3.50 + 2.25):

| Line | B1: seller under the EUR 10,000 EU threshold, no OSS option | B2: OSS; eBook treated as sold to the French buyer (or France-only SPV) | B3: OSS; EU-wide code treated as MPV, redeemed by a resident of Germany |
|---|---|---|---|
| eBook 9.99 | FR 5.5 %: 9.47 + 0.52 | FR 5.5 %: 9.47 + 0.52 | DE 7 %: 9.34 + 0.65, due at redemption |
| Card 3.50 | FR 20 %: 2.92 + 0.58 | DE 19 %: 2.94 + 0.56 | DE 19 %: 2.94 + 0.56 |
| Postage 2.25 | FR 20 %: 1.88 + 0.37 | DE 19 %: 1.89 + 0.36 | DE 19 %: 1.89 + 0.36 |
| **Excl. VAT / VAT** | **14.27 / 1.47** | **14.30 / 1.44** | **14.17 / 1.57** |
| Where declared | French return | eBook: French return; card and postage: OSS, Germany | All three lines: OSS, Germany |

Non-tax points in Example B: the French fixed price still applies (the buyer is in France, art. 3 [M-hi]); the German fixed price normally does not reach a French-language title (§5.5 [M-med]); an envelope sent to Germany triggers packaging registration there (§11.3 [M-hi]).

**Example C — standalone mode: eBook Cadeau sells only the card and postage** (the eBook was bought elsewhere, e.g. through a store gift link).

| Seller status | Card 3.50 + postage 1.52 = 5.02 TTC | Kept by the seller |
|---|---|---|
| *Franchise en base* (invoice mention "TVA non applicable, art. 293 B du CGI" [M-hi]) | No VAT charged, no input VAT deducted | 5.02 |
| Normal regime | 20 %: 4.19 excl. VAT + 0.83 VAT | 4.19 |

**Example D — eBook Cadeau's fee to the reseller** (B2B). A fee of EUR 1.00 excl. VAT is invoiced EUR 1.20 to a French reseller, who deducts the VAT; to a reseller established in Germany it is invoiced EUR 1.00 with reverse charge [M-hi]. Whether printing and mailing a card to a consumer on behalf of a foreign reseller is a service (reverse charge) or a supply of goods located in France is a question for the accountant.

**Unredeemed and withdrawn codes** (analysis): under the SPV reading the VAT paid at sale stays due if the code is never used; under the MPV reading no VAT arises on an unredeemed code. A withdrawal within 14 days is a cancelled sale: credit note, VAT reversed. Sibling research puts unused gift cards in France at about 20–25 % — [Europe 1](https://www.europe1.fr/societe/cartes-et-cheques-cadeaux-comment-loubli-de-leur-utilisation-constitue-un-business-juteux-pour-les-enseignes-4223693), [Sleeping Money](https://www.sleepingmoney.com/fr-fr/etudes/achat-revente-cartes-cadeaux-france/) (industry sources) [V2]; no figure was found for title-specific eBook codes.

---

## 7. Consumer law

### 7.1 Right of withdrawal

| Rule | Source | Conf. |
|---|---|---|
| 14 days from conclusion for digital content not on a tangible medium | Directive 2011/83 art. 9(2)(c) | [M-hi] |
| Right lost when performance has begun **and** the consumer gave prior express consent, acknowledged the loss, **and** the trader confirmed this on a durable medium | Directive 2011/83 art. 16(m) as amended by Directive 2019/2161; Code de la consommation art. L221-28 13° | [M-hi] |
| If the waiver is defective the consumer may withdraw and owes nothing | art. 14(4)(b) | [M-hi] |
| Missing withdrawal information extends the period by 12 months | art. 10 | [M-hi] |
| No withdrawal for goods "confectionnés selon les spécifications du consommateur ou nettement personnalisés" | art. 16(c); L221-28 3° | [M-hi] |
| Online **withdrawal function** ("button") for distance contracts concluded through an online interface, from 19 June 2026 | Directive (EU) 2023/2673 inserting art. 11a; French transposition not checked | [M-med] |
| No pre-ticked boxes for extra payments | Directive 2011/83 art. 22; L121-17 | [M-hi] |

Gift specifics (analysis; no authority found):

- The **buyer** is the consumer; the recipient has no withdrawal right.
- "Performance has begun" is generally read as the start of the download. Handing over a code is not that. For 14 days an **unredeemed** code can therefore be withdrawn, and the platform must be able to revoke a code and refund. A printed card may already be in the post; the terms state that withdrawal voids the code.
- For redemption inside the 14 days, the waiver is collected from the buyer at checkout, worded to cover redemption "par vous ou par la personne à qui vous remettez le code", and repeated in the confirmation e-mail. Whether a third party's download counts as performance begun at the consumer's request is untested [M-lo].
- Without a valid waiver, a recipient could redeem and the buyer could still withdraw and be refunded in full. The safe rule is: no B2C gift code is redeemable inside the 14 days unless the waiver was recorded.
- A card bearing a personal message or a name is personalised: no withdrawal for the card itself. A generic card carrying only a unique code is doubtful.

### 7.2 Conformity of digital content — Directive (EU) 2019/770

French transposition: ordonnance n° 2021-1247, Code de la consommation arts L224-25-1 ff., in force 1 Jan 2022 [M-med].

- Supply is made when the content "or any means suitable for accessing or downloading" it is made available to the consumer or to a facility chosen by the consumer (art. 5). Delivering the code may count as supply, so the availability of the redemption service becomes a conformity matter [M-hi].
- The trader bears the burden of proving supply (art. 12(1)); liability runs 2 years for a single supply, with the burden of proof on the trader for 1 year (arts 11–12); remedies: bringing into conformity, price reduction, termination with refund within 14 days (arts 14–18) [M-hi].
- **Owed to whom**: the contracting consumer. The recipient is a third party unless (a) the terms extend the guarantee to the bearer (*stipulation pour autrui*, Code civil art. 1205), or (b) the recipient enters a contract at redemption [M-med]. Recommendation: the terms state that "le porteur du code bénéficie des mêmes garanties que l'acheteur", and either person can open a support case with only the code.
- The Directive leaves copyright untouched: it gives the buyer conformity rights, not transfer rights [M].
- Reseller mode: the reseller is the liable trader; redemption downtime at eBook Cadeau is the reseller's failure to supply → availability commitment and indemnity in the B2B contract (§4.4).

### 7.3 Pre-contractual information

Directive 2011/83 art. 6(1)(r)–(s); Code de la consommation arts L111-1 and L221-5: functionality including technical protection measures, and relevant compatibility and interoperability with hardware and software [M-hi]. The buyer usually does not know the recipient's device, so the information must be on the product page **and**, in short form, on the card: format (EPUB/PDF), protection (none / watermark / LCP / Adobe DRM), "lisible sur…", "non lisible directement sur Kindle" where true.

### 7.4 Expiry and unredeemed codes

| Country | Rule for vouchers | Conf. |
|---|---|---|
| France | No statutory minimum validity; the issuer sets it, with clear prior information. Expired = lost, in principle. Limits: unfair-terms control (L212-1; R212-1 blacklists clauses that bind the consumer while the trader does not perform); 5-year general limitation period (Code civil art. 2224) | [M-med] |
| Germany | Limitation 3 years from the end of the year of purchase (§§ 195, 199 BGB); shorter expiry in standard terms is generally void (§ 307 BGB; OLG München 17 Jan 2008, 29 U 3193/07, Amazon one-year limit) | [M-med] |
| Netherlands | Minimum 2 years (since 2022) | [M-lo] |
| Ireland | Minimum 5 years (Gift Vouchers Act 2019) | [M-med] |

A code for a fully paid, named eBook is closer to a deferred delivery than to stored value: an expiry that lets the seller keep the price without delivering is exposed under unfair-terms law (analysis). Recommendation: **no expiry, or at least 5 years**, and a refund-or-substitute rule if the title is withdrawn. The reseller–publisher contract must allow delivery of sold-but-unredeemed copies after a title leaves the catalogue (§4.3).

### 7.5 Other items

- A consumer mediator must be named in the terms (L612-1) [M-hi].
- The EU ODR platform closed on 20 July 2025 (Regulation (EU) 2024/3228): old ODR links are removed [M-med].
- Contracts concluded online for EUR 120 or more must be archived 10 years (L213-1) [M-med].
- A website operated by a business must show its publisher's identity and its host (LCEN) [M-med].

---

## 8. Payment-regulation status

- **E-money** (Directive 2009/110/EC art. 2(2)): electronically stored *monetary value*, a claim on the issuer, issued on receipt of funds *to make payment transactions*, and accepted by someone *other than the issuer* [M-hi]. A code for one named eBook stores no monetary value, makes no payment, and in reseller mode is accepted only by its issuer. It fails the definition three times; the limited-network exclusion is not even needed (analysis).
- **PSD2 art. 3(k)** excludes instruments usable only at the issuer's premises or in a limited network, or for a very limited range of goods or services; art. 37(2) requires notification to the regulator above EUR 1 million of transactions in 12 months (France: ACPR; Code monétaire et financier art. L521-3 and following) [M-hi; article numbers M-lo]. EBA guidelines EBA/GL/2022/02 narrow the exclusion [M-med]. Store-value gift cards rely on it; a title-specific code does not need it.
- **Where the risk would appear** (analysis):

| Design | Why it is a problem | Decision |
|---|---|---|
| A code carrying a euro value, or usable for any title | Stored monetary value → e-money or limited-network analysis | Not built |
| A code accepted by several independent sellers, with eBook Cadeau settling between them | Acceptance by third parties; settlement = payment service | Not built |
| eBook Cadeau collecting buyers' money for publishers or authors | Holding third-party funds = payment service; the commercial-agent exclusion in art. 3(b) is narrow; also deemed-supplier VAT (§6.5) and DAC7 platform reporting (Directive (EU) 2021/514) [M-med] | Not built; if ever needed, a licensed PSP's split-payment product |
| Card carrying a store's own gift-card code bought by the giver (§3.6) | The store is the issuer; eBook Cadeau only prints what the giver bought | Allowed |

- PSD3 / Payment Services Regulation: political agreement recalled for late 2025 [M-lo]; status in November 2026 not checked.

---

## 9. GDPR design notes

### 9.1 Flows

| Flow | Personal data | Legal basis | Retention (proposal) |
|---|---|---|---|
| Print-at-home, PDF built in the browser, gift-link lane (S1) | None server-side | — | — |
| Print-at-home with a hosted gift page | Code, timestamp; giver name, recipient name and message if the giver chooses to show them on the gift page | Art. 6(1)(b) contract with the giver; 6(1)(f) for the recipient's name | Names and message: 12 months after first claim, or until cancellation |
| Hosted-file upload (S2, S3) | Uploader identity, verified e-mail, declaration, file hash; any buyer data embedded in a file | 6(1)(b); 6(1)(c) for the LCEN identification duty [M] | Declaration: life of the hosting plus the limitation period |
| Buyer account, order, invoice | Name, e-mail, billing data | 6(1)(b); 6(1)(c) for accounting | Invoices 10 years (Code de commerce art. L123-22) |
| Postal fulfilment to the recipient | Recipient name and postal address, gift message | 6(1)(f) — the recipient is not a party, so 6(1)(b) is unavailable | Printer: ≤ 30 days; service: about 90 days (delivery-claim window) |
| Redemption | Code, timestamp, IP, user agent; e-mail only if the reader chooses "send to my e-reader" | 6(1)(f) fraud prevention and proof of supply; 6(1)(c) for VAT location evidence | Security logs 6–12 months (CNIL logging guidance); proof of supply 5 years; VAT/OSS evidence 10 years, pseudonymised |

The table is [M-hi] on the legal bases and [M-med] on retention periods; all durations are proposals.

### 9.2 Roles

| Mode | Controller | Processor | Sub-processors |
|---|---|---|---|
| Standalone | eBook Cadeau | Printer, mail house, host | — |
| Reseller-integrated | Reseller | eBook Cadeau | Printer, mail house, host — each with the reseller's prior authorisation |

### 9.3 Points that matter

- **The paper card's advantage.** It needs no recipient e-mail and no account. E-mail gifting (the alternative) needs the recipient's e-mail, an Art. 14 notice in the message, and care not to turn a gift notice into marketing [M-hi].
- **Art. 14** (data not obtained from the data subject): when the buyer supplies a recipient's address, the recipient is informed at the latest at first contact — a two-line notice on the delivery slip or card back (who supplied the address, purpose, deletion delay, contact) [M-hi].
- **Minimisation and design** (arts 5(1)(c), 25): redemption without an account; a hash of the code is stored, not the code (`docs/api.md`); no third-party trackers on the redemption page, so no consent banner is needed under art. 82 of the loi Informatique et Libertés [M-med].
- **Reading choices can reveal sensitive data.** CJEU C-184/20 (2022) and C-21/23 *Lindenapotheke* (2024) treat data from which health or orientation can be inferred as Art. 9 data [M-med]. No profiling on titles; the buyer is not told whether or when the gift was redeemed unless the recipient agrees.
- **Watermarks.** A watermarked file contains personal data of whoever it was issued to (§2.4). In reseller lanes the mark names the recipient and is generated by the reseller at claim time. In standalone lanes retailer-watermarked files are refused; an author's own file is delivered unchanged.
- **Gift links are bearer secrets.** In lane S1 the link stays in the browser and in the printed PDF; it is not sent to or logged by the server (analysis).
- **Processors**: Art. 28 contract with every printer and mail house; list of sub-processors [M-hi].
- **Transfers**: EU/EEA printers and hosting preferred. The UK and Switzerland have adequacy decisions (UK renewal recalled for late 2025 [M-lo]); US vendors need the Data Privacy Framework (challenge dismissed by the General Court, T-553/23, 3 Sept 2025; appeal pending [M-lo]) or standard contractual clauses.
- **Minors**: Art. 8 applies only to consent-based processing by an online service offered directly to a child (age 15 in France, 16 in Germany) [M-hi]. Consent is avoided as a basis at redemption. A minor can accept a gift and perform everyday acts (Code civil art. 1148) [M-med].
- A record of processing (Art. 30) is required; a DPO and a DPIA probably are not, at this scale [M-med].

---

## 10. European Accessibility Act — Directive (EU) 2019/882

| Point | Content | Conf. |
|---|---|---|
| Scope | Services: "e-books and dedicated software" (art. 2(2)(e)) and "e-commerce services" (art. 2(2)(f)); products: e-readers | [M-hi] |
| Date | Applies to products placed and services provided after **28 June 2025**; existing service contracts and legacy set-ups tolerated until 28 June 2030 (art. 32) | [M-hi] |
| Micro-enterprise | Fewer than 10 staff **and** turnover or balance sheet ≤ EUR 2 million: exempt for services (art. 4(5)) | [M-hi] |
| Other escape | Disproportionate burden or fundamental alteration, with a documented assessment (art. 14) | [M-hi] |
| Requirements | Annex I: websites and apps perceivable, operable, understandable, robust (in practice EN 301 549 → WCAG 2.1 AA); accessible identification, security and payment functions; information about the service in accessible form. For eBooks: no blocking of assistive technology, structure and navigation, reflow, **DRM must not block accessibility features**, accessibility metadata | [M-hi] |
| France | Loi n° 2023-171 du 9 mars 2023 (art. 16), décret n° 2023-931 du 9 octobre 2023; control split between DGCCRF, Arcom and others. Backlist eBooks: compliance deadline recalled as 2030 | [M-med] / [M-lo] |
| Germany | Barrierefreiheitsstärkungsgesetz (BFSG), same date | [M-hi] |

For this project (analysis):

- The **redemption site** is part of an e-commerce service and the gateway to an eBook. It is built to WCAG 2.1 AA whatever the micro-enterprise exemption says: resellers are in scope and will require it by contract. An accessibility statement is published.
- **A QR code is not accessible on its own.** The card prints a short URL and a typed code in a large, high-contrast face, grouped in blocks, without look-alike characters. Rate limiting replaces CAPTCHA; any challenge needs a non-visual alternative.
- The **printed card** is outside the EAA (paper), but the same information must exist in accessible form online.
- **eBook files** are the publisher's and retailer's duty. The redemption page shows the title's accessibility metadata when supplied, and files are not wrapped in a protection that defeats screen readers.

---

## 11. Mandatory mentions, eco-labelling and EPR

### 11.1 On the card (France)

| Mention | Basis | Conf. |
|---|---|---|
| What the card gives (title, author, one digital copy), how to redeem, single use, validity, format/DRM, customer-service contact | L111-1, L221-5 (information); good practice | [M-med] |
| Seller identity; for a registered business on "documents publicitaires": SIREN, "RCS + city", registered office, legal form and capital | Code de commerce arts R123-237, R123-238 | [M-med] |
| French language (other languages may be added) | Loi n° 94-665 ("Toubon") art. 2 | [M-hi] |
| Printer's name and address on any publicly distributed print, except "ouvrages de ville ou bilboquets" (private stationery) | Loi du 29 juillet 1881 art. 3 | [M-med] |
| **Triman + Info-tri** sorting label | Code de l'environnement art. L541-9-3 (loi AGEC art. 17), décret n° 2021-835 | [M-med] |
| No "biodégradable" / "respectueux de l'environnement" claims; FSC/PEFC logos only under the printer's licence | AGEC art. 13; scheme rules (see `notes/printers.esg.md`) | [M-med] |

Not required: a price (better omitted on a gift), an ISBN (the card is not a book), legal deposit (commercial jobbing print is exempt [M-lo]). A one-off personalised card given privately is arguably an *ouvrage de ville*; a print run of promotional cards for a publisher is not (analysis).

Suggested legal line (to adapt): *« Bon pour un exemplaire numérique de "Titre" (Auteur, Éditeur), format EPUB. À retirer sur exemple.fr/c avec le code ci-dessus. Usage unique. Sans date limite. Vendu par [dénomination, forme, capital, RCS, siège]. Aide : […]. Imprimé par [imprimeur, ville]. »* + Triman/Info-tri pictogram. For postal delivery, the Art. 14 notice of §9.3 is added.

### 11.2 EPR in France

- Printed papers other than books issued to households fall under the *papiers graphiques* EPR (Code de l'environnement art. L541-10-1 3° — "à l'exception des livres"); household packaging (the envelope) falls under the packaging EPR. The two schemes were merged by loi n° 2023-305; Citeo is the main approved body (Adelphe and Léko are also approved for packaging) [M-med].
- The producer must join an approved body, obtain a **unique identifier (IDU)** from ADEME and show it in its terms of sale (art. L541-10-13) [M-med]. Very small quantities get a flat-rate declaration; whether the old 5-tonne exemption for papers survives is not known [M-lo].
- **Who is the producer** (analysis): the brand that issues the card — the reseller in reseller mode; eBook Cadeau, or the professional who orders a print run, in standalone mode. A private person printing at home is nobody's producer.
- Missing Triman: administrative fine up to EUR 15,000 per legal person (art. L541-9-4) [M-lo].
- Outlook: the Commission has challenged the Triman as a barrier to trade, and the Packaging Regulation (EU) 2025/40 (general application recalled as 12 Aug 2026, harmonised labels later) will displace national packaging labels [M-lo]. The label is designed as a swappable template element.

### 11.3 Germany and other Member States

- **VerpackG**: whoever first places filled packaging — shipping envelopes included — on the German market for consumers must register in **LUCID** *before* shipping and join a dual system; no de minimis; applies to foreign distance sellers; fines and a sales ban [M-hi]. Whether Germany has replaced the VerpackG to implement Regulation 2025/40 is not known [M-lo].
- The card itself is a product, not packaging; Germany has no statutory EPR for printed paper [M-med].
- Other Member States have their own packaging EPR; several require an authorised representative for foreign sellers [M-lo].
- Design lever: a **sealed self-mailer** (the card folds and seals to hide the code, no envelope) removes packaging EPR from the picture; only the French paper EPR remains. Otherwise a destination-country mail house acts under its own registrations, confirmed in the contract (analysis).

---

## 12. Compliance checklist by launch phase

Each item is a task, not a finding. "Counsel" items refer to the question numbers in §13.

### Phase 0 — repository, demo, documentation (now)

- [ ] Wording audit: nothing presents the product as a way to pass on a purchased eBook (CPI L335-2-1; DSA recital 20).
- [ ] Fixtures are public domain in France or written for the project; provenance recorded.
- [ ] Demo stores show the reseller entitlement flow only.
- [ ] No DRM-removal code, dependency or link.
- [ ] Upload gate in place: mandatory rights basis, DRM refusal, tests.
- [ ] Legal text templates (terms, privacy notice, card legal line) marked "draft, not reviewed by counsel".

### Phase 1 — standalone, free, print-at-home (public beta, no money taken)

- [ ] Operator identity and host shown on the site (LCEN).
- [ ] Terms of use describing the upload rules and moderation (DSA art. 14); contact points (arts 11–12); notice form (art. 16); statement-of-reasons template (art. 17).
- [ ] Declaration, automated checks, blocklist, strike policy live.
- [ ] Gift-link lane live with its host allow-list.
- [ ] Privacy notice; record of processing; retention timers running; no third-party trackers.
- [ ] Accessibility self-assessment against WCAG 2.1 AA and published statement.
- [ ] Counsel: questions 1–9 answered in writing before the hosted-file lane opens to the public.

### Phase 2 — standalone, paid printed card, delivery in France

- [ ] Business registered; VAT status decided (franchise or normal regime); invoice mentions correct.
- [ ] Terms of sale: pre-contractual information, withdrawal rules (personalised card), online withdrawal function, consumer mediator named.
- [ ] Payments through a licensed PSP; no eBook money collected.
- [ ] Printer contract: Art. 28 terms, deletion within 30 days, production site named, printer's name for the imprint line.
- [ ] Art. 14 notice printed on postal products.
- [ ] Triman/Info-tri artwork on the card; EPR membership and IDU obtained; IDU in the terms of sale.
- [ ] Card legal line complete; French text present.
- [ ] E-invoicing obligations checked with the accountant.
- [ ] Counsel / accountant: questions 12, 17, 23–25, 29.

### Phase 3 — reseller pilot (France)

- [ ] B2B contract signed with the clauses of §4.4, including processing terms and run-off.
- [ ] Reseller confirms distributor/publisher authorisation for gift entitlements (§4.3) and for covers (document 07).
- [ ] Checkout: separate lines, card unticked, card price at or above cost, free route available.
- [ ] Code terms: single title, single use, France only, no expiry; revoke and reissue tested end to end.
- [ ] Withdrawal waiver collected and confirmed on a durable medium; bearer clause for the conformity guarantee.
- [ ] Written VAT position (SPV) held by the reseller; receipts show two rates.
- [ ] Accessibility audit of the redemption pages; statement published (the reseller is in EAA scope).
- [ ] EPR: agreed in writing who is the producer of the card and the envelope.
- [ ] Security review of redemption (rate limits, code entropy, logs).
- [ ] Counsel / accountant: questions 10–16, 19–22.

### Phase 4 — EU extension (Germany first)

- [ ] VAT: OSS registration once EUR 10,000 of cross-border B2C sales is reached (or by option); written position on MPV versus sale-to-buyer; redeemer-country evidence stored.
- [ ] Territorial rights per title reflected in `redemption_territory`.
- [ ] BuchPrG: German-language titles sold at the fixed price; card charged for.
- [ ] No expiry (meets the German three-year rule and the Irish five-year rule).
- [ ] Packaging: LUCID registration and dual-system contract before the first envelope, or self-mailer only; other Member States checked one by one.
- [ ] BFSG accessibility; German-language terms, legal notice and card text.
- [ ] Eco-label table extended per destination country.
- [ ] Counsel / accountant: questions 18, 21, 22, 26, 27.

---

## 13. Questions for counsel and the accountant

### IP counsel (France)

1. Confirm §2: is there any lawful route for a private buyer to transfer a store-bought eBook file to another person, including DRM-free files whose publisher terms are permissive?
2. Does a strictly one-uploader-to-one-recipient transfer reach a "public" (making available; CPI L335-2-1 "mise à disposition du public")? No authority is known.
3. Is the upload gate of §3.3 (declaration, automated checks, blocklist, strikes) sufficient diligence to keep the DSA art. 6 exemption for lanes S2/S3, given that the service knows each file's title and ISBN?
4. Exposure of the open-source repository and its maintainers under CPI L335-2-1 for a generic hosted-file feature; wording to use and to avoid.
5. Gift-link lane: may a third party encode a store's gift link in a QR on a card that names the store? Current terms of Amazon.fr, Decitre/Vivlio, Thalia; trade-mark limits.
6. Is a buyer-identifying watermark "rights-management information" (CPI L335-3-2)? Is unchanged delivery enough, and may a rights-holder authorise format conversion without moral-right issues?
7. Current CPI numbering after the 2021 ARCOM reform (L331-5 ff., L335-3-1, L335-3-2, R335-3); LCEN art. 6 as rewritten in 2024; retention periods under décret 2021-1362.
8. Public-domain test: French war extensions and "mort pour la France"; Project Gutenberg licence (20 % royalty, trade mark); reuse terms of Gallica, Wikisource, ebooksgratuits.com.
9. Does a paid card or paid printing make the use of a CC NonCommercial work "commercial"?
10. Reseller mode: does a gift entitlement fit the usual *mandat* and distribution contracts of French e-distributors? Who is the licensee? Is the *stipulation pour autrui* analysis sound? Model clause for §4.3.
11. Exact holdings and references of Cass. 1re civ. 23 Oct 2024 (*Valve*), Cass. Feb 2025 (*Dstorage*), the *Aurélien D.* line, and any decision on eBooks specifically.

### Commercial and consumer counsel

12. Loi 2011-590 art. 4: exact wording. Is a free print-at-home PDF a *prime*? A free printed card? Is a paid card at cost safe, and below cost a disguised discount? Is a compulsory card fee a price above the fixed price? Any position of the ministry, the *médiateur du livre* or a court?
13. Does the 2011 law or its decree contain an "extra services requested by the buyer" rule like loi Lang art. 1? Class and amount of the fine (décret 2012-146); amendments since 2018.
14. Withdrawal: does the recipient's download trigger art. 16(m)? Waiver wording; French transposition and practical form of the withdrawal function due from 19 June 2026; scope of the personalised-goods exclusion for the card.
15. Can a gift recipient invoke the conformity guarantee without a contractual extension? Wording of the bearer clause.
16. Expiry: confirm "no expiry". If a reseller insists on an expiry, the minimum safe duration in France and Germany.
17. Card mentions: is the card a "document publicitaire" (Code de commerce R123-237)? Is the printer's name required (loi de 1881 art. 3)? Triman/Info-tri on a small card: exemptions, artwork, fines; who is the "producer" in reseller mode; small-issuer EPR thresholds and fees.
18. Germany: is a printed gift card a *handelsübliche Nebenleistung* (BuchPrG § 7(4))? VerpackG for envelopes and the self-mailer analysis; status of Regulation (EU) 2025/40 and of the Commission's action against the Triman.
19. Payment regulation: confirm that a title-specific code is outside e-money and payment-services rules and needs no ACPR notification; PSD3 status.
20. EAA: scope of the micro-enterprise exemption, the competent French authority, penalties, and the flow-down clause resellers will ask for.

### Accountant / VAT adviser

21. Is a download code for a named title an SPV, an MPV or a direct sale with deferred delivery (BOFiP, BMF positions; exact reasoning of CJEU C-68/23)? What enforcement of the France-only restriction is required?
22. Cross-border gifts: buyer or redeemer as the customer; evidence to keep; treatment of unredeemed codes.
23. Printed card as a separate supply at 20 %: confirm; exposure to CGI art. 278-0 for an optional line; current wording of that article.
24. Postage re-billed to the customer: taxable amount, non-deductible stamp cost, pricing to pass it through.
25. *Franchise en base*: 2026 thresholds; effect of cross-border sales and of the EU SME scheme; when to leave the micro regime given that social charges apply to turnover including re-billed print and postage (`notes/market-business.competitors-costs.md`).
26. OSS: registration timing; distance sales of cards; records.
27. Printing and mailing a card for a reseller established in another Member State: service with reverse charge, or goods supplied in France?
28. Revenue recognition and VAT for unredeemed codes; credit notes on withdrawal.
29. E-invoicing and e-reporting calendar for a small B2C seller.
30. DAC7: confirm that selling only the card service keeps the service outside platform reporting.
31. Current reduced eBook rates per Member State for 2026.

### Privacy

32. Legal basis and Art. 14 wording for the recipient's postal address.
33. Retention schedule of §9.1 against CNIL recommendations and the LCEN identification duty.
34. Controller/processor split and sub-processor chain in reseller mode; transfer status (UK adequacy, Data Privacy Framework appeal).
35. Do redeemed titles amount to Art. 9 data in the service's hands, and is a DPIA needed?

---

## Could not verify

No search succeeded for either input note or for this synthesis, so **every [M] item is unverified**. In order of impact:

1. **Loi 2011-590 art. 4**: exact wording; any position on gift cards, gift wrap or shipping with an eBook; whether the old regulatory exclusions from the notion of *prime* survive.
2. **Current store terms** (Amazon.fr Kindle, Kobo, Fnac, 7switch, Vivlio, Apple, Google): wording on transfer; whether 7switch, Cultura, Apple Books and Google Play Books offer title-level gifting in France in 2026; whether store terms allow a gift link to be re-wrapped.
3. **VAT**: SPV / MPV / direct-sale treatment of download codes; buyer versus redeemer; wording of CGI art. 278-0; 2026 rates; French franchise thresholds (two conflicting figures in the sibling note); the EU SME scheme.
4. **Paragraph numbers and quotations** of *Tom Kabinet* (paras 69–72), *Cyando* (operative part; whether an operator caught by the "deliberate intervention" test can still invoke the hosting exemption), DSA recital 20.
5. **CPI numbering after the 2021 ARCOM reform**; LCEN art. 6 after 2024; décret 2021-1362 retention periods.
6. **French decisions cited from memory**: the 1984 "cercle de famille" definition; the *Aurélien D.* reasoning; Cass. 23 Oct 2024 (*Valve*); Cass. Feb 2025 (*Dstorage*); SNE blocking orders after 2022.
7. **Withdrawal**: whether third-party redemption satisfies art. 16(m); French transposition of Directive 2023/2673.
8. **CJEU case on the French EUR 3 book-shipping minimum** (number recalled as C-366/24): outcome unknown.
9. **Expiry rules**: French case law on expiry clauses; the Dutch two-year rule; other Member States.
10. **Triman/Info-tri** for a small card, paper-EPR thresholds, Citeo fees, fine amounts; Commission action against the Triman; timetable of Regulation 2025/40; German implementation.
11. **Card mentions**: printer's name under the 1881 law; "document publicitaire" status.
12. **EAA in France**: competent authority, penalties, backlist deadline.
13. **Other fixed-price regimes** (AT, ES, BE, IT, NL) and BuchPrG § 2(2), § 7(4) wording.
14. **Project Gutenberg** licence text and the German blocking; Standard Ebooks CC0 wording; Gallica, Wikisource and ebooksgratuits.com reuse terms; CC 4.0 wording on format changes and technical measures.
15. **PSD3/PSR** status; Code monétaire et financier article numbers; **data-transfer** status (UK adequacy, T-553/23 appeal); the French e-invoicing calendar.
16. **Amazon gift mechanics**: when the publisher is paid; whether the link can be printed; the reported 2026 opening of EPUB/PDF downloads for DRM-free Kindle titles.

For a verification pass: a fresh web-search budget, or the hosts `www.legifrance.gouv.fr`, `eur-lex.europa.eu`, `curia.europa.eu`, `bofip.impots.gouv.fr`, `www.gesetze-im-internet.de`, `www.cnil.fr`, `www.economie.gouv.fr`, `www.citeo.com`, `www.verpackungsregister.org`, `www.amazon.fr`, `www.kobo.com`, `www.fnac.com`, `www.7switch.com`, `www.gutenberg.org`, `standardebooks.org` allowed in the sandbox.

---

## Sources

**URLs.** All were obtained by the search sessions of sibling notes in this repository and copied from those notes; none was reopened for this document.

Copyright and hosting:

- https://www.scl.org/10768-cjeu-sale-of-second-hand-e-books-via-a-website-is-a-communication-to-the-public/
- https://www.charlesrussellspeechlys.com/en/insights/expert-insights/tmt/2020/can-you-resell-an-e-book-online/
- https://legalblogs.wolterskluwer.com/copyright-blog/up-in-the-cloud-some-reflections-on-the-cjeu-judgment-in-vcast/
- https://ipkitten.blogspot.com/2017/11/the-vcast-decision-how-to-turn-private.html
- https://next.ink/news/86942-la-justice-europeenne-impose-decouplage-copie-privee-et-source-illicite.htm
- https://www.baumann-avocats.com/CPI/article-l122-5-du-code-de-la-propriete-intellectuelle.php
- https://www.doctrine.fr/l/texts/codes/LEGITEXT000006069414/sections/LEGISCTA000006179045
- https://www.memoireonline.com/12/06/297/m_l-exception-de-copie-privee-face-aux-dispositifs-techniques-de-protection-des-oeuvres10.html
- https://www.village-justice.com/articles/affaire-wizzgo-prend-tournant,4999.html
- https://merlin.obs.coe.int/article/8373
- https://www.freshfields.com/en/our-thinking/blogs/technology-quotient/the-digital-services-act-new-liability-rules-102iiyf
- https://www.taylorwessing.com/fr/interface/2022/the-eus-digital-services-act/online-intermediaries-and-illegal-content-under-the-digital-services-act
- https://burges-salmon.com/news-and-insight/legal-updates/data-protection/the-eu-digital-services-act-and-the-article-6-hosting-exemption
- https://www.springlex.eu/fr/packages/dsa/dsa-regulation/article-16/

Store gifting and code practice:

- https://www.amazon.fr/gp/help/customer/display.html?nodeId=GVWGP284MQ6ZRM59
- https://kdp.amazon.com/fr_FR/help/topic/G200652260
- https://www.idboox.com/news-livres/amazon-on-peut-enfin-offrir-un-livre-numerique-kindle/
- https://www.decitre.fr/offrir-un-ebook
- https://www.liseuses.net/offrir-des-ebooks-avec-tea/
- https://www.hugendubel.de/de/shortcut/ebooksverschenken
- https://www.heise.de/tipps-tricks/E-Books-verschenken-so-klappt-s-7476189.html
- https://www.dealabs.com/discussions/carte-cadeau-kobo-ou-fnac-2273026
- https://ebook-enligne.fr/blog/offrir-ebook-guide-cadeau-numerique/
- https://help.vivlio.com/hc/fr/articles/11334184838428-Je-souhaite-utiliser-un-code-promotionnel-ou-une-carte-cadeau
- https://www.publishersweekly.com/pw/print/20140922/64046-enthrill-partners-with-walmart-canadian-publishing-2014.html
- https://authors.bookfunnel.com/help/print-codes/
- https://bookwire.net/faq-download-codes

Fixed price, tax, costs:

- https://www.livreshebdo.fr/article/la-loi-sur-le-prix-en-vigueur
- https://www.livreshebdo.fr/article/prix-du-livre-numerique-une-loi-de-securite
- https://www.livreshebdo.fr/article/loi-sur-le-prix-du-livre-numerique-une-victoire-la-pyrrhus
- https://www.wipo.int/wipolex/en/legislation/details/13861
- https://www.wipo.int/wipolex/fr/legislation/details/18453
- https://www.senat.fr/dossierleg/ppl09-695.html
- https://entreprisma.fr/micro-entrepreneur
- https://developp.fr/micro-entreprise-plafonds/
- https://www.laposte.fr/tarifs-postaux-courrier-lettres-timbres
- https://www.toutsurmesfinances.com/argent/a/prix-du-timbre
- https://www.europe1.fr/societe/cartes-et-cheques-cadeaux-comment-loubli-de-leur-utilisation-constitue-un-business-juteux-pour-les-enseignes-4223693
- https://www.sleepingmoney.com/fr-fr/etudes/achat-revente-cartes-cadeaux-france/

**Primary references cited from memory** (no URL available; to be looked up by number):

- CJEU: C-263/18 *Tom Kabinet*; C-128/11 *UsedSoft*; C-682/18 and C-683/18 *YouTube/Cyando*; C-435/12 *ACI Adam*; C-265/16 *VCAST*; C-174/15 *VOB*; C-610/15 *Ziggo*; C-527/15 *Filmspeler*; C-160/15 *GS Media*; C-161/17 *Renckhoff*; C-666/18 *IT Development*; C-349/96 *Card Protection Plan*; C-41/04 *Levob*; C-463/16 *Stadion Amsterdam*; C-68/23 *M-GbR*; C-479/13 *Commission v France*; C-540/08 *Mediaprint*; C-184/20; C-21/23 *Lindenapotheke*; General Court T-553/23.
- EU acts: Directives 2001/29, 2009/24, 2019/790, 2006/112 (arts 30a, 30b, 44, 58, 59c, 73a, 78), 2016/1065, 2018/1713, 2011/83 (arts 6, 9, 10, 11a, 14, 16, 22), 2019/2161, 2023/2673, 2019/770, 2009/110, 2015/2366 (arts 3(b), 3(k), 37(2)), 2019/882, 2021/514, 2005/29; Regulations 282/2011 (arts 9a, 24a–24f), 2016/679, 2018/302, 2022/2065 (arts 3, 6–8, 11–17, 19, 23; recitals 14, 20), 2024/3228, 2025/40.
- France: CPI L122-2 to L122-5, L123-1, L123-4, L123-10, L132-17-1 ff., L137-1 ff., L331-5 ff., L335-2, L335-2-1, L335-3, L335-3-1, L335-3-2, L336-1, L336-2, R335-3; lois 81-766, 94-665, 2004-575, 2011-590, 2011-1898, 2014-344, 2021-1382, 2021-1901, 2023-171, 2023-305, 2024-449; loi du 29 juillet 1881 art. 3; décrets 2006-1763, 2011-1499, 2012-146, 2021-835, 2021-1362, 2023-931; arrêté du 4 avril 2023; Code de la consommation L111-1, L121-17, L121-19, L212-1, R212-1, L213-1, L221-5, L221-28, L224-25-1 ff., L612-1; CGI 256 ter, 257 ter, 267, 278-0, 278-0 bis A, 293 B; Code civil 1148, 1205, 1989, 2224; Code de commerce L123-22, R123-237, R123-238; Code de l'environnement L541-9-3, L541-9-4, L541-10-1, L541-10-13; Code pénal 121-7.
- French cases: Cass. 1re civ. 28 Feb 2006 and 19 June 2008 (*Mulholland Drive*); Cass. crim. 30 May 2006 and CA Aix 5 Sept 2007 (*Aurélien D.*); Cass. 1re civ. 23 Oct 2024 n° 23-13.738 (*Valve*); Cass. 1re civ. 5 Oct 2022 (*Entr'ouvert*); CA Paris 12 Apr 2023 and Cass. Feb 2025 (*Dstorage*); TJ Paris 25 Aug 2022 (Z-Library).
- Germany and others: BuchPrG §§ 2, 3, 5, 7; UStG § 3(13)–(15), § 12(2) Nr. 14; BGB §§ 195, 199, 307; UrhG §§ 53, 95a; VerpackG; BFSG; BGH I ZR 83/14, I ZR 18/11, I ZR 80/12 and the 2 June 2022 *uploaded* judgments; OLG München 29 U 3193/07; OLG Hamm 22 U 60/13; OLG Hamburg 10 U 5/11; LG Frankfurt 9 Feb 2018 (*S. Fischer v PGLAF*); OLG Köln 6 U 60/14; *Capitol Records v ReDigi* (2d Cir. 2018); *Hachette v Internet Archive* (2d Cir. 2024); Project Gutenberg Licence § 1.E; CC0 1.0 and CC 4.0 legal codes.

Repository documents used: `docs/api.md`; `docs/research/07-legal-covers-and-reseller-rights.md`; `notes/legal-covers-reseller.reseller.md`; `notes/market-business.market.md`; `notes/market-business.competitors-costs.md`; `notes/ereader-ux.ux.md`; `notes/ereader-ux.ecosystems.md`; `notes/printers.web2print.md`; `notes/printers.esg.md`.

---

## Implications for the implementation

Numbers marked "(proposal)" are design choices, not legal thresholds. Field and state names follow `docs/api.md` and document 07.

**Purpose, wording, repository**

1. No user-facing string, README sentence, package description, SEO text, test name or demo scenario presents the product as a way to pass on, share, lend or resell an eBook the user bought. CI runs a deny-list check (French, English, German phrases) over UI catalogues, documentation and fixtures; a hit fails the build.
2. The upload dialog shows, before the file picker, a statement that store purchases are personal licences, and offers the gift-link path as the first choice for "a book I bought".
3. Test fixtures and demo catalogues contain only works that are public domain in France (author dead for at least 71 years, no "mort pour la France" extension) or files written for the project. Each fixture has a provenance record: source, licence, author's year of death.
4. The demo stores implement entitlement flows only. The fake 7switch is an open-file store that issues a claim on a file marked for the recipient (`hosted_file` with `licensed_distributor`, or `reseller_licence`). The fake Fnac uses `reseller_redirect` into a recipient account. No demo shows a customer uploading a purchased file.
5. The repository contains no DRM-removal code, dependency, link or instruction.

**Upload gate (`hosted_file`)**

6. `X-Rights-Basis` is mandatory, has no default, and accepts only `own_work`, `public_domain`, `open_licence`, `licensed_distributor`. No value represents a purchased or private copy. `licensed_distributor` is accepted on the partner API only.
7. A hosted-file gift becomes claimable only after the uploader's e-mail is verified and a declaration record is stored: rights basis, declarant name and role, e-mail, UTC timestamp, SHA-256 of the file, version of the declaration text.
8. Refused with `drm_protected`: an EPUB whose `META-INF/encryption.xml` lists any algorithm other than the two font-obfuscation algorithms (IDPF, Adobe), or that contains `META-INF/rights.xml` or `META-INF/license.lcpl`; `.acsm` files; Kindle containers; encrypted PDFs.
9. Refused with `retail_copy_suspected`, with a pointer to the gift-link path: files whose metadata or text carries retailer purchase marks (buyer name or e-mail in an ex-libris, order number, retailer identifier). The pattern list is data, updatable without a release.
10. ISBN rule: an ISBN on the blocklist is refused. With `own_work`, an ISBN whose registrant prefix is not verified for the uploader's account puts the gift in `pending_review`; it is not claimable until approved.
11. `public_domain` requires the author's year of death, and the translator's when the file names one. Automatic acceptance only when the current year minus the latest death year is at least 71; 101 when "mort pour la France" is ticked; otherwise `pending_review`. A file containing the Project Gutenberg licence text is accepted on the free tier only.
12. `open_licence` takes a closed list. CC0, CC BY and CC BY-SA: all tiers. Any NonCommercial variant: free print-at-home only; a paid print order is rejected with `non_commercial_licence`. Licence URL and attribution text are mandatory and are shown on the gift page.
13. Delivery is byte-identical: the SHA-256 served equals the SHA-256 stored, asserted by an integration test. Nothing is added to or removed from the file. Format conversion exists only behind `allow_format_conversion` (default false), set by the uploader; a converted file keeps `dc:rights`, licence and attribution text unchanged, asserted by a test.
14. Download cap: 10 sessions per gift by default (`docs/api.md`); the uploader may lower it to 1. The gift page shows the count and the date of the last download. The file is deleted when the cap is reached, on cancellation, on takedown or at expiry.
15. No listing, search, catalogue, public profile or share button exists. Gift pages and download URLs send `noindex`. Download sessions use 128-bit random tokens valid 15 minutes (proposal). Unknown-code attempts are limited per client and by a global failure budget with alerting.

**Gift-link lane**

16. Standalone mode offers "store gift link" as a delivery type: the giver pastes the store's redemption URL or code. Accepted hosts come from an allow-list in configuration; initial entries are the stores recorded as offering title-level gifting (Amazon.fr, Decitre, Furet du Nord, Thalia, Hugendubel), each with the date its flow was last checked. Any other host is refused.
17. The link is encoded into the QR in the browser and never sent to the server; an integration test asserts that no request URL, body or log line contains it. The card prints the store's name as plain text (no logo), the store's own code in clear when there is one, and the store's country restriction.

**Notice and action**

18. A public notice form and an e-mail address collect the DSA art. 16 elements. Receipt is acknowledged automatically; the decision target is 24 hours. An upheld notice disables every token for the file, deletes the file, and adds its SHA-256 and ISBN to the blocklist checked at every upload. The uploader receives a statement of reasons with a way to contest. All steps are logged for 6 years (proposal, as in document 07).
19. An account is suspended after 2 upheld notices or 1 false declaration, and banned on recurrence.
20. No price depends on the number of downloads; uploaders receive no payment or credit.

**Selling, price law, VAT**

21. Standalone mode has no endpoint through which a price for an eBook is set or collected. The only sellable items are the card, printing, postage and hosting.
22. In reseller checkout (API and demo stores) the eBook line equals the publisher's price and is never changed by the gift option. The printed card is a separate optional line, unticked by default, with its own price and the standard VAT rate. Postage is a third line at the card's rate. The print-at-home PDF is never a line item.
23. A printed-card line priced at zero is rejected unless the reseller configuration carries `publisher_premium = true` (document 07, rule 29). A price below the configured `card_cost_floor` (excl. VAT) is rejected; the floor has no default and is set per reseller from real quotes.
24. The gift option is always offered without the paid card.
25. No code path in the demo stores applies a coupon or loyalty credit to the eBook line; the partner API documentation states the rule.
26. VAT rates, postage tariffs and thresholds are dated configuration records (`valid_from`), not constants. Shipped defaults: FR eBook 5.5 %, FR standard 20 %, DE eBook 7 %, DE standard 19 %, each flagged "unverified, 2026-11".
27. Receipts show each line with its rate, amount excluding VAT and VAT, rounded per line to the cent. The worked examples of §6.6 are unit tests (15.01 → 13.66 + 1.35; 15.74 → 14.27 + 1.47, 14.30 + 1.44, 14.17 + 1.57).
28. Every code carries `redemption_territory` (ISO country list, default `["FR"]`) and a derived `vat_treatment`: `spv` when the list has one country, `mpv` otherwise (working rule until the VAT adviser answers question 21; the reseller can override it). The territory is printed on the card and shown on the gift page. For `reseller_redirect` and `reseller_licence` the partner enforces it by account country and returns `redeemer_country` in the fulfilment callback; an `mpv` code cannot be marked fulfilled without it.
29. A print order whose delivery country is outside `redemption_territory` requires an explicit confirmation by the buyer ("the recipient will need an account in …") and is logged.

**Code lifecycle and consumer law**

30. States: `created → redeemed → fulfilled`, plus `cancelled` (reason `withdrawal`, `refund`, `fraud`, `takedown` or `reissued`) and `expired`. Cancellation for `withdrawal` returns `409` once the code is redeemed. Reissue creates a new code for the same order and cancels the old one atomically. Every transition is timestamped and sent to the partner webhook.
31. There is no default expiry. An `expires_at` earlier than 5 years after issue is rejected for any gift attached to a paid eBook or a paid card. The card prints the expiry date or "sans date limite".
32. Partner gift creation for a consumer sale carries `withdrawal_waiver {consented, at, text_version}`. When `consented` is false, the gift gets `redeemable_from` = purchase time + 14 days, and the gift page says when it opens.
33. The gift page and the card state, before the claim action, the format, the protection (none, watermark, LCP, Adobe DRM) and device compatibility in one line.
34. A support case can be opened with the code alone. The terms templates include the bearer clause of §7.2.
35. The card never shows a price.

**Personal data**

36. Giver name, recipient name and message are optional. A "print only" option keeps them in the browser-generated PDF and off the server. When stored, they are deleted 12 months after the first claim, or immediately on cancellation (proposal).
37. A print-at-home card in the gift-link lane sends no personal data to the server.
38. For postal orders the recipient's name and address are stored encrypted, sent only to the selected printer, and deleted from the service's database 90 days after dispatch (proposal); the printer contract requires deletion within 30 days. The Art. 14 notice is a mandatory template slot on postal products.
39. The redemption record holds the code hash, timestamp, device class and country. Raw IP address and user agent stay in security logs for at most 6 months (proposal), then are dropped. Proof-of-supply records are kept 5 years; for `mpv` codes the country evidence is kept 10 years, without the IP address.
40. The giver is not told whether or when the gift was claimed unless the recipient opts in on the gift page; the default is off.
41. Gift pages load no third-party resource (`Content-Security-Policy: default-src 'self'`), set no cookie and run no analytics.
42. `device_email` (Send to Kindle) is used for the one sending and is not stored.
43. No feature profiles users or recommends titles on the basis of what was redeemed.
44. The printer registry records each printer's production country; printers outside the EEA are disabled by default. In reseller mode a printer is used only if it is listed as an authorised sub-processor in that reseller's configuration.

**Accessibility**

45. Gift pages meet WCAG 2.1 AA: they work without JavaScript, every input is labelled, text contrast is at least 4.5:1, errors are stated in text, and there is no CAPTCHA (rate limiting instead; the `429` page states the waiting time). CI runs an automated accessibility check on every gift-page route with zero AA violations allowed.
46. Every card that carries a QR code also carries the claim URL in clear and the typed code, grouped `XXXXX-XXXXX`, in a monospaced face of at least 12 pt with contrast of at least 7:1 (proposal). Preflight fails a card lacking either text.
47. The code input accepts upper and lower case, spaces and hyphens, and maps look-alike characters as Crockford base32 specifies.
48. When the title's accessibility metadata is supplied, it is shown on the gift page.

**Printed card: mentions and eco-labelling**

49. Mandatory template slots for professional print: legal line (what the card gives, single use, validity, seller identity and company identifiers), format and protection line, support contact, redemption territory, printer's name and city, eco-label. Preflight fails a professional-print PDF with an empty mandatory slot. Print-at-home requires the first four.
50. The eco-label slot is filled from a table keyed by destination country and `valid_from` (France: Triman and Info-tri artwork for paper; other countries empty until specified). The artwork is a replaceable asset.
51. Card text is in French for delivery in France; other languages may be added, not substituted.
52. Templates contain none of the restricted environmental claims ("biodégradable", "respectueux de l'environnement", "écologique", "neutre en carbone"); a lint enforces it.
53. The issuer record (eBook Cadeau or the reseller) holds the French EPR unique identifier and, where envelopes are sent to Germany, the LUCID registration number. A postal order to Germany that uses an envelope is refused when the LUCID number is missing; products flagged `packaging = none` (sealed self-mailer) skip this check.

