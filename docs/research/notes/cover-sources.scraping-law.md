# Cover (and spine) acquisition — is it lawful? Scraping, hotlinking, TDM, database right (EU/FR, US for comparison)

Research notes, 2026-11-12. Angle: the *acquisition* step only (finding, fetching, storing a cover the user did not supply). Whether a cover may be *printed* at all is treated in the sibling notes `legal-covers-reseller.independent.md` and `legal-covers-reseller.reseller.md` (same folder); this note refers to them instead of repeating them. Not legal advice; it is a map for French IP counsel to confirm.

## 0. Method and limits — read first

**No web search succeeded in this session.** The workflow's shared search budget was already exhausted when this task started: all 5 queries I sent were refused. WebFetch was then tried on four primary-source hosts (`www.rfc-editor.org`, `eur-lex.europa.eu`, `curia.europa.eu`, `www.w3.org`) and all four are outside the sandbox allowlist. I did not work around either limit.

Consequences:

- Everything below is written **from memory** and tagged **[M]** (confident recall of a well-known authority) or **[M?]** (recall I am less sure of: date, number or detail may be off). Nothing here is [V1]/[V2] by my own searching.
- **[V2*] / [V1*]** marks a point that a sibling note verified in its own search results; the linked URLs come from those notes, not from a search of mine.
- I cite authorities by official identifier (case number, date, ECLI, statute article) rather than by URL, because the brief forbids guessed URLs. ECLIs and French decision numbers are from memory.
- A verification pass needs either a fresh search budget or the hosts above plus `www.legifrance.gouv.fr`, `www.cnil.fr`, `www.courdecassation.fr` on the allowlist. Section 13 lists what to check first.

---

## 1. Bottom line

1. "Publicly reachable" is not "free to take". In EU law three separate regimes bite on acquisition: copyright in the image, the sui generis database right of whoever assembled the catalogue, and contract/unfair competition (ToS, *parasitisme*). Each must be cleared separately [M].
2. **robots.txt is not law and not a licence.** RFC 9309 itself says its rules are "not a form of access authorization". In France and Belgium the absence of a `Disallow` grants nothing. But a `Disallow` now has real legal effect as a *prohibition*: it is the standard machine-readable opt-out under DSM art. 4(3), the EU AI Act Code of Practice names it, and the CNIL treats ignoring it as defeating the legitimate-interest basis [M].
3. **The TDM exception (DSM art. 4; CPI L122-5-3 III) authorises copying for analysis only.** It never covers showing the image to a user, keeping it after the analysis, or printing it. It is useful to us for one thing: fetching a cover transiently to derive unprotected facts (dominant colours, font class, a hash) and then deleting it [M].
4. **Linking and copying are treated oppositely.** A link or an inline frame to an image that the rightholder put online without technical restrictions is not a communication to a new public (Svensson, BestWater, VG Bild-Kunst). Downloading the same image and serving it from our own server is (Renckhoff) [M; Renckhoff V2*].
5. A real-time "look up this ISBN on retailer X and show me its cover" feature resembles the dedicated meta search engine condemned in Innoweb; systematic lookups against a French retailer's catalogue also fall under CPI L342-2 and *parasitisme* (Leboncoin v Entreparticuliers) [M].
6. US law is more permissive on both points (hiQ: scraping public pages is not a CFAA offence; Kelly and Perfect 10: thumbnails are fair use) but neither helps a service aimed at France, and even in the US hiQ lost on contract [M; thumbnails V2*].
7. Practical consequence: **no generic cover scraper.** Use an allow-list of sources with a recorded legal basis, hotlink-with-link-back for preview, store only URLs and derived features, never run an image proxy, and print only licensed or user-supplied covers.

---

## 2. Which act engages which right

