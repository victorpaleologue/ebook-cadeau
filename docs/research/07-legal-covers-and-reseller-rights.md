# Legal: what an eBook reseller may provide for printing, and the risk of using covers independently

_Status: draft — research date 2026-11-12_

> **This is research to brief a lawyer, not legal advice.** It maps the questions, the authorities and a defensible default product policy so that a French IP lawyer can confirm or correct them quickly.

**Confidence markers.** [V2] = verified in at least two independent sources during the research session; [V1] = one source; [M] = from memory, not verified in this session. Risk ratings and "my analysis" passages are assessments, not sourced rulings.

**Method limits.** This document synthesises two researcher notes (`notes/legal-covers-reseller.reseller.md`, 16 searches; `notes/legal-covers-reseller.independent.md`, 15 searches). Both researchers ran out of the shared search budget before the 25-search target. For this synthesis six further queries were attempted and all were refused (budget exhausted), so **nothing new was verified here**: every point I added is tagged [M]. Primary texts (Légifrance, gesetze-im-internet.de) could not be opened. All statute article numbers must be re-read at source before any external use. Section 10 lists what remains unverified.

---

## 1. Executive summary

### 1.1 Findings

1. **A reseller owns nothing in a cover.** It holds a contractual licence limited to "promotion and sale" of the title. French law reads such licences narrowly (CPI L. 131-3 [M]) and has no general advertising exception. Loi n° 2011-590 regulates price only and is silent on covers [V1].
2. **Printing a cover on a card is a new reproduction on a new medium** (Art & Allposters, C-419/13 [V2]). Whether a store-display licence stretches to it is arguable and untested: no French decision was found.
3. **The retailer's usual comfort does not transfer to eBooks.** Dior v Evora (C-337/95 [V2]) lets a reseller advertise goods it lawfully resells, but it rests on exhaustion, and eBooks have none (Tom Kabinet, C-263/18 [V2]). For eBooks the contract is the only basis (inference, untested).
4. **Standalone use has no exception.** The EU exception list is closed [V2]; French short quotation does not cover a full image, even reduced [V2]; re-posting a freely available image is a new act (Renckhoff, C-161/17 [V2]); US fair use protects thumbnails, not a full-size printed jacket [V2].
5. **The only statutory foothold is private copying by the end user**, at home, from a lawful source, on the user's own equipment. In French doctrine the party operating the equipment is the copier (Rannou-Graphie, Wizzgo [V1]), and the exception fails if the service supplies the work (VCAST, C-265/16 [V2]). This is a legal reason, not only a performance reason, to compose in the browser.
6. **Moral rights cannot be cleared by the publisher.** Cropping, overprinting, recolouring or extending the artwork onto a spine are recognised infringements [V2]. A cover must be reproduced untouched; the 10 mm spine, the back and the QR panel must be separate panels built from our own material.
7. **Text is safe.** Title, author and publisher name retyped as plain text, in our own layout and openly licensed fonts, is low risk in every mode and every jurisdiction examined.
8. **Every documented precedent is rights-holder opt-in.** Enthrill, Walmart–Kobo, Juniper Books, Out of Print all work on publisher agreements, own artwork or public-domain material [V2/V1]. No case of a retailer unilaterally printing cover art was found, and no litigation against these businesses either.
9. **Two regulatory side-issues are open**: a free card bundled with an eBook may be a "vente à prime" under art. 4 of loi 2011-590 [M]; the card is a separate supply at the standard VAT rate, not the eBook rate [M].
10. **Largest unknowns**: the actual reseller-facing terms of Eden Livres, Immatériel/7switch, Numilog, Interforum, Hachette and Kobo/Fnac; the cover-licence terms of Dilicom/FEL and Electre; whether a gift to a third party is "usage privé du copiste".

### 1.2 Traffic-light table

Modes: **S-home** = standalone, user prints at home; **S-pro** = standalone, professional print ordered through the service; **R-B** = reseller-integrated, reseller holds only its existing "promotion and sale" licence; **R-A** = reseller-integrated, express print clause or per-title publisher opt-in.

Green = offer by default. Amber = offer only with the stated condition; counsel to confirm. Red = do not offer.

| # | What is put on the card | S-home | S-pro | R-B | R-A |
|---|---|---|---|---|---|
| 1 | Typographic card: title, author, publisher name as retyped text; own layout; open fonts; no logos | Green | Green | Green | Green |
| 2 | Front cover, unaltered, full panel | Amber: user-supplied local file, composed client-side, one card. Red if the service fetches the file or renders server-side | Red. Amber only if the orderer declares to be the rights holder (author, publisher) and warrants it | Red by policy (legal rating medium to high, §3.5) | Green towards the publisher; Amber residue for third-party artwork (§2) |
| 3 | Front cover, unaltered, reduced size on a neutral ground | Amber (as row 2) | Red (as row 2) | Amber: only if the licence covers promotion "sur tous supports" and the reseller declares so in writing | Green |
| 4 | Cover cropped, overprinted, recoloured, or extended onto the spine (including AI outpainting) | Red (not offered) | Red | Red | Red unless adaptation is expressly licensed and the artist's consent is warranted: then Amber |
| 5 | Generated spine: flat colour, title retyped in an open font | Green | Green | Green | Green |
| 6 | Generated spine imitating a collection's look or the cover's hand lettering | Red | Red | Red | Amber with express consent |
| 7 | Publisher's real spine or flat full-cover file | n/a | n/a (Green if orderer is the publisher) | Red (not supplied, not licensed) | Green scaled as supplied; Amber if re-fitted to 10 mm (adaptation consent) |
| 8 | Publisher, imprint or collection logo used separately (spine, back) | Red | Red | Red | Amber: express consent and brand guidelines |
| 9 | Back-cover blurb from the feed | Red (not fetched) | Red | Amber | Green if flagged |
| 10 | Extract of the book, author photograph | Red | Red | Red | Amber: per-title flag |
| 11 | Film/TV tie-in, licensed-character, fine-art, rights-managed covers | Red | Red | Red | Red unless cleared per title |
| 12 | Public-domain artwork (artist dead more than 70 years), source without contractual restriction, no distortion | Green | Green | Green | Green |
| 13 | Preview thumbnail in the UI, never printed | Amber: hotlinked from a source whose terms allow display | Amber | Green on the reseller's own pages | Green |
| 14 | QR code, redemption code, reseller's own logo, giver's personal message | Green | Green | Green | Green |

### 1.3 Decisions recommended

- **Default output in every mode is the typographic card.** Artwork is an upgrade that needs a recorded rights basis.
- **Printed covers in reseller mode require a per-title flag** backed by a written publisher authorisation, plus a warranty and indemnity from the reseller.
- **Standalone covers are user-supplied, client-side, print-at-home only.** The service never fetches a cover into a printable file and never sends an unlicensed cover to a printer.
- **Covers are never modified.** The spine is typeset, not derived from the artwork.

---

## 2. Who owns what on a cover

A cover is a stack of separately owned elements. The publisher is the gatekeeper, but its consent is only as good as what it holds.

