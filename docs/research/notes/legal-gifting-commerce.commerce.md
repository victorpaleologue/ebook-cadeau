# Legal — gifting an eBook: fixed price, VAT, consumer law, payments, data protection, accessibility, labelling

Research notes, 2026-11-12. Angle: commerce, tax, consumer, data. **Not legal advice** — a map of questions and likely answers for a French lawyer and a VAT adviser to confirm.

**Method caveat — read first.** No web search could be run in this session: the shared search budget was already exhausted when this task started (5 queries sent, 5 refused), and direct fetches of `eur-lex.europa.eu` and `www.legifrance.gouv.fr` failed (hosts outside the sandbox allowlist). **Every statement below is therefore [M] = from memory, not verified in this session.** I add a recall grade: **[M-hi]** (text I know well), **[M-med]**, **[M-lo]** (verify before relying on it). No [V1]/[V2] claim exists in this file. Legal texts are cited by official number so they can be checked; I give no URL I did not see, except four that a sibling note obtained from its own searches (marked "sibling").

---

## 0. Bottom line

1. **Gifting does not breach the French fixed eBook price** as long as the buyer pays the publisher's public price. The danger is the **card**: a *free*, professionally printed and mailed card attached to the sale looks like a *prime* (loi 2011-590 art. 4) or an indirect discount. A free print-at-home PDF is low risk. [M-med]
2. **Sell the printed card as an optional, separately priced service line** (opt-in, never pre-ticked), priced at or above cost. This solves price law, VAT rate separation and the "extra payment" rule in one move. [M-med]
3. **VAT**: eBook at the reduced rate (FR 5.5 %, DE 7 %), card and postage at the standard rate (FR 20 %). Keep them as two supplies; a bundled single price risks the *highest* rate on everything (CGI art. 278-0). [M-med]
4. A code for one named eBook is a **voucher** in VAT terms. It is a single-purpose voucher (VAT due at sale) only if the place of supply is known at issue — i.e. **redemption restricted to one Member State**. An EU-wide code is arguably multi-purpose (VAT at redemption, at the redeemer's country rate). [M-med]
5. **Withdrawal**: the buyer keeps a 14-day right until the eBook is actually downloaded, unless the express waiver is collected at checkout. The system needs a *revoke unredeemed code* function. [M-hi on the rule, M-lo on its application to third-party redemption]
6. **Conformity guarantee** is owed to the buyer (the contracting consumer), not automatically to the recipient. Extend it contractually to the code bearer. [M-med]
7. A title-specific code is **not e-money** and not a payment instrument. Do not let eBook Cadeau collect eBook money on behalf of publishers. [M-hi]
8. **GDPR**: the paper card's real advantage — no recipient e-mail, no account needed. With client-side (WASM) PDF generation, print-at-home can process no personal data server-side at all. Postal fulfilment brings recipient name and address, an Art. 14 notice and a printer DPA. [M-hi]
9. **EAA** (in force since 28 June 2025) covers e-books and e-commerce. A micro-enterprise is exempt for services, but resellers are not and will flow WCAG 2.1 AA down by contract. A QR code alone is not accessible: always print a short URL and typed code. [M-hi]
10. **Printed card for French households**: Triman + Info-tri, EPR registration (unique identifier, Citeo or another approved body), printer's name, company identifiers, French language. Shipping envelopes to Germany trigger LUCID registration with no de minimis. [M-med]

---

## 1. Fixed book price

### 1.1 France — loi n° 2011-590 du 26 mai 2011, décret n° 2011-1499 du 10 novembre 2011

Text of the law: WIPO Lex [original](https://www.wipo.int/wipolex/en/legislation/details/13861) and [consolidated to 2018](https://www.wipo.int/wipolex/fr/legislation/details/18453) (sibling URLs, not opened here); legislative file at the [Sénat](https://www.senat.fr/dossierleg/ppl09-695.html) (sibling); trade summary in [Livres Hebdo](https://www.livreshebdo.fr/article/la-loi-sur-le-prix-en-vigueur) (sibling).

| Provision | Content (from memory) | Recall |
|---|---|---|
| Art. 1 | Covers a *livre numérique* that is a work of the mind, sold in digital form and either published in print or printable by its content and composition, "à l'exception des éléments accessoires propres à l'édition numérique" | M-hi |
| Art. 2 | Any person established in France who publishes an eBook for commercial distribution in France must set a public sale price "pour tout type d'offre à l'unité ou groupée"; the price "peut différer en fonction du contenu de l'offre et de ses modalités d'accès ou d'usage". Exception for collective/professional licences (research, higher education) | M-hi |
| Art. 3 | That price "s'impose aux personnes proposant des offres de livres numériques aux acheteurs situés en France" — extraterritorial, keyed to the **buyer's** location | M-hi |
| Art. 4 | "Les ventes à primes de livres numériques ne sont autorisées (…) que si elles sont proposées par l'éditeur (…) simultanément et dans les mêmes conditions à l'ensemble des personnes mentionnées à l'article 3" | M-med |
| Art. 5 | Publisher's trade discount must reflect the retailer's qualitative services | M-hi |
| Décret 2011-1499 | Defines "contenu de l'offre" (all or part of one or more eBooks plus associated features), "modalités d'accès" (download, streaming, removable medium) and "modalités d'usage" (private/collective, duration, printing/copying/transfer rights) | M-med |
| Décret n° 2012-146 du 30 janvier 2012 | Breach = contravention de 3e classe (max €450, ×5 for legal persons), per offence | M-lo |
| Loi n° 2014-344 (Hamon) | Created the *médiateur du livre*; sworn agents can record breaches. In February 2015 the mediator held that unlimited-subscription offers priced by the platform breached the 2011 law — proof that novel offers are scrutinised | M-med |

Differences from print (loi n° 81-766, "loi Lang"): **no 5 % retailer discount** for eBooks — the price is a floor and a ceiling [M-med]. Loi Lang art. 1 lets a retailer add "les frais ou rémunérations correspondant à des prestations supplémentaires exceptionnelles expressément réclamées par l'acheteur et dont le coût a fait l'objet d'un accord préalable" [M-hi]; loi Lang art. 6 bans primes unless offered by the publisher to all retailers [M-hi]. I do not recall an equivalent "extra services" sentence in the 2011 law [M-lo], but the logic transfers: a service the buyer expressly asks and pays for is not part of the book price.

What is a *prime*: a product or service given **free** with a purchase (former Code de la consommation art. L121-35, now L121-19) [M-med]. General consumer law has allowed primes since 2011 unless they are an unfair practice; the book-specific ban is a stricter special rule. Its compatibility with Directive 2005/29 is arguable after CJEU *Mediaprint* (C-540/08, 2010 — Austrian ban on bonuses with newspapers precluded) [M-med]; do not build on that argument.

### 1.2 Scenarios for the card

| Scenario | Reading | Risk |
|---|---|---|
| Buyer pays the public price; recipient downloads | Price paid in full; the law does not care who reads | None |
| Free **print-at-home PDF** | A presentation of the delivered code, negligible value; comparable to free gift wrap, which bookshops give without being accused of primes | Low |
| Free **printed and mailed card** (cost €2–7) | Free good with market value tied to the sale → *prime* under art. 4, unless the *publisher* offers it to all retailers on the same terms; alternatively an indirect discount | Medium–high |
| **Paid optional card**, own price line, ≥ cost | Separate service expressly requested; not a prime (primes are free) | Low |
| Paid card far below cost | Disguised-discount argument by a competitor, the SNE/SLF or the mediator | Medium |
| Publisher defines a "gift offer" with its own public price (art. 2 allows prices to vary by access modality) | Lawful if that price binds every retailer | Low; needs publisher action |
| Free or cheap **shipping of the card** | Not caught by the book shipping floor (§1.3) | Low |
| Author/publisher **gives away** codes (festival, press) | No sale, so no price to respect. A publisher *selling* direct must charge its own public price | Low |
| Self-published author sells card + eBook at a fair | Author = publisher if established in France; eBook part must equal the price shown at all retailers; show the split on the receipt | Low |
| Buyer abroad, recipient in France | Art. 3 is keyed to the buyer; French price arguably not binding. Publisher territorial rights still apply | Unclear |

### 1.3 Loi "Darcos" — loi n° 2021-1901 du 30 décembre 2021

Amended loi Lang art. 1: delivery of a **printed** book may not be free unless collected in a bookshop, and must be charged at a minimum set by order. Arrêté du 4 avril 2023: **€3 minimum** for orders containing less than €35 of new books, €0.01 above; in force 7 October 2023 [M-hi]. Amazon's challenge went from the Conseil d'État to the CJEU in 2024 (case number recalled as C-366/24) [M-lo]; outcome not known to me.

Effect on this project: **none on the eBook or on the card alone**. The card is not a *livre* (no work reproduced; fiscal definition of a book not met). The floor applies only if a parcel also contains new printed books — then the retailer's ordinary book-shipping rule governs the parcel. I do not recall the Darcos law changing the 2011 eBook price law [M-lo].

### 1.4 Germany — Buchpreisbindungsgesetz (BuchPrG)

| Point | Content | Recall |
|---|---|---|
| § 2(1) Nr. 3 (since 1 Sept 2016) | Covers products that reproduce or substitute books, "wie zum Beispiel zum dauerhaften Zugriff angebotene elektronische Bücher" | M-hi |
| § 3 | Whoever sells commercially to end buyers **in Germany** must charge the fixed price — applies to foreign sellers since 2016 | M-hi |
| § 5 | Publisher sets the price, may set different prices per edition/format | M-hi |
| § 7(4) | No breach if the retailer, on the occasion of a book sale, gives goods of low value, bears shipping or special procurement costs, or provides "andere handelsübliche Nebenleistungen" | M-med |
| Case law | BGH 23 July 2015, I ZR 83/14 (*Gutscheinaktion beim Buchankauf*): vouchers that fund price-bound books breach § 3. Trade practice treats about 2 % of the book price as "low value" | M-med / M-lo |

Germany is therefore **more permissive on the card** than France: gift wrapping is the textbook *handelsübliche Nebenleistung*, and a low-value insert is expressly allowed. A printed, mailed card worth more than the eBook is not "low value"; charge for it.

### 1.5 Other EU regimes (brief, all [M-lo] unless noted)

| Country | Fixed price for print | eBooks covered? |
|---|---|---|
| Austria | Yes (BPrBG, recast 2023) | Yes, since 2014; max 5 % retailer discount |
| Spain | Yes (Ley 10/2007, max 5 % discount) | Yes in principle — the statutory definition of *libro* includes digital |
| Italy | Yes (Legge 128/2011 "Levi", Legge 15/2020: max 5 %) | No — eBooks outside the law |
| Netherlands | Yes — **not abolished** (Wet op de vaste boekenprijs, 2005) [M-med] | No |
| Belgium | Flanders (decree of 2016, in force 2017) and Wallonia-Brussels (décret du 19 octobre 2017) | Wallonia-Brussels: yes; Flanders: not sure |
| No fixed price | UK, IE, SE, FI, DK, PL, CZ, CH (abolished/rejected) | — |

---

## 2. VAT

### 2.1 Rates (as recalled, to verify for 2026)

| Supply | FR | DE | Others |
|---|---|---|---|
| eBook (download) | 5.5 % — CGI art. 278-0 bis A, books "y compris ceux fournis par téléchargement" [M-med] | 7 % since 18 Dec 2019, § 12(2) Nr. 14 UStG [M-hi] | BE 6, NL 9, ES 4, IT 4, AT 10, LU 3, PT 6, PL 5, SE 6, IE 0 (since 2024), DK 25 [M-med/lo] |
| Printed card, envelope, postage charged to the customer | 20 % | 19 % | BE 21, NL 21, ES 21, IT 22, AT 20, LU 17 [M-hi] |
| Fee for generating a print-at-home PDF | Electronically supplied service, standard rate | same | same |

Legal basis for reduced eBook rates: Directive (EU) 2018/1713, after CJEU C-479/13 *Commission v France* (2015) had condemned the earlier French 5.5 % [M-hi]. Postage charged by the supplier is part of the taxable amount of the goods (VAT Directive art. 78) [M-hi]. A card is not a "book" for VAT anywhere I know of.

### 2.2 One supply or two?

- CJEU *Card Protection Plan* (C-349/96, 1999): each supply is normally distinct; an element is ancillary when it is not an aim in itself for the customer but a means of better enjoying the principal supply. *Levob* (C-41/04, 2005): elements so closely linked that splitting them is artificial form one supply. *Stadion Amsterdam* (C-463/16, 2018): one supply takes one rate [M-hi].
- France codified this in CGI art. 257 ter and **art. 278-0**: a single operation made of non-ancillary elements at different rates bears the **highest rate** [M-med].
- Consequences: (i) a free print-at-home PDF is ancillary — whole price at 5.5 %; (ii) an *optional, separately priced* printed card is a distinct supply at 20 %; optional purchase and separate pricing are strong indicators, not conclusive; (iii) a compulsory "gift pack" at one price risks 20 % on the eBook too. A taxpayer could argue a paid card is ancillary to the eBook (5.5 % on the card); conservative position is 20 %.

### 2.3 Vouchers — Directive (EU) 2016/1065 (VAT Directive arts. 30a, 30b, 73a; vouchers issued from 1 Jan 2019)

| Term | Definition / rule | Recall |
|---|---|---|
| Voucher | Instrument carrying an obligation to accept it as consideration for a supply, where the goods/services or the potential suppliers are indicated on it or in its terms | M-hi |
| Single-purpose (SPV) | Place of supply **and** VAT due are known at issue. Each transfer by a taxable person acting in its own name is treated as the supply itself; redemption is not a taxable event | M-hi |
| Multi-purpose (MPV) | Any other voucher. VAT only at redemption; taxable amount = price paid for the voucher (art. 73a) | M-hi |
| Transposition | FR: CGI art. 256 ter (*bon à usage unique / à usages multiples*). DE: § 3(13)–(15) UStG (*Einzweck-/Mehrzweck-Gutschein*) | M-med |
| CJEU C-68/23 *M-GbR* (18 April 2024) | Prepaid codes for digital content limited to users in Germany are SPVs: the "place of supply" test looks at the final supply to the end user, and a territorial restriction in the terms makes it known | M-med |

Application: a code for one named eBook has a known rate **only if the redeemer's country is fixed**, because electronically supplied services to consumers are taxed where the customer is (VAT Directive art. 58). So:

| Design | VAT treatment | Operational effect |
|---|---|---|
| Code redeemable only by users located in France (terms + location check) | SPV → FR 5.5 % due at sale | Simple; VAT is due even on never-redeemed codes |
| Code redeemable EU-wide | Arguably MPV → VAT at redemption in the redeemer's Member State, via OSS | Needs location evidence at redemption; breakage untaxed; price received is fixed while the rate varies |
| Treat the sale as an immediate supply of the eBook to the **buyer** with deferred download | VAT at payment, buyer's country | Same result as SPV for domestic gifts; defensible alternative reading |

The geo-blocking Regulation (EU) 2018/302 does not prevent a territorial restriction: art. 4(1)(b) excludes services whose main feature is access to copyright works, eBooks included [M-hi].

### 2.4 Which customer — buyer or recipient?

Outside the voucher rules, the VAT "customer" is the party to the legal relationship who pays: the **buyer** [M-med]. Location evidence follows Implementing Regulation (EU) 282/2011 arts. 24a–24f: two non-contradictory items (billing address, IP, bank country…), or one item if such cross-border supplies stay under €100,000 a year [M-med]. Under an MPV reading it is the **redeemer**. This buyer/recipient split is the main open tax question for cross-border gifts; get a written position from a VAT adviser before selling outside France.

### 2.5 Cross-border mechanics

- **€10,000** EU-wide annual threshold for cross-border B2C digital services plus intra-EU distance sales of goods (art. 59c); below it a supplier established in one Member State may charge home VAT; above it destination VAT through the **Union OSS** (since 1 July 2021), records kept 10 years [M-hi].
- A printed card mailed to a consumer in another Member State is an intra-EU distance sale of goods (destination standard rate above the threshold) [M-hi].
- French *franchise en base* (micro-entreprise): no VAT charged below national thresholds (figures disputed in the sibling market note); it does not shelter supplies located in other Member States beyond the €10,000 threshold, subject to the EU SME scheme applicable since 1 Jan 2025 [M-lo].
- **Art. 9a of Regulation 282/2011**: whoever "takes part" in a supply of electronic services through a portal is presumed to be the supplier unless the real provider is explicitly shown as such and the intermediary neither authorises the charge or delivery nor sets the terms [M-hi]. If eBook Cadeau sold eBooks for authors it would become the deemed VAT supplier. **Standalone mode should sell only the card service**, never the eBook.
- Service fee invoiced to a reseller: ordinary B2B service (FR 20 %, or reverse charge to an EU business customer, art. 44) [M-hi].

---

## 3. Consumer law

### 3.1 Right of withdrawal

| Rule | Source | Recall |
|---|---|---|
| 14 days from conclusion for digital content not on a tangible medium | Directive 2011/83 art. 9(2)(c) | M-hi |
| Right lost when performance has begun **and** the consumer gave prior express consent, acknowledged the loss, **and** the trader confirmed this on a durable medium | Directive 2011/83 art. 16(m) as amended by Directive 2019/2161; Code de la consommation art. L221-28 13° | M-hi |
| If the waiver is defective the consumer may withdraw and owes nothing | art. 14(4)(b) | M-hi |
| Missing withdrawal information extends the period by 12 months | art. 10 | M-hi |
| No withdrawal for goods "confectionnés selon les spécifications du consommateur ou nettement personnalisés" | art. 16(c); L221-28 3° | M-hi |
| Online **withdrawal function** ("button") required for distance contracts concluded through an online interface, from 19 June 2026 | Directive (EU) 2023/2673 inserting art. 11a | M-med |
| No pre-ticked boxes for extra payments | Directive 2011/83 art. 22; L121-17 | M-hi |

Gift specifics (my analysis, no authority found):

- The **buyer** is the consumer; the recipient has no withdrawal right.
- "Performance has begun" is generally read as the start of the download. Handing over a code is not that. So for 14 days an **unredeemed** code can be withdrawn → the platform must be able to revoke a code and refund. A printed card may already be in the post; state in the terms that withdrawal voids the code.
- For redemption inside the 14 days, collect the waiver from the buyer at checkout, worded to cover redemption "par vous ou par la personne à qui vous remettez le code", and repeat it in the confirmation e-mail. Whether a third party's download counts as performance begun at the consumer's request is untested [M-lo].
- A card bearing a personal message or name is personalised: no withdrawal for the card itself. A generic card with only a unique code is doubtful.

### 3.2 Conformity of digital content — Directive (EU) 2019/770

French transposition: ordonnance n° 2021-1247, Code de la consommation arts. L224-25-1 et seq., in force 1 Jan 2022 [M-med].

- Supply is made when the content "or any means suitable for accessing or downloading" it is made available to the consumer or to a facility chosen by the consumer (art. 5) — delivering the code may count as supply, so the redemption service's availability becomes a conformity matter [M-hi].
- Trader bears the burden of proving supply (art. 12(1)); liability 2 years for a single supply, burden of proof on the trader for 1 year (arts. 11–12); remedies: bring into conformity, price reduction, termination with refund within 14 days (arts. 14–18) [M-hi].
- **Owed to whom**: the contracting consumer. The recipient is a third party unless (a) the terms extend the guarantee to the bearer (*stipulation pour autrui*, Code civil art. 1205), or (b) the recipient enters a contract at redemption [M-med]. Recommendation: state that "le porteur du code bénéficie des mêmes garanties que l'acheteur", and let either of them open a support case with only the code.
- Reseller mode: the reseller is the liable trader; redemption downtime at eBook Cadeau is the reseller's failure to supply → needs an SLA and an indemnity in the B2B contract.

### 3.3 Pre-contractual information

Directive 2011/83 art. 6(1)(r)–(s); Code de la consommation arts. L111-1 and L221-5: functionality including technical protection measures, and relevant compatibility and interoperability with hardware and software [M-hi]. The buyer usually does not know the recipient's device, so the information must be on the product page **and** in short form on the card: format (EPUB/PDF), protection (none / watermark / LCP / Adobe DRM), "lisible sur…", "non lisible directement sur Kindle" where true.

### 3.4 Expiry and unredeemed codes

| Country | Rule for vouchers | Recall |
|---|---|---|
| France | No statutory minimum validity; the issuer sets it, with clear prior information. Expired = lost, in principle. Limits: unfair-terms control (L212-1; R212-1 blacklists clauses that bind the consumer while the trader does not perform); 5-year general limitation period (Code civil art. 2224) | M-med |
| Germany | Limitation 3 years from the end of the year of purchase (§§ 195, 199 BGB); shorter expiry in standard terms is generally void (§ 307 BGB; OLG München 17 Jan 2008, 29 U 3193/07, Amazon one-year limit) | M-med |
| Netherlands | Minimum 2 years (since 2022) | M-lo |
| Ireland | Minimum 5 years (Gift Vouchers Act 2019) | M-med |

A code for a fully paid, named eBook is closer to a deferred delivery than to stored value: an expiry that lets the seller keep the price without delivering is exposed under unfair-terms law. Recommendation: **no expiry, or at least 5 years**, and a refund-or-substitute rule if the title is withdrawn. The reseller–publisher contract must allow delivery of sold-but-unredeemed copies after a title leaves the catalogue, and must say when the sale is reported for royalties (at purchase, not redemption). Breakage levels: see sibling `market-business.market.md` §6.

Other items: consumer mediator must be named in the terms (L612-1) [M-hi]; the EU ODR platform closed on 20 July 2025 (Regulation (EU) 2024/3228) — remove old ODR links [M-med]; contracts concluded online for €120 or more must be archived 10 years (L213-1) [M-med].

---

## 4. Payments and e-money

- **E-money** (Directive 2009/110/EC art. 2(2)): electronically stored *monetary value*, a claim on the issuer, issued on receipt of funds *to make payment transactions*, and accepted by someone *other than the issuer* [M-hi]. A code for one named eBook stores no monetary value, makes no payment, and in reseller mode is accepted only by its issuer. It fails the definition three times; the limited-network exclusion is not even needed.
- **PSD2 art. 3(k)** excludes instruments usable only at the issuer's premises or in a limited network, or for a very limited range of goods or services; art. 37(2) requires notification to the regulator above €1 million of transactions in 12 months (France: ACPR; Code monétaire et financier art. L521-3 and following) [M-hi; article numbers M-lo]. EBA guidelines EBA/GL/2022/02 narrow the exclusion [M-med]. Store-value gift cards rely on it; a title-specific code does not need it.
- **Where the risk would appear**: (i) a code carrying a euro value or usable for any title; (ii) a code accepted by several independent sellers with eBook Cadeau settling between them; (iii) eBook Cadeau collecting buyers' money for publishers (holding third-party funds = payment service; the commercial-agent exclusion in art. 3(b) is narrow). Avoid all three; if marketplace payments ever become necessary, use a licensed PSP's split-payment product.
- PSD3 / Payment Services Regulation: political agreement late 2025 [M-lo]; status in November 2026 not checked.

---

## 5. The upload-and-QR scheme (commerce-side remarks; copyright detail is in the sibling notes)

- An individual who uploads an eBook they *bought* so that someone else can download it makes a reproduction and a communication outside the licence. There is no exhaustion for downloads (CJEU *Tom Kabinet*, C-263/18, 2019) and the private-copy exception (CPI art. L122-5 2°) covers only copies for the copier's own private use [M-hi]. **Uploads must be limited to rights holders** (authors, publishers), public-domain or freely licensed works.
- For individuals, the lawful path is a "bring your own code" card: the QR carries a gift code or link bought from a retailer; eBook Cadeau never touches the file.
- When files are hosted: hosting provider under the Digital Services Act (Regulation (EU) 2022/2065) — liability shield if passive (art. 6), notice-and-action mechanism (art. 16), contact points; platform-level duties do not apply to micro and small enterprises (art. 19) [M-hi]. Uploader warranty, takedown, redemption caps per file.
- Not selling the eBook also keeps eBook Cadeau out of DAC7 platform reporting (Directive (EU) 2021/514) and art. 9a deemed-supplier status [M-med].

---

## 6. GDPR

| Flow | Personal data | Role of eBook Cadeau | Legal basis | Retention (proposal) |
|---|---|---|---|---|
| Print-at-home, PDF built in the browser | None server-side if the code is pre-issued; otherwise code + timestamp | Controller (standalone) / processor (reseller) | — / contract | — |
| Buyer account, order, invoice | Name, e-mail, billing data | Controller or processor | Art. 6(1)(b) contract; 6(1)(c) for accounting | Invoices 10 years (Code de commerce art. L123-22) |
| Postal fulfilment to the recipient | Recipient name + postal address, gift message | Controller/processor; printer = (sub-)processor | Art. 6(1)(f) — recipient is not a party, so 6(1)(b) is unavailable | Delete at printer ≤ 30 days; at service after delivery-claim window (≈ 90 days) |
| Redemption | Code, timestamp, IP, user agent; e-mail only if the reader chooses "send to my e-reader" | idem | 6(1)(f) fraud prevention and proof of supply; 6(1)(c) for VAT location evidence | Security logs 6–12 months (CNIL logging guidance); proof of supply 5 years; VAT/OSS evidence 10 years, pseudonymised |

Points that matter:

- **Art. 14** (data not obtained from the data subject): when the buyer gives a recipient's address, the recipient must be informed at the latest at first contact — print a two-line notice on the delivery slip or card back (who supplied the address, purpose, deletion delay, contact) [M-hi].
- **E-mail gifting** (the alternative) needs the recipient's e-mail, an Art. 14 notice in the message, and care not to turn a gift notice into marketing. The paper card avoids all of this.
- **Data minimisation / by design** (arts. 5(1)(c), 25): redemption without an account; store a hash of the code; no third-party trackers on the redemption page (then no consent banner is needed under art. 82 of the loi Informatique et Libertés) [M-med].
- **Reading choices can reveal sensitive data.** CJEU C-184/20 (2022) and C-21/23 *Lindenapotheke* (2024) treat data from which health or orientation can be inferred as Art. 9 data [M-med]. Do not profile on titles; do not tell the buyer when or whether the gift was redeemed unless the recipient agrees.
- **Processors**: Art. 28 contract with every printer and mail house; list of sub-processors; reseller mode = reseller controller → eBook Cadeau processor → printer sub-processor (prior authorisation).
- **Transfers**: prefer EU/EEA printers and hosting. UK and Switzerland have adequacy decisions (UK renewed late 2025 [M-lo]); US vendors need the Data Privacy Framework (challenge dismissed by the General Court, T-553/23, 3 Sept 2025; appeal pending [M-lo]) or SCCs.
- **Minors**: Art. 8 applies only to consent-based processing by an online service offered directly to a child (age 15 in France, 16 in Germany) [M-hi]. Avoid consent as a basis at redemption. A minor can accept a gift and perform everyday acts (Code civil art. 1148) [M-med].
- Record of processing (Art. 30) required; DPO and DPIA probably not, at this scale.

---

## 7. European Accessibility Act — Directive (EU) 2019/882

| Point | Content | Recall |
|---|---|---|
| Scope | Services: "e-books and dedicated software" (art. 2(2)(e)) and "e-commerce services" (art. 2(2)(f)); products: e-readers | M-hi |
| Date | Applies to products placed and services provided after **28 June 2025**; existing service contracts and legacy set-ups tolerated until 28 June 2030 (art. 32) | M-hi |
| Micro-enterprise | Fewer than 10 staff **and** turnover or balance sheet ≤ €2 million: exempt for services (art. 4(5)) | M-hi |
| Other escape | Disproportionate burden or fundamental alteration, with a documented assessment (art. 14) | M-hi |
| Requirements | Annex I: websites and apps perceivable, operable, understandable, robust (in practice EN 301 549 → WCAG 2.1 AA); accessible identification, security and payment functions; information about the service in accessible form. For eBooks: no blocking of assistive technology, structure and navigation, reflow, **DRM must not block accessibility features**, accessibility metadata | M-hi |
| France | Loi n° 2023-171 du 9 mars 2023 (art. 16), décret n° 2023-931 du 9 octobre 2023; control split between DGCCRF, Arcom and others. Backlist eBooks: compliance deadline recalled as 2030 | M-med / M-lo |
| Germany | Barrierefreiheitsstärkungsgesetz (BFSG), same date | M-hi |

For this project:

- **Redemption site** = part of an e-commerce service and the gateway to an eBook. Build to WCAG 2.1 AA regardless of the micro-enterprise exemption: resellers are in scope and will require it; publish an accessibility statement.
- **QR is not accessible on its own.** Print a short URL and a typed code in a large, high-contrast face; group the code in blocks; avoid look-alike characters. Any CAPTCHA or rate-limit challenge needs a non-visual alternative.
- The **printed card** itself is outside the EAA (paper), but the same information must exist in accessible form online.
- **eBook files**: publisher/retailer duty. Show the title's accessibility metadata on the redemption page, and do not wrap files in a protection that defeats screen readers.

---

## 8. Mandatory mentions on the card and packaging

### 8.1 On the card (France)

| Mention | Basis | Recall |
|---|---|---|
| What the card gives (title, author, one digital copy), how to redeem, single use, validity, format/DRM, customer-service contact | L111-1, L221-5 (information); good practice | M-med |
| Seller identity; for a registered business on "documents publicitaires": SIREN, "RCS + city", registered office, legal form and capital | Code de commerce arts. R123-237, R123-238 | M-med |
| French language (other languages may be added) | Loi n° 94-665 ("Toubon") art. 2 | M-hi |
| Printer's name and address on any publicly distributed print, except "ouvrages de ville ou bilboquets" (private stationery) | Loi du 29 juillet 1881 art. 3 | M-med |
| **Triman + Info-tri** sorting label | Code de l'environnement art. L541-9-3 (loi AGEC art. 17), décret n° 2021-835 | M-med |
| No "biodégradable" / "respectueux de l'environnement" claims; FSC/PEFC logos only under the printer's licence | AGEC art. 13; scheme rules (see sibling ESG note) | M-med |

Not required: price (better omitted on a gift), ISBN (the card is not a book), legal deposit (commercial jobbing print is exempt [M-lo]). A one-off personalised card given privately is arguably an *ouvrage de ville*; a print run of promotional cards for a publisher is not.

Suggested legal line (to adapt): *« Bon pour un exemplaire numérique de "Titre" (Auteur, Éditeur), format EPUB. À retirer sur exemple.fr/c avec le code ci-dessus. Usage unique. Sans date limite. Vendu par [dénomination, forme, capital, RCS, siège]. Aide : […]. Imprimé par [imprimeur, ville]. »* + Triman/Info-tri pictogram.

### 8.2 EPR in France

- Printed papers other than books issued to households fall under the *papiers graphiques* EPR (Code de l'environnement art. L541-10-1 3° — "à l'exception des livres"); household packaging (the envelope) falls under the packaging EPR. The two schemes were merged by loi n° 2023-305; Citeo is the main approved body (Adelphe, Léko also approved for packaging) [M-med].
- The producer (the brand that issues the card: the reseller in reseller mode, eBook Cadeau or the ordering professional in standalone mode) must join an approved body, obtain a **unique identifier (IDU)** from ADEME and show it in its terms of sale (art. L541-10-13) [M-med]. Very small quantities get a flat-rate declaration; whether the old 5-tonne exemption for papers survives is not known to me [M-lo].
- Missing Triman: administrative fine up to €15,000 per legal person (art. L541-9-4) [M-lo].
- Outlook: the Commission has challenged the Triman as a barrier to trade, and the Packaging Regulation (EU) 2025/40 (general application 12 Aug 2026, harmonised labels later) will displace national packaging labels [M-lo]. Design the label as a swappable template element.

### 8.3 Germany and other Member States

- **VerpackG**: whoever first places filled packaging — shipping envelopes included — on the German market for consumers must register in **LUCID** *before* shipping and join a dual system; no de minimis; applies to foreign distance sellers; fines and a sales ban [M-hi]. The card itself is a product, not packaging; Germany has no statutory EPR for printed paper [M-med].
- Other Member States have their own packaging EPR (several require an authorised representative for foreign sellers) [M-lo].
- Design lever: a **sealed self-mailer** (the card folds and seals to hide the code, no envelope) removes packaging EPR from the picture; only the French paper EPR remains. Otherwise let a destination-country mail house act under its own registrations and confirm this in the contract.

---

## 9. Design decisions that follow

1. Checkout: eBook line at the publisher's price + optional card line (unticked), separate VAT rates on the receipt.
2. Free tier = print-at-home PDF only. Free printed cards only as a publisher-backed offer open to all retailers.
3. Code terms: single title, single use, redemption territory configurable (default France → SPV), no expiry or ≥ 5 years.
4. Code states: issued → activated → redeemed, plus **revoked** (withdrawal, loss, fraud) and **reissued**.
5. Waiver checkbox and durable-medium confirmation at checkout; withdrawal function online.
6. Bearer-benefits clause for the conformity guarantee; support by code alone.
7. Card template slots: legal line, format/DRM line, Art. 14 notice (postal mode), Triman/Info-tri, printer's name.
8. Standalone mode sells the card service only; uploads restricted to rights holders; "bring your own code" for individuals.
9. No recipient data in print-at-home; address data deleted on a timer; Art. 28 contract with each printer.
10. WCAG 2.1 AA redemption page with typed-code path.

---

## Sources

No search result was obtained in this session. URLs below come from the sibling note `legal-covers-reseller.reseller.md` (its own searches); they were not opened here.

- https://www.wipo.int/wipolex/en/legislation/details/13861 — loi n° 2011-590 (original)
- https://www.wipo.int/wipolex/fr/legislation/details/18453 — loi n° 2011-590 (consolidated 2018)
- https://www.senat.fr/dossierleg/ppl09-695.html — legislative file
- https://www.livreshebdo.fr/article/la-loi-sur-le-prix-en-vigueur — trade summary

Primary references cited from memory (official identifiers, no URL; to be read on Légifrance / EUR-Lex / gesetze-im-internet):

- FR: loi n° 2011-590; décret n° 2011-1499; décret n° 2012-146; loi n° 81-766 arts. 1, 6; loi n° 2021-1901; arrêté du 4 avril 2023; Code de la consommation arts. L111-1, L121-17, L121-19, L212-1, R212-1, L213-1, L221-5, L221-28, L224-25-1 s., L612-1; CGI arts. 256 ter, 257 ter, 278-0, 278-0 bis A; Code civil arts. 1148, 1205, 2224; Code de commerce arts. L123-22, R123-237, R123-238; Code de l'environnement arts. L541-9-3, L541-9-4, L541-10-1, L541-10-13; décret n° 2021-835; loi n° 2023-305; loi du 29 juillet 1881 art. 3; loi n° 94-665 art. 2; loi n° 2023-171 art. 16; décret n° 2023-931; CPI art. L122-5.
- DE: BuchPrG §§ 2, 3, 5, 7; UStG § 3(13)–(15), § 12(2) Nr. 14; BGB §§ 195, 199, 307; VerpackG; BFSG; BGH I ZR 83/14; OLG München 29 U 3193/07.
- EU: Directives 2006/112/EC (arts. 30a, 30b, 44, 58, 59c, 73a, 78, 369a s.), (EU) 2016/1065, (EU) 2018/1713, 2011/83/EU (arts. 6, 9, 10, 11a, 14, 16, 22), (EU) 2019/2161, (EU) 2023/2673, (EU) 2019/770, 2009/110/EC, (EU) 2015/2366 (arts. 3(b), 3(k), 37(2)), (EU) 2019/882, (EU) 2021/514, 2005/29/EC; Regulations (EU) 282/2011 (arts. 9a, 24a–24f), (EU) 2016/679, (EU) 2018/302, (EU) 2022/2065, (EU) 2024/3228, (EU) 2025/40.
- CJEU: C-349/96 *CPP*; C-41/04 *Levob*; C-463/16 *Stadion Amsterdam*; C-68/23 *M-GbR*; C-479/13 *Commission v France*; C-540/08 *Mediaprint*; C-263/18 *Tom Kabinet*; C-184/20; C-21/23; T-553/23.

## Could not verify

Everything in this file is unverified in this session. The items most in need of checking, in order of impact:

1. **Loi 2011-590 art. 4** exact wording, and any ministry, mediator or court position on whether a free gift card, gift wrap or shipping accompanying an eBook is a *prime* or a disguised discount. No precedent known to me.
2. Whether the 2011 law or its decree contains an "extra services requested by the buyer" rule like loi Lang art. 1; amount and class of the fine (décret 2012-146); any amendment after 2018, including by the Darcos law.
3. **VAT**: whether tax authorities (BOFiP, BMF) treat a download code for a named title as an SPV, an MPV or a direct sale; buyer-versus-redeemer place of supply for cross-border gifts; exact reasoning of C-68/23; wording of CGI art. 278-0; current reduced rates per Member State for 2026; French *franchise en base* thresholds and the EU SME scheme.
4. Outcome of the CJEU case on the French €3 book-shipping minimum (number recalled as C-366/24).
5. Whether third-party redemption satisfies art. 16(m) of Directive 2011/83; French transposition and practical form of the withdrawal function due from 19 June 2026.
6. Whether French courts would let a gift recipient invoke the digital-content conformity guarantee without a contractual extension.
7. Gift-voucher expiry rules: French case law on expiry clauses; the Dutch 2-year rule; other Member States.
8. PSD3/PSR status in November 2026; exact Code monétaire et financier article numbers for the limited-network exclusion.
9. EAA in France: which authority controls eBooks and e-commerce, penalty amounts, treatment of backlist titles until 2030; status of EN 301 549 v4.
10. **Triman/Info-tri** for a small printed card: size exemptions, exact signage for *papiers graphiques*, the 5-tonne paper threshold, Citeo fees for small issuers, fine amounts; state of the Commission's action against the Triman; PPWR labelling timetable; whether Germany has replaced the VerpackG to implement the PPWR.
11. Whether a commercial gift card needs the printer's name under the 1881 press law, and whether it counts as a *document publicitaire* for Code de commerce mentions.
12. Other fixed-price regimes for eBooks (AT, ES, BE-Flanders, BE-Wallonia, IT, NL): all from memory.
13. Data-transfer status: UK adequacy renewal, appeal against T-553/23; CNIL retention recommendations.
14. Actual practice of French resellers (Fnac/Kobo, 7switch, Cultura, Decitre) for eBook gifting terms, expiry and withdrawal wording — not researched.

For a verification pass: the two hosts tried and refused here were `www.legifrance.gouv.fr` and `eur-lex.europa.eu`. The official sites of BOFiP, the CJEU, the German federal law portal, the CNIL, the DGCCRF, Citeo and the German packaging register (LUCID) would also be needed, or a fresh web-search budget.