| Step in a "find the missing cover" pipeline | Right or rule engaged | Usable basis |
|---|---|---|
| Fetch and parse an HTML page or API response to find an image URL | Reproduction (InfoSoc art. 2) of the page; extraction from a database (Dir. 96/9 art. 7); contract (ToS) | TDM art. 4 if access is lawful and no opt-out; insubstantial-part rule for databases; API licence |
| Keep the URL, ISBN, dimensions, hash | None: facts are not protected (subject to database right if done at scale) | — |
| Fetch the image bytes server-side | Reproduction of the cover | Licence, or TDM (temporary, analysis only) |
| Keep the bytes (cache, CDN) | Reproduction, continuing | Licence only; TDM copies must be deleted |
| Show the image from our server | Communication to the public (InfoSoc art. 3) — Renckhoff | Licence only |
| Show it by inline link to the source server | No new public if source is authorised and unprotected — Svensson line | Case law, plus source ToS |
| User's browser renders and caches it | Temporary copy, InfoSoc art. 5(1) (PRCA v NLA "Meltwater", C-360/13) | Statutory exception [M] |
| Embed in the PDF / print | Reproduction on a new medium; distribution if we ship | Licence, or the user's own private copy (see sibling note) |

---

## 3. robots.txt and RFC 9309

**What the standard is.** RFC 9309, "Robots Exclusion Protocol", IETF Proposed Standard, September 2022 (Koster, Illyes, Zeller, Sassman). It standardises the 1994 convention. Points that matter for a compliant fetcher [M]:

- The rules "are not a form of access authorization"; the Security Considerations add that the protocol "is not a substitute for valid content security measures".
- Longest-match rule; on a tie `Allow` wins; user-agent matching is case-insensitive.
- Follow at least five redirects; parse at least the first 500 KiB.
- A 4xx on `/robots.txt` means "no restrictions"; a 5xx or network failure means **assume complete disallow**.
- Do not use a cached copy for more than 24 hours unless the file is unreachable.

**Legal weight in the EU.**

| Question | Answer | Authority | Conf. |
|---|---|---|---|
| Does "no Disallow" grant permission to copy or display? | No in France: assignments must be express and are read narrowly (CPI L131-3). No in Belgium: Copiepresse v Google (CA Bruxelles, 5 May 2011) rejected the "opt-out" theory of copyright. EU level: implied consent needs prior, effective, individual information of the author (Soulier and Doke, C-301/15, 16 Nov 2016). | statute, cases | [M] |
| German exception | BGH Vorschaubilder I (I ZR 69/08, 29 Apr 2010): putting images online without blocking crawlers is "schlichte Einwilligung" to *usual search-engine* thumbnail display. Limited to search engines; not transferable to product imagery. | case | [M] |
| Does a `Disallow` bind a crawler? | Not as such; no EU statute makes robots.txt mandatory. | — | [M] |
| Is it an opt-out from TDM? | Yes, the paradigm "machine-readable means" of DSM art. 4(3) and recital 18. The GPAI Code of Practice (published 10 July 2025, under AI Act art. 53(1)(c)) commits signatories to crawlers that read and follow robots.txt as specified in RFC 9309. | directive, soft law | [M] |
| Must the opt-out be machine-readable? | Directive: recital 18 says so for online content. **France is wider**: CPI R122-28 (décret n° 2022-928 of 23 June 2022) lets the opposition be expressed "par tout moyen", "notamment" by machine-readable means, metadata or a site's terms of use. So in France a plain ToS clause is an opt-out. | regulation | [M] |
| German/Dutch case law | LG Hamburg, 27 Sep 2024, 310 O 227/23 (Kneschke v LAION): dataset creation covered by the research TDM exception; obiter, a natural-language reservation may count as machine-readable. On appeal the OLG Hamburg (5 U 104/24, 10 Dec 2025) upheld dismissal and, as I recall, held the natural-language reservation was not machine-readable at the 2021 copying date. Rb. Amsterdam, 30 Oct 2024 (DPG Media v HowardsHome): a robots.txt naming only some bots was not an effective reservation against another bot. | cases | [M?] |
| Is robots.txt an "effective technological measure" (InfoSoc art. 6) so that ignoring it is circumvention under VG Bild-Kunst? | Probably not: it restricts nothing technically. My analysis; no authority known. | — | [M] |
| Other weight | Ignoring it is evidence of bad faith for *parasitisme* and removes the "reasonable expectations" limb of GDPR legitimate interest (CNIL, section 8). | — | [M] |

Other opt-out signals a fetcher should read: W3C Community Group **TDMRep** (`tdm-reservation` HTTP header, `/.well-known/tdmrep.json`, HTML `<meta name="tdm-reservation">`, also embeddable in EPUB/PDF), driven by the French EDRLab and recommended to French publishers by the SNE [M?]; `X-Robots-Tag` / meta `noai`, `noimageai` (non-standard); IPTC "Data Mining" photo metadata; the IETF "AI Preferences" (aipref) work on extending robots.txt [M?].

