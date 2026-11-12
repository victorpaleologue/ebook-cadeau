# Legal — the reseller's position: what an eBook reseller may hand to a print service, and the risk of using covers independently

Research notes, 2026-11-12. Angle: the reseller (7switch, Fnac/Kobo-type) that offers an "eBook cadeau" printed card at checkout.
**Not legal advice** — this is a research map for a French IP lawyer to confirm, not a substitute for one.

**Method caveat.** The shared web-search budget ran out after 16 successful searches (4 more were refused), below the 25+ target. Everything tagged **[M]** is from memory and was *not* checked in this session; statute article numbers in particular must be re-read on Légifrance. Tags: **[V2]** = ≥2 independent sources in this session, **[V1]** = one source, **[M]** = memory.

---

## 0. Bottom line

1. A reseller holds **no property right** in a cover. It holds a contractual, purpose-limited licence ("for the promotion and sale of the work") whose scope French law reads narrowly. No statute gives a retailer a general right to reproduce covers to advertise what it sells.
2. Printing the cover on a physical card is a **new reproduction on a new medium**. Whether the existing promotional licence stretches to it is arguable, not settled; I found no French decision on the point.
3. The retailer's usual EU-law comfort (Dior v Evora: a reseller may advertise goods it lawfully resells) rests on **exhaustion**, and there is no exhaustion for eBooks (Tom Kabinet). So for eBooks everything turns on the contract.
4. Even a publisher's express consent can be hollow: publishers often hold cover art on thin paper (commission + invoice), under stock licences with print-run/merchandise limits, or under film tie-in licences. Moral rights of the illustrator/photographer cannot be waived by the publisher.
5. Safe design: **publisher opt-in per title** (feed flag) for cover printing, cover reproduced **uncropped and unaltered**, spine/QR panels built from own typographic material *outside* the cover image, and a **text-only fallback card** for titles without confirmed rights.

---

## 1. Where the reseller's rights come from

### 1.1 Statute: loi n° 2011-590 (prix du livre numérique)

