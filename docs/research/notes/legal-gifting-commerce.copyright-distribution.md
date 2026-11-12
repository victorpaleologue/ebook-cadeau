# Legal — can an eBook be given? The "file upload → QR download" scheme, exhaustion, DRM, private copy, hosting status, and the lawful configurations

Research notes, 2026-11-12. Angle: copyright and distribution law of the gifting mechanism itself (covers, fonts and the detail of price law / VAT are in companion notes). **Not legal advice** — a map for a French IP lawyer to confirm.

**Method caveat — read first.** No web search could be run for this note: the workflow's shared search budget (200 calls) was already used up when this task started; all 5 queries I sent were refused. WebFetch is blocked by the sandbox allowlist (tried: `eur-lex.europa.eu`, `en.wikipedia.org`, `www.gutenberg.org`, `standardebooks.org`). Consequently **nothing here is [V1] or [V2] by my own searches.** Tags used:

- **[M]** = from memory, not verified in this session. Case numbers, dates and article numbers are given so they can be checked on Curia / EUR-Lex / Légifrance; no URL is given for them because none appeared in a search result.
- **[S2] / [S1]** = fact and URL taken from a sibling note in this repo (`legal-covers-reseller.*.md`, `market-business.*.md`), where the agent that ran the search tagged it V2 / V1. Not re-checked by me.

Everything in §9 is my own recommendation, not sourced.

---

## 0. Bottom line

1. **A private buyer cannot lawfully "give" an eBook bought in a store by uploading the file for someone else.** There is no exhaustion for eBooks (Tom Kabinet), the upload and the download are unauthorised reproductions, the store licence is personal and non-transferable, and the French private-copy exception does not reach a copy made for another person from a source that is not lawful.
2. With DRM the scheme additionally requires circumvention (offence for the user; a heavier offence for anyone supplying the means). With a watermark (*tatouage numérique*) the file stays traceable to the original buyer.
3. A service **whose stated purpose is passing on purchased eBooks** would not be a neutral host: DSA recital 20 and YouTube/Cyando remove the safe harbour for services offered to facilitate infringement, and French criminal law has a specific offence for tools "manifestement destinés" to unauthorised sharing (CPI L335-2-1).
4. Four configurations are lawful: (1) the **reseller sells a gift entitlement** and licenses the recipient on redemption; (2) **rights-holders** distribute their own files; (3) **public-domain / openly licensed** works; (4) the card carries a **store code or gift link**, not a file.
5. Standalone mode must therefore drop "upload the eBook you bought" as a use case, and replace it for commercial titles with "paste the gift link / code the store gave you". File hosting stays, restricted to (2) and (3), behind declarations, automated checks, notice-and-action and a strike policy.
6. Keeping delivery **one token → one recipient, unlisted, unindexed** matters legally: it keeps the service a plain "hosting service" under the DSA (not an "online platform") and outside art. 17 of the DSM Directive.

---

## 1. The scheme, act by act