---

## 4. Terms-of-use breach — Ryanair v PR Aviation

- **C-30/14, 15 Jan 2015, EU:C:2015:10.** Directive 96/9 does not apply to a database protected neither by copyright nor by the sui generis right; therefore its arts. 6(1), 8 and 15, which make users' minimum rights mandatory, do not stop the owner of such a database from restricting use **by contract**, subject to national law [M].
- The paradox: for a *protected* database, the lawful user may extract and re-use insubstantial parts for any purpose and contrary clauses are void (Dir. art. 8(1), art. 15; CPI L342-3 1° and final paragraph). For an *unprotected* one, the ToS can forbid everything [M].
- Whether the ToS bind is national contract law. In PR Aviation the user had to tick a box. In France general conditions bind only if brought to the other party's knowledge and accepted (C. civ. art. 1119); a footer link with no acceptance step (browse-wrap) is weak; a logged-in account or an API key is strong [M].
- France, same claimant: Cass. com., 10 Feb 2015, n° 12-26.023 (Ryanair v Opodo) rejected Ryanair's database claim because its investment went into *creating* the data [M?].
- Germany for contrast: BGH, 30 Apr 2014, I ZR 224/12 ("Flugvermittlung im Internet"): screen-scraping in breach of ticked ToS is not unfair hindrance unless a technical barrier is overcome [M].
- **API terms are contracts we sign.** Typical cover-API terms as I recall them (all [M?], to be re-read): Open Library Covers API — rate-limited by ISBN (about 100 requests per 5 minutes per IP), meant for display on public pages, no bulk download, courtesy link back, no rights warranty; Google Books API — thumbnails with attribution/link, restrictions on caching; Amazon Product Advertising API — images only to send traffic to Amazon, short caching window, link mandatory. None of these grants print rights, and none can grant more than the provider itself holds.

---

## 5. Sui generis database right

**Rule.** Dir. 96/9 art. 7 (France: CPI L341-1 to L343-7, loi n° 98-536 of 1 July 1998): the maker who made a substantial investment in *obtaining, verifying or presenting* the contents may prohibit extraction or re-utilisation of all or a substantial part (quantitatively or qualitatively), and the **repeated and systematic** extraction of insubstantial parts that conflicts with normal exploitation (art. 7(5); CPI L342-2). Term 15 years, renewed by new substantial investment. Criminal penalties exist in France (L343-4) [M]. Only EU-based makers qualify (art. 11): Internet Archive or Google LLC do not; Fnac Darty, Decitre, Babelio, Electre, Dilicom, MVB and Amazon's EU entities can [M].

| Case | Holding | Meaning for cover lookup | Conf. |
|---|---|---|---|
| British Horseracing Board v William Hill, C-203/02, 9 Nov 2004 (with the Fixtures Marketing cases) | Investment in *creating* data does not count; only investment in collecting, checking, presenting existing data. Art. 7(5) targets cumulative reconstitution of a substantial part. | A publisher gets no database right in its own covers merely by producing them. Aggregators and retailers that collect from many publishers do qualify. | [M] |
| Directmedia, C-304/07, 9 Oct 2008 | "Extraction" needs no technical copy; transfer after consultation suffices. | Manual or semi-manual harvesting is still extraction. | [M] |
| Football Dataco v Sportradar, C-173/11, 18 Oct 2012 | Re-utilisation is located where the targeted public is. | A service targeting France is judged in France. | [M] |
| Innoweb v Wegener, C-202/12, 19 Dec 2013 | A dedicated meta search engine that forwards user queries in real time to a database and shows the results in its own interface re-utilises the *whole* database. | A real-time ISBN-to-cover relay over a retailer's catalogue has the same structure. | [M] |
| CV-Online Latvia v Melons, C-762/19, 3 Jun 2021 | Indexing and copying a freely accessible database is extraction/re-utilisation, prohibited **if it risks the maker's ability to recoup its investment**; courts must balance against competitors' and users' interest in information and innovation. | Link-back and no substitution lower the risk; they do not remove it. | [M] |
| Leboncoin v Entreparticuliers — CA Paris, 2 Feb 2021; Cass. 1re civ., 5 Oct 2022, n° 21-16.307 | Classified-ads site is a database maker; scraping its property listings was extraction of a qualitatively substantial part; damages upheld. | French courts do enforce this against scrapers. | [M] |