| Point | Finding | Conf. |
|---|---|---|
| Publisher fixes the public price; it binds everyone selling to buyers in France | [Livres Hebdo](https://www.livreshebdo.fr/article/la-loi-sur-le-prix-en-vigueur), [Le Monde du Droit](https://www.lemondedudroit.fr/pi-tic/278-propriete-intellectuelle-industrielle/33565-prix-du-livre-numerique-publication-de-la-loi.html), text on [WIPO Lex](https://www.wipo.int/wipolex/en/legislation/details/13861) | [V2] |
| Before the law, publishers controlled price through **contrats de mandat** (agency) with e-retailers; the law was seen as a "second safety net" should those contracts be invalidated; mandate retailers stayed under their contracts | [Livres Hebdo](https://www.livreshebdo.fr/article/prix-du-livre-numerique-une-loi-de-securite), [Livres Hebdo](https://www.livreshebdo.fr/article/loi-sur-le-prix-du-livre-numerique-une-victoire-la-pyrrhus) | [V1] |
| Publisher must take the retailer's qualitative promotion/advice services into account when setting the discount | same sources; [Sénat file](https://www.senat.fr/dossierleg/ppl09-695.html) | [V1] |
| Consolidated text (as amended to 6 Aug 2018) exists on [WIPO Lex](https://www.wipo.int/wipolex/fr/legislation/details/18453); later amendments not checked | | [V1] |
| The law says **nothing** about covers, metadata or promotional material. It regulates price only | reading of the summaries above | [V1] |
| Art. 4 restricts **ventes à primes**: a premium with an eBook is allowed only if the *publisher* offers it simultaneously and on the same terms to all retailers | | [M] — verify |

Consequences for the card:

- **A free printed card bundled with the eBook is plausibly a "prime"** [M]. A reseller should either charge for the card as a separate, separately-invoiced service (not a book; standard VAT rate rather than the book rate [M]), or have the publisher offer it to all resellers. A card priced below cost could also be attacked as a disguised discount on the fixed price. This is a regulatory risk independent of copyright.
- Under a **mandat**, the retailer sells in the publisher's name and for its account; Code civil art. 1989 — "le mandataire ne peut rien faire au-delà de ce qui est porté dans son mandat" [M]. An agent has *less* room for own-initiative promotional formats than a buyer-reseller.

### 1.2 Contract: what the upstream chain typically grants

I could not obtain the reseller-facing terms of Eden Livres, Immatériel, Numilog, Interforum or Hachette (not public / not surfaced). What published French model contracts show about the upstream position:

| Source | What it says | Conf. |
|---|---|---|
| [Ex Aequo author contract (2021)](https://editions-exaequo.com/wp-content/uploads/2021/01/AAA-contrat-type-auteur-V4-du-09-01-2021-.pdf) | Publisher may mandate third parties for diffusion/promotion/distribution; any use of cover, visuals or summaries is subject to the **publisher's prior consent** | [V1] |
| [SGDL model contract (Oct 2024)](https://www.sgdl.org/images/phocadownload/JURIDIQUE/Contrat_type_version_octobre2024.pdf) | Responsibility for promotional elements lies with the publisher; cover choice is the publisher's exclusive competence | [V1] |
| [LDT specimen contract](https://ldt-editions.fr/wp-content/uploads/2023/04/SPECIMEN_-_CONTRAT_DEDITION.pdf) | Author's prior approval for presentation, cover and promotional texts; free consultation of extracts for strictly promotional purposes earns no royalty | [V1] |
| [Librinova CGV](https://www.librinova.com/cgv-auteur) | Author allows "reasonable extracts" as promotional documentation, up to 15 % of the book | [V1] |

Pattern: the right to promote flows author → publisher → (distributor) → retailer, each step "for promotion", with the publisher as gatekeeper. Typical retailer licence wording in e-distribution agreements is "to reproduce and display cover art, metadata and excerpts solely to market, promote and sell the eBooks through the store" [M]. Two French rules make that wording narrow:

- **Strict interpretation of assignments**: CPI L. 131-3 requires each right, and its extent, destination, place and duration, to be specified; what is not granted is retained (also L. 122-7) [M]. A licence for display "on the site" does not obviously include print.
- **No general advertising exception**. InfoSoc Directive art. 5(3)(j) lets Member States allow use "for advertising the public exhibition or sale of artistic works"; France transposed only the narrow judicial-sale-catalogue case (CPI L. 122-5 3° d) — [Le Monde du Droit](https://www.lemondedudroit.fr/pi-tic/278-propriete-intellectuelle-industrielle/58593-protection-par-le-droit-dauteur-des-catalogues-dune-maison-de-vente-aux-encheres-mis-en-ligne.html) [V1]. The short-quotation exception does not cover a full reproduction of a graphic work, even reduced (Ass. plén. 5 Nov 1993, Fabris/Utrillo) [M].

**Fnac-specific chain issue** [M]: Fnac's eBook catalogue is operated with Kobo; the publisher licences may sit with Rakuten Kobo, not Fnac. A sub-licence chain publisher → distributor → Kobo → Fnac → eBook Cadeau → printer must be permitted at every link.

### 1.3 Metadata feeds: ONIX, Dilicom, Electre

- ONIX 3.0 carries covers as "supporting resources" in the marketing-collateral block: resources "offered for promotional use by supply chain partners" (images, audio, video, sample chapters) — [BISG ONIX 3.0 intro](https://es.slideshare.net/bisg/bisg-webcast-onix-for-books-v30-introduction-1787290) [V1].
- `<ContentAudience>` (List 154): code 01 "Restricted" = distribution by agreement between the parties to the exchange, not to be displayed without agreement; other codes (search-engine index, bloggers…) target audiences, not media — [List 158 notes](https://hanmoto.com/pub/onix/codelists/onix-codelist-158.htm), [Issue 50 changes](https://booknetcanada.atlassian.net/wiki/download/attachments/1378909/ONIX_BookProduct_Codelists_Issue_50_Changes.pdf?api=v2) [V1].
- `<ResourceFeature>` (List 160): 01 **Required credit** ("Photo Jerry Bauer", "© Magnum Photo"), 02 Caption, 03 **Copyright holder** — "indicative only, as the resource can be used without consultation" — [BIC List 160](https://home.bic-media.com/onix_info/3-0-2/codelists/onix-codelist-160.htm), [data2type](https://repository.data2type.de/ONIX/v_3.0/html/si.List160.html) [V2].
- **ONIX has no field meaning "may be printed on a physical object"** — I found none; ResourceFeature otherwise describes image attributes (background colour, perspective) — [BookNet Canada](https://booknetcanada.ca/blog/2021/02/02/onix-codelists-issue-52-released/) [V1]. The licence is implicit ("promotional use by supply-chain partners") and governed by the trading agreement. A publisher opt-in therefore needs a proprietary flag or a side agreement.
- **Dilicom**: the Distrimage service supplies FEL front-cover visuals; free for use inside bookshop management software, but "pour tout autre usage (internet, extranet), il convient de prendre contact avec le service clients de DILICOM" — [SLF, 2012](https://www.syndicat-librairie.fr/le_service_distrimage_de_dilicom) [V1]. Covers from trade databases are licensed **per use**; print is not a default use. Current Dilicom terms ([CGV 2021](http://dilicms.centprod.com/file/dilicom/file.do?path=/Conditions+g%C3%A9n%C3%A9rales/CGV+-+Dilicom.pdf)) did not surface an image-rights clause in the extract returned.
- **Electre**: terms not found.

### 1.4 Customary tolerance

Covers are protected works; authorisation is to be sought from the publisher, who knows the rightsholders; layout designer, illustrator and photographer may each hold rights — [Guichet du Savoir](https://www.guichetdusavoir.org/question/voir/45700), [Pierrat, Village de la Justice](https://www.village-justice.com/articles/tirer-couverture,54716.html) [V2]. In practice "les ayants-droit s'opposent rarement à une reproduction de la couverture à un format inférieur à celui de l'ouvrage et dans un but d'information du lecteur" — [Guichet du Savoir](https://www.guichetdusavoir.org/question/voir/139502) [V1]. This is tolerance, not a right, and its two conditions (smaller format, informational purpose) are exactly what a full-size shelf object strains.

---

## 2. Promotion/packaging of the sale, or separate merchandising?

### 2.1 EU case law

| Case | Holding | Effect on the card | Conf. |
|---|---|---|---|
| **Dior v Evora**, C-337/95 (1997) | A reseller of lawfully marketed goods may use the mark in advertising customary in its trade unless it seriously damages the mark's reputation; the same principles apply to copyright in the packaging shown in leaflets — [CMS](https://cms.law/en/gbr/legal-updates/brands-ecj-cases-parfums-christian-dior-v-evora-bv-official-distribution-networks-parallel-imports-and-the-exhaustion-of-rights), [swarb](https://swarb.co.uk/?p=111802), [judgment](https://mansfield.bailii.org/eu/cases/EUECJ/1997/C33795.html) | Best argument for print bookshops showing covers in leaflets. But it is an exhaustion case about *goods put on the market* | [V2] |
| **Tom Kabinet**, C-263/18 (2019) | Supplying an eBook by download is communication to the public, not distribution; **no exhaustion** — [SCL](https://www.scl.org/10768-cjeu-sale-of-second-hand-e-books-via-a-website-is-a-communication-to-the-public/), [Charles Russell Speechlys](https://www.charlesrussellspeechlys.com/en/insights/expert-insights/tmt/2020/can-you-resell-an-e-book-online/) | The Dior rationale has no exhaustion to stand on for eBooks. The reseller's advertising rights are purely contractual (my inference, untested) | [V2] |
| **Art & Allposters**, C-419/13 (2015) | Exhaustion does not cover a work whose **medium has been altered** (poster → canvas); the new object needs fresh authorisation — [IPKat](https://ipkitten.blogspot.com/2015/01/breaking-news-cjeu-says-exhaustion-only.html?m=0), [Kluwer](https://legalblogs.wolterskluwer.com/copyright-blog/allposters-ecj-decision-no-exhaustion-of-rights-in-modifications-of-the-copyright-work/), [IPPT](https://www.ippt.eu/items/ippt20150122-cjeu-allposters-v-pictoright) | A cover licensed for screen display, printed on card stock, is a new reproduction in a new medium | [V2] |

### 2.2 Factors a French court would weigh (my analysis; no decision found)

| Points toward "accessory of the sale" | Points toward "separate merchandising" |
|---|---|
| Exactly one card per eBook licence sold; never sold alone | Charged separately; produced by a third party for profit |
| Card is the delivery vehicle (carries the code), like the box of a download-code product | Designed to stay on a shelf after redemption: decorative function |
| Cover used to identify the product purchased | Cover at or near full size, print resolution, with a spine: it *imitates the book object* |
| Licence says promotion "sur tous supports" / "par tous moyens" | Licence says "on the Site/Store"; print not mentioned (L. 131-3) |

Assessment: with a licence limited to the online store, **printing is outside scope** more likely than not. With "tous supports" promotional wording it becomes defensible. With an express clause it is clean as against the publisher (not necessarily against third-party art owners, §3).

---

## 3. Who owns what in a cover

| Layer | Usual owner | Typical limit | Effect on the card | Conf. |
|---|---|---|---|---|
| Layout / typographic composition | Publisher (in-house, possibly œuvre collective) or freelance designer if original | — | Publisher consent usually suffices | [V2] ([Guichet](https://www.guichetdusavoir.org/question/voir/45700), [Pierrat](https://www.village-justice.com/articles/tirer-couverture,54716.html)) |
| Commissioned illustration | Illustrator; publisher is assignee | Cass. 1re civ., 2 July 2014, n° 13-24.359 (Magnard-Vuibert): commission + assignment of the reproduction right, not a contrat d'édition; **publisher must prove the assignment and its terms**; there, only purchase orders and invoices — [Le Monde du Droit](https://www.lemondedudroit.fr/decryptages/9965-arret-magnard-vuibert-exclusion-du-regime-du-contrat-dedition-pour-la-contribution-accessoire-une-oeuvre-composite.html), [Livres Hebdo](https://www.livreshebdo.fr/article/tirer-la-couverture) | Assignment often covers "the book and its promotion" only; a publisher's consent to a card may exceed what it holds | [V2] |
| Stock image (royalty-free) | Agency / contributor | Standard licences: printed books capped at **500,000 copies**; **standalone products/merchandise need an Extended licence** — [Depositphotos](https://help.depositphotos.com/hc/en-us/articles/33734087656466-Image-Licensing-for-Authors-Publishers), [Adobe forum](https://community.adobe.com/t5/stock-discussions/use-of-adobe-stock-photo-on-book-e-book-cover/m-p/8940006), [ScribeCount](https://scribecount.com/author-resource/protecting-books/book-cover-rights-for-indie-authors-do-you-really-own-the-cover-you-paid-for) (ScribeCount contradicts the cap for some vendors) | Licence is personal to the publisher and not sub-licensable to a retailer [M]; a paid decorative card can be read as merchandise | [V2] |
| Rights-managed photo (Arcangel, Trevillion, Getty RM…) | Agency / photographer | Licensed per use: format, territory, print run, term | Card almost certainly outside the licence | [M] |
| Film / TV tie-in art, licensed characters | Studio / licensor | Edition-specific, time-limited; actors' image rights (C. civ. art. 9) | **Exclude by default** | [M] |
| Fine-art reproduction | Artist / estate, often via ADAGP | Per-use tariff | Exclude unless cleared | [M] |
| Publisher / imprint / collection logo | Publisher (trade marks) | §5 | Fine inside the untouched cover; not separately | [M] |

An ActuaLitté headline in the results ("[Photo du tournage d'À bout de souffle : la justice blanchit Flammarion](https://actualitte.com/article/132591/droit-justice/photo-du-tournage-d-a-bout-de-souffle-la-justice-blanchit-flammarion)") shows that litigation over photographs used by publishers does occur; I did not get the article's content.

---

## 4. Moral rights (droit au respect de l'œuvre et du nom, CPI L. 121-1)

- Having permission to use a work does not dispense from respecting moral rights — [APIE / economie.gouv](https://www.economie.gouv.fr/node/40472) [V1].
- Case law treats **cropping, mirrored reproduction, colourisation and cut-outs (détourage)** as infringements; a photo may not be cropped, altered or used in a montage without the author's express consent — [Livres Hebdo](https://www.livreshebdo.fr/article/pop-et-droit-moral), [practical note](https://www.alsace.catholique.fr/wp-content/uploads/sites/14/2016/04/notions-cle-droit-image.pdf) [V2].
- In the "pop" case reported by Livres Hebdo, a reproduction that did not present the work in its integrity, omitted the signature and changed the format cost **€15,000** in damages — [Livres Hebdo](https://www.livreshebdo.fr/article/pop-et-droit-moral) [V1].
- Not automatic: in a Bordeaux judgment of 29 Feb 2024 claimants invoking cropping and missing credit lost on all claims (grounds not visible) — [Dalloz](https://www.dalloz-actualite.fr/document/tj-bordeaux-29-fevr-2024-n-2308086) [V1]; and courts sanction "abus notoire" of post-mortem moral rights — [Lextenso](https://www.labase-lextenso.fr/l-essentiel-droit-de-la-propriete-intellectuelle/2026-n2/abus-notoire-du-droit-moral-post-mortem-DPI203n9) [V1].
- Moral rights are inalienable: a publisher cannot waive them for the illustrator [M].

| Operation on the cover | Moral-right exposure |
|---|---|
| Proportional scaling, CMYK conversion | Low |
| Cropping to fit a panel, trimming for bleed inside the artwork | **High** — add bleed *outside* the image |
| Overprinting QR / code / "cadeau" ribbon on the cover | **High** |
| Extending the artwork onto the spine (stretch, mirror, generative outpainting) | **High** — alteration, and an adaptation |
| Spine as a separate panel: plain colour, title retyped in own font | Low for copyright (mind §5) |
| Dropping the credit (usually on the back cover or copyright page, not the front) | Medium — print the credit; ONIX List 160 code 01 carries it [V2] |

---

## 5. Trade marks and trade dress — from memory, searches not run

- Publisher, imprint and collection names ("Folio", "Le Livre de Poche", "Pocket", "10/18"…) are registered trade marks, typically including class 16 (printed matter) [M].
- A reseller may use a mark **referentially** to designate the genuine product (Directive 2015/2436 art. 14(1)(c); CPI L. 713-6; BMW v Deenik C-63/97) provided the use follows honest practices and does not suggest a commercial link [M]. The logo *as part of the unaltered cover* fits this. Dior v Evora adds the limit of serious damage to reputation [V2].
- Lifting the logo out of the cover and placing it on a generated spine or back panel is trade-mark use on a printed product not made by the publisher — likely infringement, and it says "official publisher object" when it is not [M].
- Collection trade dress (Gallimard's "Blanche": cream ground, red/black rules, NRF monogram) can be defended through copyright, figurative marks and **parasitisme** (C. civ. art. 1240) [M]. A generated spine that mimics the collection's real spine is riskier than a neutral one. No case law verified.
- Titles: an original title is protected (CPI L. 112-4) [M]; retyping the title to designate the book is referential and low-risk.

---

## 6. Precedents and analogies

| Precedent | How rights were handled | Conf. |
|---|---|---|
| **Enthrill** (Canada, 2013–16): eBook gift cards in Walmart Canada, Sears, Safeway, etc.; 3,000+ locations in 2015 | **Publisher agreements** (HarperCollins Canada, Scholastic, Harlequin, Dundurn…); publishers add titles to Enthrill's catalogue and pay ~5 % of digital list price per code (min $0.25, max $1.50); authors may print their own cards from codes — [Publishers Weekly](https://www.publishersweekly.com/pw/print/20140922/64046-enthrill-partners-with-walmart-canadian-publishing-2014.html), [The Book Designer](https://www.thebookdesigner.com/e-books-for-real-e-book-gift-cards-from-livrada-and-enthrill/), [Build Book Buzz](https://buildbookbuzz.com/e-book-gift-cards/) | [V2] |
| **Livrada** (title-specific cards at Target, 2012) | Covered alongside Enthrill in [The Book Designer](https://www.thebookdesigner.com/e-books-for-real-e-book-gift-cards-from-livrada-and-enthrill/); rights mechanics not retrieved | [V1]/[M] |
| **Walmart × Kobo "digital book cards"** (2018, ~40 novels) | Curated title list — implies per-title publisher participation (inference) — [Boing Boing](https://boingboing.net/2018/08/22/walmarts-selling-ebooks-now.html/amp) | [V1] |
| **Amazon book-specific Kindle cards** (2016) | Existence only — [The eBook Reader](https://blog.the-ebook-reader.com/2016/01/22/kindle-cards-new-book-specific-kindle-gift-cards) | [V1] |
| Game "code in a box", platform POSA cards | Made or approved by the publisher/platform under brand guidelines; retailers do not self-produce title art | [M] |
| Music / audiobook download cards (Dropcards) | Ordered by the label, artist or author — the rightsholder — [The Book Designer](https://www.thebookdesigner.com/?p=36235) | [V1]/[M] |
| PLV, printed retailer catalogues | PLV is supplied by the publisher; chain catalogues reproduce covers within publisher-funded co-op advertising | [M] |

Reading: every title-specific physical card precedent I could find was **rightsholder-initiated or rightsholder-opted-in**. I found none where a retailer unilaterally printed cover art under its general promotional licence. French precedents (cards sold in bookshops) could not be searched.

---

## 7. Risk-rated list: what the reseller can hand to a print service

Scenarios — **A**: express publisher authorisation for the printed card. **B**: only the existing "promotion and sale" licence. **C**: no licence (cover scraped or user-supplied).

| # | Element | A | B | C | Notes |
|---|---|---|---|---|---|
| 1 | Title, author name, publisher name as **plain retyped text**; ISBN | Low | Low | Low | Facts / referential use. Do not suggest endorsement |
| 2 | **Front cover as supplied** (uncropped, unaltered, credited) | Low–Med | **Med** | **High** | A is not "Low" because of third-party art (§3). B hinges on "tous supports" wording and the prime rule |
| 3 | Front cover cropped / overprinted / extended onto the spine | Med | **High** | **High** | Moral rights + adaptation; publisher cannot waive |
| 4 | Generated spine: neutral background, retyped title, own-licensed font | Low | Low–Med | Low–Med | Rises if it imitates the collection's trade dress or cover lettering |
| 5 | **Back-cover blurb** (texte de quatrième) from the feed | Low | Med | High | Protected text; supplied for store display. Prefer own short text or none |
| 6 | **Publisher / collection logo** used separately (spine, back) | Med | **High** | **High** | Trade-mark use on printed matter; needs express consent |
| 7 | **Extract** of the book printed on the card | Med | **High** | **High** | Extract licences target online preview; link to it via QR instead |
| 8 | Author photograph | Med | High | High | Photographer's rights + image rights; avoid |
| 9 | Film tie-in / licensed-character / fine-art covers | **High** unless specifically cleared | High | High | Exclude by default; the publisher flags exceptions |
| 10 | Review quotes | Low–Med | Med | Med | Fine inside the untouched cover; separate reuse needs care |
| 11 | Reseller's own logo; QR code; redemption code | Low | Low | Low | "QR Code" is a Denso Wave word mark [M] |

Liability: contrefaçon is civil and criminal (CPI L. 335-2: up to 3 years / €300,000 [M]); good faith is no defence in civil infringement [M]. Reseller (principal), composition service and printer can all be sued. The realistic *commercial* risk for a reseller is less a damages award than a publisher or distributor suspending the catalogue feed.

---

## 8. Clauses to check or add (reseller ↔ publisher / e-distributor)

**Check in the existing agreement**
1. Definition of "éléments de promotion" / "métadonnées" / "matériel promotionnel": are cover, blurb, extract, logos listed?
2. Purpose and **media**: "sur le Site" vs "sur tous supports / par tous moyens"; any mention of print, PLV, catalogues.
3. **Sub-licensing / subcontracting** ("prestataires", "sous-traitants") — needed for the composition service and printers.
4. Right to **resize, crop, adapt**; credit obligations.
5. Prior-approval clause for promotional operations; under a mandat, the agent's instructions on promotion.
6. Clauses on **primes, bundles, gifts, discounts** (loi 2011-590 art. 4 [M]).
7. Trade-mark licence scope and brand guidelines.
8. Takedown delay and the publisher's own rights warranty over supplied material.
9. Territory (where printed cards ship) and term (cards outlive a title's withdrawal).

**Add (amendment or side letter)**
- Express authorisation to reproduce the front cover "sur une carte imprimée ou imprimable remise à l'acheteur d'un exemplaire numérique de l'œuvre, à raison d'une carte par exemplaire vendu", directly or through named categories of subcontractors.
- Integrity rules: no cropping or overprint; spine and inside panels composed outside the cover image; mandatory credit line.
- **Per-title opt-in/opt-out** via a feed flag; default exclusion of tie-in, licensed-character and rights-managed covers.
- Publisher warranty that it holds cover rights for this use, or a duty to flag titles where it does not.
- Whether the publisher supplies the real spine / full-cover file (cleaner than generating one).
- Logo use: yes/no, with guidelines. Blurb use: yes/no.
- Statement that the card is a paid ancillary service (or a publisher-offered premium available to all retailers), with the publisher acknowledging the pricing structure.
- Takedown within 24–48 h for new generation; sell-off rule for cards already printed.

---

## 9. Rights warranty / indemnity wording (reseller ↔ eBook Cadeau)

Draft for counsel, in French (operative language for FR contracts). Limits: an indemnity shifts cost, not liability — rightsholders can still sue eBook Cadeau directly; it is worth the reseller's solvency; and a party that knowingly took part in an infringement may be refused the benefit of the warranty [M].

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

English gist: reseller warrants written publisher authorisation for print via subcontractors (X.2); narrow sub-licence (X.3); the service commits to integrity, credits, own-material spine, 24 h takedown, per-ISBN log (X.4); reseller indemnifies against third-party IP/personality/unfair-competition claims (X.5); carve-outs and reciprocal indemnity for what the service adds — fonts, generated spines (X.6); indemnity outside the general cap, plus insurance (X.7); text-only fallback (X.8). Between professionals, limitation clauses are valid unless they void an essential obligation (C. civ. art. 1170) [M].

---

## 10. Product implications

- Feed/API field per title: `cover_print_authorised` (bool), `credit_line`, `logo_use`, `blurb_use`; default false.
- Three render modes: (1) authorised cover, untouched, with bleed added outside the artwork; (2) cover at reduced size on a neutral ground, for a "promotion tous supports" licence; (3) typographic card, no cover.
- Never crop, overprint or outpaint the cover; QR code and codes go on the inside panels.
- Per-ISBN kill switch and generation log (supports X.4 d–e).
- Price the card as a separate service line; do not bundle it free with the eBook until the prime question is cleared.
- Standalone mode (user uploads a cover): the service actively composes the file, so a hosting safe harbour is doubtful (active role, L'Oréal v eBay C-324/09 [M]) and the private-copy exception does not shelter the service [M]. Covered in the companion notes on independent use.

---

## Sources

Statute and contracts
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
- https://cms.law/fr/fra/legal-updates/le-contrat-d-edition-a-l-ere-du-numerique (appeared in results; not read)

Metadata
- https://es.slideshare.net/bisg/bisg-webcast-onix-for-books-v30-introduction-1787290
- https://hanmoto.com/pub/onix/codelists/onix-codelist-158.htm
- https://home.bic-media.com/onix_info/3-0-2/codelists/onix-codelist-160.htm
- https://repository.data2type.de/ONIX/v_3.0/html/si.List160.html
- https://booknetcanada.ca/blog/2021/02/02/onix-codelists-issue-52-released/
- https://booknetcanada.atlassian.net/wiki/download/attachments/1378909/ONIX_BookProduct_Codelists_Issue_50_Changes.pdf?api=v2
- https://www.syndicat-librairie.fr/le_service_distrimage_de_dilicom
- http://dilicms.centprod.com/file/dilicom/file.do?path=/Conditions+g%C3%A9n%C3%A9rales/CGV+-+Dilicom.pdf

Cover ownership, moral rights
- https://www.guichetdusavoir.org/question/voir/45700
- https://www.guichetdusavoir.org/question/voir/139502
- https://www.village-justice.com/articles/tirer-couverture,54716.html
- https://www.livreshebdo.fr/article/tirer-la-couverture
- https://www.lemondedudroit.fr/decryptages/9965-arret-magnard-vuibert-exclusion-du-regime-du-contrat-dedition-pour-la-contribution-accessoire-une-oeuvre-composite.html
- https://www.lemondedudroit.fr/pi-tic/278-propriete-intellectuelle-industrielle/58593-protection-par-le-droit-dauteur-des-catalogues-dune-maison-de-vente-aux-encheres-mis-en-ligne.html
- https://www.livreshebdo.fr/article/pop-et-droit-moral
- https://www.economie.gouv.fr/node/40472
- https://www.alsace.catholique.fr/wp-content/uploads/sites/14/2016/04/notions-cle-droit-image.pdf
- https://www.dalloz-actualite.fr/document/tj-bordeaux-29-fevr-2024-n-2308086
- https://www.labase-lextenso.fr/l-essentiel-droit-de-la-propriete-intellectuelle/2026-n2/abus-notoire-du-droit-moral-post-mortem-DPI203n9
- https://actualitte.com/article/132591/droit-justice/photo-du-tournage-d-a-bout-de-souffle-la-justice-blanchit-flammarion (headline only)
- https://help.depositphotos.com/hc/en-us/articles/33734087656466-Image-Licensing-for-Authors-Publishers
- https://community.adobe.com/t5/stock-discussions/use-of-adobe-stock-photo-on-book-e-book-cover/m-p/8940006
- https://scribecount.com/author-resource/protecting-books/book-cover-rights-for-indie-authors-do-you-really-own-the-cover-you-paid-for

EU case law
- https://mansfield.bailii.org/eu/cases/EUECJ/1997/C33795.html
- https://cms.law/en/gbr/legal-updates/brands-ecj-cases-parfums-christian-dior-v-evora-bv-official-distribution-networks-parallel-imports-and-the-exhaustion-of-rights
- https://swarb.co.uk/?p=111802
- https://ipkitten.blogspot.com/2015/01/breaking-news-cjeu-says-exhaustion-only.html?m=0
- https://legalblogs.wolterskluwer.com/copyright-blog/allposters-ecj-decision-no-exhaustion-of-rights-in-modifications-of-the-copyright-work/
- https://www.ippt.eu/items/ippt20150122-cjeu-allposters-v-pictoright
- https://www.scl.org/10768-cjeu-sale-of-second-hand-e-books-via-a-website-is-a-communication-to-the-public/
- https://www.charlesrussellspeechlys.com/en/insights/expert-insights/tmt/2020/can-you-resell-an-e-book-online/

Precedents
- https://www.publishersweekly.com/pw/print/20140922/64046-enthrill-partners-with-walmart-canadian-publishing-2014.html
- https://www.thebookdesigner.com/e-books-for-real-e-book-gift-cards-from-livrada-and-enthrill/
- https://buildbookbuzz.com/e-book-gift-cards/
- https://boingboing.net/2018/08/22/walmarts-selling-ebooks-now.html/amp
- https://blog.the-ebook-reader.com/2016/01/22/kindle-cards-new-book-specific-kindle-gift-cards
- https://www.thebookdesigner.com/?p=36235

## Could not verify

Search budget exhausted after 16 successful queries; the following are open.

1. **Reseller-facing terms of French e-distributors** (Eden Livres, Immatériel/7switch, Numilog, Interforum, Hachette) and of Kobo/Fnac: the actual wording on promotional elements, media, sub-licensing. The most important gap; only obtainable from the contracts themselves.
2. **Loi 2011-590 art. 4 (ventes à primes)** and exact article wording — from memory; read on Légifrance. Whether a paid or free card is a "prime" or a disguised discount is untested.
3. **Any French decision** on a retailer reproducing book covers in print (catalogue, PLV) without consent — none found.
4. **Electre** licence terms for cover images; **current Dilicom** terms for covers (only a 2012 SLF notice found).
5. **ONIX**: full List 154 table; whether List 158 has spine / full-cover codes; any usage-constraint element for supporting resources in ONIX 3.1 — not confirmed.
6. **Trade marks / trade dress**: registrations and classes for Folio, Blanche, etc.; French case law on collection trade dress and parasitisme; BMW v Deenik and Directive 2015/2436 citations — all from memory.
7. **CPI article numbers and cases cited from memory**: L. 131-3, L. 122-7, L. 122-5, L. 112-4, L. 335-2, L. 713-6; Fabris/Utrillo (Ass. plén. 1993); C. civ. art. 1989, 1170, 1240; L'Oréal v eBay.
8. **Dior v Evora** copyright paragraphs: holding confirmed via secondary sources; paragraph text not read.
9. **French precedents** for physical eBook cards; **Fnac / 7switch current gifting flows**; **Livrada, Walmart–Kobo, Kindle cards** rights mechanics; **music, audiobook and game-key cards** — not searched or headline only.
10. **Rights-managed agency terms** (Arcangel, Trevillion, Getty) and whether RF "standard" licences classify a paid cover card as merchandise — vendor-specific, unverified. ScribeCount contradicts the 500,000-copy cap for some vendors.
11. **VAT treatment** of the card versus the eBook — from memory.
12. Whether the Cass. 2 July 2014 decision is published or "inédit" — commentary is inconsistent.