| Layer | Usual owner | Typical limit and authority | Effect on the card | Conf. |
|---|---|---|---|---|
| Commissioned illustration | Illustrator; publisher is assignee | Cass. 1re civ., 2 July 2014, n° 13-24.359 (Magnard-Vuibert): a cover commission is a contract for services with an assignment of the reproduction right, not a contrat d'édition; **the publisher must prove the assignment and its terms**; there, only purchase orders and invoices existed — [Le Monde du Droit](https://www.lemondedudroit.fr/decryptages/9965-arret-magnard-vuibert-exclusion-du-regime-du-contrat-dedition-pour-la-contribution-accessoire-une-oeuvre-composite.html), [Livres Hebdo](https://www.livreshebdo.fr/article/tirer-la-couverture), [ActuaLitté](https://actualitte.com/article/46544/distribution/realiser-la-seule-couverture-d-039-un-livre-n-039-oblige-pas-au-contrat-d-039-edition) | The assignment often covers "the book and its promotion" only. A publisher's consent to a card may exceed what it holds | [V2] |
| Layout, composition (maquette) | Publisher (in-house, possibly œuvre collective, CPI L. 113-2 and L. 113-5 [M]) or freelance designer | The layout designer is also a rights holder — [Guichet du Savoir 139502](https://www.guichetdusavoir.org/question/voir/139502), [45700](https://www.guichetdusavoir.org/question/voir/45700), [Pierrat, Village de la Justice](https://www.village-justice.com/articles/tirer-couverture,54716.html) | Publisher consent usually suffices | [V2] |
| Stock image, royalty-free | Agency, contributor | Standard licences: printed books capped at 500,000 copies; **standalone products and merchandise need an Extended licence** — [Depositphotos](https://help.depositphotos.com/hc/en-us/articles/33734087656466-Image-Licensing-for-Authors-Publishers), [Adobe forum](https://community.adobe.com/t5/stock-discussions/use-of-adobe-stock-photo-on-book-e-book-cover/m-p/8940006); [ScribeCount](https://scribecount.com/author-resource/protecting-books/book-cover-rights-for-indie-authors-do-you-really-own-the-cover-you-paid-for) contradicts the cap for some vendors, so the cap is vendor-specific (terms as seen 2026-11) | Licence is personal to the publisher and not sub-licensable to a retailer [M]. A paid decorative card can be read as merchandise | [V2] |
| Rights-managed photograph (Arcangel, Trevillion, Getty RM) | Agency, photographer | Licensed per use: format, territory, print run, term | Card almost certainly outside the licence | [M] |
| Film/TV tie-in art, licensed characters | Studio, licensor | Edition-specific, time-limited; actors' image rights (C. civ. art. 9) | Exclude by default | [M] |
| Fine-art reproduction | Artist or estate, often through ADAGP | Per-use tariff | Exclude unless cleared | [M] |
| Hand-drawn title lettering | Letterer; publisher as assignee | A graphic work (CPI L. 112-2 [M]); see `notes/legal-fonts-genai.fonts.md` | Cannot be "retyped"; tracing or regenerating it is an adaptation | [M] |
| Title as text | Author, publisher | An original title is protected, and afterwards against confusing reuse for a work of the same kind (CPI L. 112-4) | Retyping the title to designate the very book is referential use | [M] |
| Blurb (texte de quatrième), extracts | Publisher, author | Supplied for store display; extract licences target online preview. Model contracts: extracts "for strictly promotional purposes" ([LDT specimen](https://ldt-editions.fr/wp-content/uploads/2023/04/SPECIMEN_-_CONTRAT_DEDITION.pdf)), up to 15 % of the book ([Librinova CGV](https://www.librinova.com/cgv-auteur)) | Not printable without consent | [V1] |
| Author photograph | Photographer; plus the author's image right (C. civ. art. 9 [M]) | — | Avoid | [M] |
| Publisher, imprint, collection logos | Publisher (registered trade marks, typically class 16 [M]) | §6 | Acceptable inside the untouched cover; not separately | [M] |
| Moral rights (name, integrity) | The author personally, then heirs | Perpetual, inalienable (CPI L. 121-1 [M]). Cropping an image to fit a cover format without express consent was treated as an integrity issue: TGI Paris, 5 Oct 2007, n° 06/05137 ([Légifrance](https://www.legifrance.gouv.fr/juri/id/JURITEXT000018862984)) [V1] | The publisher cannot waive them for the illustrator or photographer | [V1]/[M] |

Three points follow.

1. **Permission runs through the publisher, but is not exhausted by it.** Covers are protected works; authorisation is to be sought from the publisher, who knows the rights holders [V2] ([Guichet du Savoir](https://www.guichetdusavoir.org/question/voir/45700), [Pierrat](https://www.village-justice.com/articles/tirer-couverture,54716.html)). Litigation over photographs used by publishers does occur; one headline seen but not read: [ActuaLitté, À bout de souffle / Flammarion](https://actualitte.com/article/132591/droit-justice/photo-du-tournage-d-a-bout-de-souffle-la-justice-blanchit-flammarion).
2. **Lack of originality is not a safe harbour in France.** CA Versailles, 22 March 2022 (Zulma v Elior) denied copyright in the cover of "L'Embellie" for lack of demonstrated creative choices, but still found **parasitisme** for its reuse in an advertising campaign — [Fournol 1](https://www.fournol-avocat.fr/actualite/2022/8/17/la-difficile-reconnaissance-de-loriginalit-des-couvertures-de-livre), [Fournol 2](https://www.fournol-avocat.fr/actualite/2022/8/18/parasitisme-reconnu-pour-la-couverture-dun-livre) [V1]. A "plain" typographic cover is therefore not free to take (C. civ. art. 1240 [M]).
3. **The text's status says nothing about the cover.** A public-domain novel in a recent edition carries protected artwork for the artist's life plus 70 years (CPI L. 123-1 [M]); French moral rights survive that. Photographs are fully protected if they are the author's own intellectual creation (Painer, C-145/10 — [IPKat](https://ipkitten.blogspot.com/2011/12/all-photos-are-created-equal-painer.html), [Kluwer](https://legalblogs.wolterskluwer.com/copyright-blog/another-piece-of-the-puzzle-or-is-it-cjeu-on-photographs-as-copyright-works/)) [V2].

---

## 3. The reseller's rights and the "promotion of the work" scope

### 3.1 Statute: loi n° 2011-590 of 26 May 2011 (prix du livre numérique)

| Point | Source | Conf. |
|---|---|---|
| The publisher fixes the public price; it binds everyone selling to buyers in France | [Livres Hebdo](https://www.livreshebdo.fr/article/la-loi-sur-le-prix-en-vigueur), [Le Monde du Droit](https://www.lemondedudroit.fr/pi-tic/278-propriete-intellectuelle-industrielle/33565-prix-du-livre-numerique-publication-de-la-loi.html), text on [WIPO Lex](https://www.wipo.int/wipolex/en/legislation/details/13861) | [V2] |
| Before the law, publishers controlled price through **contrats de mandat** (agency) with e-retailers; the law was a "second safety net"; mandate retailers stayed under their contracts | [Livres Hebdo](https://www.livreshebdo.fr/article/prix-du-livre-numerique-une-loi-de-securite), [Livres Hebdo](https://www.livreshebdo.fr/article/loi-sur-le-prix-du-livre-numerique-une-victoire-la-pyrrhus) | [V1] |
| The publisher must take the retailer's qualitative promotion and advice services into account when setting the discount | same; [Sénat file](https://www.senat.fr/dossierleg/ppl09-695.html) | [V1] |
| Consolidated text as amended to 6 Aug 2018 on [WIPO Lex](https://www.wipo.int/wipolex/fr/legislation/details/18453); later amendments not checked | | [V1] |
| The law says **nothing** about covers, metadata or promotional material | reading of the summaries above | [V1] |
| Art. 4: a **vente à prime** with an eBook is allowed only if the *publisher* offers it simultaneously and on the same terms to all retailers | not checked on Légifrance | [M] |

Consequences:

- **A free printed card bundled with the eBook is plausibly a "prime"** [M]. Until counsel clears it, the reseller should charge for the card as a separate, separately invoiced service, or have the publisher offer it to all resellers. A card priced below cost could also be attacked as a disguised discount on the fixed price. This risk is independent of copyright.
- **VAT**: the card is a separate supply at the standard rate, not the eBook rate (5.5 % in France) [M]; a code for one identified eBook is a single-purpose voucher (Directive 2016/1065) per `notes/legal-gifting-commerce.copyright-distribution.md` [M].
- **Under a mandat** the retailer sells in the publisher's name and for its account; "le mandataire ne peut rien faire au-delà de ce qui est porté dans son mandat" (C. civ. art. 1989) [M]. An agent has less room for own-initiative promotional formats than a buyer-reseller.

### 3.2 Contract: what the upstream chain grants

The reseller-facing terms of Eden Livres, Immatériel, Numilog, Interforum and Hachette were not obtainable (not public). Published French model contracts show the upstream position:

| Source | What it says | Conf. |
|---|---|---|
| [Ex Aequo author contract (2021)](https://editions-exaequo.com/wp-content/uploads/2021/01/AAA-contrat-type-auteur-V4-du-09-01-2021-.pdf) | The publisher may mandate third parties for diffusion, promotion, distribution; any use of the cover, visuals or summaries needs the **publisher's prior consent** | [V1] |
| [SGDL model contract (Oct 2024)](https://www.sgdl.org/images/phocadownload/JURIDIQUE/Contrat_type_version_octobre2024.pdf) | Promotional elements are the publisher's responsibility; cover choice is the publisher's exclusive competence | [V1] |
| [LDT specimen contract](https://ldt-editions.fr/wp-content/uploads/2023/04/SPECIMEN_-_CONTRAT_DEDITION.pdf) | Author's prior approval for presentation, cover and promotional texts; free consultation of extracts for strictly promotional purposes earns no royalty | [V1] |
| [Librinova CGV](https://www.librinova.com/cgv-auteur) | Author allows "reasonable extracts" as promotional documentation, up to 15 % of the book | [V1] |

(A CMS article on the digital publishing contract appeared in results but was not read: [CMS](https://cms.law/fr/fra/legal-updates/le-contrat-d-edition-a-l-ere-du-numerique).)

The right to promote flows author → publisher → (e-distributor) → retailer, each step "for promotion", with the publisher as gatekeeper. Typical retailer wording in e-distribution agreements is "to reproduce and display cover art, metadata and excerpts solely to market, promote and sell the eBooks through the store" [M]. Two French rules make that narrow:

- **Strict reading of assignments.** CPI L. 131-3 requires each right assigned to be mentioned separately and the field of exploitation to be delimited as to extent, destination, place and duration; what is not granted is retained (also L. 122-7) [M]. A licence to display "on the site" does not obviously include print.
- **No general advertising exception.** InfoSoc Directive art. 5(3)(j) lets Member States allow use "for advertising the public exhibition or sale of artistic works"; France transposed only the narrow judicial-sale catalogue case (CPI L. 122-5 3° d) — [Le Monde du Droit](https://www.lemondedudroit.fr/pi-tic/278-propriete-intellectuelle-industrielle/58593-protection-par-le-droit-dauteur-des-catalogues-dune-maison-de-vente-aux-encheres-mis-en-ligne.html) [V1]. Short quotation does not cover a full image (§4.2).

**Fnac chain** [M]: Fnac's eBook catalogue is operated with Kobo; the publisher licences may sit with Rakuten Kobo, not Fnac. A sub-licence chain publisher → distributor → Kobo → Fnac → eBook Cadeau → printer must be permitted at every link.

### 3.3 Metadata feeds: ONIX, Dilicom, Electre

- ONIX 3.0 carries covers as "supporting resources" in the marketing-collateral block, described as resources "offered for promotional use by supply chain partners" — [BISG ONIX 3.0 introduction](https://es.slideshare.net/bisg/bisg-webcast-onix-for-books-v30-introduction-1787290) [V1].
- `<ContentAudience>` (List 154): code 01 "Restricted" means distribution by agreement between the exchanging parties, not to be displayed without agreement; other codes target audiences, not media — [List 158 notes](https://hanmoto.com/pub/onix/codelists/onix-codelist-158.htm), [Issue 50 changes](https://booknetcanada.atlassian.net/wiki/download/attachments/1378909/ONIX_BookProduct_Codelists_Issue_50_Changes.pdf?api=v2) [V1].
- `<ResourceFeature>` (List 160): 01 **Required credit** ("Photo Jerry Bauer", "© Magnum Photo"), 02 Caption, 03 **Copyright holder**, "indicative only, as the resource can be used without consultation" — [BIC List 160](https://home.bic-media.com/onix_info/3-0-2/codelists/onix-codelist-160.htm), [data2type](https://repository.data2type.de/ONIX/v_3.0/html/si.List160.html) [V2].
- **ONIX has no field meaning "may be printed on a physical object"**: none was found; ResourceFeature otherwise describes image attributes — [BookNet Canada](https://booknetcanada.ca/blog/2021/02/02/onix-codelists-issue-52-released/) [V1]. The licence is implicit and governed by the trading agreement. **A publisher opt-in therefore needs a proprietary feed field or a side agreement.**
- ONIX does mark media tie-in editions (List 21 edition type "MDT") [M], which can drive an automatic exclusion.
- **Dilicom**: the Distrimage service supplies FEL front-cover visuals, free for use inside bookshop management software; "pour tout autre usage (internet, extranet), il convient de prendre contact avec le service clients de DILICOM" — [SLF, 2012](https://www.syndicat-librairie.fr/le_service_distrimage_de_dilicom) [V1]. Trade-database covers are licensed per use; print is not a default use. The current [Dilicom CGV (2021)](http://dilicms.centprod.com/file/dilicom/file.do?path=/Conditions+g%C3%A9n%C3%A9rales/CGV+-+Dilicom.pdf) did not surface an image-rights clause in the extract returned.
- **Electre**: terms not found.

### 3.4 Customary tolerance

In practice "les ayants-droit s'opposent rarement à une reproduction de la couverture à un format inférieur à celui de l'ouvrage et dans un but d'information du lecteur" — [Guichet du Savoir 139502](https://www.guichetdusavoir.org/question/voir/139502) [V1]. This is tolerance, revocable at will, not a right. Its two conditions (smaller format, informational purpose) are exactly what a full-size shelf object strains. Commercial use requires asking [V2 on the principle, with [Pierrat](https://www.village-justice.com/articles/tirer-couverture,54716.html)].

### 3.5 Promotion of the sale, or separate merchandising?

| Case | Holding | Effect on the card | Conf. |
|---|---|---|---|
| **Dior v Evora**, C-337/95 (4 Nov 1997) | A reseller of lawfully marketed goods may use the mark in advertising customary in its trade unless it seriously damages the mark's reputation; the same principles apply to copyright in the packaging shown in leaflets — [CMS](https://cms.law/en/gbr/legal-updates/brands-ecj-cases-parfums-christian-dior-v-evora-bv-official-distribution-networks-parallel-imports-and-the-exhaustion-of-rights), [swarb](https://swarb.co.uk/?p=111802), [judgment](https://mansfield.bailii.org/eu/cases/EUECJ/1997/C33795.html) | The best argument for print bookshops showing covers in leaflets. But it is an exhaustion case about goods put on the market, and about advertising, not about manufacturing a new object | [V2] |
| **Tom Kabinet**, C-263/18 (19 Dec 2019) | Supplying an eBook by download is communication to the public, not distribution; **no exhaustion** — [SCL](https://www.scl.org/10768-cjeu-sale-of-second-hand-e-books-via-a-website-is-a-communication-to-the-public/), [Charles Russell Speechlys](https://www.charlesrussellspeechlys.com/en/insights/expert-insights/tmt/2020/can-you-resell-an-e-book-online/) | The Dior rationale has no exhaustion to stand on for eBooks. The reseller's advertising rights are purely contractual (inference, untested) | [V2] |
| **Art & Allposters**, C-419/13 (22 Jan 2015) | Exhaustion does not cover a work whose **medium has been altered** (poster to canvas); the new object needs fresh authorisation — [IPKat](https://ipkitten.blogspot.com/2015/01/breaking-news-cjeu-says-exhaustion-only.html?m=0), [Kluwer](https://legalblogs.wolterskluwer.com/copyright-blog/allposters-ecj-decision-no-exhaustion-of-rights-in-modifications-of-the-copyright-work/), [IPPT](https://www.ippt.eu/items/ippt20150122-cjeu-allposters-v-pictoright) | A cover licensed for screen display, printed on card stock, is a new reproduction in a new medium | [V2] |

Factors a French court would weigh (analysis; no decision found):

| Points toward "accessory of the sale" | Points toward "separate merchandising" |
|---|---|
| Exactly one card per eBook licence sold; never sold alone | Charged separately; produced by a third party for profit |
| The card is the delivery vehicle (it carries the code), like the box of a download-code product | Designed to stay on a shelf after redemption: decorative function |
| The cover identifies the product purchased | Cover at or near full size, print resolution, with a spine: it imitates the book object |
| Licence says promotion "sur tous supports" or "par tous moyens" | Licence says "on the Site/Store"; print not mentioned (L. 131-3) |

**Assessment.** With a licence limited to the online store, printing is outside scope more likely than not. With "tous supports" promotional wording it becomes defensible, more so at reduced size. With an express clause it is clean as against the publisher, though not necessarily against third-party art owners (§2). Note the tension between the first copyright factor and the prime rule (§3.1): charging separately helps on price law and hurts on "accessory of the sale".

### 3.6 Precedents

| Precedent | How rights were handled | Conf. |
|---|---|---|
| **Enthrill** (Canada, 2013–16): eBook gift cards at Walmart Canada, Sears, Safeway; 3,000+ locations in 2015 | **Publisher agreements** (HarperCollins Canada, Scholastic, Harlequin, Dundurn); publishers add titles to the catalogue and pay about 5 % of digital list price per code (min $0.25, max $1.50; 2014–15 figures); authors may print their own cards from codes — [Publishers Weekly](https://www.publishersweekly.com/pw/print/20140922/64046-enthrill-partners-with-walmart-canadian-publishing-2014.html), [The Book Designer](https://www.thebookdesigner.com/e-books-for-real-e-book-gift-cards-from-livrada-and-enthrill/), [Build Book Buzz](https://buildbookbuzz.com/e-book-gift-cards/) | [V2] |
| **Livrada** (title-specific cards at Target, 2012) | Covered with Enthrill in [The Book Designer](https://www.thebookdesigner.com/e-books-for-real-e-book-gift-cards-from-livrada-and-enthrill/); rights mechanics not retrieved | [V1]/[M] |
| **Walmart × Kobo "digital book cards"** (2018, about 40 novels) | Curated title list, implying per-title publisher participation (inference) — [Boing Boing](https://boingboing.net/2018/08/22/walmarts-selling-ebooks-now.html/amp) | [V1] |
| **Amazon book-specific Kindle cards** (2016) | Existence only — [The eBook Reader](https://blog.the-ebook-reader.com/2016/01/22/kindle-cards-new-book-specific-kindle-gift-cards) | [V1] |
| Music and audiobook download cards (Dropcards) | Ordered by the label, artist or author, i.e. the rights holder — [The Book Designer](https://www.thebookdesigner.com/?p=36235) | [V1]/[M] |
| Game "code in a box", platform POSA cards | Made or approved by the publisher or platform under brand guidelines | [M] |
| PLV, printed retailer catalogues | PLV is supplied by the publisher; chain catalogues reproduce covers within publisher-funded co-op advertising | [M] |
| **Juniper Books** (custom jackets, US) | Own original artwork; series jackets "used with permission" of publisher and author or agent; customer designs only if "the copyright for the artwork [belongs] to you or [is] public domain"; a complete Anne of Green Gables set was long blocked by copyright on two titles — [FAQ](https://juniperbooks.com/a/faq), [Custom FAQ](https://juniperbooks.com/pages/custom-faqs), [Bridgerton](https://juniperbooks.com/products/bridgerton-jackets-only), [ACOTAR](https://junipercustom.com/products/a-court-of-thorns-and-roses-5-book-jackets-only-set), [Anne](https://junipercustom.com/products/complete-anne-of-green-gables-8-book-set-jackets-only-mto) | [V1] (company's own pages) |
| **Out of Print** (cover apparel, US) | Licenses cover art from publishers, authors, estates; fees flow to them — [Axial](https://www.axial.net/company/out-of-print), [Fortune 2014](https://dc.fortune.com/2014/10/31/the-little-literary-t-shirt-company-that-could/), [Freelancers Union](https://blog.freelancersunion.org/2013/06/25/books-shirts-story-out-print/), [Cool Mom Picks](https://coolmompicks.com/blog/2010/04/07/you-can-totally-judge-me-by-a/) | [V2] |
| **LibraryThing / Goodreads** (display only) | Covers from Amazon under its terms (link-back mandatory), from members, and on a fair-use theory; bulk redistribution forbidden; Goodreads volunteers were told third-party bookseller covers could not be reused — [LibraryThing](https://blog.librarything.com/category/covers/), [1M covers](https://blog.librarything.com/2008/08/a-million-free-covers-from-librarything/), [Goodreads](https://www.goodreads.com/topic/show/1277733-source-of-book-covers?page=1) | [V1] |
| German publishers | Some grant blanket permission for review use (Rowohlt per [WBS](https://www.wbs.legal/it-und-internet-recht/verwendung-der-fotografie-eines-buchcovers-im-rahmen-einer-buchrezension-ohne-zustimmung-des-urhebers-14143/); [Gmeiner blogger FAQ](https://www.gmeiner-verlag.de/images/stories/gmeiner-bloggerinnen-faq.pdf)) | [V1] each |

Reading: every business found that prints cover art on a sold object works on licences, own artwork or public-domain material. No litigation was found against Juniper Books or Out of Print (Litographs was not searched), and none against a bookseller for displaying covers of books it sells. The disputes found concern third parties reusing a cover outside bookselling (Zulma/Elior, Moulinsart) and photographers pursuing image reuse. The absence of case law reflects tolerance of promotional display, not permission for merchandise. French precedents for physical eBook cards could not be searched.

### 3.7 What the reseller can hand to the print service

Scenarios — **A**: express publisher authorisation for the printed card. **B**: only the existing "promotion and sale" licence. **C**: no licence (cover scraped or user-supplied).

| # | Element | A | B | C | Notes |
|---|---|---|---|---|---|
| 1 | Title, author name, publisher name as **plain retyped text**; ISBN | Low | Low | Low | Facts and referential use. Do not suggest endorsement |
| 2 | **Front cover as supplied** (uncropped, unaltered, credited) | Low–Med | **Med** | **High** | A is not "Low" because of third-party art (§2). B hinges on "tous supports" wording and the prime rule |
| 3 | Front cover cropped, overprinted, or extended onto the spine | Med | **High** | **High** | Moral rights plus adaptation; the publisher cannot waive |
| 4 | Generated spine: neutral background, retyped title, own-licensed font | Low | Low–Med | Low–Med | Rises if it imitates the collection's trade dress or the cover lettering |
| 5 | **Back-cover blurb** from the feed | Low | Med | High | Protected text supplied for store display. Prefer own short text or none |
| 6 | **Publisher or collection logo** used separately (spine, back) | Med | **High** | **High** | Trade-mark use on printed matter; needs express consent |
| 7 | **Extract** printed on the card | Med | **High** | **High** | Link to the online extract through the QR instead |
| 8 | Author photograph | Med | High | High | Photographer's rights plus image rights; avoid |
| 9 | Film tie-in, licensed-character, fine-art covers | **High** unless specifically cleared | High | High | Exclude by default; the publisher flags exceptions |
| 10 | Review quotes | Low–Med | Med | Med | Fine inside the untouched cover; separate reuse needs care |
| 11 | Reseller's own logo; QR code; redemption code | Low | Low | Low | "QR Code" is a Denso Wave word mark [M] |

**Resolution of a difference between the notes.** The reseller notes rate row 2 under scenario B as "Medium" and propose a reduced-size render mode for "tous supports" licences. The independent notes recommend artwork on printed cards only when licensed for that purpose. Both are kept, in this order: the product treats scenario B as "no print rights" by default (typographic card); the reduced-size mode exists but is enabled only when the reseller declares in writing that its licence covers promotion on print media. Reason: the reseller cannot be sure of the scope without reading its own contracts, and the commercial downside (a publisher or distributor suspending the catalogue feed) is larger than the gain.

### 3.8 Contract clauses (reseller ↔ publisher or e-distributor)

**Check in the existing agreement**

1. Definition of "éléments de promotion", "métadonnées", "matériel promotionnel": are cover, blurb, extract, logos listed?
2. Purpose and **media**: "sur le Site" versus "sur tous supports / par tous moyens"; any mention of print, PLV, catalogues.
3. **Sub-licensing and subcontracting** ("prestataires", "sous-traitants"): needed for the composition service and the printers.
4. Right to **resize, crop, adapt**; credit obligations.
5. Prior-approval clause for promotional operations; under a mandat, the agent's instructions on promotion.
6. Clauses on **primes, bundles, gifts, discounts** (loi 2011-590 art. 4 [M]).
7. Trade-mark licence scope and brand guidelines.
8. Takedown delay and the publisher's own rights warranty over supplied material.
9. Territory (where printed cards ship) and term (cards outlive a title's withdrawal).

**Add (amendment or side letter)**

- Express authorisation to reproduce the front cover "sur une carte imprimée ou imprimable remise à l'acheteur d'un exemplaire numérique de l'œuvre, à raison d'une carte par exemplaire vendu", directly or through named categories of subcontractors.
- Integrity rules: no cropping or overprint; spine and inside panels composed outside the cover image; mandatory credit line.
- **Per-title opt-in or opt-out** through a feed flag; default exclusion of tie-in, licensed-character and rights-managed covers.
- Publisher warranty that it holds cover rights for this use, or a duty to flag titles where it does not.
- Whether the publisher supplies the real spine or the flat full-cover file ("couverture à plat"), and whether it may be re-fitted to a 10 mm spine.
- Logo use: yes or no, with guidelines. Blurb use: yes or no.
- Statement that the card is a paid ancillary service (or a publisher-offered premium available to all retailers), with the publisher acknowledging the pricing structure.
- Takedown within 24–48 h for new generation; sell-off rule for cards already printed.

---

## 4. Standalone use: exceptions by jurisdiction, and who is the copier

### 4.1 EU frame

- **Closed list.** Art. 5 of Directive 2001/29 is exhaustive; neither Member States nor courts can add exceptions, even on fundamental-rights grounds: Funke Medien (C-469/17, para 64), Pelham (C-476/17, para 65), Spiegel Online (C-516/17, para 49), all 29 July 2019 — [Kluwer on Spiegel Online](https://legalblogs.wolterskluwer.com/copyright-blog/reflections-on-the-cjeus-judgment-in-spiegel-online-is-there-a-golden-intersection-between-freedom-of-expression-and-eu-copyright-law-part-i/), [Kluwer on Pelham](https://legalblogs.wolterskluwer.com/copyright-blog/sample-sample-in-my-song-can-they-tell-where-you-are-from-the-pelham-judgment-part-ii/), restated in the AG opinion in C-590/23 ([FRA](https://fra.europa.eu/en/node/49752)) [V2]. There is no "it helps sell the book" defence and no EU fair use.
- **Re-posting is a new act.** Renckhoff (C-161/17, 7 Aug 2018): copying a photo freely and lawfully available on one site and posting it on another is a new communication to a new public; the absence of technical restrictions at the source is irrelevant — [IRIS Merlin](https://merlin.obs.coe.int/article/8373), [Columbia GFoE](https://globalfreedomofexpression.columbia.edu/cases/land-nordrhein-westfalen-v-dirk-renckhoff/), [Mondaq](https://www.mondaq.com/uk/copyright/730320/the-risks-of-reposting-online-content-cjeu-rules-on-the-unauthorised-reposting-of-copyright-protected-works-on-the-internet) [V2]. "The cover is all over the web" is no defence for a service that fetches and re-serves it.
- **Private copying through a service.** VCAST (C-265/16, 29 Nov 2017): a cloud service can in principle sit inside the private-copy exception, but the user must have lawful access to the source, and where the provider itself makes the work available without authorisation the exception fails — [Kluwer](https://legalblogs.wolterskluwer.com/copyright-blog/up-in-the-cloud-some-reflections-on-the-cjeu-judgment-in-vcast/), [IPKat](https://ipkitten.blogspot.com/2017/11/the-vcast-decision-how-to-turn-private.html), [LK Shields](https://lkshields.ie/news-insights/publication/eu-court-defines-boundaries-for-cloud-based-services-recorded-tv-programmes), [Quintais/Rendas](https://pure.uva.nl/ws/files/26578344/EU_copyright_law_and_the_Cloud.pdf) [V2]. **Design rule: a tool that helps the user copy something the user already has may be covered; a service that also supplies the work is not.**
- **Lawful source only.** Private-copy regimes cannot cover unlawful sources — [Next](https://next.ink/news/86942-la-justice-europeenne-impose-decouplage-copie-privee-et-source-illicite.htm) [V1]; ACI Adam, C-435/12 [M].
- **Exhaustion** frees only further distribution of the tangible copy sold; it never authorises a new reproduction (Art & Allposters [V2]) and does not exist for eBooks (Tom Kabinet [V2]).
- Art. 5(3)(j) (advertising the exhibition or sale of artistic works, "excluding any other commercial use"), 5(3)(i) (incidental inclusion), 5(3)(k) (parody) [M]: a faithful cover is not a parody (Deckmyn, C-201/13 [M]); the others are treated per country below.
- Art. 17 of the DSM Directive (content-sharing platforms) does not fit: the service does not give the public access to user uploads [M].
- Art. 5(2)(b) limits private copying to natural persons and non-commercial ends; art. 5(5) adds the three-step test [M]. Copydan (C-463/12) accepted that a private copy may be made with a device belonging to a third party [M], which supports the "tool" position.

### 4.2 France (CPI)

- **Principle.** Any reproduction without consent is unlawful (L. 122-4 [M]); contrefaçon is also a criminal offence (L. 335-2: up to 3 years and EUR 300,000 [M]).
- **Short quotation (L. 122-5 3° a) does not work for images.** "La représentation intégrale d'une œuvre, quelles que soient sa forme et sa durée, ne peut s'analyser comme une courte citation": Cass. 1re civ., 4 July 1995, Antenne 2 ([text](https://affairesjuridiques.aphp.fr/textes/cour-de-cassation-civile-1ere-4-juillet-1995-antenne-2-droits-dauteur-courte-citation-representation-integrale-de-loeuvre/)); a photograph reproduced in reduced format is still a full reproduction: Cass. 1re civ., 7 Nov 2006 ([Squire Patton Boggs](https://larevue.squirepattonboggs.com/droits-d-auteur-reproduction-d-une-photographie_a334.html)); same line in Fabris v Loudmer (1991) and the Utrillo/France 2 case ([IRIS Merlin](https://merlin.obs.coe.int/article/284)); summarised by [Livres Hebdo](https://www.livreshebdo.fr/article/citations-dartistes) [V2]. The reseller notes cite the same line as "Ass. plén., 5 Nov 1993, Fabris/Utrillo" [M]: this is the plenary decision in the Fabris v Loudmer litigation (Fabris held Utrillo's rights), so the two notes agree; the verified 1995 and 2006 decisions are kept as the primary authorities.
- **Accessory theory.** Cass. 1re civ., 12 May 2011, n° 08-20.651 ("Être et avoir"): illustrations visible in the background of a documentary were an accessory inclusion outside the monopoly — [Squire Patton Boggs](https://larevue.squirepattonboggs.com/etre-et-avoir-ou-les-auxiliaires-du-droit-d-auteur_a1468.html), [Le Monde du Droit](https://www.lemondedudroit.fr/pi-tic/278-propriete-intellectuelle-industrielle/33575-affaire-qetre-et-avoirq-de-linclusion-fortuite-dune-oeuvre.html), [IRIS Merlin](https://merlin.obs.coe.int/article/5813) [V2]. On the card the cover is the main subject of the front panel, so the theory is unavailable (application).
- **Private copy (L. 122-5 2°).** Text: copies "réalisées à partir d'une source licite et strictement réservées à l'usage privé du copiste et non destinées à une utilisation collective" — [Baumann](https://www.baumann-avocats.com/CPI/article-l122-5-du-code-de-la-propriete-intellectuelle.php), [Doctrine](https://www.doctrine.fr/l/texts/codes/LEGITEXT000006069414/sections/LEGISCTA000006179045) [V2]. Three difficulties:
  1. *Who is the copiste?* See §4.7.
  2. *"Usage privé du copiste"*: a card handed to a friend is not obviously the copier's own use. The independent notes found no case and call it unsettled. The sibling note on gifting recalls that in the Aurélien D. litigation (CA Montpellier 2005; Cass. crim., 30 May 2006; CA Aix-en-Provence, 5 Sept 2007) copies lent to friends counted against private use [M]. The point therefore leans against the user, and the tier is rated Amber, not Green.
  3. The 2° carve-out for "copies des œuvres d'art destinées à être utilisées pour des fins identiques à celles pour lesquelles l'œuvre originale a été créée" [M]: a cover reproduced to serve as a cover on a shelf is uncomfortably close.
- **No general equivalent of art. 5(3)(j)**; the only catalogue exception is for judicial-sale catalogues (L. 122-5 3° d) [V1, §3.2].
- **Licence scope is read narrowly** (L. 131-3 [M]): a reseller's right to display a cover on its product page does not by itself include printing it on a gift object.
- **Litigation climate.** Damages set at three times the evaded licence fee for a photo reused on a third-party site (TJ Bordeaux, 26 June 2025, per [Kohen Avocats](https://kohenavocats.com/concurrent-copie-site-internet-textes-photos-fiches-faire-cesser-chiffrer-2026/)) [V1]; systematic reuse of auction-catalogue content led to a contrefaçon and parasitisme judgment against Artprice ([UGGC](https://www.uggc.com/condamnation-dartprice-com-pour-contrefacon-et-concurrence-parasitaire/)) [V1]; Moulinsart obtained an infringement finding over reproduction of Tintin cover elements (Cass. 1re civ., 26 May 2011, n° 09-71.083, [Légifrance](https://www.legifrance.gouv.fr/juri/id/JURITEXT000024085051/); facts from a search summary only) [V1].

### 4.3 Germany (UrhG, [statute](https://www.gesetze-im-internet.de/urhg/BJNR012730965.html))

- **§ 51 (Zitat)**: a Bildzitat needs an inner link between image and text; a cover used as decoration next to a review does not qualify, and bloggers "in der Regel" hold no rights in the cover — [WBS 1](https://www.wbs.legal/it-und-internet-recht/verwendung-der-fotografie-eines-buchcovers-im-rahmen-einer-buchrezension-ohne-zustimmung-des-urhebers-14143/), [WBS 2](https://www.wbs.legal/internetrecht/fotorecht-bildrecht/verwendung-der-fotografie-eines-buchcovers-im-rahmen-einer-buchrezension-ohne-zustimmung-des-urhebers-32292/) [V1]. Author credit is a substantive condition of the quotation defence (LG Köln, 14 O 291/24, per [Ferner](https://www.ferner-alsdorf.de/lg-koeln-notwendigkeit-der-urheberangabe-bei-der-berufung-auf-die-zitatschranke/)) [V1].
- **§ 58 (Katalogbildfreiheit)**: lets the organiser (gallery, art dealer, auction house) reproduce works publicly exhibited or intended for public sale, as far as needed to promote the event — [Haufe](https://www.haufe.de/id/beitrag/2-urheberrecht-c-werke-in-ausstellungen-HI15189406.html), [§ 58 text](https://lexmea.de/de/gesetz/urhg/58) [V1]. It concerns selling the artwork itself. An analogous application to covers is rejected as soon as the site has a commercial purpose ([WBS](https://www.wbs.legal/internetrecht/fotorecht-bildrecht/verwendung-der-fotografie-eines-buchcovers-im-rahmen-einer-buchrezension-ohne-zustimmung-des-urhebers-32292/)) [V1]; traders who advertise with book-cover images "tragen ein gewisses rechtliches Risiko" and rely on lived practice ([IT-Recht Kanzlei](https://www.it-recht-kanzlei.de/Urheberrecht-Versto%C3%9F-Produktfotos-Produktbeschreibung-Werke.html)) [V1]. § 58 gives nothing for a printed product.
- **§ 53(1)**: single copies by a natural person for private use, not from an obviously unlawful source; they may be made by a third party if free of charge, or if on paper by photomechanical or similar processes; copies may not be distributed to the public (§ 53(6)); private use includes the close personal circle; for automated services the customer who triggers the process is the maker (BGH "Internet-Videorecorder") — all [M]. Germany is the most favourable jurisdiction for print-at-home.
- **§ 57 (unwesentliches Beiwerk)**: only for elements that could be swapped without anyone noticing (BGH "Möbelkatalog") [M]; a cover on the front panel is the opposite.
- **Enforcement**: image Abmahnungen (cease-and-desist with contractual penalty, damages, lawyer's fees) are routine — [IT-Recht Kanzlei FAQ](https://www.it-recht-kanzlei.de/abmahnung-urheberrecht-bilderklau-faq.html) [V1]. Highest practical enforcement risk of the jurisdictions examined.

### 4.4 United Kingdom (CDPA 1988)

- Fair dealing is purpose-bound. Quotation (s.30(1ZA), added 2014): work already public, fair dealing, no more than required, sufficient acknowledgement — [legislation.gov.uk](https://www.legislation.gov.uk/ukpga/1988/48/section/30), [2014 Regulations](https://legislation.gov.uk/uksi/2014/2356/regulation/3/made?view=plain), [UCL](https://blogs.ucl.ac.uk/copyright/2023/02/24/fair-dealing-week-2023-part-2-three-fair-dealing-exceptions) [V2]. It formally reaches photographs, but a whole image is unlikely to qualify; news reporting excludes photographs (s.30(2)) [V1].
- No private-copying exception: s.28B was quashed in 2015 [M]. s.63 (advertising the sale of an artistic work) and s.31 (incidental inclusion) do not fit [M]. Published editions also carry a 25-year typographical-arrangement right (s.8) [M].

### 4.5 United States (comparison)

- **Thumbnails are fair use for search and identification**: Kelly v Arriba Soft, 336 F.3d 811 (9th Cir. 2003); Perfect 10 v Amazon (9th Cir. 2007) — [opinion](https://m.openjurist.org/336/f3d/811), [CRS RL33810](https://www.everycrsreport.com/reports/RL33810.html) [V2]; library associations supported that position as amici ([EFF brief](https://eff.org/files/filenode/Perfect10_v_Google/effperfect10vgoogleamicus.pdf)) [V1]. LibraryThing states that showing covers for sale, rental or commentary is fair use "in most circumstances" ([LibraryThing blog](https://blog.librarything.com/category/covers/)) [V1] (a site's opinion, not a ruling).
- The reasoning rests on small, low-resolution images serving a different function. A full-size, print-resolution cover on a decorative object sold for money has the same expressive purpose as the original; after Warhol v Goldsmith (2023) that weighs against fair use [M].
- **First sale** (17 USC § 109) limits only the distribution right in a particular lawful copy and is never a defence to reproduction — [DOJ manual](https://www.justice.gov/usam/title9/crm01854.htm), [Copyright Alliance](https://copyrightalliance.org/?p=1446) [V2].
- **Print-on-demand intermediaries are not reliably "passive"**: the Sixth Circuit reversed a finding that Redbubble was a mere transactional intermediary (OSU v Redbubble, 2021, trade mark) — [Fross Zelnick](https://www.frosszelnick.com/u-s-court-of-appeals-for-the-sixth-circuit-redbubble-could-be-liable-for-direct-infringement-arising-from-print-on-demand-service/), [Goldman](https://blog.ericgoldman.org/archives/2021/02/ip-lawsuits-against-print-on-demand-vendors-continue-to-vex-the-courts-osu-v-redbubble-more.htm) [V2]; in Australia Redbubble was held to infringe copyright, with nominal damages (Pokémon v Redbubble [2017] FCA 1541, [WILMap](https://wilmap.stanford.edu/node/30497)) [V1]. No EU judgment on print-on-demand platforms was found.

### 4.6 Other EU Member States

Not researched. Private copying is optional under art. 5(2)(b) and its conditions differ by state (who may make the copy, whether a third party may assist, which categories of work) [M]. The harmonised baseline in §4.1 applies everywhere; before launching print-at-home with user covers in Belgium, the Netherlands, Spain or Italy, the local private-copy text must be read.

### 4.7 Who is the copier

| Architecture | Who makes the copy | Consequence |
|---|---|---|
| User picks a local file; composition runs in the browser (WASM); user prints on own printer | The user. The service supplies a tool | The private-copy argument is the user's (strong in DE, arguable in FR, unavailable in UK). Service exposure is limited to tool-supplier theories (complicité par fourniture de moyens, C. pén. art. 121-7; faute, C. civ. art. 1240 [M]), which depend on how the tool is designed and marketed |
| Same input, but the PDF is rendered on our server | France: the service. Under the Rannou-Graphie line (CA Paris, 8 Oct 1982; Cass. 1re civ., 7 March 1984) the copier is the party operating the equipment for customers ([thesis summary](https://www.memoireonline.com/12/06/297/m_l-exception-de-copie-privee-face-aux-dispositifs-techniques-de-protection-des-oeuvres10.html)) [V1]; the online recorder Wizzgo could not shelter behind its users' private copying (TGI Paris, réf., 6 Nov 2008, [Village de la Justice](https://www.village-justice.com/articles/affaire-wizzgo-prend-tournant,4999.html)) [V1] | The service is the copier and has no exception. Germany may differ for fully automated processes [M] |
| The service fetches the cover by ISBN and inserts it | The service supplies the work (VCAST) and makes a new communication (Renckhoff) [V2] | The user's exception does not protect the service |
| The service orders a print run from a printer and ships it | The service and the printer reproduce; the service distributes | No exception in the EU or UK; weak fair use in the US. Licence or rights-holder status required |

Marketing matters for the first row: the tool must be presented as a neutral composer whose default is the typographic card, not as a way to "print any book's cover".

### 4.8 Scenarios and risk matrix (exposure of the service; L = low, M = medium, H = high)

| Scenario | France | Germany | Other EU (harmonised baseline) | UK | US |
|---|---|---|---|---|---|
| 1a. Print-at-home, user-supplied cover, client-side only | L–M (copiste = user; gift and "fins identiques" doubts) | L (§ 53) | L–M (private copy is optional per state [M]) | M in theory (no exception), L in practice | L–M |
| 1b. Print-at-home, server-rendered PDF | M–H (service = copiste) | L–M | M | M | M |
| 1c. Print-at-home, cover fetched by the service | H | H (Abmahnung) | H | M–H | M |
| 2. Service prints and ships an unlicensed cover | H (civil and criminal) | H | H | H | H |
| 3a. Preview of a user-supplied file, in the browser | L | L | L | L | L |
| 3b. Preview thumbnail fetched by the service | M (tolerance only) | M–H | M | L–M | L (Kelly) |
| 4. Public-domain text, recent artwork | as 1–3 for that artwork | same | same | same (plus s.8 right [M]) | same |
| 4'. Public-domain text, artwork itself public domain | L (moral rights: no distortion) | L | L | L | L |
| 5. Typographic or user-designed card | L | L | L | L | L |
| Any scenario plus modification of the artwork (crop, overlay, generated spine continuing the art) | +1 level (droit au respect) | +1 | +1 | + | derivative-work issue |

Severity is asymmetric: per-image damages are usually modest (licence-fee multiples), but an automated service repeats the act for every title, and an injunction hits the whole feature. For scenario 3b, use only feeds whose terms allow display: Amazon data requires a link-back ([LibraryThing](https://blog.librarything.com/category/covers/)); Goodreads' own librarians were told that covers from Amazon or other booksellers could not legally be reused ([Goodreads thread](https://www.goodreads.com/topic/show/1277733-source-of-book-covers?page=1)) [V1]. Acquisition law (scraping, database right, hotlinking) is treated in `notes/cover-sources.scraping-law.md`.

---

## 5. Moral rights and alterations, including the generated spine

### 5.1 Rules

- Having permission to use a work does not dispense from respecting moral rights — [APIE / economie.gouv](https://www.economie.gouv.fr/node/40472) [V1].
- French case law treats **cropping, mirrored reproduction, colourisation and cut-outs (détourage)** as infringements; a photo may not be cropped, altered or used in a montage without the author's express consent — [Livres Hebdo](https://www.livreshebdo.fr/article/pop-et-droit-moral), [practical note](https://www.alsace.catholique.fr/wp-content/uploads/sites/14/2016/04/notions-cle-droit-image.pdf) [V2]; cropping to fit a cover format: TGI Paris, 5 Oct 2007 ([Légifrance](https://www.legifrance.gouv.fr/juri/id/JURITEXT000018862984)) [V1].
- In the "pop" case reported by Livres Hebdo, a reproduction that did not present the work in its integrity, omitted the signature and changed the format cost **EUR 15,000** in damages — [Livres Hebdo](https://www.livreshebdo.fr/article/pop-et-droit-moral) [V1].
- Not automatic: in a Bordeaux judgment of 29 Feb 2024 claimants invoking cropping and a missing credit lost on all claims (grounds not visible) — [Dalloz](https://www.dalloz-actualite.fr/document/tj-bordeaux-29-fevr-2024-n-2308086) [V1]; courts sanction "abus notoire" of post-mortem moral rights — [Lextenso](https://www.labase-lextenso.fr/l-essentiel-droit-de-la-propriete-intellectuelle/2026-n2/abus-notoire-du-droit-moral-post-mortem-DPI203n9) [V1].
- Moral rights are perpetual and inalienable (CPI L. 121-1): a publisher cannot waive them for the illustrator [M]. A publisher opt-in clears the economic right only.
- Integrity can also be infringed without touching the pixels, by a context that distorts the spirit of the work (French case law on works reused in advertising) [M]. An untouched cover on a tasteful gift card is a weak case for such a claim, but not a null one.
- Elsewhere: Germany protects against distortion that prejudices the author's legitimate interests (UrhG § 14, with § 39 on alterations by licensees) and requires naming (§ 13) [M]; UK integrity right requires prejudice to honour or reputation and can be waived (CDPA ss.80, 87) [M]. France is the strictest of the three.

### 5.2 Operations on the cover

| Operation | Moral-right exposure | Policy |
|---|---|---|
| Proportional scaling, CMYK conversion | Low | Allowed |
| Letterboxing on a neutral ground when the cover's aspect ratio differs from the panel | Low | Default |
| Cropping to fit a panel; trimming for bleed inside the artwork | **High** | Forbidden. Add bleed outside the image |
| Non-uniform scaling (stretch to fill) | **High** | Forbidden |
| Overprinting a QR code, redemption code, "cadeau" ribbon, fold or crop marks on the cover | **High** | Forbidden. Those go on other panels |
| Recolouring, filters, background removal | **High** | Forbidden |
| Extending the artwork onto the spine (stretch, mirror, generative outpainting) | **High**: alteration and adaptation | Forbidden unless adaptation is expressly licensed and the artist's consent is warranted |
| Dropping the credit (usually on the back cover or copyright page, not the front) | Medium | Print the credit; ONIX List 160 code 01 carries it [V2] |

### 5.3 The 10 mm spine

The card needs a spine the eBook never had. Options, from safest to riskiest:

| Option | Copyright and moral rights | Trade dress | Verdict |
|---|---|---|---|
| Separate panel, flat neutral colour, title and author retyped in an openly licensed font from our own catalogue | Low. Retyping the title designates the book (referential) | Low if the layout is our own | **Default in every mode** |
| Same, with the panel colour set to one dominant colour measured from the cover | Low (analysis): a single colour value is not a reproduction of the work. No pixels, textures or gradients are taken | Low | Allowed; counsel to confirm |
| Publisher's real spine artwork, supplied by the publisher, scaled uniformly | Low with consent | None | Allowed in mode R-A |
| Publisher's real spine re-fitted to 10 mm | Medium: a real spine is rarely 10 mm wide, so fitting it means cropping, re-laying out or non-uniform scaling, i.e. adaptation | None | Only with an express adaptation clause; otherwise use the typeset spine |
| Spine reproducing the cover's own title lettering (cut out, traced, or regenerated by an image model) | High: reproduction of part of the cover, a crop, and for hand lettering an adaptation of a graphic work | Medium–High for franchise titles (trade marks) | Not offered without express licence |
| Spine that continues the cover artwork (texture sampled from the edge, mirror, outpainting) | High (§5.2) | — | Not offered without express licence and warranted artist consent |
| Spine imitating a recognisable collection (colours, rules, logo position) | Medium | **High**: parasitisme (§6) | Not offered |

The font-identification and generative paths are analysed in `notes/legal-fonts-genai.fonts.md` and `notes/legal-fonts-genai.genai.md`; their conclusion matches this table: matching an open look-alike font is acceptable, re-drawing the cover's lettering or outpainting the artwork is the riskiest operation in the product and stays behind a licence flag.

---

## 6. Trade marks and trade dress

Searches on this section were not run by either researcher; everything is [M] unless marked.

- **Registered marks.** Publisher, imprint and collection names and logos ("Folio", "Le Livre de Poche", "Pocket", "10/18") are registered trade marks, typically including class 16 (printed matter). Series and franchise titles are often registered too.
- **Referential use.** A third party may use a mark to designate the genuine product of the proprietor, provided the use follows honest practices and does not suggest a commercial link (Directive (EU) 2015/2436 art. 14(1)(c) and 14(2); CPI L. 713-6; BMW v Deenik, C-63/97). Naming the publisher in plain text, and the logo as part of the unaltered cover, fit this. Dior v Evora adds the limit of serious damage to reputation [V2].
- **Separate logo use.** Lifting the logo out of the cover and placing it on a generated spine or back panel is trade-mark use on a printed product not made by the publisher. It is likely infringement, and it signals "official publisher object" when it is not.
- **Collection trade dress.** A collection's look (Gallimard's "Blanche": cream ground, red and black rules, NRF monogram) can be defended through copyright, figurative marks and **parasitisme** (C. civ. art. 1240). Zulma v Elior shows parasitisme succeeding for a book cover even where copyright failed [V1]. The Cour de cassation's commercial chamber tightened the parasitisme test on 26 June 2024 (the claimant must identify an individualised economic value and an intention to ride on it), per `notes/legal-fonts-genai.fonts.md` [M]. A generated spine that mimics a collection's real spine is riskier than a neutral one.
- **Titles.** An original title is protected (CPI L. 112-4); retyping it to designate the book is referential and low risk.
- **No false origin.** The card should say who made it (reseller or eBook Cadeau) and must not reproduce the book's EAN barcode or present itself as the publisher's product. A card that looks like the book but contains none could also raise a misleading-practice question (C. conso. L. 121-2); a plain "carte cadeau eBook" mention answers it.
- **Retailer marks in our own material.** "Fnac", "Kobo" and "7switch" are third-party marks. Public demos, test fixtures, screenshots and the open-source repository must not use their names, logos or look; demo stores should carry invented brands. The same applies to real covers in fixtures and marketing: use invented titles, public-domain artwork or covers licensed by a partner.
- "QR Code" is a registered word mark of Denso Wave; the usual practice is an acknowledgement line where the term is printed.

---

## 7. Liability, takedown, warranties, insurance

### 7.1 Who is exposed

| Actor | Role | Exposure |
|---|---|---|
| Reseller (mode R) | Principal: decides to offer the card, supplies the cover | Contrefaçon (civil and criminal), breach of its distribution contract, loss of the catalogue feed |
| eBook Cadeau, composing on a server or ordering the print | Reproduces and distributes | Co-infringer; no hosting exemption for its own acts (§7.3) |
| eBook Cadeau, supplying a browser tool only | Tool supplier | Indirect theories only (§4.7) |
| Printer | Reproduces | Co-infringer in principle; printers' terms usually pass the risk back to the customer through a rights warranty [M] |
| End user printing at home | Copier | Private-copy argument; practically never pursued |
| Publisher opting in | Licensor | Liable to its illustrator or photographer if it grants more than it holds (§2) |

Good faith is no defence to civil infringement in France [M]. Contrefaçon is also a délit (CPI L. 335-2: up to 3 years and EUR 300,000 [M]). Damages take into account the claimant's loss, moral prejudice and the infringer's profits, or a lump sum at least equal to the fee that would have been due (CPI L. 331-1-3 [M]). Reference amounts found: EUR 15,000 in the "pop" case [V1]; three times the evaded licence fee (TJ Bordeaux, 2025) [V1]. In Germany the usual instrument is the Abmahnung [V1]. For a reseller the realistic commercial risk is less a damages award than a publisher or distributor suspending the feed.

Cross-border: infringement is judged under the law of the country for which protection is claimed (Rome II Regulation, art. 8(1) [M]). Shipping a card to Germany or Belgium brings that country's law and courts.

### 7.2 Regulatory side-issues

- Prime rule and fixed price (§3.1) [M].
- VAT on the card at the standard rate (§3.1) [M].
- Misleading presentation and false origin (§6) [M].

### 7.3 Intermediary status (DSA, Regulation (EU) 2022/2065)

- Art. 6 keeps the hosting exemption for providers that store information at the user's request, have no actual knowledge and act expeditiously once aware. It is lost where the provider plays an active role giving it knowledge or control (the line from Google France and L'Oréal v eBay); injunctions remain possible (art. 6(4)); art. 6(3) removes it for marketplaces that lead consumers to believe the product comes from the platform itself — [Freshfields](https://www.freshfields.com/en/our-thinking/blogs/technology-quotient/the-digital-services-act-new-liability-rules-102iiyf), [Taylor Wessing](https://www.taylorwessing.com/fr/interface/2022/the-eus-digital-services-act/online-intermediaries-and-illegal-content-under-the-digital-services-act), [BEUC](https://www.beuc.eu/sites/default/files/publications/ConsumerPro_DSA-complement_to_TBD_2023.pdf), [Busch report](https://digitalplanet.tufts.edu/wp-content/uploads/2023/02/DD-Report_2-Christoph-Busch-11.30.22.pdf) [V2].
- **Where hosting status helps and where it does not.** It can cover a user-uploaded file stored at the user's request. It cannot cover (i) covers the service selects or fetches by ISBN (own content), (ii) printing and shipping, where the service manufactures the object (Redbubble analogy), (iii) anything rendered under the service's brand as its own product (art. 6(3) logic). Both notes agree on this. Client-side composition is stronger than a safe harbour: nothing is hosted.
- France: LCEN (loi 2004-575, art. 6), aligned on the DSA since the 2024 "SREN" law [M].
- US: DMCA § 512 requires a registered agent and a repeat-infringer policy [M]; many print-on-demand sites follow it ([IPWatchdog](https://www.ipwatchdog.com/2021/03/09/litigation-threatening-burst-redbubble-courts-weigh-ip-implications-redbubbles-unique-online-marketplace/id=130712/)) [V1].

### 7.4 Notice and takedown

- Art. 16 DSA: every hosting provider needs an electronic notice mechanism; a sufficiently precise notice gives actual knowledge; decisions must be timely, diligent, non-arbitrary and objective — [Burges Salmon](https://burges-salmon.com/news-and-insight/legal-updates/data-protection/the-eu-digital-services-act-and-the-article-6-hosting-exemption), [art. 16 text](https://www.springlex.eu/fr/packages/dsa/dsa-regulation/article-16/) [V2]. Contact points (arts. 11–12), statement of reasons (art. 17), small-enterprise carve-out from platform-only duties (art. 19) [M].
- **Delay.** The reseller notes propose 24–48 h (24 business hours in the draft clause); the independent notes propose same-day removal. Kept: a contractual commitment of 24 business hours, an operating target of the same business day.
- **Scope of a takedown.** Stop new generation for the ISBN (or the publisher's whole ISBN prefix on request); purge cached covers; regenerate unprinted orders as typographic cards; no recall of cards already delivered (sell-off rule to be agreed in the contract).

### 7.5 Warranty chain

Rights and warranties must be back-to-back: publisher → reseller (clause in §3.8) → eBook Cadeau (article below) → printer (the printer's terms). An indemnity shifts cost, not liability: rights holders can still sue eBook Cadeau directly; it is worth the reseller's solvency; and a party that knowingly took part in an infringement may be refused the benefit of the warranty [M].

Draft for counsel, in French (operative language for French contracts):

> **Article X — Contenus Éditoriaux ; garantie**
>
> **X.1** « Contenus Éditoriaux » désigne, pour chaque Titre, la première de couverture, les textes de présentation, titre, nom(s) d'auteur, mentions de crédit et tout autre élément transmis par le Revendeur.
>
> **X.2 Déclarations du Revendeur.** Pour chaque Titre transmis avec l'indicateur « impression de la couverture autorisée », le Revendeur déclare et garantit : (a) détenir de l'éditeur ou de son distributeur une autorisation écrite couvrant la reproduction des Contenus Éditoriaux sur un support imprimé ou imprimable remis à l'acheteur d'un exemplaire numérique du Titre, y compris par l'intermédiaire de sous-traitants ; (b) que cette autorisation couvre le territoire de livraison, la durée du Contrat et une Carte par exemplaire numérique vendu ; (c) que les mentions de crédit transmises sont complètes ; (d) n'avoir connaissance d'aucune restriction de tiers (visuel sous licence limitée, affiche de film, personnage sous licence) non couverte par cette autorisation.
>
> **X.3 Sous-licence.** Le Revendeur concède au Prestataire, pour la seule exécution du Service et pour la durée du Contrat, une sous-licence non exclusive et non cessible de reproduction des Contenus Éditoriaux, que le Prestataire peut exercer par l'intermédiaire des imprimeurs listés en Annexe.
>
> **X.4 Engagements du Prestataire.** Le Prestataire s'engage à : (a) reproduire la couverture sans recadrage, surimpression ni modification autre que la mise à l'échelle proportionnelle et la conversion colorimétrique nécessaires à l'impression ; (b) faire figurer les mentions de crédit transmises ; (c) composer le dos et les autres volets avec ses propres éléments typographiques, hors de l'image de couverture, et ne reproduire isolément aucune marque ou logo de l'éditeur sans autorisation expresse ; (d) cesser toute génération pour un Titre dans les [24] heures ouvrées suivant notification de retrait ; (e) tenir un journal des Cartes générées par ISBN.
>
> **X.5 Garantie.** Le Revendeur garantit le Prestataire contre toute réclamation ou action d'un tiers (éditeur, auteur, illustrateur, photographe, agence, titulaire de marque, organisme de gestion collective) alléguant que la reproduction des Contenus Éditoriaux conformément au Contrat porte atteinte à un droit de propriété intellectuelle ou de la personnalité, ou constitue un acte de concurrence déloyale ou parasitaire. Il prend en charge les condamnations, les indemnités transactionnelles qu'il a approuvées, les frais et honoraires raisonnables de défense et les coûts de retrait. Le Prestataire notifie la réclamation sous [10] jours ouvrés, coopère à la défense et ne transige pas sans l'accord du Revendeur.
>
> **X.6 Exclusions et réciprocité.** La garantie ne couvre pas les réclamations résultant (i) d'un manquement du Prestataire à l'article X.4, (ii) des éléments fournis par le Prestataire (polices, fonds, éléments générés, logiciels), pour lesquels le Prestataire garantit le Revendeur dans les mêmes termes, (iii) d'une génération postérieure au délai de retrait.
>
> **X.7 Plafond.** Les limitations de responsabilité de l'article [●] ne s'appliquent pas au présent article [variante : plafond spécifique de [●] € par sinistre]. Chaque partie justifie d'une assurance responsabilité civile professionnelle couvrant la contrefaçon.
>
> **X.8 Repli.** En l'absence de l'indicateur visé en X.2, le Prestataire génère une Carte typographique ne reproduisant ni la couverture ni aucune marque figurative.

English gist: the reseller warrants written publisher authorisation for print through subcontractors (X.2); narrow sub-licence (X.3); the service commits to integrity, credits, an own-material spine, 24 h takedown and a per-ISBN log (X.4); the reseller indemnifies against third-party IP, personality and unfair-competition claims (X.5); carve-outs and a reciprocal indemnity for what the service adds, i.e. fonts and generated spines (X.6); indemnity outside the general cap, plus insurance (X.7); text-only fallback (X.8). Between professionals, limitation clauses are valid unless they void an essential obligation (C. civ. art. 1170) [M].

**User terms (standalone).** A warranty of rights or of private use, one print per gift code, no resale. This supports the private-copy narrative and gives a contractual basis to refuse service; as an indemnity against a consumer it is weak.

### 7.6 Insurance

Not researched; all [M]. General professional-liability (RC pro) policies often exclude intellectual-property infringement or cover it only through a specific extension (media or technology errors-and-omissions cover). Intentional fault is uninsurable (C. assur. L. 113-1) and criminal fines cannot be insured. Action: ask a broker for a quote that names "atteinte aux droits de propriété intellectuelle" for a print-on-demand composition service, with defence costs, and ask whether cover is conditional on a rights-clearance procedure (the flags and logs in the last section are that procedure). Clause X.7 requires the reseller to hold equivalent cover.

---

## 8. Recommended product policy and safe fallbacks

### 8.1 Tiers

| Tier | What it is | Where it may be used | Condition |
|---|---|---|---|
| **T0 — Typographic card** | Title, author, publisher name as text; our templates; open fonts; no logos | Everywhere: home print, professional print, reseller mode | None. This is the default and the universal fallback |
| **T1 — Licensed cover** | Unaltered cover on the front panel, credit line printed | Home print and professional print, reseller mode | Per-title flag backed by a written publisher authorisation; reseller warranty (art. X) |
| **T1r — Reduced-size cover** | Unaltered cover at reduced size on a neutral ground | Reseller mode | Reseller's written declaration that its licence covers promotion on print media; disabled by default |
| **T2 — Rights-holder cover** | Cover supplied by the author or publisher ordering their own cards | Home print and professional print, standalone | Declaration and warranty by the orderer (the Juniper Custom rule); stock-licence reminder |
| **T3 — User-supplied cover, private** | Cover file picked by the user in the browser | Home print only, standalone | Client-side only; file never reaches our servers; one card per gift code; user confirms private, non-commercial use |
| **T4 — Public-domain artwork** | Period or first-edition cover whose artist died more than 70 years ago | Everywhere | Artist death date recorded; source without contractual restriction; no distortion |

### 8.2 Rules

1. **Default output is T0** in every mode and on every print route.
2. **Artwork on a professionally printed card requires T1, T2 or T4.** No professional printing of unlicensed covers.
3. **The service never fetches a cover into a printable artefact** in standalone mode. ISBN lookup returns text metadata. A thumbnail may be shown in the UI only from a source whose terms allow it, and it is not embedded in the PDF.
4. **Covers are never modified** (§5.2). The 10 mm spine is typeset (§5.3).
5. **Tie-in, licensed-character, fine-art and rights-managed covers are excluded** unless the publisher clears the title specifically.
6. **No separate use of logos, blurbs, extracts or author photographs** without a per-title flag.
7. **Public-domain titles**: the cover is a separate work; offer T4 only when the artist's death date clears life plus 70.
8. **Takedown**: public notice form, same-business-day target, ISBN and ISBN-prefix blocklist, log of notices and decisions.
9. **UK and US users**: same policy; nothing printed relies on fair use or fair dealing.
10. **Card pricing in reseller mode**: a separate, separately invoiced line until the prime question is cleared.
11. **Written opinion from French IP counsel** before enabling T1r, T3 in France, or any professional printing of third-party covers.

### 8.3 Mitigations and their cost

| Mitigation | Effect | Cost |
|---|---|---|
| Client-side (WASM) composition, no server copy of user covers | Keeps the user as the only copier; removes the hosting question | Already the architecture |
| Typographic card as default and fallback | Removes third-party artwork entirely | Template and font work |
| Reseller contract clause, warranty and indemnity, per-title feed flag | Clears professional printing for participating titles | Negotiation; depends on the publishers' upstream grants |
| Publisher opt-in programme (allow-list by ISBN prefix; free, non-exclusive, revocable, purpose-limited licence; no alteration; credit line) | Turns tolerance into a licence | Outreach; start with independent publishers and self-published authors |
| Opt-out blocklist by ISBN prefix, honoured quickly | Limits exposure with litigious rights holders | Low |
| Notice-and-action process (art. 16 form, logged decisions, contact point; DMCA agent if US traffic) | Preserves the safe harbour where it applies; shows good faith | Low |
| No alteration; separate typeset spine; QR inside; credit "Couverture : © …" | Reduces moral-rights and parasitisme claims | Layout constraint |
| No scraping of retailer or aggregator sites for print assets | Avoids Renckhoff, database-right and contract claims | Fewer covers in preview |

---

## 9. Questions for counsel

**Reseller mode**

1. Does a licence to use the cover "pour la promotion et la vente", without media wording, cover a printed card handed to the buyer of the eBook (CPI L. 131-3)? Does "sur tous supports" change the answer? Does reduced size?
2. Is a card given free with an eBook a "vente à prime" under art. 4 of loi 2011-590, and is a card sold below cost a disguised discount on the fixed price? What VAT rate applies to the card, and does bundling affect the eBook rate?
3. Under a contrat de mandat, may the retailer launch this format without the publisher's instruction (C. civ. art. 1989)?
4. For Fnac-type chains (publisher → distributor → Kobo → Fnac → service → printer), which link must grant the sub-licence, and can the reseller's warranty (art. X) be relied on if one link is missing?
5. Is a publisher's opt-in sufficient where the cover uses a commissioned illustration (Cass. 2 July 2014) or a stock image? Is a paid card "merchandise" under standard stock licences? What wording should the publisher's warranty carry?
6. What do Dilicom/FEL and Electre licences allow for covers obtained through them?

**Standalone mode**

7. With composition entirely in the user's browser, is the user the "copiste" (Rannou-Graphie, Wizzgo)? Is a single card given to a friend "usage privé du copiste"? Does the art-work carve-out in L. 122-5 2° ("fins identiques") apply to a cover reproduced as a cover?
8. What is the service's exposure as tool supplier (C. pén. art. 121-7; C. civ. art. 1240), and which design or marketing choices raise it?
9. For professional printing ordered by a self-declared rights holder, is a declaration plus warranty adequate diligence, or is some verification expected? Does the DSA hosting exemption apply to any part of that flow?
10. In Germany, could a single paid paper copy ordered by a consumer fall under § 53(1) sentence 2 UrhG, and what is the Abmahnung exposure for cards shipped to Germany?

**Moral rights, spine, marks**

11. Is an untouched cover placed next to a generated spine and back panel an infringement of integrity by context? Is inset placement with a neutral frame safer than full bleed?
12. Is setting the spine colour to a dominant colour measured from the cover free of any reproduction issue?
13. May the publisher's logo appear as part of the unaltered cover on an object not made by the publisher (referential use, class 16)? Should the card carry an "objet non édité par l'éditeur" line?
14. Can a purely typographic card that takes the form of a book jacket for a named title be attacked as parasitisme after the June 2024 Cour de cassation decisions? Does L. 112-4 (titles) add anything?
15. What credit line should be printed when the feed supplies none?

**Liability and operations**

16. Are clauses X.5 and X.7 enforceable as drafted, and what cap is market practice? What insurance wording is needed?
17. What takedown delay and sell-off rule are reasonable, and how long should generation logs be kept (civil and criminal limitation)?
18. Criminal exposure of the directors and of the printer; what the printer's terms should say.
19. Public-domain artwork: reliance on faithful reproductions of two-dimensional works (DSM Directive art. 14 [M]) and on host terms; "mort pour la France" and wartime extensions.

---

## 10. Reconciliation of the notes, and what is not verified

### 10.1 Differences between the two notes

| Topic | Reseller notes | Independent notes | Kept |
|---|---|---|---|
| Dior v Evora, Art & Allposters, Tom Kabinet | [V2], with sources | [M] | [V2]: the reseller notes supply two or more sources for each |
| Cass. 1re civ., 2 July 2014 (cover commissions) | [V2], n° 13-24.359 | [V1] | [V2]: three sources across the notes. Whether the decision is published is unclear |
| Authority for "no short quotation of an image" | Ass. plén., 5 Nov 1993 [M] | Cass. 1995 and 2006, Fabris v Loudmer 1991 [V2] | Same line of cases; the verified 1995 and 2006 decisions are primary |
| Cover under the existing promotional licence | "Medium" risk; reduced-size mode proposed | Artwork only when licensed for print | Default is no cover; reduced-size mode behind a written reseller declaration (§3.7) |
| Takedown delay | 24–48 h; 24 business hours in the clause | Same day | 24 business hours contractual, same-business-day target |
| Gift and private copy | Not addressed ("does not shelter the service") | Unsettled, L–M for France | Amber; the gifting note's recollection of Aurélien D. [M] leans against the user |
| Stock-image cap of 500,000 copies | [V2], but ScribeCount contradicts for some vendors | — | Treated as vendor-specific; the "merchandise needs an extended licence" rule is the operative point |

### 10.2 Not verified

1. **Reseller-facing terms** of Eden Livres, Immatériel/7switch, Numilog, Interforum, Hachette, Kobo/Fnac. The most important gap; obtainable only from the contracts.
2. **Loi 2011-590 art. 4** wording and later amendments; VAT treatment of the card.
3. **CPI and Code civil article numbers cited from memory**: L. 112-2, L. 112-4, L. 113-2, L. 113-5, L. 121-1, L. 122-4, L. 122-7, L. 123-1, L. 131-3, L. 331-1-3, L. 335-2, L. 713-6; the "fins identiques" carve-out and L. 122-5 3° d) wording; C. civ. arts. 9, 1170, 1240, 1989; C. pén. art. 121-7; C. conso. L. 121-2; C. assur. L. 113-1; LCEN after the 2024 SREN law.
4. **Cases cited from memory**: Ass. plén. 5 Nov 1993; the Wizzgo appeal outcome; Aurélien D.; ACI Adam; Copydan; Deckmyn; BMW v Deenik; L'Oréal v eBay; Google France; the BGH cases (Internet-Videorecorder, Möbelkatalog); the June 2024 parasitisme judgments; Warhol v Goldsmith. Dior v Evora's copyright paragraphs were confirmed through secondary sources only.
5. **Decisions seen only through a search summary**: Cass. 26 May 2011, n° 09-71.083 (Moulinsart); TGI Paris, 5 Oct 2007; TJ Bordeaux, 29 Feb 2024 (grounds not visible). The Zulma/Elior damages amount was not seen.
6. **Germany and UK from memory**: wording of § 53(1) and (6), §§ 13, 14, 39, 57; UK s.28B quashing, ss.8, 31, 63, 80, 87.
7. **Trade marks and trade dress**: registrations and classes for the collections named; French case law on collection trade dress. No search was run.
8. **ONIX**: full List 154; whether List 158 has spine or full-cover codes; any usage-constraint element for supporting resources in ONIX 3.1; the List 21 "MDT" code.
9. **No source found** for: a French or German judgment against a bookseller for displaying covers; any EU judgment on print-on-demand platform liability; SNE or Börsenverein written positions on cover reuse; any French decision on a retailer printing covers in a catalogue or PLV without consent; French precedents for physical eBook cards; Electre terms; rights-managed agency terms.
10. **Insurance** market practice: not researched.
11. **Other EU Member States'** private-copy rules: not researched.
12. **Leads seen but not read**: a German article titled "Buchverkauf im Internet verboten" ([ChannelPartner](https://www.channelpartner.de/article/3897785/buchverkauf-im-internet-verboten.html)), which may concern warnings to online sellers over a book cover; an ActuaLitté piece on reproducing a comics panel for home decoration ([link](https://actualitte.com/article/106953/edito/il-voulait-decorer-sa-maison-en-reproduisant-une-case-de-bd)).
13. Risk levels throughout are assessments, not sourced ratings.

---

## 11. Sources

All links below appeared in the researchers' search results. No new source was added in this synthesis.

**French statute, contracts, trade practice**
- https://www.livreshebdo.fr/article/la-loi-sur-le-prix-en-vigueur
- https://www.livreshebdo.fr/article/prix-du-livre-numerique-une-loi-de-securite
- https://www.livreshebdo.fr/article/loi-sur-le-prix-du-livre-numerique-une-victoire-la-pyrrhus
- https://www.lemondedudroit.fr/pi-tic/278-propriete-intellectuelle-industrielle/33565-prix-du-livre-numerique-publication-de-la-loi.html
- https://www.wipo.int/wipolex/en/legislation/details/13861
- https://www.wipo.int/wipolex/fr/legislation/details/18453
- https://www.senat.fr/dossierleg/ppl09-695.html
- https://editions-exaequo.com/wp-content/uploads/2021/01/AAA-contrat-type-auteur-V4-du-09-01-2021-.pdf
- https://www.sgdl.org/images/phocadownload/JURIDIQUE/Contrat_type_version_octobre2024.pdf
- https://ldt-editions.fr/wp-content/uploads/2023/04/SPECIMEN_-_CONTRAT_DEDITION.pdf
- https://www.librinova.com/cgv-auteur
- https://cms.law/fr/fra/legal-updates/le-contrat-d-edition-a-l-ere-du-numerique (not read)
- https://www.syndicat-librairie.fr/le_service_distrimage_de_dilicom
- http://dilicms.centprod.com/file/dilicom/file.do?path=/Conditions+g%C3%A9n%C3%A9rales/CGV+-+Dilicom.pdf
- https://www.guichetdusavoir.org/question/voir/45700
- https://www.guichetdusavoir.org/question/voir/139502

**Metadata (ONIX)**
- https://es.slideshare.net/bisg/bisg-webcast-onix-for-books-v30-introduction-1787290
- https://hanmoto.com/pub/onix/codelists/onix-codelist-158.htm
- https://home.bic-media.com/onix_info/3-0-2/codelists/onix-codelist-160.htm
- https://repository.data2type.de/ONIX/v_3.0/html/si.List160.html
- https://booknetcanada.ca/blog/2021/02/02/onix-codelists-issue-52-released/
- https://booknetcanada.atlassian.net/wiki/download/attachments/1378909/ONIX_BookProduct_Codelists_Issue_50_Changes.pdf?api=v2

**French case law and commentary**
- https://www.village-justice.com/articles/tirer-couverture,54716.html
- https://www.livreshebdo.fr/article/tirer-la-couverture
- https://www.lemondedudroit.fr/decryptages/9965-arret-magnard-vuibert-exclusion-du-regime-du-contrat-dedition-pour-la-contribution-accessoire-une-oeuvre-composite.html
- https://actualitte.com/article/46544/distribution/realiser-la-seule-couverture-d-039-un-livre-n-039-oblige-pas-au-contrat-d-039-edition
- https://www.lemondedudroit.fr/pi-tic/278-propriete-intellectuelle-industrielle/58593-protection-par-le-droit-dauteur-des-catalogues-dune-maison-de-vente-aux-encheres-mis-en-ligne.html
- https://www.livreshebdo.fr/article/pop-et-droit-moral
- https://www.economie.gouv.fr/node/40472
- https://www.alsace.catholique.fr/wp-content/uploads/sites/14/2016/04/notions-cle-droit-image.pdf
- https://www.dalloz-actualite.fr/document/tj-bordeaux-29-fevr-2024-n-2308086
- https://www.labase-lextenso.fr/l-essentiel-droit-de-la-propriete-intellectuelle/2026-n2/abus-notoire-du-droit-moral-post-mortem-DPI203n9
- https://actualitte.com/article/132591/droit-justice/photo-du-tournage-d-a-bout-de-souffle-la-justice-blanchit-flammarion (headline only)
- https://affairesjuridiques.aphp.fr/textes/cour-de-cassation-civile-1ere-4-juillet-1995-antenne-2-droits-dauteur-courte-citation-representation-integrale-de-loeuvre/
- https://larevue.squirepattonboggs.com/droits-d-auteur-reproduction-d-une-photographie_a334.html
- https://merlin.obs.coe.int/article/284
- https://www.livreshebdo.fr/article/citations-dartistes
- https://larevue.squirepattonboggs.com/etre-et-avoir-ou-les-auxiliaires-du-droit-d-auteur_a1468.html
- https://www.lemondedudroit.fr/pi-tic/278-propriete-intellectuelle-industrielle/33575-affaire-qetre-et-avoirq-de-linclusion-fortuite-dune-oeuvre.html
- https://merlin.obs.coe.int/article/5813
- https://www.baumann-avocats.com/CPI/article-l122-5-du-code-de-la-propriete-intellectuelle.php
- https://www.doctrine.fr/l/texts/codes/LEGITEXT000006069414/sections/LEGISCTA000006179045
- https://www.memoireonline.com/12/06/297/m_l-exception-de-copie-privee-face-aux-dispositifs-techniques-de-protection-des-oeuvres10.html
- https://www.village-justice.com/articles/affaire-wizzgo-prend-tournant,4999.html
- https://next.ink/news/86942-la-justice-europeenne-impose-decouplage-copie-privee-et-source-illicite.htm
- https://www.fournol-avocat.fr/actualite/2022/8/17/la-difficile-reconnaissance-de-loriginalit-des-couvertures-de-livre
- https://www.fournol-avocat.fr/actualite/2022/8/18/parasitisme-reconnu-pour-la-couverture-dun-livre
- https://www.legifrance.gouv.fr/juri/id/JURITEXT000024085051/
- https://www.legifrance.gouv.fr/juri/id/JURITEXT000018862984
- https://kohenavocats.com/concurrent-copie-site-internet-textes-photos-fiches-faire-cesser-chiffrer-2026/
- https://www.uggc.com/condamnation-dartprice-com-pour-contrefacon-et-concurrence-parasitaire/
- https://actualitte.com/article/106953/edito/il-voulait-decorer-sa-maison-en-reproduisant-une-case-de-bd (not read)

**Stock-image licences**
- https://help.depositphotos.com/hc/en-us/articles/33734087656466-Image-Licensing-for-Authors-Publishers
- https://community.adobe.com/t5/stock-discussions/use-of-adobe-stock-photo-on-book-e-book-cover/m-p/8940006
- https://scribecount.com/author-resource/protecting-books/book-cover-rights-for-indie-authors-do-you-really-own-the-cover-you-paid-for

**EU case law and DSA**
- https://mansfield.bailii.org/eu/cases/EUECJ/1997/C33795.html
- https://cms.law/en/gbr/legal-updates/brands-ecj-cases-parfums-christian-dior-v-evora-bv-official-distribution-networks-parallel-imports-and-the-exhaustion-of-rights
- https://swarb.co.uk/?p=111802
- https://ipkitten.blogspot.com/2015/01/breaking-news-cjeu-says-exhaustion-only.html?m=0
- https://legalblogs.wolterskluwer.com/copyright-blog/allposters-ecj-decision-no-exhaustion-of-rights-in-modifications-of-the-copyright-work/
- https://www.ippt.eu/items/ippt20150122-cjeu-allposters-v-pictoright
- https://www.scl.org/10768-cjeu-sale-of-second-hand-e-books-via-a-website-is-a-communication-to-the-public/
- https://www.charlesrussellspeechlys.com/en/insights/expert-insights/tmt/2020/can-you-resell-an-e-book-online/
- https://legalblogs.wolterskluwer.com/copyright-blog/reflections-on-the-cjeus-judgment-in-spiegel-online-is-there-a-golden-intersection-between-freedom-of-expression-and-eu-copyright-law-part-i/
- https://legalblogs.wolterskluwer.com/copyright-blog/sample-sample-in-my-song-can-they-tell-where-you-are-from-the-pelham-judgment-part-ii/
- https://fra.europa.eu/en/node/49752
- https://legalblogs.wolterskluwer.com/copyright-blog/up-in-the-cloud-some-reflections-on-the-cjeu-judgment-in-vcast/
- https://ipkitten.blogspot.com/2017/11/the-vcast-decision-how-to-turn-private.html
- https://lkshields.ie/news-insights/publication/eu-court-defines-boundaries-for-cloud-based-services-recorded-tv-programmes
- https://pure.uva.nl/ws/files/26578344/EU_copyright_law_and_the_Cloud.pdf
- https://merlin.obs.coe.int/article/8373
- https://globalfreedomofexpression.columbia.edu/cases/land-nordrhein-westfalen-v-dirk-renckhoff/
- https://www.mondaq.com/uk/copyright/730320/the-risks-of-reposting-online-content-cjeu-rules-on-the-unauthorised-reposting-of-copyright-protected-works-on-the-internet
- https://ipkitten.blogspot.com/2011/12/all-photos-are-created-equal-painer.html
- https://legalblogs.wolterskluwer.com/copyright-blog/another-piece-of-the-puzzle-or-is-it-cjeu-on-photographs-as-copyright-works/
- https://www.freshfields.com/en/our-thinking/blogs/technology-quotient/the-digital-services-act-new-liability-rules-102iiyf
- https://www.taylorwessing.com/fr/interface/2022/the-eus-digital-services-act/online-intermediaries-and-illegal-content-under-the-digital-services-act
- https://burges-salmon.com/news-and-insight/legal-updates/data-protection/the-eu-digital-services-act-and-the-article-6-hosting-exemption
- https://www.beuc.eu/sites/default/files/publications/ConsumerPro_DSA-complement_to_TBD_2023.pdf
- https://www.springlex.eu/fr/packages/dsa/dsa-regulation/article-16/
- https://digitalplanet.tufts.edu/wp-content/uploads/2023/02/DD-Report_2-Christoph-Busch-11.30.22.pdf

**Germany**
- https://www.gesetze-im-internet.de/urhg/BJNR012730965.html
- https://lexmea.de/de/gesetz/urhg/58
- https://www.haufe.de/id/beitrag/2-urheberrecht-c-werke-in-ausstellungen-HI15189406.html
- https://www.wbs.legal/it-und-internet-recht/verwendung-der-fotografie-eines-buchcovers-im-rahmen-einer-buchrezension-ohne-zustimmung-des-urhebers-14143/
- https://www.wbs.legal/internetrecht/fotorecht-bildrecht/verwendung-der-fotografie-eines-buchcovers-im-rahmen-einer-buchrezension-ohne-zustimmung-des-urhebers-32292/
- https://www.it-recht-kanzlei.de/Urheberrecht-Versto%C3%9F-Produktfotos-Produktbeschreibung-Werke.html
- https://www.it-recht-kanzlei.de/abmahnung-urheberrecht-bilderklau-faq.html
- https://www.ferner-alsdorf.de/lg-koeln-notwendigkeit-der-urheberangabe-bei-der-berufung-auf-die-zitatschranke/
- https://www.gmeiner-verlag.de/images/stories/gmeiner-bloggerinnen-faq.pdf
- https://www.channelpartner.de/article/3897785/buchverkauf-im-internet-verboten.html (not read)

**UK and US**
- https://www.legislation.gov.uk/ukpga/1988/48/section/30
- https://legislation.gov.uk/uksi/2014/2356/regulation/3/made?view=plain
- https://blogs.ucl.ac.uk/copyright/2023/02/24/fair-dealing-week-2023-part-2-three-fair-dealing-exceptions
- https://m.openjurist.org/336/f3d/811
- https://www.everycrsreport.com/reports/RL33810.html
- https://eff.org/files/filenode/Perfect10_v_Google/effperfect10vgoogleamicus.pdf
- https://www.justice.gov/usam/title9/crm01854.htm
- https://copyrightalliance.org/?p=1446
- https://www.frosszelnick.com/u-s-court-of-appeals-for-the-sixth-circuit-redbubble-could-be-liable-for-direct-infringement-arising-from-print-on-demand-service/
- https://blog.ericgoldman.org/archives/2021/02/ip-lawsuits-against-print-on-demand-vendors-continue-to-vex-the-courts-osu-v-redbubble-more.htm
- https://wilmap.stanford.edu/node/30497
- https://www.ipwatchdog.com/2021/03/09/litigation-threatening-burst-redbubble-courts-weigh-ip-implications-redbubbles-unique-online-marketplace/id=130712/

**Precedents and industry practice**
- https://www.publishersweekly.com/pw/print/20140922/64046-enthrill-partners-with-walmart-canadian-publishing-2014.html
- https://www.thebookdesigner.com/e-books-for-real-e-book-gift-cards-from-livrada-and-enthrill/
- https://buildbookbuzz.com/e-book-gift-cards/
- https://boingboing.net/2018/08/22/walmarts-selling-ebooks-now.html/amp
- https://blog.the-ebook-reader.com/2016/01/22/kindle-cards-new-book-specific-kindle-gift-cards
- https://www.thebookdesigner.com/?p=36235
- https://juniperbooks.com/a/faq
- https://juniperbooks.com/pages/custom-faqs
- https://juniperbooks.com/products/bridgerton-jackets-only
- https://junipercustom.com/products/a-court-of-thorns-and-roses-5-book-jackets-only-set
- https://junipercustom.com/products/complete-anne-of-green-gables-8-book-set-jackets-only-mto
- https://www.axial.net/company/out-of-print
- https://dc.fortune.com/2014/10/31/the-little-literary-t-shirt-company-that-could/
- https://blog.freelancersunion.org/2013/06/25/books-shirts-story-out-print/
- https://coolmompicks.com/blog/2010/04/07/you-can-totally-judge-me-by-a/
- https://blog.librarything.com/category/covers/
- https://blog.librarything.com/2008/08/a-million-free-covers-from-librarything/
- https://www.goodreads.com/topic/show/1277733-source-of-book-covers?page=1

---

## Implications for the implementation

Numbers marked (proposal) are design choices derived from the analysis, not legal thresholds; counsel should confirm them.

**Rights model**

1. Every cover asset carries a mandatory rights basis with exactly these values: `none` (default), `print_licence` (T1), `promo_all_media` (T1r), `rights_holder_declared` (T2), `user_private_copy` (T3), `public_domain` (T4). No code path may place a raster or vector cover in a print file when the basis is `none`.
2. The default output of every entry point (web, command line, reseller API) is the typographic card: title, subtitle, author names, publisher name and ISBN as plain text, set in fonts from the project's own openly licensed catalogue. No logo, no blurb, no barcode.
3. Print-route gate: home-print PDF accepts every basis except `none`; a professional print order accepts only `print_licence`, `rights_holder_declared` and `public_domain`; `promo_all_media` yields the reduced-size layout only; `user_private_copy` never reaches a print order.
4. The command-line and self-hosted tools require an explicit rights-basis argument to embed a cover and print a one-line notice that the operator is responsible for the rights; without the argument they produce the typographic card.

**Cover integrity (enforced in layout and again in preflight)**

5. A cover is placed with uniform scaling only: horizontal and vertical scale factors equal within 0.1 %; rotation 0°; the clip box equals the image box (no crop); no colour change other than ICC conversion to the output profile.
6. Aspect-ratio mismatch is resolved by letterboxing on a flat neutral ground, never by cropping or stretching.
7. No object may intersect the cover's bounding box: QR code, redemption code, ribbons, text, fold marks and crop marks are placed on other panels or outside the trim. Preflight fails the file otherwise.
8. Bleed is 3 mm and is filled with the flat ground colour, not with mirrored or extended artwork. By default the cover is inset at least 3 mm inside the trim on its outer edges so that trimming tolerance never cuts the artwork. Full-bleed placement (trimming up to 3 mm of artwork) is allowed only when the title carries `trim_authorised = true`.
9. Reduced-size layout (`promo_all_media`): the cover occupies at most 60 % of the front panel's width and height (at most 36 % of its area) (proposal), on a neutral ground, with the title retyped beside it.
10. When a cover is printed, a credit line is printed on the back or inside panel at 6 pt minimum (proposal): the feed's credit text when supplied (ONIX List 160 codes 01 and 03), otherwise "Couverture : © {publisher}" with `credit_missing = true` recorded in the generation log.

**Spine (10 mm) and other panels**

11. The spine is always a separate panel. Default: flat colour, title and author typeset from metadata text in an open font. The colour may be a single dominant colour computed from the cover; no pixels, textures or gradients are sampled from the cover.
12. Outpainting, mirroring or stretching the artwork onto the spine, cutting out or regenerating the cover's title lettering, and any image-model output derived from the cover are disabled unless the title carries `adaptation_authorised = true` and the basis is `print_licence` or `rights_holder_declared`. The flag defaults to false.
13. A publisher-supplied spine or flat full-cover file is used as supplied with uniform scaling. Re-fitting it to 10 mm requires `adaptation_authorised = true`; otherwise the typeset spine is used.
14. Templates are original designs. No template may reproduce a publisher's collection layout; no template takes a publisher logo. Publisher, imprint and collection logos are accepted only inside the unaltered cover image, or as separate elements when `logo_use = true`.
15. Blurbs, extracts, review quotes and author photographs are printed only when the corresponding per-title flag is true (`blurb_use`, `extract_use`, `author_photo_use`); all default to false.
16. Every card carries an imprint line naming who produced it (reseller or eBook Cadeau) and the mention "carte cadeau eBook" in the card's language. The book's EAN barcode is never reproduced. Where "QR Code" is printed as a term, the Denso Wave acknowledgement line is added.

**Standalone mode**

17. A cover enters the standalone flow only through a local file picked or dropped in the browser. Decoding, analysis and PDF composition run client-side in WebAssembly. There is no upload endpoint for cover images in the home-print flow, and an integration test asserts that no network request carries the image bytes.
18. The service has no server-side proxy or fetcher that inserts a cover into a PDF. ISBN lookup returns text metadata only.
19. A preview thumbnail of a cover the service has no rights in is at most 400 px on its long edge (proposal), is hotlinked by the browser from a source whose terms allow display, is not cached or re-served by our servers, is not read into a canvas, and is never embedded in the PDF. Covers found by the acquisition tools are stored as `none` with their source URL and the date the source's terms were last read.
20. On cover import the user must choose one declaration, stored with the session: "I am the author or publisher, or hold a licence" (`rights_holder_declared`); "copy for my private use, from a book I bought" (`user_private_copy`); "the artwork is in the public domain" (`public_domain`, requires the artist's year of death). No choice means no cover.
21. `user_private_copy` produces exactly one jacket per gift code: one jacket per PDF, no multi-up imposition, no professional-print button, and the generation counter for that code blocks a second cover-bearing PDF.
22. `public_domain` is accepted automatically only when the current year minus the artist's year of death is at least 71; the interface warns about French extensions for authors "morts pour la France".
23. A professional print order with `rights_holder_declared` requires an account, the declarant's name, role and email, and acceptance of the rights warranty; the declaration is logged with the order.

**Reseller API**

24. Per-title fields, all defaulting to the restrictive value: `cover_rights` (`none`, `promo_all_media`, `print_licence`), `adaptation_authorised`, `trim_authorised`, `logo_use`, `blurb_use`, `extract_use`, `author_photo_use`, `credit_line`, `excluded_reason` (`tie_in`, `licensed_character`, `rights_managed`, `fine_art`), `cleared_per_title`, `territories` (ISO country list for delivery), `licence_reference`, `valid_until`.
25. A title with an `excluded_reason`, or flagged as a media tie-in edition in ONIX, is forced to the typographic card unless `cleared_per_title = true`.
26. When `cover_rights = none` the API ignores any image payload and does not store it. Cover files received under a licence are cached for the life of the order plus 30 days (proposal) and purged on takedown.
27. A cover-bearing card is not shipped to a country outside the title's `territories`; the order falls back to the typographic card.
28. `promo_all_media` is disabled per reseller until the reseller's written declaration is recorded in its account configuration.
29. In reseller checkout flows (including the demo stores) the card is a separate line item with its own price and the standard VAT rate; a price of zero is rejected unless the reseller configuration carries `publisher_premium = true`.

**Takedown, logging, metadata**

30. A blocklist keyed by ISBN and by ISBN registrant prefix is checked at every generation: enforced server-side for reseller and print orders, and fetched as a signed list by the browser for home print. A blocked title yields the typographic card.
31. Takedown: new generation stops within 24 business hours of a valid notice (target: same business day); cached covers are purged; unprinted orders are regenerated as typographic cards; delivered cards are not recalled.
32. A public notice form collects the DSA art. 16 elements (reasons, ISBN or URL, notifier's name and email, good-faith statement), acknowledges receipt, and records the decision with its reasons. A contact point is published.
33. Server-side generations are logged per card: ISBN, rights basis, flags, reseller identifier, licence reference, template, printer, timestamp. Retention: 6 years (proposal, to cover civil and criminal limitation). Client-side home-print generations log nothing about the cover.
34. Each PDF embeds in its XMP metadata the rights basis, credit line, licence reference and generator version, so that a printed card can be traced to its authorisation.

**Tests and fixtures**

35. Integration tests assert the gates: a reseller feed with `cover_rights = none` yields a PDF with no image on the front panel; a crop, overlay or non-uniform scale fails preflight; a blocked ISBN yields the typographic card; a `user_private_copy` session cannot create a print order.
36. Demo stores, fixtures, screenshots and documentation use invented retailer brands and invented or public-domain books with own-made or public-domain covers. No real retailer or publisher name, logo or look, and no real in-copyright cover, is committed to the repository.