Public bodies: bibliographic metadata of the BnF is under the Licence Ouverte (since 2014) and the Open Data Directive 2019/1024 bars public bodies from using the database right to block re-use. That covers **metadata, not cover images**, which remain third-party works [M?].

---

## 6. Text-and-data-mining exceptions

| | DSM art. 3 / CPI L122-5-3 II | DSM art. 4 / CPI L122-5-3 III |
|---|---|---|
| Beneficiary | Research organisations, cultural heritage institutions | Anyone, any purpose including commercial |
| Condition | Lawful access; scientific research | Lawful access **and** no reservation by the rightholder |
| Opt-out | None; contract cannot override (art. 7(1)) | Yes (art. 4(3)); in France by any means (R122-28) |
| Retention | Secure storage, may keep for verification | Only as long as needed for the mining; French text: copies "détruites à l'issue de la fouille" |
| Rights covered | Reproduction; database extraction | Same, plus software and press-publisher reproduction |

All [M]. France transposed by ordonnance n° 2021-1518 of 24 Nov 2021; the database-right mirror is in CPI L342-3 [M?].

**What TDM allows us to do:** fetch a cover from a lawfully accessible page, analyse it automatically "to generate information" (art. 2(2)) — dominant colours, title bounding box, typeface classification, perceptual hash to check that a user-supplied image matches the ISBN — then delete the copy. The derived facts are not protected and may be stored and used to build *our own* typographic spine. Training our font-identification model on such images is also TDM, subject to the same opt-outs and deletion [M, my application].

**What TDM does not allow:**

- Showing the fetched image to anyone: communication to the public (InfoSoc art. 3) is outside arts. 3–4 [M].
- Keeping it as a cache or asset library [M].
- Printing it or embedding it in the PDF [M].
- Output that reproduces the expression: LG München I, 11 Nov 2025, 42 O 14139/24 (GEMA v OpenAI) held memorised and regurgitated works fall outside the exception [M?]. A generative "outpainting" of the cover onto the spine is therefore not sheltered by TDM.
- Ignoring `robots.txt`, TDMRep or, in France, a ToS clause.
- Bypassing a paywall or login: access would not be "lawful" (recital 14) [M].

---

## 7. Hotlinking versus copying (communication to the public)