| # | Act | Who | Right engaged | Defence? |
|---|---|---|---|---|
| 1 | Remove DRM to obtain a transferable file (if any) | Buyer | Protection of technical measures: InfoSoc art. 6; CPI L331-5, L335-3-1 | None for this purpose [M] |
| 2 | Upload: a copy is fixed on the service's server | Buyer triggers; service runs the equipment | Reproduction: InfoSoc art. 2; CPI L122-3, L122-4 | Private copy fails: not "usage privé du copiste"; the service is not a natural person (§5) |
| 3 | File stored and offered behind the QR / code | Service | Reproduction; making available (InfoSoc art. 3; CPI L122-2) if a "public" exists (§7.1) | DSA art. 6 hosting exemption — conditional, and only an exemption from liability (§7) |
| 4 | Recipient downloads: new copy on the device | Recipient | Reproduction | Private copy fails: source not lawful (§5) |
| 5 | Buyer deletes own copy ("forward and delete") | Buyer | — | Irrelevant: no exhaustion to trigger (§2) |
| 6 | Breach of the store licence | Buyer | Contract; a licence breach touching IP rights can also be sued as *contrefaçon* (CJEU IT Development v Free Mobile, C-666/18, 18 Dec 2019; Cass. 1re civ., 5 Oct 2022, Entr'ouvert v Orange) [M] | — |

*Contrefaçon* is civil and criminal: CPI L335-2 / L335-3, up to 3 years and EUR 300,000 [M].

---

## 2. No digital exhaustion

**Tom Kabinet** — CJEU (Grand Chamber), 19 Dec 2019, C-263/18, *Nederlands Uitgeversverbond and Groep Algemene Uitgevers v Tom Kabinet Internet BV*, ECLI:EU:C:2019:1111.

- Holding: supplying an eBook to the public by download, for permanent use, is **communication to the public** (more precisely "making available") under art. 3(1) of Directive 2001/29, not distribution under art. 4(1). The exhaustion rule of art. 4(2) therefore does not apply — [SCL](https://www.scl.org/10768-cjeu-sale-of-second-hand-e-books-via-a-website-is-a-communication-to-the-public/), [Charles Russell Speechlys](https://www.charlesrussellspeechlys.com/en/insights/expert-insights/tmt/2020/can-you-resell-an-e-book-online/) [S2].
- Reasoning [M]: (i) the WIPO Copyright Treaty and its agreed statements confine "distribution" to tangible copies; (ii) recitals 28–29 of the Directive: exhaustion "does not arise in the case of services and on-line services in particular"; art. 3(3): the communication right is never exhausted; (iii) economics: a digital copy does not deteriorate, so a "second-hand" copy is a perfect substitute for a new one; (iv) **new public**: the user licence that normally accompanies an eBook authorises reading only by the user who downloaded it, on his or her own equipment, so anyone else is a public the right-holder did not take into account (paras 69–72 as I recall them).
- The defendant's "reading club" and its request that sellers delete their copy changed nothing.

**UsedSoft v Oracle** — CJEU (Grand Chamber), 3 July 2012, C-128/11, ECLI:EU:C:2012:407 [M]. Exhaustion applies to a downloaded copy of a **computer program** sold with a perpetual licence, provided the seller makes his own copy unusable. Tom Kabinet distinguishes it: Directive 2009/24 is *lex specialis* and treats tangible and intangible copies of software alike; an eBook is not a computer program.

Corroboration and comparison [M]:

| Jurisdiction | Decision | Point |
|---|---|---|
| France | Cass. 1re civ., 23 Oct 2024, n° 23-13.738, *UFC-Que Choisir v Valve* (confirming CA Paris, 21 Oct 2022; TGI Paris, 17 Sept 2019 had held the opposite) | No exhaustion for dematerialised video games; non-resale clause not struck down. France's highest court follows the Tom Kabinet line for non-software content |
| France | CSPLA report on "la seconde vie des biens culturels numériques" (2015) | Official position against a second-hand digital market |
| Germany | OLG Hamm, 15 May 2014, 22 U 60/13 (audiobooks); OLG Hamburg, 10 U 5/11 (eBooks) | Store terms forbidding resale / transfer of downloads upheld |
| USA | *Capitol Records v ReDigi*, 2d Cir. 2018 | Transfer of a digital file is a reproduction; first sale no defence |
| EU (lending) | *VOB v Stichting Leenrecht*, C-174/15, 10 Nov 2016 | "One copy, one user" e-lending is possible only for public libraries under the lending derogation, from a lawful source — no help to private persons |

Consequence: "giving", "lending", "reselling" and "swapping" a purchased eBook file are all the same legally — each needs the right-holder's authorisation. A gift can only be *built* by the party entitled to grant a licence.

---

## 3. What store licences say about transfer and gifting

Current terms could not be opened. The table gives the wording as I remember it; re-read each before relying on it.

| Store | Licence wording on transfer | Own gifting function |
|---|---|---|
| Amazon Kindle | Kindle Store Terms of Use: content is "licensed, not sold"; the user may not "sell, rent, lease, distribute, broadcast, sublicense, or otherwise assign any rights" to the content, nor circumvent DRM [M]. Since 2025 the US buy button itself speaks of purchasing a licence [M]. "Download & transfer via USB" was withdrawn in Feb 2025 [M] | Yes: "Acheter pour d'autres personnes" — buyer receives prepaid redemption links or enters the recipient's e-mail; recipient must be in the same country; unredeemed copies can be reassigned or refunded — [Amazon.fr help](https://www.amazon.fr/gp/help/customer/display.html?nodeId=GVWGP284MQ6ZRM59), [KDP help](https://kdp.amazon.com/fr_FR/help/topic/G200652260), [IDBoox](https://www.idboox.com/news-livres/amazon-on-peut-enfin-offrir-un-livre-numerique-kindle/) [S2] |
| Kobo / Kobo by Fnac | Limited, non-exclusive, non-transferable, revocable licence for personal non-commercial use [M] | No title-level gift found; users fall back on gift cards — [Dealabs](https://www.dealabs.com/discussions/carte-cadeau-kobo-ou-fnac-2273026), [ebook-enligne.fr](https://ebook-enligne.fr/blog/offrir-ebook-guide-cadeau-numerique/) [S1] |
| Decitre / Furet du Nord (Vivlio) | Not retrieved | Yes: "Offrir un ebook", gift e-mail on a chosen date — [Decitre](https://www.decitre.fr/offrir-un-ebook), [Liseuses.net](https://www.liseuses.net/offrir-des-ebooks-avec-tea/) [S2] |
| Thalia / Hugendubel (tolino) | "Einfaches, nicht übertragbares Nutzungsrecht" is the usual German wording [M] | Yes: "Verschenken" — [Hugendubel](https://www.hugendubel.de/de/shortcut/ebooksverschenken), [heise](https://www.heise.de/tipps-tricks/E-Books-verschenken-so-klappt-s-7476189.html) [S2] |
| Apple Books, Google Play Books | Personal, non-commercial use rules [M] | E-mail gifting of a specific book in some countries; redeemable only in the country of purchase [M] |
| 7switch (Immatériel) | Not retrieved. Sells many titles with watermark or without protection, shown per title [M] | Not verified (I have an unconfirmed recollection of an "offrir" option) |

Pattern: every store that allows a gift does it by **selling a fresh entitlement redeemed into the recipient's account**. None lets a customer move a copy already in their library. Sanctioned sharing exists only as household features (Amazon Family Library, Apple Family Sharing) [M].

---

## 4. DRM and watermark

**Legal protection of technical measures** [M — re-read on Légifrance; the 2021 ARCOM reform renumbered part of this chapter]

| Rule | Content |
|---|---|
| InfoSoc art. 6 | Member States must protect effective technological measures against circumvention and against trafficking in circumvention tools. Art. 6(4), 4th subpara.: the duty to let users benefit from exceptions does **not** apply to works made available on demand on agreed contractual terms — i.e. to eBook stores |
| CPI L331-5 | Effective technical measures are protected; they must not block interoperability |
| CPI L335-3-1 | I: knowingly circumventing a technical measure, otherwise than for research: fine of **EUR 3,750**. II: procuring or offering means designed or specially adapted to circumvent: **6 months and EUR 30,000** |
| CPI R335-3 (décret 2006-1763) | Holding or using a circumvention tool for personal use: 4th-class *contravention* (max EUR 750) |
| InfoSoc art. 7; CPI L331-11, L335-3-2 | Removing or altering electronic rights-management information, or distributing works from which it was removed: EUR 3,750; supplying means: 6 months and EUR 30,000 |

Implications:

- A DRM-protected file (Adobe ADEPT, Kindle, Kobo, Apple FairPlay, Readium LCP) is useless to a recipient unless the DRM is removed or account credentials are shared. The service must never host, link to or describe DeDRM tools: that is the L335-3-1 II offence, and it is "providing tools specifically intended for illegal sharing" in the Cyando sense (§7).
- Technical aside [M]: an Adobe `.acsm` or an LCP `.lcpl` is a licence token bound at fulfilment. Issued **by the reseller for the recipient**, it is the lawful delivery vehicle (architecture: `Delivery::ResellerLicence`). Passed on by a buyer without the store's consent, it involves no circumvention but still breaches the personal licence.
- **Watermark (*tatouage*)** is not an access-control measure, so copying a watermarked file is not circumvention. But: (i) the file carries the original buyer's identifiers (visible ex-libris and/or hidden marks); if the recipient's copy leaks, the trail ends at the **buyer**; (ii) stripping or altering it can fall under L335-3-2 where the mark carries licence or rights information — whether a purely buyer-identifying mark is "rights-management information" is debated [M]; (iii) the file contains personal data of the buyer (GDPR). Design rule: deliver files byte-identical; never "clean", re-package or optimise an uploaded EPUB.

---

## 5. Private copy and the "cercle de famille"

- **Two different exceptions.** CPI L122-5 **1°** covers "représentations privées et gratuites effectuées exclusivement dans un **cercle de famille**" — performances, not copies. L122-5 **2°** covers copies "réalisées à partir d'une **source licite** et strictement réservées à l'**usage privé du copiste** et non destinées à une utilisation collective" — text per [Baumann](https://www.baumann-avocats.com/CPI/article-l122-5-du-code-de-la-propriete-intellectuelle.php), [Doctrine](https://www.doctrine.fr/l/texts/codes/LEGITEXT000006069414/sections/LEGISCTA000006179045) [S2]. The "cercle de famille" is therefore not the test for copies; it is sometimes borrowed by analogy to describe tolerated private use.
- **Cercle de famille is narrow**: relatives or very close friends "unis de façon habituelle par des liens familiaux ou d'intimité" (T. corr. Paris, 24 Jan 1984, commonly quoted) [M].
- **A copy made for someone else is not "l'usage privé du copiste".** In the *Aurélien D.* litigation (CA Montpellier, 10 Mar 2005; Cass. crim., 30 May 2006; on remand CA Aix-en-Provence, 5 Sept 2007) the final conviction relied, among other things, on copies lent to friends exceeding private use [M]. I know of no decision treating a copy handed to a friend as covered.
- **Lawful source.** Added to L122-5 2° by loi n° 2011-1898 of 20 Dec 2011; at EU level, *ACI Adam* (C-435/12, 10 Apr 2014) excludes copies from unlawful sources — [Next](https://next.ink/news/86942-la-justice-europeenne-impose-decouplage-copie-privee-et-source-illicite.htm) [S1]. The recipient's download is made from a server copy that nobody authorised.
- **Who is the copyist?** Under the Rannou-Graphie line the party operating the copying equipment for customers is the copyist — [thesis summary](https://www.memoireonline.com/12/06/297/m_l-exception-de-copie-privee-face-aux-dispositifs-techniques-de-protection-des-oeuvres10.html) [S1]; the online recorder Wizzgo could not shelter behind its users — [Village de la justice](https://www.village-justice.com/articles/affaire-wizzgo-prend-tournant,4999.html) [S1]. CJEU *VCAST* (C-265/16): a service may assist private copying only if it does not itself give access to the work — [Kluwer](https://legalblogs.wolterskluwer.com/copyright-blog/up-in-the-cloud-some-reflections-on-the-cjeu-judgment-in-vcast/), [IPKat](https://ipkitten.blogspot.com/2017/11/the-vcast-decision-how-to-turn-private.html) [S2].
- **Not a right.** *Mulholland Drive* (Cass. 1re civ., 28 Feb 2006 and 19 June 2008): private copy is an exception that cannot be asserted against DRM, and it is subject to the three-step test (now the last paragraphs of L122-5) [M]. Giving away a substitute for a sale conflicts with normal exploitation.
- Germany is more generous on paper (§ 53(1) UrhG allows copies for the close personal circle), but § 53(4)(b) requires consent for an essentially complete copy of a book, and contract plus § 95a (DRM) apply; whether § 53(4)(b) covers eBooks is not settled as far as I know [M].

Answer to "does giving a copy to a friend qualify?": **no in France** (wrong beneficiary, unlawful source for the recipient, three-step test, contract), and not safely anywhere in the EU.

---

## 6. Enforcement context: HADOPI → ARCOM, and the book trade

- HADOPI (lois 2009-669 and 2009-1311) ran the *réponse graduée* against subscribers whose connection was used on **peer-to-peer** networks ("négligence caractérisée", 5th-class *contravention*). It merged with the CSA into **ARCOM** on 1 Jan 2022 (loi n° 2021-1382 of 25 Oct 2021) [M]. The graduated response does not reach direct download or a private transfer service; an individual gifter is not exposed through it.
- ARCOM also regulates technical protection measures (successor of the ARMT), keeps lists of infringing services, handles mirror-site blocking, and is France's **Digital Services Coordinator** under the DSA (loi "SREN" n° 2024-449 of 21 May 2024) [M].
- The book trade enforces collectively: the SNE obtained ISP blocking of some 200 Z-Library domains (TJ Paris, 25 Aug 2022) on the basis of CPI L336-2, with later extensions reported [M]. A service perceived as an eBook-passing tool should expect the same route — an L336-2 injunction can target any intermediary, exempt from liability or not.
- Realistic exposure: for the individual, low probability but traceability through the watermark; for the service, notices, injunctions and — decisive commercially — refusal by publishers, e-distributors and resellers to work with it. The standalone mode's reputation conditions the reseller-integrated mode.

---

## 7. Status of the service: host, communicator, or accomplice

### 7.1 Does the service itself communicate to the public?

**YouTube / Cyando** — CJEU (Grand Chamber), 22 June 2021, joined cases C-682/18 and C-683/18, ECLI:EU:C:2021:503 [M]. Cyando ran the file-locker "Uploaded". The operator of a sharing or file-hosting platform does **not** itself communicate to the public unless it contributes, beyond merely providing the platform, to giving access in breach of copyright. Indicators:

1. specific knowledge of an infringing file and no expeditious removal;
2. knowing "in a general sense" that users infringe, and not deploying the technological measures expected of a reasonably diligent operator;
3. taking part in selecting content, **providing tools specifically intended for illegal sharing**, or **knowingly promoting such sharing** — which a financial model encouraging it may show.

Applied to a service marketed as "give an eBook you bought": indicator 3 is met by the product description itself and indicator 2 by the absence of a rights check. The earlier line points the same way: *Ziggo / The Pirate Bay* (C-610/15), *Filmspeler* (C-527/15), *GS Media* (C-160/15) [M]; reposting a freely accessible work is a new communication (*Renckhoff*, C-161/17 — [IRIS Merlin](https://merlin.obs.coe.int/article/8373) [S2]).

The "public" requirement is the one real argument for a one-recipient design: one buyer sending to one named friend is not an indeterminate, fairly large group. It is weak for the *service*: Tom Kabinet counts persons who access the same work successively, the service is open to anyone, and the reproduction right (server copy, recipient copy) has no "public" threshold at all. Untested — do not build on it.

### 7.2 Hosting exemption (DSA) and LCEN

| Rule | Content | Source |
|---|---|---|
| DSA art. 6 | No liability for stored content without actual knowledge, or if removed expeditiously once aware; lost where the provider plays an active role giving knowledge or control; injunctions remain possible | [Freshfields](https://www.freshfields.com/en/our-thinking/blogs/technology-quotient/the-digital-services-act-new-liability-rules-102iiyf), [Taylor Wessing](https://www.taylorwessing.com/fr/interface/2022/the-eus-digital-services-act/online-intermediaries-and-illegal-content-under-the-digital-services-act) [S2] |
| DSA recital 20 | No exemption where the provider "deliberately collaborates" with users, e.g. offers the service "with the main purpose of facilitating illegal activities", including by making that purpose explicit | [M] |
| DSA art. 7, art. 8 | Voluntary own-initiative checks do not forfeit the exemption; no general monitoring obligation | [M] |
| DSA art. 16 | Every host needs an electronic notice mechanism; a precise, substantiated notice gives actual knowledge; timely, diligent, objective decisions | [Burges Salmon](https://burges-salmon.com/news-and-insight/legal-updates/data-protection/the-eu-digital-services-act-and-the-article-6-hosting-exemption), [art. 16 text](https://www.springlex.eu/fr/packages/dsa/dsa-regulation/article-16/) [S2] |
| DSA arts 11–15, 17 | Contact points, terms describing moderation, statement of reasons to the affected user; transparency report not required of micro/small enterprises | [M] |
| DSA art. 3(i), 3(k), recital 14; art. 19 | "Online platform" = host that **disseminates to the public** (potentially unlimited number of third parties). A one-token transfer is not dissemination to the public, so only the basic hosting duties apply; micro/small enterprises are in any case exempt from the platform section | [M] |
| DSM Directive 2019/790 art. 2(6), art. 17 (CPI L137-1 ff.) | Direct liability of services whose main purpose is to give **the public** access to large amounts of user-uploaded works, organised and promoted for profit. No public access → not such a service | [M] |
| LCEN (loi 2004-575) art. 6 | Aligned on the DSA by the 2024 SREN law; duty to keep data identifying content providers (décret 2021-1362); "manifestement illicite" standard from Cons. const. 2004-496 DC | [M] |

File-locker case law [M]: in France, *Nintendo v Dstorage (1fichier)* — CA Paris, 12 Apr 2023, upheld by the Cour de cassation in Feb 2025: a host that leaves notified content online loses the exemption; a proper notice suffices, no court order needed. In Germany, BGH *Alone in the Dark* (2012) and *File-Hosting-Dienst* (2013) (RapidShare: heightened duties when the business model invites infringement), then the *uploaded* judgments of 2 June 2022 applying Cyando.

**Why the exemption is fragile here.** The service builds a card from the title, author and ISBN, so it has specific knowledge of *which work* each stored file is. For an in-commerce title uploaded by someone who is not the publisher, it "ought to know" what is going on. That is an active role with knowledge, not passive storage.

### 7.3 French criminal add-ons [M]

- CPI **L335-2-1**: 3 years and EUR 300,000 for knowingly publishing or making available software "manifestement destiné à la mise à disposition du public non autorisée d'œuvres", or inciting its use, advertising included. Relevant to both the hosted service and the open-source repository: purpose statements, README and marketing copy are evidence.
- Complicity by supplying means (Code pénal art. 121-7); civil measures against such software (CPI L336-1) and injunctions against any intermediary (L336-2).
- Moving the transfer into the browser (WASM / WebRTC, no server copy) does not cure the scheme: the recipient's copy is still unauthorised, and these provisions target the tool and its publisher, not only a host. (My assessment.)

---

## 8. The lawful configurations

### 8.1 The reseller sells a gift entitlement (recommended for all commercial titles)

- **Structure**: the buyer pays the reseller; the reseller issues a single-use entitlement (code / link); on redemption the content is licensed to the **recipient's** account, with DRM or watermark personalised to the recipient. The buyer never holds a copy. This is the first, authorised making-available — no exhaustion question arises.
- **Contract analysis (mine)**: a sale with a *stipulation pour autrui* (Code civil arts 1205 ff.): the buyer obtains the reseller's promise to license a beneficiary; revocable until acceptance — which matches Amazon's "reassign or refund if unredeemed" [S2]. The recipient accepts the store terms at redemption.
- **Publisher authorisation**: the reseller sells under a *mandat* or distribution licence, so the gift mechanism must fit it. Points to check: who the "end user" is; when the sale is reported and royalties fall due (purchase vs redemption — Amazon's rule not verified); territory tested on buyer and recipient; title withdrawn before redemption; price changes between purchase and redemption. Under loi n° 2011-590 the publisher's price binds every seller to buyers in France — [Livres Hebdo](https://www.livreshebdo.fr/article/la-loi-sur-le-prix-en-vigueur), [WIPO Lex](https://www.wipo.int/wipolex/en/legislation/details/13861) [S2] — so the gift is sold at the fixed price and the card is invoiced separately (companion note: *prime* risk).
- **Precedent**: Enthrill's title-specific cards ran on publisher agreements — [Publishers Weekly](https://www.publishersweekly.com/pw/print/20140922/64046-enthrill-partners-with-walmart-canadian-publishing-2014.html) [S2].
- **eBook Cadeau's role**: issue/print the code and route redemption (`ResellerRedirect`, `ResellerLicence`). It never stores the book, so it performs no copyright-relevant act on it.

### 8.2 Rights-holders distributing their own files

- Self-published authors: lawful, unless bound by exclusivity (KDP Select forbids distributing the digital book elsewhere during enrolment [M]) or by a publishing contract — an author who has assigned digital rights (CPI L132-17-1 ff.) cannot hand out files without the publisher.
- Publishers: need digital rights for the territory, plus rights in translations, illustrations and embedded fonts.
- The uploader must grant the service a narrow hosting licence (reproduce and transmit to the designated recipient). If the code is *sold*, the fixed-price law and the 5.5 % VAT rate come into play (other note).
- Existing practice: BookFunnel print codes — [BookFunnel](https://authors.bookfunnel.com/help/print-codes/) [S2]; Bookwire download codes — [Bookwire](https://bookwire.net/faq-download-codes) [S1].

### 8.3 Public domain and open licences

| Source / licence | Usable? | Conditions and traps [M] |
|---|---|---|
| Public domain **in France** | Yes | Life + 70 (CPI L123-1); +30 years for authors "morts pour la France" (L123-10); translators, annotators and illustrators have their own terms; moral rights are perpetual (name, integrity); 25-year right in posthumous publications (L123-4). The cover is a separate work |
| Project Gutenberg | With care | Texts are public domain **in the USA** (works published through 1930, as of 2026), not necessarily in the EU: LG Frankfurt, 9 Feb 2018 (S. Fischer v PGLAF) led PG to block German users. "Project Gutenberg" is a registered trade mark: either remove every PG reference from the file, or follow the PG licence — full licence attached and, if any fee is charged, a **royalty of 20 % of gross profits** (licence § 1.E.8) |
| Standard Ebooks | Yes for the edition | Its own work (markup, typography, cover) is dedicated under **CC0 1.0**; underlying texts are US public domain only — same territorial check |
| Wikisource, Gallica, ebooksgratuits.com, BEQ | Case by case | Site terms may restrict commercial reuse (BnF for Gallica scans; ebooksgratuits non-commercial); BEQ follows Canadian terms |
| CC0, CC BY, CC BY-SA | Yes | Attribution (title, author, source, licence) on card or redemption page; **no DRM or other technical restriction** may be added |
| CC BY-NC(-SA/-ND) | Free print-at-home only | A paid card or paid professional printing risks being "commercial"; German courts have read NC both narrowly and broadly (LG / OLG Köln 2014). Route to manual review or refuse the paid tier |
| Internet Archive "lending" copies | No | Controlled digital lending held unlawful in *Hachette v Internet Archive* (2d Cir. 2024); not a lawful source |

### 8.4 Codes or coupons for a store, not files

- **Gift link**: the user buys the title through the store's own gift function (Amazon, Decitre/Vivlio, Thalia…) and pastes the redemption link or code; the card's QR encodes it. No file, no copyright act; handle the link client-side as a bearer secret.
- **Store credit + recommendation**: where no title-level gift exists (Kobo/Fnac [S1]), the card carries a store gift-card code and names the suggested title. Note that Vivlio gift cards do not cover eBooks — [Vivlio help](https://help.vivlio.com/hc/fr/articles/11334184838428-Je-souhaite-utiliser-un-code-promotionnel-ou-une-carte-cadeau) [S2].
- eBook Cadeau must not itself sell stored value redeemable at third parties (e-money / payment-services rules; limited-network exclusion to be checked) [M]. A 100 % coupon funded at the full public price respects the fixed-price law; a discount coupon does not [M].

---

## 9. What standalone mode must restrict or require (recommendations)

**Upload policy matrix**

| What the user has | File hosting | Offer instead |
|---|---|---|
| eBook bought in a store, with DRM | **Refuse** (detect and block) | Gift-link card (§8.4) |
| eBook bought in a store, watermark or DRM-free | **Refuse** | Gift-link card |
| Own work (author, publisher) | Accept with declaration | — |
| Public domain in France / CC0 / BY / BY-SA | Accept with licence selection | — |
| CC NC | Free tier only, or review | — |
| US-only public domain | Refuse for FR/EU recipients | Link to the source site |

**Requirements**

1. **Purpose and wording.** Never describe the service (UI, README, store listing, SEO) as a way to pass on, share, lend or resell purchased eBooks. Say the opposite in the upload dialog: store purchases are personal licences; use the store's gift function.
2. **Declaration** before each upload: a rights basis chosen from a closed list (author / publisher with digital rights / public domain in France / named open licence), identity and verified e-mail of the uploader, warranty and indemnity, and a licence to the service limited to hosting and delivery. Store the declaration with timestamp and file hash.
3. **Automated licence check** (protected by DSA art. 7):
   - reject DRM containers (`META-INF/encryption.xml` beyond font obfuscation, `rights.xml`, `license.lcpl`, Kindle/KFX DRM, `.acsm`);
   - flag retailer watermarks and ex-libris pages, and retailer identifiers in metadata;
   - read `dc:rights` and the ISBN; an ISBN belonging to an in-commerce title whose publisher is not the verified uploader blocks the upload or sends it to review;
   - for "public domain": require the author's death year; apply life + 70 and the French extensions; check translator and edition.
4. **DRM-free only; byte-identical delivery.** No conversion that would drop rights information; no DRM added to CC works; no DeDRM help anywhere.
5. **One-recipient token.** Unguessable code (≥ 128 bits behind the short typed code, rate-limited), single redemption with a small retry allowance, expiry, deletion of the file after redemption or expiry, `noindex`, no catalogue, no search, no public profile, no "share again" button.
6. **Notice-and-action** (DSA art. 16): public form and e-mail; acknowledge; decide fast (target 24 h); disable the token and delete the file; send the uploader a statement of reasons with redress; keep a log. Add the file hash and ISBN to a blocklist so the same work does not reappear (the measure Cyando expects).
7. **Repeat-infringer policy**: suspend the account after two upheld notices or one false declaration, ban on recurrence. It is a legal duty only for online platforms (DSA art. 23) and a condition of the US DMCA safe harbour (§ 512(i)) [M], but it is the evidence of diligence a court looks for.
8. **Business model**: charge for the card and printing, never per download; no reward to uploaders.
9. **Records and contacts**: single contact point, terms describing these rules, identification data kept as LCEN requires, EU hosting, GDPR notice covering uploader, recipient and any buyer data inside files.
10. **Repository hygiene** (L335-2-1): demo fixtures and tests use public-domain or own-made files; fake stores demonstrate the *reseller entitlement* flow, not customer-to-customer transfer.

---

## 10. Adjacent points (pointers only; see other notes) [M]

- **Fixed price**: loi 2011-590 binds anyone selling eBooks to buyers in France, including eBook Cadeau if it ever collects the price of a title.
- **VAT**: a code for one identified eBook is a single-purpose voucher (Directive 2016/1065): VAT at the eBook rate (5.5 % in France) is due when the code is sold; the card is a separate supply at the standard rate.
- **Consumer law**: the withdrawal exclusion for digital content (C. conso. L221-28 13°) only bites once supply has begun with express consent; an unredeemed gift code therefore stays, in principle, within the buyer's 14-day withdrawal right (my reading).
- **Digital Content Directive 2019/770** leaves copyright untouched; it gives the buyer conformity rights, not transfer rights.

---

## Sources

**Reused from sibling notes in this repository** (URLs appeared in those agents' search results; not re-checked here)

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
- https://www.livreshebdo.fr/article/la-loi-sur-le-prix-en-vigueur
- https://www.wipo.int/wipolex/en/legislation/details/13861
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

**Primary references cited from memory (no URL available in this session; look up by number)**

- CJEU: C-263/18 Tom Kabinet; C-128/11 UsedSoft; C-682/18 and C-683/18 YouTube/Cyando; C-435/12 ACI Adam; C-265/16 VCAST; C-174/15 VOB; C-610/15 Ziggo; C-527/15 Filmspeler; C-160/15 GS Media; C-161/17 Renckhoff; C-666/18 IT Development.
- EU acts: Directive 2001/29 (arts 2, 3, 4, 5(2)(b), 5(5), 6, 7; recitals 28–29); Directive 2009/24; Directive 2019/790 (arts 2(6), 17); Regulation 2022/2065 (arts 3, 6–8, 11–17, 19, 23; recitals 14, 18, 20); Directive 2016/1065; Directive 2019/770.
- France: CPI L122-2 to L122-5, L123-1, L123-4, L123-10, L132-17-1 ff., L137-1 ff., L331-5 ff., L335-2, L335-2-1, L335-3, L335-3-1, L335-3-2, L336-1, L336-2, R335-3; lois 2004-575, 2006-961, 2009-669, 2009-1311, 2011-590, 2011-1898, 2021-1382, 2024-449; décrets 2006-1763, 2021-1362; Code civil arts 1205 ff.; Code pénal art. 121-7; C. conso. L221-28.
- French cases: Cass. 1re civ. 28 Feb 2006 and 19 June 2008 (Mulholland Drive); Cass. crim. 30 May 2006 and CA Aix 5 Sept 2007 (Aurélien D.); Cass. 1re civ. 23 Oct 2024 n° 23-13.738 (Valve); Cass. 1re civ. 5 Oct 2022 (Entr'ouvert); CA Paris 12 Apr 2023 and Cass. Feb 2025 (Dstorage); TJ Paris 25 Aug 2022 (Z-Library).
- Other: OLG Hamm 22 U 60/13; OLG Hamburg 10 U 5/11; BGH I ZR 18/11, I ZR 80/12 and the 2 June 2022 *uploaded* judgments; LG Frankfurt 9 Feb 2018 (S. Fischer v PGLAF); OLG Köln 6 U 60/14; Capitol Records v ReDigi (2d Cir. 2018); Hachette v Internet Archive (2d Cir. 2024); Project Gutenberg Licence § 1.E; CC0 1.0 and CC 4.0 legal codes.

## Could not verify

Zero searches succeeded, so **every [M] item is unverified**. Priority list for a follow-up pass or for counsel:

1. **Current store terms** — Amazon Kindle Store Terms of Use (amazon.fr version), Kobo, Fnac (CGV livres numériques), 7switch, Vivlio, Apple, Google: exact wording on transfer; whether 7switch, Cultura, Apple Books and Google Play Books offer title-level gifting in France in 2026.
2. **Amazon gift mechanics**: when the publisher/author is paid (purchase or redemption), whether the gift link can be printed, exchange-for-credit rule in France. The reported 2026 opening of EPUB/PDF downloads for DRM-free Kindle titles.
3. **Paragraph numbers and quotations** of Tom Kabinet (paras 69–72), Cyando (operative part, and whether an operator caught by the "deliberate intervention" test can still invoke the hosting exemption), DSA recital 20.
4. **CPI numbering after the 2021 ARCOM reform**: L331-5 to L331-11, L335-3-1, L335-3-2, R335-3/R335-4; ARCOM's technical-measures competence; LCEN art. 6 as rewritten in 2024; retention periods of décret 2021-1362.
5. **French authorities on copies given to friends**: the 1984 "cercle de famille" definition (court and date), the Aix-en-Provence 2007 reasoning; any decision on eBooks specifically — none known.
6. **Cass. 1re civ. 23 Oct 2024 (Valve)** and **Cass. Feb 2025 (Dstorage)**: numbers, dates, exact holdings. SNE blocking orders after 2022.
7. **Whether a buyer-identifying watermark is "rights-management information"** (InfoSoc art. 7 / L335-3-2): no authority known.
8. **"Public" in a strict one-recipient transfer service**: no case found or known; my assessment only.
9. **Project Gutenberg** licence text (20 % royalty, trade-mark conditions) and the state of the German blocking; **Standard Ebooks** CC0 wording; Gallica / BnF, ebooksgratuits.com and Wikisource reuse terms; whether Readium LCP's passphrase sharing is contractually tolerated by French resellers.
10. **German points**: § 53(4)(b) UrhG and eBooks; OLG Hamm / OLG Hamburg references; OLG Köln on "NonCommercial".
11. **Payment-services / e-money analysis** for cards carrying store credit; VAT voucher treatment; fixed-price law details (art. numbers) — from memory; covered by companion notes.
12. Hosts to allow in the sandbox for a verification pass: `eur-lex.europa.eu`, `curia.europa.eu`, `www.legifrance.gouv.fr`, `www.gutenberg.org`, `standardebooks.org`, `www.amazon.fr`, `www.kobo.com`, `www.fnac.com`, `www.7switch.com`.