| Case | Holding | Conf. |
|---|---|---|
| Svensson, C-466/12, 13 Feb 2014 | A clickable link to a work freely available with the rightholder's consent reaches no new public. Different if the link bypasses access restrictions. | [M] |
| BestWater, C-348/13, order of 21 Oct 2014 | Same for framing/embedding. | [M] |
| GS Media, C-160/15, 8 Sep 2016 | Link to an *unauthorised* copy infringes if the linker knew or should have known; **knowledge is presumed when the link is posted for profit**. | [M] |
| Renckhoff, C-161/17, 7 Aug 2018 | Copying a freely available photo and posting it on another site is a new communication to a new public; the rightholder loses control once a second copy exists. Sources: [IRIS Merlin](https://merlin.obs.coe.int/article/8373), [Columbia GFoE](https://globalfreedomofexpression.columbia.edu/cases/land-nordrhein-westfalen-v-dirk-renckhoff/), [Mondaq](https://www.mondaq.com/uk/copyright/730320/the-risks-of-reposting-online-content-cjeu-rules-on-the-unauthorised-reposting-of-copyright-protected-works-on-the-internet). | [V2*] |
| VG Bild-Kunst, C-392/19 (Grand Chamber), 9 Mar 2021 | Framing that circumvents **effective technological measures** against framing is a communication to a new public. The rightholder can limit its consent only through such measures, not by contract alone (legal certainty). | [M] |

Reading for covers:

- **Inline display from the source server is the only unlicensed display route with CJEU support**, and only if (i) the source posted the cover with the rightholder's consent — a publisher's own site yes, a retailer under licence yes, an unknown blog no, and as a commercial service we carry the GS Media presumption; (ii) the source uses no hotlink protection (Referer checks, signed or expiring URLs, tokens). If it does, stop [M].
- Copyright is not the only constraint on hotlinking: source ToS, bandwidth free-riding as *parasitisme*, and data protection — the visitor's IP address goes to the third-party host (compare LG München I, 20 Jan 2022, 3 O 17493/20, Google Fonts) [M].
- France has treated image-search thumbnails as licensable: CPI L136-1 to L136-4 (loi n° 2016-925, amended 2019) set up collective management for automated image-referencing services [M?]. There is no French thumbnail exception; see the sibling note on short quotation.
- **A server-side image proxy is a re-host.** Browsers only let our WASM code read remote pixels if the source sends CORS headers. Working around a missing CORS header with our own proxy means we fetch, copy and transmit a file we selected: Renckhoff/VCAST territory, and no "mere conduit" shelter under DSA art. 4 because we select the content [M, my application; VCAST V2* via [Kluwer](https://legalblogs.wolterskluwer.com/copyright-blog/up-in-the-cloud-some-reflections-on-the-cjeu-judgment-in-vcast/)].

---

## 8. CNIL guidance on scraping (GDPR)

- Guidance of 30 April 2020 on re-using publicly accessible online data for commercial prospecting: public data remain personal data; check the source's ToS; consent needed for prospecting [M].
- AI how-to sheets, finalised 19 June 2025, including the sheet on legitimate interest and *moissonnage*: define collection criteria in advance, exclude unnecessary categories, delete irrelevant data at once, and **do not collect from sites that object by robots.txt or CAPTCHA** [M?].
- Sanctions: Clearview AI, EUR 20 million (Oct 2022); KASPR, EUR 240,000 (Dec 2024) for harvesting LinkedIn contact details that users had restricted [M?].
- Relevance to covers is limited but not zero: author names are personal data (legitimate interest is comfortable for bibliographic identification); retailer pages carry customer reviews (do not ingest); some covers show identifiable persons (*droit à l'image*, C. civ. art. 9). The useful takeaway is institutional: the French regulator treats robots.txt as binding on a good-faith scraper [M].

---

## 9. French unfair competition (*parasitisme*) and criminal law

- Basis: C. civ. art. 1240. Parasitisme = placing oneself in another's wake to profit, without paying, from its efforts, know-how, reputation or investment. No competition relationship and no risk of confusion required [M].
- Cass. com., 26 June 2024, n° 23-13.535 (Maisons du Monde v Auchan) tightened proof: the claimant must show an identified, individualised economic value and the defendant's intent to ride on it [M?].
- It fills gaps left by IP: a cover held non-original still attracted a parasitisme award (CA Versailles, 22 Mar 2022, Zulma v Elior — [Fournol](https://www.fournol-avocat.fr/actualite/2022/8/18/parasitisme-reconnu-pour-la-couverture-dun-livre)) [V1*]; systematic reuse of auction catalogues led to infringement plus parasitic competition against Artprice ([UGGC](https://www.uggc.com/condamnation-dartprice-com-pour-contrefacon-et-concurrence-parasitaire/)) [V1*].
- Application: lifting a retailer's catalogue imagery, which it paid to license, normalise and host, to power a paid gift-card product is the textbook pattern, even where copyright and database right both fail. Imitating a collection's look on a generated spine raises the same issue (sibling note).
- Criminal: Code pénal art. 323-1 (fraudulent access to or remaining in an automated data-processing system) and 323-3 (fraudulent extraction of data, added 2014). Cass. crim., 20 May 2015, n° 14-81.336 ("Bluetouff") upheld a conviction for remaining in, and taking files from, an extranet reachable through a search engine once the defendant knew access was meant to be restricted. Public pages are outside this; logging in under false pretences, defeating CAPTCHAs or rotating IPs after a block moves towards it [M].

---

## 10. United States, for comparison

| Topic | Position | Conf. |
|---|---|---|
| Scraping public pages | hiQ Labs v LinkedIn, 938 F.3d 985 (9th Cir. 2019), reaffirmed 31 F.4th 1180 (2022) after Van Buren v US (2021): not "without authorization" under the CFAA. On remand hiQ was held in breach of LinkedIn's User Agreement and accepted a USD 500,000 judgment and injunction (Dec 2022). | [M] |
| Contract and preemption | Meta v Bright Data (N.D. Cal., Jan 2024): ToS did not reach logged-out scraping. X Corp v Bright Data (N.D. Cal., May 2024): state-law claims over public data preempted by the Copyright Act. | [M?] |
| Database right | None (Feist v Rural, 1991). | [M] |
| Thumbnails | Kelly v Arriba Soft, 336 F.3d 811 (9th Cir. 2003) and Perfect 10 v Amazon, 508 F.3d 1146 (9th Cir. 2007): search thumbnails are transformative fair use — [opinion](https://m.openjurist.org/336/f3d/811), [CRS RL33810](https://www.everycrsreport.com/reports/RL33810.html). | [V2*] |
| Inline linking | Perfect 10 "server test": no display by the site that only links; reaffirmed in Hunley v Instagram (9th Cir. 2023), rejected in Goldman v Breitbart (S.D.N.Y. 2018). | [M] |
| Limits | Warhol v Goldsmith (2023): same-purpose commercial use weighs against fair use. A full-size printed cover is not a thumbnail. | [M] |

The US picture explains why US-hosted services (Open Library, Goodreads, LibraryThing) show covers freely. It gives a France-targeted service nothing: applicable law follows the country of protection (Rome II art. 8) and jurisdiction follows accessibility (Pinckney C-170/12, Hejduk C-441/13) [M].

---

## 11. Risk by acquisition route

Risk to the service, France-first. (a) locate and preview to the user; (b) store image bytes on our side; (c) print on the card. My assessment, not a sourced rating.

| # | Route | (a) Locate + preview | (b) Store | (c) Print |
|---|---|---|---|---|
| 1 | Rightholder upload, or publisher/reseller feed **with an explicit print clause** | Low | Low | Low |
| 2 | Reseller's ONIX feed cover, "promotional use" licence only (reseller mode) | Low | Low (inside the reseller's licence, as its processor) | Medium until a print clause exists (CPI L131-3) |
| 3 | User's own file: cover extracted **in the browser** from the user's EPUB, or own scan | Low | Low if never uploaded; Medium if we host it (DSA notice duties) | Home print: Low–Medium (user's private copy). Pro print: High unless the user is the rightholder |
| 4 | Inline link to the publisher's own page image (e.g. its `og:image`), link back, no technical protection, robots/TDMRep clean | Low–Medium | High (Renckhoff) — keep URL and derived features only | High through us |
| 5 | Cover API with display terms (Open Library, Google Books), used as the terms say | Low–Medium (third-party rights not cleared; IP leak to host) | Medium–High (caching limits; Renckhoff) | High (outside terms, no licence, and too small anyway) |
| 6 | On-demand scrape of a retailer/aggregator page (Amazon, Fnac, Decitre, Babelio, Goodreads) | Medium–High (ToS, L342-2, Innoweb, parasitisme) | High | High |
| 7 | Bulk crawl or mirror of a cover catalogue | High | High | High |
| 8 | Image-search results of unknown provenance | Medium–High (GS Media presumption) | High | High |
| 9 | TDM-only fetch: derive palette / font class / hash, then delete | No display. Low–Medium if access lawful and no opt-out | Image: must be deleted. Derived facts: Low | Derived colour and font choice on our own spine: Low (watch series look-alikes) |
| 10 | Public-domain cover (artist dead > 70 years) from an open-licence host | Low | Low | Low (check host terms; moral rights against distortion) |
| 11 | Any route that defeats a block (login, CAPTCHA, IP rotation, hotlink protection, spoofed user-agent) | High | High | High |

**Spines.** Spine or full-jacket images are almost never in public feeds, so "finding" one means scraping shelf photos or printer files: routes 6–8, with the same analysis as covers plus the photographer's right in the photo. The lawful default is to generate our own spine; route 9 can inform its colour and typeface.

**Best technical fit for standalone mode:** route 3. An EPUB carries its cover as a manifest item; reading it in WASM needs no network acquisition at all. As I recall, Readium LCP requires cover images to stay unencrypted, so no protection measure is touched [M?].

---

## 12. Practical policy

Fetcher rules (for the `covers` crate and any server-side job):

1. **Allow-list, not open web.** Each source has a recorded basis: contract, open licence, or statutory exception, with the date its terms were last read.
2. **Honest identification.** Fixed user-agent with a URL describing the bot and an opt-out contact. No spoofing.
3. **Obey every reservation signal**: robots.txt per RFC 9309 (5xx = disallow, 24 h cache), TDMRep header / well-known file / meta tag, `noai`-type tags, and for French sites the ToS text. Log the signal state with each fetch as evidence.
4. **Never defeat a barrier.** Stop on 401, 403, 429, CAPTCHA or hotlink protection; no retries through other IPs.
5. **On-demand, low rate, no enumeration.** One ISBN per user action; no real-time relay of a retailer's search (Innoweb); no background crawling of catalogues (L342-2).
6. **Store facts, not pixels.** URL, timestamp, hash, dimensions, palette, font class. Image bytes only for licensed tiers. TDM copies are deleted when the analysis ends.
7. **No image proxy.** If the source does not allow cross-origin reads, the image is not available to the composer.

Product rules:

8. **Preview** = inline link to an authorised source plus a visible link back and source label, or the user's own file. Mention third-party image hosts in the privacy notice; prefer EU hosts.
9. **Print** = licensed cover (routes 1–2 with print clause), user-supplied cover for home printing, public-domain cover, or the typographic jacket. A cover found by routes 4–8 is never embedded in a PDF by us.
10. **Opt-out and takedown** (ISBN-prefix blocklist, same-day removal) are mitigation only: after Soulier and Doke an opt-out offer does not legalise an unlicensed use.
11. **Do not ingest** reviews, ratings or other personal data from scraped pages.

---

## 13. Sources

URLs below were **not** returned by searches of mine (none ran). They are copied from the sibling notes, where they appeared in that agent's search results:

- [IRIS Merlin — Renckhoff](https://merlin.obs.coe.int/article/8373)
- [Columbia Global Freedom of Expression — Renckhoff](https://globalfreedomofexpression.columbia.edu/cases/land-nordrhein-westfalen-v-dirk-renckhoff/)
- [Mondaq — Renckhoff](https://www.mondaq.com/uk/copyright/730320/the-risks-of-reposting-online-content-cjeu-rules-on-the-unauthorised-reposting-of-copyright-protected-works-on-the-internet)
- [Kluwer Copyright Blog — VCAST](https://legalblogs.wolterskluwer.com/copyright-blog/up-in-the-cloud-some-reflections-on-the-cjeu-judgment-in-vcast/)
- [Fournol — parasitisme pour la couverture d'un livre (Zulma v Elior)](https://www.fournol-avocat.fr/actualite/2022/8/18/parasitisme-reconnu-pour-la-couverture-dun-livre)
- [UGGC — Artprice](https://www.uggc.com/condamnation-dartprice-com-pour-contrefacon-et-concurrence-parasitaire/)
- [Kelly v Arriba Soft, 336 F.3d 811](https://m.openjurist.org/336/f3d/811)
- [CRS RL33810 — search engines and fair use](https://www.everycrsreport.com/reports/RL33810.html)

Authorities cited from memory, by identifier (to be opened on EUR-Lex / Curia / Légifrance in a verification pass):

- EU legislation: Directive 96/9/EC (arts. 7, 8, 10, 11, 15); Directive 2001/29/EC (arts. 2, 3, 5(1), 6); Directive (EU) 2019/790 (arts. 2(2), 3, 4, 7; recitals 14, 18); Regulation (EU) 2024/1689 art. 53(1)(c) and the GPAI Code of Practice of 10 July 2025; Regulation (EU) 2022/2065 art. 4; Directive (EU) 2019/1024; Rome II Regulation art. 8.
- CJEU: C-203/02 BHB v William Hill (EU:C:2004:695); C-304/07 Directmedia (EU:C:2008:552); C-5/08 Infopaq (EU:C:2009:465); C-173/11 Football Dataco v Sportradar (EU:C:2012:642); C-202/12 Innoweb (EU:C:2013:850); C-466/12 Svensson (EU:C:2014:76); C-360/13 PRCA v NLA (EU:C:2014:1195); C-348/13 BestWater (EU:C:2014:2315); C-30/14 Ryanair v PR Aviation (EU:C:2015:10); C-160/15 GS Media (EU:C:2016:644); C-301/15 Soulier and Doke (EU:C:2016:878); C-161/17 Renckhoff (EU:C:2018:634); C-392/19 VG Bild-Kunst (EU:C:2021:181); C-762/19 CV-Online Latvia v Melons (EU:C:2021:434); C-170/12 Pinckney; C-441/13 Hejduk.
- France: CPI L122-5-3, R122-28, L131-3, L136-1 to L136-4, L341-1 to L343-7; ordonnance n° 2021-1518; décret n° 2022-928; loi n° 98-536; C. civ. arts. 9, 1119, 1240; Code pénal arts. 323-1, 323-3; Cass. com. 10 Feb 2015 n° 12-26.023; Cass. crim. 20 May 2015 n° 14-81.336; CA Paris 2 Feb 2021 and Cass. 1re civ. 5 Oct 2022 n° 21-16.307; Cass. com. 26 June 2024 n° 23-13.535; CNIL guidance of 30 Apr 2020 and AI recommendations of 19 June 2025; CNIL sanctions Clearview (2022) and KASPR (2024).
- Germany, Netherlands, Belgium: BGH I ZR 69/08; BGH I ZR 224/12; LG Hamburg 310 O 227/23; OLG Hamburg 5 U 104/24; LG München I 42 O 14139/24 and 3 O 17493/20; Rb. Amsterdam 30 Oct 2024 (DPG Media v HowardsHome); CA Bruxelles 5 May 2011 (Copiepresse v Google).
- US: hiQ v LinkedIn (938 F.3d 985; 31 F.4th 1180); Van Buren v US (2021); Meta v Bright Data (2024); X Corp v Bright Data (2024); Perfect 10 v Amazon (508 F.3d 1146); Hunley v Instagram (2023); Goldman v Breitbart (2018); Feist v Rural (1991); Warhol v Goldsmith (2023).
- Standards: RFC 9309 (Sept 2022); W3C Community Group TDM Reservation Protocol; Readium LCP specification.

---

## 14. Could not verify

Nothing in this note was verified by my own searching. In order of how much the policy depends on it:

1. **Current terms of each candidate cover source**: Open Library Covers API (rate limit, bulk rule, link-back), Google Books API (caching, attribution), Amazon PA-API (and whether it is being replaced), BnF catalogue cover service, Gallica commercial-reuse terms (I have an unconfirmed memory that commercial reuse of public-domain Gallica images became free in 2024), ISBNdb, LibraryThing. Risk rows 5 and 10 depend on them.
2. **Actual robots.txt, TDMRep and ToS state** of the French sites we might touch (publishers, Fnac, Decitre, Babelio, leslibraires.fr, 7switch). Whether the SNE's TDMRep recommendation is as widely deployed as I believe.
3. **CPI R122-28 wording** ("par tout moyen … conditions générales d'utilisation") and the exact numbering of the TDM items in L342-3. The French-is-wider-than-the-Directive point rests on this.
4. **OLG Hamburg, 5 U 104/24** (date, and its holding on natural-language reservations); **Rb. Amsterdam DPG Media v HowardsHome**; **LG München I GEMA v OpenAI** reasoning; status of the pending CJEU reference C-250/25 (Like Company v Google), which I did not use.
5. **CNIL**: date and exact wording of the final scraping sheet (19 June 2025) and whether respecting robots.txt is framed as mandatory; KASPR amount and date.
6. **French decision numbers and dates**: Ryanair v Opodo (n° 12-26.023), Leboncoin (n° 21-16.307), Maisons du Monde v Auchan (n° 23-13.535), Bluetouff (n° 14-81.336); penalties in L343-4 and Code pénal 323-1/323-3.
7. **CPI L136-1 to L136-4** (image-search collective management): current text and whether it was ever implemented.
8. **ECLI numbers** and paragraph-level holdings of the CJEU cases; RFC 9309 quotations and numeric limits (five redirects, 500 KiB, 24 hours).
9. **US district-court outcomes** (hiQ remand and settlement figures; Meta and X v Bright Data).
10. **Readium LCP** rule that cover images stay unencrypted; whether Adobe ADEPT and Kobo/Kindle files behave the same.
11. **No authority found or known** for: robots.txt as an "effective technological measure"; a French judgment treating a robots.txt violation as a fault in itself; a French judgment on image hotlinking as parasitisme; any EU decision on cover-lookup services specifically; whether ONIX has a spine or full-jacket resource code (also open in the sibling note).
