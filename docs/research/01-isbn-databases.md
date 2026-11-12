# ISBN & book-metadata databases we are allowed to access

_Status: fact-checked — research date 2026-11-12_

> **Fact-check pass (2026-11-12).** Web search worked for this pass. Claims marked **[FC]** were re-verified with fresh searches or by re-reading the primary file; corrections are written in place and listed in the "Verification log" at the end and in `factcheck/isbn-databases.md`. Trade sources (§4.2–4.4, §4.6), the legal authorities of §5 and all live HTTP behaviour were **not** re-checked and keep their original markers.

> **Scope.** This document answers one question: from which databases may the service fetch **text metadata** (title, contributors, publisher, date, identifiers) for a book identified by its ISBN, and on what terms. It is research to brief the implementation, not legal advice. Finding and previewing cover images is covered in `02-cover-sources.md`; whether a cover may be printed is decided in `07-legal-covers-and-reseller-rights.md`. Where the documents meet, doc 07 governs.

**Confidence markers.** [V2] = verified in at least two independent sources; [V1] = one source; [M] = from memory, not verified in this session. Qualifiers kept from the researcher notes: "[M, low]" = shaky recall, do not code or contract against it without a check; [sib] = fact and link taken from a sibling note of this repository, not re-opened here. "Re-read" means I opened the same file again today; it does not raise a marker. Hit-rate words ("high", "medium", "low") are judgement, never measurements.

**Method limits — read first.**

- **No web search ran**, for either researcher or for this synthesis. The shared budget of 200 searches was already spent; the five queries I sent were refused. WebFetch and every vendor or API host are blocked by the sandbox proxy. I did not work around either limit, and **no endpoint was called live**.
- "Source" in this document therefore means a primary file read on `raw.githubusercontent.com`, `pypi.org` or `registry.npmjs.org`: an operator's own repository (Open Library, DNB, KB Sweden, Etalab, Crossref, OpenAlex, Hardcover, the BnF hackathon wiki), or client code that hard-codes an endpoint (Zotero translators, isbnlib plugins, Inventaire, VuFind, Koha). For [V2], "independent" means independent code bases or documents.
- The two researcher notes are `notes/isbn-databases.open.md` (68 links) and `notes/isbn-databases.trade.md` (52 links plus 3 [sib]). For this synthesis I re-opened 25 of their files to spot-check the load-bearing claims and read 5 new ones (§6.5). Every spot-check agreed with the notes.
- **Not verified by anyone**: every price; every trade contract or subscription term; the wording of the Google Books terms; every CORS header except Open Library's; every live HTTP behaviour; every hit rate. GPL, LGPL and AGPL code (Zotero translators, calibre, Inventaire, isbnlib plugins, Koha, VuFind, python-stdnum, bookshops) was read for facts only. None of it may be copied into this BSD-3 repository.

**What this pass changed against the notes.**

| Point | Notes said | Now | Evidence |
|---|---|---|---|
| Licence Ouverte terms | Commercial reuse and caching allowed, attribution required [M] | [V1] for the text of Licence Ouverte 2.0: reuse "for commercial or non-commercial purposes, worldwide and for an unlimited period"; copying, adapting and redistributing allowed; one condition, naming the source and "the date of the most recent update"; a hyperlink to the source can satisfy it; the grant includes the grantor's "sui generis rights of database producers". Which version BnF and ABES apply today stays [M] | Etalab: [English text](https://raw.githubusercontent.com/etalab/licence-ouverte/master/open-licence.md), [French text](https://raw.githubusercontent.com/etalab/licence-ouverte/master/LO.md) |
| Registration-group routing | Groups other than 978-2, 979-10, 979-12, 979-8 are [M] | [V1]: 978-3 German language, 978-84 Spain, 978-88 Italy, 978-90 and 978-94 Netherlands, 978-91 Sweden, 978-951 and 978-952 Finland, 978-972 and 978-989 Portugal, 979-11 Korea, 979-13 Spain. There is no 979-0 ISBN group | [isbn.dat](https://raw.githubusercontent.com/arthurdejong/python-stdnum/master/stdnum/isbn.dat), generated from the Agency's range file dated 2026-01-04 |
| Google Books `country` parameter | Documented on `volumes/{id}`; use on the list call is [M] | [V1]: the discovery document defines `country` on `volumes/{volumeId}` only. The list method has no such parameter. A French view of a title needs a second call, or relies on the caller's IP | [discovery document, rev. 20240214](https://raw.githubusercontent.com/googleapis/google-api-go-client/main/books/v1/books-api.json) |
| BnF SRU query form | One ISBN per query | [V1]: the isbnlib plugin sends the ISBN-10 **and** the ISBN-13 in one CQL query and calls it a "Quirk (see issue #1)". Old records can carry only the ISBN-10 | [isbnlib-bnf source](https://raw.githubusercontent.com/xlcnd/isbnlib-bnf/master/isbnlib_bnf/_bnf.py) |
| BnF "API SRU" wiki page | Not read | It documents **Gallica's** SRU (`gallica.bnf.fr/SRU`, at most 50 records per response), not the catalogue général. Two different services; only the catalogue one answers ISBN lookups | [BnF wiki, API-SRU](https://raw.githubusercontent.com/wiki/hackathonBnF/hackathon2016/API-SRU.md) |
| DNB data licence | CC0 [M] | Still [M]. The DNBLab README says DNB offers its bibliographic data "frei zugänglich" [V1]; its own licence line, "CC BY 3.0", covers the tutorial material and must not be read as the data licence | [DNBLab README](https://raw.githubusercontent.com/deutsche-nationalbibliothek/dnblab/main/README.md) |
| lobid data licence | CC0 [M] | Still [M]. The repository README states only the code licence (EPL); the API page links a usage policy on lobid.org that could not be opened | [lobid README](https://raw.githubusercontent.com/hbz/lobid-resources/master/README.md) |
| ONIX spine codes | `02-cover-sources.md` says List 158 has no spine-only code; the trade note says codes 56 and 57 exist | Re-read: 56 "Spine image" ("2D, portrait orientation") and 57 "Spine panorama image" ("2D, image spans multiple upright spines") exist in code lists Issue 75 [V1]. Doc 02 read an older copy of the list | [Jonix List 158](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ResourceContentTypes.java) |
| Dilicom price encoding | Not stated | [V1]: "FEL à la demande" returns the price as a zero-padded integer in thousandths of a euro (`00013000` = EUR 13.00) | [bookshops scraper](https://raw.githubusercontent.com/vindarel/bookshops/master/bookshops/frFR/dilicom/dilicomScraper.py) |
| Reseller licence and CPI L. 131-3 | The trade note cites L. 131-3 for reading the reseller's licence narrowly | Corrected by the fact-check of doc 07: L. 131-3 governs authors' contracts only. See §6.1 | `07-legal-covers-and-reseller-rights.md` §3.2 |

---

## 1. Executive summary

### 1.1 Findings

1. **The best metadata source is not a database.** In standalone mode it is the user's own EPUB (its OPF metadata) plus what the user types. In reseller mode it is the ONIX record the reseller already holds for the title. Both are authoritative and raise no database-licence question. ISBN databases are a convenience: they pre-fill a form from a typed or scanned ISBN.
2. **"Allowed" falls into four classes.** (A) Open: no key, a licence that permits commercial reuse and storage. (B) Terms-bound: a free API whose terms limit storage, branding or use. (C) Contract: paid or negotiated access. (D) Excluded: the terms or the law bar our use. §5.5 says what each class lets us do.
3. **Class A is large enough for France and the main EU languages.** BnF catalogue général and data.bnf.fr (Licence Ouverte) [V1 for the licence statement, dated 2016], Libris (CC0) [V1], Open Library (contributions under CC0) [V1], Crossref [V1], BookBrainz core data (CC0) [V1]; and, from memory only: SUDOC (Licence Ouverte), DNB, K10plus, lobid, Finna, BNE, Wikidata, Inventaire, DOAB/OAPEN (CC0) [M].
4. **For a French title, ask BnF first.** ISBNs starting 978-2 (French language) and 979-10 (France) go to the BnF SRU service, which needs no key [V2 for the endpoint; FC: confirmed on bnf.fr]. Route other registration groups to their national library (§3.1).
5. **979-prefixed ISBNs have no ISBN-10** [V2]. Any provider keyed on ISBN-10 fails silently on recent French titles. The reverse also holds: old BnF records may carry only the ISBN-10, so the query must send both forms for 978 ISBNs [V1]. BnF also documents a dedicated index, `bib.fuzzyIsbn`, that matches EAN, ISBN-13 and ISBN-10 in one operand [FC] (§3.3).
6. **An eBook has its own ISBN, and catalogues know print ISBNs better** [M]. Expect misses on eBook ISBNs at BnF [FC: BnF has no per-title deposit step for digital publications yet and counted 2,606 digital books against 77,341 printed books deposited in 2025]. Google Books is the open-ish source most likely to know an eBook ISBN [M]; DNB covers German eBooks well [M]. None of this was measured.
7. **Class B is one useful source: Google Books — but not for a paid product without Google's permission.** API key; about 1,000 requests per day by default [FC, secondary sources]; branding guidelines [FC]; no permanent copy [M]. **Correction [FC]:** the Books API has its own terms on top of the Google APIs terms, and they say: "You may not charge users any fee for the use of your application, unless you have entered into a separate agreement with Google or obtained Google's written permission", and "You may not reorder or otherwise alter the results". Treat it as usable only in a free flow, or after written permission; otherwise keep it off.
8. **No French trade database is open, and none was priced.** Dilicom (FEL, Hub), Electre, Tite-Live and Decitre ORB are subscriber services for booksellers and libraries [V1 for ORB's subscription and Dilicom's credentials; M for the rest]. Whether a company that is neither can subscribe is unknown. As of 2026-11 no price could be confirmed for any of them.
9. **No commercial source grants a right to print a cover** [M]. Buying a database solves pre-fill, not the cover.
10. **ONIX 3.0/3.1 is the canonical trade feed** (current: code lists Issue 75 [FC: EDItEUR's site]; ONIX 3.1.3 per the Jonix README only [V1, not confirmed by search]). It can carry a full jacket (List 158 code 29 [V2]) and a spine image (code 56 [FC: added in Issue 70; code 57 in Issue 71]). It carries terms for assets (audience, territory, dates, credit) [V2] and, since 3.1.1, a per-image licence slot (`EpubLicense`) [V1]. It has no code meaning "may be printed on a physical object".
11. **Browser-direct lookup is verified for one source only**: Open Library sends `Access-Control-Allow-Origin: *` on `.json` and `/api/` paths [V1]. For every SRU service CORS is unverified. Plan a server-side relay for **text metadata**. This is not the image proxy that docs 02 and 07 forbid (§6.1).
12. **Excluded**: Amazon (PA-API 5.0 shut off on 15 May 2026, replaced by the Creators API [FC]; terms bar offline and printed use [M]), Goodreads, LibraryThing for commercial use, Babelio, Fnac, Kobo, Decitre's website, OCLC WorldCat's internal API, the ISBN Agency's publisher register [M unless marked].

### 1.2 Recommended provider priority — standalone mode

Steps 0–2 need no network. A step runs only when the steps above it left title or first author empty. "Default" is the shipped setting.

| # | Source | Class | Used for | Storage | Default |
|---|---|---|---|---|---|
| 0 | ISBN parsing with a range table generated from the Agency's file | — | Validity, hyphenation, registration group (language area), ISBN-10 form | Built in | On |
| 1 | The user's EPUB: OPF metadata read in the browser | Own file | All text fields and identifiers | Stays in the browser | On |
| 2 | Typed input | — | Everything; overrides every other source | — | On |
| 3 | ONIX 3.0/3.1 file from a publisher, author platform or e-distributor (opt-in) | Rights holder | All fields; the only channel that can carry a print permission for the cover | Per account | On when the feature ships |
| 4 | National library routed by registration group. 978-2 and 979-10: BnF SRU, then data.bnf.fr | A | Pre-fill | Cache with source and date | On |
| 5 | Open Library | A | Pre-fill, all groups | Cache | On |
| 6 | Google Books | B | Pre-fill for misses, mainly eBook ISBNs | None beyond the session | On only when a key is configured **and** the flow charges the user nothing or Google's written permission is on file [FC] |
| 7 | Inventaire, Wikidata; SUDOC `isbn2ppn` for academic titles | A | Last resort | Cache | Off until the hit-rate test shows a gain |
| 8 | ISBNdb | C, self-serve | Pre-fill | Per its terms | Off |
| 9 | One contracted French base (Decitre ORB or Electre), Dilicom "FEL à la demande" for EAN and fixed-price validation | C | Pre-fill and validation only | Per contract | Off |

Order of steps 4 to 6 follows the open note; steps 8 and 9 follow the trade note. Both notes agree.

### 1.3 Recommended provider priority — reseller-integrated mode

| # | Source | Used for | Rule |
|---|---|---|---|
| 1 | The title payload sent by the reseller: an ONIX `<Product>` or the same fields as JSON | Identifiers, title, contributors, publisher, cover link with its terms | Authoritative. Never overridden by a catalogue |
| 2 | The same payload, evaluated per title | `ContentAudience`, `Territory`, `ContentDate` 14/15, `EpubUsageConstraint`, outlet restrictions | One pure function returns allow, deny or needs opt-in (§4.6) |
| 3 | A print permission obtained at distributor level | Cover on the card | Contract rider or a named `EpubLicense`. Without it: typographic card, no cover image |
| 4 | Class A catalogues, queried on the print ISBN from `RelatedProduct` | Filling a missing non-critical field | Optional. Marked with its source |
| 5 | — | — | No trade database under our own name. No catalogue shared across resellers: we act as each reseller's subcontractor |

### 1.4 Sources we must not use

Amazon Creators API; Goodreads; LibraryThing (commercial use); Babelio; the Fnac, Kobo and Decitre websites; the internal search API of worldcat.org; the ISBN Agency's Global Register of Publishers; AFNIL's site; any dump of scraped catalogues. Reasons are in §3.9, §4.5 and §5.4.

---

## 2. Master comparison table

Legend. **Class**: A open, B terms-bound, C contract, D excluded. **CORS**: "verified" = read in the operator's code; "stated" = the operator says so; "unverified" = unknown, assume a relay is needed. **Verdict**: Primary = use first; Tier 1 = routed national library; Tier 2 = general open fallback; Tier 3 = display-only; Optional = off by default; Hold = do not code until a live check; Skip = no value for us; Excluded = must not be used.

### 2.1 Open, public and library sources

| Source | Coverage | Access | Key / limits | Data licence | Commercial use OK? | Covers? | CORS | Verdict |
|---|---|---|---|---|---|---|---|---|
| **BnF catalogue général** (FR) | French legal deposit; 14 million bibliographic records in 2016 [V1]. High for French print, partial for eBooks [M, low] | SRU 1.2 XML; also Z39.50 and OAI-PMH [V2] | None. No published limit found [FC: unsupported either way]. `maximumRecords` defaults to 10, at most 1000 [FC] | Licence Ouverte since 1 January 2014, Dewey authority records excepted [FC] | Yes, with attribution [FC; one BnF portal sentence about paid commercial reuse appears to concern digitised documents — re-read it] | Thumbnail endpoint [V1]; not covered by the metadata licence [M] | Unverified | **Tier 1** for 978-2 and 979-10 (A) |
| **data.bnf.fr** (BnF) | Same catalogue as linked data: works, authors, editions | SPARQL JSON; RDF dumps (editions: 39 GB RDF/XML) [V2] | None | Licence Ouverte [V1] | Yes, with attribution | No | Unverified | Tier 1 back-up; work and author identifiers (A) |
| **SUDOC** (ABES, FR) | Union catalogue of French academic libraries | `isbn2ppn` web service, XML or JSON [V1]; record by PPN [M] | None. Limits unknown | Licence Ouverte (Etalab), ABES policy since 2012 [FC] | Yes [FC] | No | Unverified | Optional, academic titles (A) |
| **Open Library** (Internet Archive, US) | 192 million records of all types in the Nov 2021 dump [V1]. Print-heavy, patchy for French eBooks [M] | REST JSON; dumps on archive.org [V1] | None. 180 requests/min per IP on JSON and API paths, 1 request/s on HTML pages [V1, FC re-read]. Published policy: 1 request/s unidentified, 3 with a User-Agent naming the application and a contact [FC] | Contributions under CC0 [V1] | Yes | Yes; no licence on the images [M] | **Verified**: `*` on `.json` and `/api/` [V1] | **Tier 2**, all groups (A) |
| **Google Books** | Very broad, including eBooks; `saleInfo.isEbook`, per-country sale data [V1]. High for eBook ISBNs [M] | REST JSON | API key; about 1,000 requests/day by default [FC, secondary] | Google APIs terms **plus the Books API terms**: no fee charged to users without Google's written permission; results not to be reordered or altered; branding guidelines [FC]. No permanent database [M] | Display only, and **not in a paid application without written permission** [FC]; storage no [M] | Thumbnails about 80 to 1280 px wide [V1]; no reuse right | Yes [M] | **Tier 3**, display-only (B) |
| **DNB** (DE) | German legal deposit, including online publications [M] | SRU 1.1 XML: MARC21-xml, oai_dc, RDFxml [V2] | None; no registration since at least 2023 [FC] | CC0 1.0 since 1 July 2015 [FC] | Yes [FC] | Cover endpoint fed by MVB, rights unclear [V1] | Unverified | Tier 1 for 978-3 (A) |
| **K10plus** (GBV + BSZ, DE) | German academic union catalogue | SRU XML [V1; FC for endpoint and `pica.isb`] | None | CC0 [M; FC found "open data" and "public domain" statements but none naming CC0] | Yes [M] | No | Unverified | German back-up (A) |
| **lobid-resources** (hbz, DE) | North Rhine-Westphalia union catalogue | JSON-LD REST, JSONP, weekly JSON-lines dump [V1] | None. A usage policy exists, not read [V1] | CC0 1.0, stated by hbz and carried in each record [FC] | Yes [FC] | No | JSONP documented [V1] | German back-up (A) |
| **Libris** (KB, SE) | Swedish union catalogue | JSON-LD `find`, SRU, OAI-PMH, SPARQL [V1] | None. User-Agent required; KB may rate-limit or block [V1, FC re-read] | CC0 **for the Swedish national bibliography and authority records** [FC: that is what KB's text says; other union-catalogue records are not expressly covered] | Yes for those records | No | Docs show a browser `fetch()` [V1]; header unverified | Tier 1 for 978-91 (A) |
| **Finna** (FI) | Finnish libraries, archives, museums | REST JSON [V1] | None | Metadata CC0, images excluded [FC] | Yes [FC] | Per-record rights [V1 for the endpoint] | Unverified [FC: nothing found] | Tier 1 for 978-951, 978-952 (A) |
| **BNE datos** (ES) | Spanish national library | SPARQL JSON [V1] | None | CC0 [M] | Yes [M] | No | Unverified | Tier 1 for 978-84, 979-13 (A) |
| **KB** (NL) | Dutch union catalogue (GGC) | SRU XML [V2]; RDF dumps [V1] | None | Bibliographic sets open; some KB datasets are CC BY-NC-ND [V1] | Check per collection | No | Unverified | Tier 1 for 978-90, 978-94 after a licence check |
| **swisscovery** (SLSP, CH); **OBV** (AT) | Swiss and Austrian academic networks | Alma SRU XML [V1 for the Swiss base URL; M, low for Austria] | None | Unknown | Unknown | No | Unverified | Hold |
| **ICCU / OPAC SBN** (IT) | Italian union catalogue | 2020 HTML-form URL returning UNIMARC [V1, stale] | None | Unknown | Unknown | — | — | Hold: find the current API |
| **KBR**, Unicat (BE) | Belgian national library; union catalogue | No public ISBN API found; Unicat SRU, age unknown [V1] | — | Unknown | Unknown | — | — | Hold |
| **PORBASE** (BNP, PT) | Portuguese union catalogue | Dublin Core XML by URN [V1, 2021] | None | Unknown | Unknown | No | Unverified | Hold |
| **Library of Congress** (US) | US deposit | SRU XML [V2, sources predate 2025]; JSON on loc.gov [M] | None; loc.gov limits [M] | US government work [M] | Yes [M] | No | Unknown | Skip: low value. LoC still lists `lx2.loc.gov`; 2026 reports say only `http://lx2.loc.gov:210/lcdb` answers and the `https` form fails the TLS handshake; the "2025 platform change" itself is unsupported [FC] |
| **British Library / BNB** (UK) | British national bibliography | None I can vouch for [M, low] | — | — | — | — | — | Hold |
| **Wikidata** | Low for editions | SPARQL JSON [V2] | None. User-Agent policy; 60 s timeout [FC]; "5 parallel queries per IP" unsupported [FC] | CC0 [M] | Yes | Rarely (P18) | Yes [M] | Optional, last resort (A) |
| **Inventaire.io** (FR association) | Aggregator: re-queries BnF, BNE, Open Library, Wikidata [V1] | REST JSON [V1] | None; limits unknown | Data CC0 [M]; server code AGPL-3.0 [V1] | Yes [M] | User uploads | [M, low] | Optional, last resort (A) |
| **Crossref** | Scholarly books with a DOI | REST JSON [V1] | None. Polite pool with `mailto`; limits in response headers [V1]. Since 1 December 2025: list requests (ours) 1 request/s, concurrency 1 in the public pool; 3 requests/s, concurrency 3 in the polite pool [FC] | "no claims of ownership"; may be "cached and incorporated" [V1] | Yes | No | Yes [M] | Skip unless academic titles matter (A) |
| **OpenAlex** | Scholarly works | REST JSON | Free API key required since 2026-02-13; `mailto` ignored [FC: OpenAlex announcement and deprecations page] | CC0 [V1] | Yes | No | Yes [V1] | **Not usable: no ISBN filter** [V1] |
| **DOAB / OAPEN** | Open-access books | REST, OAI-PMH [M] | None | CC0 [M] | Yes [M] | — | Unknown | Skip |
| **BookBrainz** (MetaBrainz) | Very low | REST by BBID, JSON dumps [M] | None | Core data CC0; other data CC BY-SA 4.0 [V1] | Yes for core data | No | Unknown | Skip |
| **Hardcover** | English-centric reading community | GraphQL [V1] | Token. Free: 5,000/day, 60/min, burst 10; 30 s timeout; beta [V1] | No rights claimed on the data; user-owned data barred for commercial products; no LLM training [V1] | Limited [V1] | User uploads; takedown policy advised [V1] | "CORS is open" but browser use forbidden to protect the token [V1] | Skip (B) |
| **HathiTrust** | Digitised library holdings | JSON, at most 20 identifiers per call [V1] | None | — | — | No | Presumably yes [V1] | Skip |
| **Internet Archive** | Scanned books | Search and metadata JSON [M] | None | Per item | — | Scans | Unknown | Skip |
| **Europeana; Trove (AU); NDL (JP)** | Heritage objects; Australia; Japan | REST | Europeana and Trove need a key [V1]; NDL none [M] | CC0 metadata [M]; Trove terms [M]; NDL CC BY 4.0 with commercial conditions [M, low] | Varies | — | [M] | Skip |
| **OCLC WorldCat** | Largest union catalogue | No open ISBN service [V1]/[M] | WSKey and subscription [M] | OCLC terms | No | — | — | **Excluded** (D) |
| **International ISBN Agency** | Registration ranges; publisher register | `RangeMessage.xml` download [V2]; register is web search only [M] | None | No licence found [M, low] | Ranges as facts | — | — | Use the range file; never scrape the register |

### 2.2 Trade and commercial sources

No price in this table could be confirmed as of 2026-11, except ISBNdb's public price list [FC].

| Source | Coverage | Access | Key / limits | Data licence | Commercial use OK? | Covers? | CORS | Verdict |
|---|---|---|---|---|---|---|---|---|
| **The reseller's ONIX feed** (e-distributors, Dilicom Hub numérique) | Every title the reseller sells | ONIX 3.0, some 2.1 [V1]; web services [M] | The reseller's contracts and GLN [M] | Contract: present, promote and sell the title [M] | Only as the reseller's subcontractor | Front cover with pixel size, MD5 and terms [V1] | Server to server | **Primary in reseller mode** (C) |
| **Dilicom FEL**, **FEL à la demande** (FR) | Titles commercially available in France [M] | Bulk file [M]; SOAP lookup by EAN, 100 EANs per call [V1] | Dilicom user and password [V1] | Members' own trade use, no redistribution [M] | By contract only | **None** in "à la demande" [V1] | n/a | Optional, later: EAN and fixed-price validation (C) |
| **Dilicom Distrimage** (FR) | Front covers for FEL titles | File or URL service [M] | Members [M] | Free inside bookshop software; any other use needs Dilicom's agreement [sib] | Print is not a default use | Yes, by contract | n/a | Not usable without a specific agreement |
| **Electre NG** (FR) | In-print, forthcoming and out-of-print French titles, with summaries and covers [M] | Web application and REST API [M] | Annual subscription [M] | Subscriber's internal use and own catalogue display [M] | By contract | Licensed option [M] | n/a | Optional, later: pre-fill (C) |
| **Decitre ORB** (FR) | "a french database of bibliographic information available by subscription" [V1] | REST JSON, user and key [V2] | Subscription | Terms not read | By contract | Cover URLs, thumbnail and original [V1] | n/a | Optional, later: pre-fill; best-documented API (C) |
| **Tite-Live Médiabase** (FR) | Base shipped with its bookshop software [M] | Bundled [M] | Tite-Live customers only [M] | Customer bookshop's own tools [M] | No | For customers [M] | n/a | Not available |
| **VLB** (MVB, DE) | German books in print | REST and ONIX; cover API with a token [V1] | Paid licence [M] | Terms of service bind display [V1]; use limited to promoting the title [M] | By contract | Token-gated [V1] | n/a | Later, if Germany justifies it (C) |
| **CB, Boekenbank (NL/BE); DILVE (ES); Informazioni Editoriali (IT); Nielsen BookData, BDS (UK); Bowker (US)** | National trade bases | Feeds, portals, web services [M] | Subscription or trade status [M] | By contract [M] | By contract | By contract | n/a | Later, per country (C) |
| **ISBNdb** | Global; depth for French eBooks unknown | REST, self-serve [M] | Paid plans, public price list read 2026-11 [FC]: Basic USD 14.99 (5,000 searches/day, 1 call/s), Premium USD 35.99 (15,000/day, 3 calls/s), Pro USD 99.99 (50,000/day, 5 calls/s), Enterprise by quote | Use inside your own application, no resale as a dataset [M] | Yes for pre-fill [M] | Unknown provenance, no rights warranty [M] | Key must stay server-side | Optional: only after a hit-rate test (C) |
| **Amazon Creators API** | Amazon catalogue | OAuth2 credentials from the Associates portal [V2] | Associates account with qualifying sales [M; one 2026 source says 10 in the previous 30 days, FC] | Content only to send traffic to Amazon; no offline or printed use [M] | **No** | Yes, but barred | — | **Excluded** (D) |
| **Goodreads; LibraryThing; Babelio; Kobo; Fnac; Bookshop.org** | Community and retailer catalogues | No new keys; non-commercial key; no public catalogue API [M] | — | Site terms [M] | No | — | — | **Excluded** (D) |

---

## 3. Per-source notes and request examples

### 3.0 Conventions

The example is ISBN **9782070360024** — *L'Étranger* by Albert Camus in Gallimard's Folio series [M for the title; the arithmetic below is checked].

| Form | Value | Needed by |
|---|---|---|
| ISBN-13, no hyphens | `9782070360024` | Most services |
| ISBN-13, hyphenated | `978-2-07-036002-4` | data.bnf.fr, Wikidata |
| ISBN-10, no hyphens | `2070360024` | BnF SRU (second operand), KB retry |
| ISBN-10, hyphenated | `2-07-036002-4` | data.bnf.fr, Wikidata P957 |
| Registration group | 978-2, "French language" [V1] | Routing |

Hyphenation needs the Agency's range table: group 2, registrant range 00–19, hence registrant `07` ([isbn.dat](https://raw.githubusercontent.com/arthurdejong/python-stdnum/master/stdnum/isbn.dat)) [V1]. A 979 ISBN has no ISBN-10 form ([isbn3 README](https://raw.githubusercontent.com/inventaire/isbn3/main/README.md), [Open Library vendors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/core/vendors.py)) [V2].

All examples are plain `GET` requests unless marked. Patterns are code, not citations; the evidence for each is linked beside it. Every request should carry the service's User-Agent (requirement 12).

### 3.1 Routing by registration group

Send the first network lookup to the national library of the ISBN's registration group. Groups read today in the range table [V1]; the mapping to a library is our choice.

| Prefix | Agency name in the range file | First provider | Then |
|---|---|---|---|
| 978-2 | French language | BnF SRU | data.bnf.fr, Open Library |
| 979-10 | France | BnF SRU | data.bnf.fr, Open Library |
| 978-3 | German language | DNB SRU | K10plus, lobid |
| 978-84, 979-13 | Spain | BNE datos | Open Library |
| 978-88, 979-12 | Italy | Open Library (SBN on hold) | — |
| 978-90, 978-94 | Netherlands | KB SRU (after the licence check) | Open Library |
| 978-91 | Sweden | Libris | Open Library |
| 978-951, 978-952 | Finland | Finna | Open Library |
| 978-972, 978-989 | Portugal | Open Library (PORBASE on hold) | — |
| 978-0, 978-1, 979-8 | English language; United States | Open Library | Google Books |
| Any other | — | Open Library | Google Books |

978-2 is a language area, not a country: Belgian, Swiss and Québec publishers share it [V1 for the agency name]. BnF legal deposit covers what is published or distributed in France [M], so a French-language title never distributed in France can miss at BnF and hit at Open Library or Google Books.

### 3.2 Sources that are not databases: the EPUB and ONIX

- **EPUB OPF.** The package document carries `dc:title`, `dc:creator`, `dc:publisher`, `dc:language`, `dc:date` and `dc:identifier`, where the ISBN usually appears as `urn:isbn:…` or as a bare number [M]. Reading it happens in the browser; nothing is uploaded. Publisher metadata inside EPUBs is uneven [M], so the form stays editable.
- **ONIX.** See §4.1. In reseller mode the reseller's record is the source of truth. In standalone mode an uploaded ONIX file from a publisher or author platform is the only channel that can carry a print permission.

### 3.3 BnF — catalogue général and data.bnf.fr

**Catalogue général, SRU** [V2: [Zotero BnF ISBN](https://raw.githubusercontent.com/zotero/translators/master/BnF%20ISBN.js), [isbnlib-bnf on PyPI](https://pypi.org/pypi/isbnlib-bnf/json) and its [source](https://raw.githubusercontent.com/xlcnd/isbnlib-bnf/master/isbnlib_bnf/_bnf.py)]:

```
https://catalogue.bnf.fr/api/SRU?version=1.2&operation=searchRetrieve
  &query=(bib.isbn%20all%20%222070360024%22%20or%20bib.isbn%20all%20%229782070360024%22)
  &recordSchema=dublincore&maximumRecords=5
```

- Send both ISBN forms for a 978 ISBN, as isbnlib-bnf does [V1]. For a 979 ISBN send the single operand `bib.isbn%20all%20%22{isbn13}%22`.
- **Alternative documented by BnF [FC]:** the index `bib.fuzzyIsbn` is a "fuzzy" search across EAN, ISBN-13 and ISBN-10, complementary to the exact `bib.isbn` and `bib.ean`. BnF's manual finds a record that holds only an ISBN-10 with `bib.fuzzyIsbn all "9782070363735"` ([criteria table](https://www.bnf.fr/sites/default/files/2019-04/tableau_criteres_sru.pdf), [query manual](https://www.bnf.fr/sites/default/files/2019-04/manuel_requetes_sru.pdf)). One operand then covers both forms. Not tested live, and the capitalisation of the index name differs between renderings (`bib.fuzzyIsbn` / `bib.fuzzyISBN`): read it in the Explain response before coding. Keep the two-operand form as the default.
- `maximumRecords` defaults to 10 and may not exceed 1000 ([service description](https://www.bnf.fr/sites/default/files/2019-04/service_sru_bnf.pdf)) [FC]. BnF's own example query is `bib.isbn all "2-07-036373-2"` [FC].
- Without `recordSchema` the records come back as `marcxchange`; the Zotero translator keeps only those [V1]. `recordSchema=dublincore` is what isbnlib-bnf requests [V1]. `intermarcXchange` appears in BnF's examples [FC]; `unimarcXchange` is still [M].
- The clients above ask for one record. Several records can share an ISBN (reprints) [M], so ask for up to 5 and choose (requirement 9).
- Titles need cleaning: Inventaire runs a `cleanupBnfTitle` step on BnF titles ([resolver](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/bnf/get_bnf_entry_from_isbn.ts)) [V1]. Author headings arrive as "Surname, Forename (dates)" [M].
- **Do not confuse it with Gallica's SRU** at `gallica.bnf.fr/SRU`, which searches the digital library and caps responses at 50 records ([BnF wiki](https://raw.githubusercontent.com/wiki/hackathonBnF/hackathon2016/API-SRU.md)) [V1].
- Other protocols: Z39.50 at `z3950.bnf.fr:2211`, database `TOUT-UTF8`, public login `Z3950` / `Z3950_BNF` [V2: [BnF wiki](https://raw.githubusercontent.com/wiki/hackathonBnF/hackathon2016/API-Z39.50.md), [Koha](https://raw.githubusercontent.com/Koha-Community/Koha/main/installer/data/mysql/en/optional/sample_z3950_servers.yml)]; OAI-PMH at `catoai.bnf.fr/oai2/OAIHandler` [V1: [BnF wiki](https://raw.githubusercontent.com/wiki/hackathonBnF/hackathon2016/API-OAI-PMH.md), 2016 text]. We need neither.

**data.bnf.fr, SPARQL** [V2: [Inventaire](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/bnf/get_bnf_entry_from_isbn.ts), [KBR BELTRANS](https://raw.githubusercontent.com/kbrbe/beltrans-data-integration/main/data-sources/bnf/README.md)]:

```
https://data.bnf.fr/sparql?default-graph-uri=&format=json&timeout=10000&query={url-encoded query}

PREFIX bnf-onto: <http://data.bnf.fr/ontology/bnf-onto/>
SELECT ?edition WHERE {
  { ?edition bnf-onto:isbn "2-07-036002-4" }
  UNION { ?edition bnf-onto:isbn "978-2-07-036002-4" }
  UNION { ?edition bnf-onto:ean "9782070360024" }
} LIMIT 5
```

`bnf-onto:isbn` holds hyphenated values and `bnf-onto:ean` the 13 digits [V1, Inventaire]; `bibo:isbn13` and `bibo:isbn10` also exist [V1, BELTRANS]. The full editions dump is 39 GB of RDF/XML [V1]; we do not mirror it.

**Licence.** "L'ensemble des métadonnées descriptives est disponible sous Licence Ouverte" ([BnF wiki](https://raw.githubusercontent.com/wiki/hackathonBnF/hackathon2016/Home.md), 2016) [V1]. Confirmed on BnF's current pages [FC]: all descriptive metadata under the State's Licence Ouverte since 1 January 2014 ([conditions](https://www.bnf.fr/fr/conditions-de-reutilisations-des-donnees-de-la-bnf), [catalogue conditions](https://catalogue.bnf.fr/conditions.do)), Dewey authority records excepted; data.bnf.fr says reuse is free for any use, commercial included, with attribution ([about](https://data.bnf.fr/fr/about)). BnF asks reusers to keep the source mention and the retrieval date. The version number (1.0 or 2.0) was not seen. The licence text allows commercial reuse with attribution [V1] (§5.2). The same page states that Gallica **images** are for non-commercial use [V1]: the open licence is about metadata.

**Cover endpoint.** `https://catalogue.bnf.fr/couverture?appName=NE&idArk=ark:/12148/cb{id}&couverture=1`, with placeholder bodies of 4566 or 4658 bytes [V1, Inventaire]. The BnF wiki lists a "Couverture Service" for the catalogue [V1]. Image rights are not granted by the metadata licence [M]; see doc 02.

**Limits and CORS.** No published rate limit found [FC: searched, nothing either way]. CORS unverified: use the relay.

**eBooks [FC].** BnF states that publishers currently have no deposit step for digital publications: they are collected through web harvesting, and a per-title deposit circuit is still being studied ([help centre](https://www.bnf.fr/fr/centre-d-aide/depot-legal)). Key figures: 77,341 printed books and 2,606 digital books deposited in 2025 ([chiffres clés](https://www.bnf.fr/fr/depot-legal-chiffres-cles)). An eBook ISBN will usually miss; query the print ISBN.

### 3.4 SUDOC (ABES)

```
https://www.sudoc.fr/services/isbn2ppn/9782070360024        (header  Accept: text/json)
```

Returns the PPN (record number) of matching records; `ean2ppn` and `issn2ppn` also exist and accept comma-separated batches ([node-sudoc 1.0.4, Inist-CNRS](https://registry.npmjs.org/sudoc)) [V1]. That client dates from 2022 and uses `http://`; the `https` form above is [M]. The record itself is then at `https://www.sudoc.fr/{ppn}.rdf` or `.xml` [M]. Licence Ouverte (Etalab): ABES policy since 2012, and data.gouv.fr lists the Sudoc API under an open licence ([ABES](https://abes.fr/en/api-et-web-services/), [data.gouv.fr](https://www.data.gouv.fr/dataservices/api-systeme-universitaire-de-documentation-sudoc)) [FC]. Value: low to medium, academic holdings.

### 3.5 Open Library

```
https://openlibrary.org/isbn/9782070360024.json                 (302 to /books/OL…M.json)
https://openlibrary.org/api/books?bibkeys=ISBN:9782070360024&format=json&jscmd=data
https://openlibrary.org/search.json?q=9782070360024
```

Evidence [V2]: [Open Library code.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/plugins/openlibrary/code.py), [Inventaire](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/openlibrary/get_openlibrary_entry_from_isbn.ts), [BookWyrm](https://raw.githubusercontent.com/bookwyrm-social/bookwyrm/main/bookwyrm/management/commands/initdb.py).

- **Rate limits, from the deployed nginx configuration** ([nginx.conf](https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/nginx.conf), [web_nginx.conf](https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/web_nginx.conf)) [V1, re-read]: 180 requests per minute per IP (burst 25) on `/api/*` and `*.json`; 1 request per second on HTML pages; covers have their own zone at 400 per minute. The limit is keyed on the IP when the request has no session cookie and neither the IP nor the User-Agent is allow-listed. A 429 answer is JSON and reads "Too Many Requests. Consider using https://openlibrary.org/developers/dumps."
- The public documentation asks for a User-Agent with an application name and a contact, and quotes 1 request per second unidentified, 3 identified ([developers/api](https://openlibrary.org/developers/api)) [FC]. It warns that violations "may result in aggressive rate limiting or blocking". Doc 02 recalls a further figure for ISBN-keyed cover requests (100 per 5 minutes) [M, low]. These concern different endpoints; §6.1 states the design value.
- **CORS**: `Access-Control-Allow-Origin: *` on any path ending in `.json` and on `/api/` ([processors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/plugins/openlibrary/processors.py)) [V1, re-read]. The [public-apis list](https://raw.githubusercontent.com/public-apis/public-apis/master/README.md) still says "No"; the source code wins.
- **Licence**: edits are contributed under CC0 ([i18n strings](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/i18n/messages.pot)) [V1]. No licence on cover images [M].
- **Missing ISBNs**: the site can import from Amazon and Better World Books on demand ([vendors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/core/vendors.py)) [V1], but plain `/isbn/` lookups run with `allow_import=False` [V1, re-read]. A miss stays a miss.
- **Dumps**: `ol_dump_YYYY-MM-DD` items in the `ol_exports` collection on archive.org; 192 million records, 29 GB in November 2021 ([oldump.sh](https://raw.githubusercontent.com/internetarchive/openlibrary/master/scripts/oldump.sh)) [V1]. Monthly frequency is [M]. Not needed at our volume.
- **Internet Archive** itself: `https://archive.org/advancedsearch.php?q=isbn:9782070360024&fl[]=identifier&output=json`, then `https://archive.org/metadata/{identifier}` [M]. No use for us.

### 3.6 Google Books

```
https://www.googleapis.com/books/v1/volumes?q=isbn:9782070360024&key={KEY}
https://www.googleapis.com/books/v1/volumes/{volumeId}?country=FR&key={KEY}
```

Evidence [V2]: [discovery document](https://raw.githubusercontent.com/googleapis/google-api-go-client/main/books/v1/books-api.json) (root URL `https://books.googleapis.com/`), [calibre-web](https://raw.githubusercontent.com/janeczku/calibre-web/master/cps/metadata_provider/google.py), [isbnlib](https://pypi.org/pypi/isbnlib/json).

- `country` exists on the second call only [V1]. The list call takes `q`, `langRestrict`, `printType`, `projection`, `maxResults` and others, but no country. Results of the list call depend on the caller's IP location [M]; a relay hosted in the EU is therefore part of the design.
- Useful fields: `volumeInfo` (title, authors, publisher, publishedDate, industryIdentifiers), `saleInfo.isEbook`, `saleInfo.country`, `accessInfo.epub` [V1].
- Image links come in six widths from about 80 to about 1280 px [V1]. They are candidates for preview only (doc 02).
- **Terms.** Google APIs Terms of Service — no scraping, no permanent database, cache only as long as cache headers allow [M]; Books branding guidelines exist as a separate page ([branding](https://developers.google.cn/books/branding?hl=en)) [FC]; exact "Powered by Google" wording not read. Default quota about 1,000 requests per day per project, raisable on request [FC, developer-forum and third-party sources; no official quota page retrieved]. Light use without a key may still be served [M].
- **Books API additional terms — correction [FC].** The Books API has its own terms ([developers.google.com/books/terms](https://developers.google.com/books/terms?csw=1); console copy dated 25 April 2012). As returned by the search tool: "You may not charge users any fee for the use of your application, unless you have entered into a separate agreement with Google or obtained Google's written permission"; "You may not reorder or otherwise alter the results returned by the Google Books API Family"; the APIs "are not intended to replace commercial services". I could not open the page itself, so read the clause before relying on either reading. Until then: Google Books is not available to a flow in which the user pays.

### 3.7 Germany: DNB, K10plus, lobid

```
https://services.dnb.de/sru/dnb?version=1.1&operation=searchRetrieve&recordSchema=MARC21-xml&maximumRecords=1&query=num%3D9782070360024
https://sru.k10plus.de/opac-de-627?version=1.1&operation=searchRetrieve&query=pica.isb=9782070360024&maximumRecords=1
https://lobid.org/resources/search?q=isbn:9782070360024&format=json
```

- DNB [V2]: [DNBLab SRU tutorial](https://raw.githubusercontent.com/deutsche-nationalbibliothek/dnblab/main/DNB_SRU_Tutorial.ipynb), [calibre-dnb](https://raw.githubusercontent.com/citronalco/calibre-dnb/master/__init__.py). Schemas MARC21-xml, oai_dc, RDFxml; the tutorial pages by 100 records [V1]. No access token appears in current client code [V2]. Data licence CC0 1.0: DNB has offered all its metadata free of charge under CC0 since 1 July 2015 ([press release](https://www.dnb.de/EN/Ueber-uns/Presse/ArchivPM2015/metadatenCC0.html)); the SRU address is unchanged on DNB's page updated 2 November 2026 ([SRU page](https://www.dnb.de/EN/Professionell/Metadatendienste/Datenbezug/SRU/sru_node.html)) [FC]. Cover endpoint `portal.dnb.de/opac/mvb/cover?isbn=`, fed by the trade body MVB, rights unclear [V1].
- K10plus [V1]: [Zotero K10plus ISBN](https://raw.githubusercontent.com/zotero/translators/master/K10plus%20ISBN.js), last updated 2024-10. Returns MARCXML. CC0 [M; not confirmed by search, which found only "open data" statements].
- lobid [V1]: [API page source](https://raw.githubusercontent.com/hbz/lobid-resources/master/web/app/views/api.scala.html). The `isbn:` field name is [M]. Weekly full dump in JSON lines, daily updates, JSONP [V1]. CC0 1.0, stated by hbz and present as a licence field in API records ([API page](https://lobid.org/resources/api)) [FC].

### 3.8 Other national and union catalogues

| Source | Request for the example ISBN | Notes |
|---|---|---|
| **Libris** (SE) | `https://libris.kb.se/find.jsonld?q=9782070360024` | Also SRU at `https://libris.kb.se/sru/libris?operation=searchRetrieve&version=1.2&query=…` [V1: [find](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/reference/find.en.md), [SRU](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/reference/sru.en.md)]. "The Swedish national bibliography and Swedish authority records are freely available without restrictions. The license form is Creative Commons Zero" — the statement names those two sets, not every union-catalogue record [FC, re-read]; KB may rate-limit or block; anonymous clients may be blocked without warning ([terms](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/index.en.md), [User-Agent](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/howto/user_agent.en.md)) [V1]. The legacy `xsearch` API is being retired ([placeholder page](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/howto/xsearch_migration.sv.md)) [V1] |
| **Finna** (FI) | `https://api.finna.fi/api/v1/search?lookfor=9782070360024&type=AllFields` | [V1: [finna-client](https://pypi.org/pypi/finna-client/json)]. An `isbn:` field query is [M]. Covers at `api.finna.fi/Cover/Show?id=…` [V1], rights per record |
| **BNE datos** (ES) | `https://datos.bne.es/sparql?format=json&query=…` with `?e <https://datos.bne.es/def/P3013> "9782070360024"` | Try plain and hyphenated, 10 and 13 digits [V1: [Inventaire](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/bne/get_bne_entry_from_isbn.ts)] |
| **KB** (NL) | `https://jsru.kb.nl/sru/sru?version=1.2&operation=searchRetrieve&startRecord=0&x-collection=GGC&recordSchema=dc&maximumRecords=1&query=identifier.ISBN%20exact%209782070360024` | [V2: [KB python API](https://raw.githubusercontent.com/KBNLresearch/KB-python-API/master/kb/nl/api/sru.py), [isbnlib-kb](https://pypi.org/pypi/isbnlib-kb/json)]. The plugin literally sends `query=query=identifier.ISBN…` and retries with the ISBN-10: test both forms live. Dumps at `data.bibliotheken.nl/files/` [V1: [BELTRANS](https://raw.githubusercontent.com/kbrbe/beltrans-data-integration/main/data-sources/kb/README.md)]. Some KB datasets are CC BY-NC-ND ([README](https://raw.githubusercontent.com/KBNLresearch/KB-python-API/master/README.rst)) [V1] |
| **swisscovery** (CH) | `https://swisscovery.slsp.ch/view/sru/41SLSP_NETWORK?version=1.2&operation=searchRetrieve&recordSchema=marcxml&query=alma.isbn=9782070360024` | Base URL [V1: [almasru](https://pypi.org/pypi/almasru/json)]; index name [M]. OBV / ÖNB (AT) has the same shape [M, low] |
| **ICCU / OPAC SBN** (IT) | — | Only a 2020 HTML-form URL on `opac.sbn.it/sbn3.0/opaclib` [V1, stale: [isbnlib-sbn](https://pypi.org/pypi/isbnlib-sbn/json)] |
| **KBR**, Unicat (BE) | — | KBR's own project obtained data by catalogue export; Unicat SRU at `http://www.unicat.be/sru` [V1: [BELTRANS KBR](https://raw.githubusercontent.com/kbrbe/beltrans-data-integration/main/data-sources/kbr/README.md), [Catmandu-SRU](https://raw.githubusercontent.com/LibreCat/Catmandu-SRU/master/README.md), age unknown] |
| **PORBASE** (PT) | `http://urn.porbase.org/isbn/dc/xml?id=9782070360024` | [V1, 2021: [isbnlib-porbase](https://pypi.org/pypi/isbnlib-porbase/json)] |
| **Library of Congress** | `https://lx2.loc.gov/sru/lcdb?operation=searchRetrieve&version=1.1&query=bath.ISBN=^9782070360024&maximumRecords=1` | [V2: [Zotero](https://raw.githubusercontent.com/zotero/translators/master/Library%20of%20Congress%20ISBN.js) (last updated 2023-08), [isbnlib-loc](https://pypi.org/pypi/isbnlib-loc/json), [Koha](https://raw.githubusercontent.com/Koha-Community/Koha/main/installer/data/mysql/en/optional/sample_z3950_servers.yml)]. All predate 2025. Status 2026 [FC]: LoC's server page still lists `lx2.loc.gov`; developers report that SRU answers on `http://lx2.loc.gov:210/lcdb`, that the `https` form does not complete a TLS handshake, and that `www.loc.gov` serves a Cloudflare challenge to server-side clients ([LoC servers](https://www.loc.gov/standards/sru/resources/lcServers.html), [report](https://github.com/guycorbaz/mybibli/pull/503)). JSON alternative `https://www.loc.gov/books/?q=9782070360024&fo=json` [M] |
| **British Library / BNB** | — | `bnb.data.bl.uk` is what [public-apis](https://raw.githubusercontent.com/public-apis/public-apis/master/README.md) still lists [M, low] |
| **NDL Search** (JP) | `https://ndlsearch.ndl.go.jp/api/opensearch?isbn=9782070360024` | [M]. Zotero's [NDL translator](https://raw.githubusercontent.com/zotero/translators/master/National%20Diet%20Library%20Catalogue.js) still targets the dead `iss.ndl.go.jp` [V1] |
| **Trove** (AU) | API v3 with an `X-API-KEY` header | [M]; key requirement [V1: public-apis] |
| **Europeana** | `https://api.europeana.eu/record/v2/search.json?wskey={KEY}&query=9782070360024` | Key [V1: [pyeuropeana](https://pypi.org/pypi/pyeuropeana/json)]; pattern [M] |
| **HathiTrust** | `https://catalog.hathitrust.org/api/volumes/brief/isbn/9782070360024.json` | Batch form `…/brief/json/isbn:{a}\|isbn:{b}`, at most 20 [V1: [VuFind preview.js](https://raw.githubusercontent.com/vufind-org/vufind/dev/themes/bootstrap5/js/preview.js)]; bulk "hathifiles" have an ISBN column [V1: [hathifiles](https://raw.githubusercontent.com/hathitrust/hathifiles/main/README.md)] |

### 3.9 Open knowledge bases

**Wikidata** [V2: [Inventaire resolver](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/wikidata/get_wikidata_entry_from_isbn.ts), [BookWyrm mappings](https://raw.githubusercontent.com/bookwyrm-social/bookwyrm/main/bookwyrm/connectors/inventaire.py); endpoint in [wikibase-sdk](https://raw.githubusercontent.com/maxlath/wikibase-sdk/main/README.md)]:

```
https://query.wikidata.org/sparql?format=json&query={url-encoded query}

SELECT ?item WHERE {
  { ?item wdt:P212 "978-2-07-036002-4" } UNION { ?item wdt:P212 "9782070360024" }
  UNION { ?item wdt:P957 "2-07-036002-4" } UNION { ?item wdt:P957 "2070360024" }
}
```

P212 is the ISBN-13 property, P957 the ISBN-10. CC0 [M]. Query-service limits: 60 s timeout [FC]; the figure "5 parallel queries per IP" could not be found anywhere and is withdrawn [FC]; User-Agent policy [M]. The graph split went live on 9 May 2025: scholarly items moved to `query-scholarly.wikidata.org`, book editions stay on the main endpoint ([WDQS graph split](https://meta.wikimedia.org/wiki/WikiCite/WDQS_graph_split)) [FC]. Hit rate for editions is low [M].

**Inventaire.io** [V1: [BookWyrm](https://raw.githubusercontent.com/bookwyrm-social/bookwyrm/main/bookwyrm/management/commands/initdb.py)]:

```
https://inventaire.io/api/entities?action=by-uris&uris=isbn:9782070360024
```

A French association's service that resolves an ISBN by querying BnF, BNE, Open Library and Wikidata [V1]. Data CC0 [M]; server code AGPL-3.0 ([README](https://raw.githubusercontent.com/inventaire/inventaire/main/README.md)) [V1], so read it for facts only. It adds little over calling the same sources ourselves.

**Crossref** [V1: [REST API doc](https://raw.githubusercontent.com/CrossRef/rest-api-doc/master/README.md), marked deprecated]:

```
https://api.crossref.org/works?filter=isbn:9782070360024&mailto=contact@example.org
```

"Crossref asserts no claims of ownership to individual items of bibliographic metadata … may be cached and incorporated into the user's content and systems" [V1]. A `mailto` in the query or the User-Agent routes to the polite pool; limits are announced in `X-Rate-Limit-Limit` and `X-Rate-Limit-Interval` headers [V1]. Since 1 December 2025 the limits depend on the request type ([announcement](https://www.crossref.org/blog/announcing-changes-to-rest-api-rate-limits/)) [FC]: public pool 5 requests/s for one record, 1 request/s for lists, concurrency 1; polite pool 10 and 3 requests/s, concurrency 3. A `filter=isbn:` call is a list request. Covers scholarly books with DOIs only.

**OpenAlex.** The works filter list has no ISBN filter ([filter-works](https://raw.githubusercontent.com/ourresearch/openalex-docs/main/api-entities/works/filter-works.md)) [V1]. Not usable for ISBN lookup, whatever its key policy (§6.1). Data CC0 ([README](https://raw.githubusercontent.com/ourresearch/openalex-docs/main/README.md)) [V1].

**DOAB / OAPEN.** `https://directory.doabooks.org/rest/search?query=isbn:9782070360024&expand=metadata` [M]; an older OAI endpoint appears in [regluit](https://raw.githubusercontent.com/EbookFoundation/regluit/master/core/loaders/doab.py) [V1]. Open-access books only.

**BookBrainz.** REST by BBID and search, JSON dumps [M]. Core data CC0, other public data CC BY-SA 4.0 ([licensing page](https://raw.githubusercontent.com/metabrainz/bookbrainz-site/master/src/client/components/pages/licensing.tsx)) [V1]. Very low coverage.

**Hardcover** [V1: [API guide](https://raw.githubusercontent.com/hardcoverapp/hardcover-docs/main/src/content/docs/api/Getting-Started.mdx), re-read]. GraphQL `POST https://api.hardcover.app/v1/graphql` with `authorization: Bearer {token}`. Free plan: 5,000 requests per day, 60 per minute, burst 10; Supporter: 50,000 per day; 30 s query timeout; "in beta", tokens may be reset. "we make no copyright or proprietary rights over this data"; a commercial product "may not use data owned by users"; no training of language models; images are user uploads. "CORS is open" but queries "are not allowed to run in the browser". Scoped personal tokens replaced the old ones in August 2026 [V1]. The ISBN query shape was not read. Low value for French titles: skip.

**Project Gutenberg, Standard Ebooks.** Public-domain editions without ISBNs [M]. Standard Ebooks' bulk feeds are tied to a Patrons Circle account ([feeds page](https://raw.githubusercontent.com/standardebooks/web/master/www/feeds/index.php)) [V1, weak]. Out of scope for ISBN lookup.

### 3.10 OCLC WorldCat — excluded

- Classify is discontinued: the isbnlib plugin is marked "DISCONTINUED" ([PyPI](https://pypi.org/pypi/isbnlib-oclc/json)) [V1]; web interface and API discontinued on 31 January 2024 ([OCLC](https://help.oclc.org/Librarian_Toolbox/Troubleshooting/Why_is_OCLC_discontinuing_OCLC_Classify)) [FC].
- The Search API needs a WSKey and a subscription [M].
- Zotero's translator (updated 2026-08-28) calls an internal `search.worldcat.org/api/search` with a cookie and a response-decryption step ([source](https://raw.githubusercontent.com/zotero/translators/master/Open%20WorldCat.js)) [V1]. That is not a public API. OCLC litigates against bulk scraping [M]. **Do not replicate it**, and do not use datasets derived from such scraping.

### 3.11 International ISBN Agency

- **Range message.** Open-source libraries regenerate their hyphenation tables from `https://www.isbn-international.org/export_rangemessage.xml` ([python-stdnum updater](https://raw.githubusercontent.com/arthurdejong/python-stdnum/master/update/isbn.py), [isbn3](https://raw.githubusercontent.com/inventaire/isbn3/main/README.md), [biblys/isbn](https://raw.githubusercontent.com/biblys/isbn/main/README.md)) [V2]. The file carries a serial number and a date [V1].
- **Licence: not found.** None of the three projects states one; they ship derived tables under MIT or LGPL. From memory the Agency offers the file free of charge for implementing ISBN handling and keeps its copyright [M, low]. Decision: generate our table from the Agency's XML with our own script, record serial and date, treat prefix ranges as facts, and never copy python-stdnum's LGPL `isbn.dat`.
- **Global Register of Publishers**: free web search of publisher prefixes, no API, no bulk download, all rights reserved [M]. AFNIL, the French agency, has no public API [M]. Do not scrape either.

### 3.12 eBook ISBNs

- Each format of a title has its own ISBN (print, EPUB, PDF) [M]. A lookup on the eBook ISBN often misses where the print ISBN hits.
- Reseller mode: take the print ISBN from the ONIX `RelatedProduct` block. Standalone mode: fall back to a title and author search and let the user confirm.
- BnF: digital legal deposit of eBooks is recent and its cataloguing is partial [M, low]. The lookup fields work on what is catalogued [V2].
- Google Books exposes `saleInfo.isEbook` and `accessInfo.epub` [V1] and is the source most likely to know an eBook ISBN [M].
- DNB has collected online publications under legal deposit for years [M].

### 3.13 Changed terms or shut down, 2024–2026

| Service | What happened | Confidence |
|---|---|---|
| OCLC Classify | Discontinued on 31 January 2024 | [FC] |
| OCLC WorldCat website | Internal search API behind a cookie and response decryption | [V1] |
| OpenAlex | API key required and credit-based limits from 2026-02-13; `mailto` ignored | [FC] |
| Hardcover | Scoped tokens (Aug 2026), published limits, LLM-training ban | [V1] |
| Open Library | Amazon PA-API 5 being replaced by the "Creators API" in its importer; explicit nginx limits and bot rules [V1]. The October 2024 breach and the end of *Hachette v. Internet Archive* affected lending, not metadata [M] | [V1] / [M] |
| Amazon | PA-API 5.0 deprecated 30 April 2026 and shut off 15 May 2026; replaced by the Creators API (§4.5) | [FC] |
| Crossref | GitHub REST docs deprecated [V1]; per-request-type limits since 2025-12-01 (lists: 1 request/s public, 3 polite) | [V1] / [FC] |
| Libris | Legacy `xsearch` on its way out | [V1], date unknown |
| British Library / BNB | Crippled by the October 2023 cyber-attack; linked-data access offline through 2024, relaunched later | [M, low] |
| NDL Search | Relaunched January 2024 on `ndlsearch.ndl.go.jp`; old URLs dead | [M] |
| Trove | API v2 retired for v3; keys reissued | [M, low] |
| Library of Congress | "New platform in 2025" unsupported; `lx2.loc.gov` still listed, reported reachable over plain HTTP on port 210 only | [FC] |
| Wikidata Query Service | Graph split live on 9 May 2025 | [FC] |
| DNB | "Blurb" hyperlinks in SRU/OAI data paused in July 2026, back with a new structure on 25 August 2026; old links stop on 25 February 2027 ([DNB](https://www.dnb.de/EN/Professionell/Metadatendienste/Kataloganreicherung/Inhaltstexte/inhaltstexte.html)) | [FC] |
| Goodreads | No new developer keys since 8 December 2020 | [FC] |
| ONIX | 3.1.1 (2024) added per-resource `EpubUsageConstraint` and `EpubLicense` [V1]; Issue 75 current, List 158 code 56 added in Issue 70 and 57 in Issue 71 [FC]; release 3.1.3 not confirmed by search | [V1] / [FC] |

---

## 4. French and EU trade sources

Vendor-side facts in this section are [M] unless a link is given. No vendor site, price list or contract was read by anyone. Treat §4.2 to §4.5 as a briefing for vendor calls.

### 4.1 ONIX for Books: the feed a reseller already holds

ONIX is the XML format in which publishers and distributors send product records to retailers.

- **Versions.** Jonix, a Java library generated from EDItEUR's schema, tracks ONIX 3.1.3 with code lists Issue 75 in its `2026-10` release; it moved to 3.1.0 in 2023-04, 3.1.1 in 2024-04 and 3.1.2 in 2025-01 ([README](https://raw.githubusercontent.com/zach-m/jonix/master/README.md)) [V1, re-read]. onixcheck validates "ONIX versions 2.1, 3.0, and 3.1 against the official XML Schema" ([PyPI](https://pypi.org/pypi/onixcheck/json)) [V1]. ONIX 3.1 is in production tooling [V2].
- **France.** The e-distributor Immatériel publishes an MIT "ONIX 3.0 & 2.1 parser for Ruby"; contributors include TEA "The Ebook Alternative" (now Vivlio) and Bookmate, both on the receiving end ([README](https://raw.githubusercontent.com/immateriel/im_onix/master/README.md), [gemspec](https://raw.githubusercontent.com/immateriel/im_onix/master/im_onix.gemspec)) [V1]. French eBook feeds are ONIX 3.0, with 2.1 still handled.
- ONIX and its code lists are free to implement under EDItEUR's licence [M].

**Where a cover sits and which fields carry its terms.** Path: `Product / CollateralDetail / SupportingResource / ResourceVersion / ResourceLink`. The schema calls `SupportingResource` "a supporting resource used for marketing and promotional purposes, eg a cover image" ([Jonix](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-onix3/src/main/java/com/tectonica/jonix/onix3/SupportingResource.java)) [V1]. Quotes below are EDItEUR text relayed by Jonix.

| Element | List | Content | Use for the card | Conf. |
|---|---|---|---|---|
| `ResourceContentType` | 158 | 01 Front cover; 02 Back cover; 03 Cover / pack; 27 thumbnail (transitional); **29 Full cover** "Includes cover, back cover, spine and - where appropriate - any flaps"; 45 Cover holding image (placeholder); 51 Cover flap image; **56 Spine image** "2D, portrait orientation"; 57 Spine panorama image | Prefer 29, or 01 + 56 + 02. Never print 45. Otherwise generate the spine | 01–55 [V2] ([im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-158.yml), [Jonix](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ResourceContentTypes.java)); 56–57 [V1] Jonix |
| `ContentAudience` | 154 | 00 Unrestricted; 01 Restricted = "Distribution by agreement between the parties to the ONIX exchange … material which is not for general distribution"; 02 Booktrade; 03 End customers; 07 Press; 09 Search engine index; 10 Social media; 13 LLM training | Use only 00 or 03 | 00–10 [V2] ([im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-154.yml), [Jonix](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ContentAudiences.java)); 11–13 [V1] |
| `Territory` in `SupportingResource` | — | Added at 3.0.3: "the resource should be used by recipients in the specified territory only" | Filter on the country of the order | [V2] (Jonix; [im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/lib/onix/supporting_resource.rb)) |
| `ResourceFeature` | 160 | 01 Required credit: "Credit that must be displayed when a resource is used"; 02 Caption; 03 Copyright holder "(indicative only, as the resource can be used without consultation)"; 07 alternative text; 08 background colour; 09 image attribute (List 256) | Print 01 on the card; store 03; reuse 07 for accessibility | [V2] ([im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-160.yml), [Jonix](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ResourceFeatureTypes.java)) |
| `ResourceForm` | 161 | 01 Linkable: the recipient "may embed the URL … and need not host an independent copy". 02 Downloadable: for "the ONIX recipient's direct use"; the recipient "must host an independent copy" | Printing needs the bytes: fetch server-side in the reseller's name; never hotlink a 02 resource from the buyer's browser | Codes [V2] ([im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-161.yml)); text [V1] ([Jonix](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ResourceForms.java)) |
| `ResourceVersionFeature` | 162 | 01 file format; 02 / 03 image height / width in pixels; 04 filename; 06 MD5; 07 exact size; 08 SHA-256 | Resolution check before download; integrity check after | [V2] ([im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-162.yml), Jonix) |
| `ContentDate` in `ResourceVersion` | 155 | 14 From date ("embargoed until this date"); 15 Until date ("may be referenced or used"); 17 Last updated; 27 / 28 Available from / until (download window) | Refuse outside 14–15; re-fetch when 17 changes | Codes [V2] ([im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-155.yml)); text [V1] ([Jonix](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ContentDateRoles.java)) |
| `EpubUsageConstraint` in `ResourceVersion` | 145 / 146 | "Added `<EpubUsageConstraint>` and `<EpubLicense>` at revision 3.1.1". Status 01 permitted, 02 permitted with limit, 03 prohibited. Type 11 Text and data mining covers "the product metadata or supporting resources". Type 02 Print means "Make physical copy of extract" — of the book, not of the cover | Read and honour it. Today it mainly expresses the text-and-data-mining opt-out | [V1] ([ResourceVersion](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-onix3/src/main/java/com/tectonica/jonix/onix3/ResourceVersion.java), [EpubUsageConstraint](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-onix3/src/main/java/com/tectonica/jonix/onix3/EpubUsageConstraint.java), [List 145](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/UsageTypes.java), [List 146](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/UsageStatuss.java)) |
| `EpubLicense` in `ResourceVersion` | 218 | Licence name plus a link to an expression: 01 human-readable, 02 professional-readable, 10 ONIX-PL, 20 ODRL | The standard slot for a publisher's "print on a gift card allowed" licence | [V1] Jonix; codes [V1] ([im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-218.yml)) |

**What ONIX does not say.** No audience, usage type or feature means "may be reproduced on a physical object". The default remains "promotional use by supply-chain partners" ([BISG](https://es.slideshare.net/bisg/bisg-webcast-onix-for-books-v30-introduction-1787290)) [sib], bounded by the trading contract. List 158 code 99 "License" concerns the licence of the product content and is deprecated in favour of `<EpubLicense>` [V1]. A print permission therefore needs a named `EpubLicense` on the cover's `ResourceVersion` (3.1.1 or later) or a proprietary flag agreed with the distributor. Doc 07 reaches the same conclusion from other sources.

**Text and data mining.** List 145 code 11 lets a publisher prohibit mining of "supporting resources", which includes covers [V1]. Identifying a font on a cover, or training a spine model on covers, is mining; under the EU DSM Directive a machine-readable reservation removes the commercial exception [M]. Honour status 03.

**Subjects, identifiers, outlets.**

- Subject schemes (List 27): 29 CLIL (France, four-digit codes, published on a Dilicom-hosted domain, `clil.centprod.com`); 28 Thèmes Electre; 93–99 Thema; A5 Rameau; 26 Warengruppen-Systematik (Germany); 32 NUR (Netherlands [M]); 53–58 CCE (Italian); 51 Spanish ISBN UDC subset ([im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-27.yml), [Jonix](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/SubjectSchemeIdentifiers.java)) [V2]. CLIL and Thema are free to use [M].
- Electre is wired into the standard: List 44 codes 28 "Identifiant Editeur Electre" and 30 "Identifiant Marque Electre" ([Jonix](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/NameIdentifierTypes.java)) [V1]. Parties are otherwise identified by GLN, the GS1 location number (List 44 and [List 92](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/SupplierIdentifierTypes.java), code 06) [V2]; im_onix exposes `publisher_gln` and `distributor_gln` ([product.rb](https://raw.githubusercontent.com/immateriel/im_onix/master/lib/onix/product.rb)) [V1].
- French actors have ONIX retail outlet codes (List 139): DIL Dilicom, ELE Electre, EPA E-Pagine, FNC Fnac, IMM Immatériel.fr, IZN Izneo, NUM Numilog, TEA Vivlio, YOU Youboox, YSC Youscribe, KBO Kobo, DEM De Marque ([im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-139.yml), [Jonix](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/OnixRetailSalesOutletIdss.java)) [V2]. Publishers use them in `SalesRestriction / SalesOutlet` to include or exclude named retailers [V1]. We have no code: a publisher would name us as a proprietary outlet.
- Parser details read in im_onix: [resource_version.rb](https://raw.githubusercontent.com/immateriel/im_onix/master/lib/onix/resource_version.rb), [collateral_detail.rb](https://raw.githubusercontent.com/immateriel/im_onix/master/lib/onix/collateral_detail.rb), [price.rb](https://raw.githubusercontent.com/immateriel/im_onix/master/lib/onix/price.rb); code lists [144](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-144.yml) (protection type) and [145](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-145.yml).

### 4.2 France — trade databases

| Source | What it is | Who can get access | Interface | Licence constraints | Covers |
|---|---|---|---|---|---|
| **Dilicom — FEL** (Fichier Exhaustif du Livre) | Interprofessional EDI hub of the French book trade. FEL is the reference file of titles commercially available: EAN, title, publisher, distributor, fixed retail price, availability, dimensions, CLIL [M] | Dilicom members: booksellers, distributors, publishers; software vendors and other third parties by specific contract [M]. The portal URL in a README example carries a 13-digit, GLN-like number [V1] | Bulk file with updates [M] | Members' own trade use, no redistribution [M]. General terms: [CGV 2021](http://dilicms.centprod.com/file/dilicom/file.do?path=/Conditions+g%C3%A9n%C3%A9rales/CGV+-+Dilicom.pdf) [sib] | Separate service: Distrimage |
| **Dilicom — FEL à la demande** | Per-EAN lookup of the FEL | Same; needs a user and a password | SOAP, EAN only, no free-text search, up to 100 EANs per request ([scraper](https://raw.githubusercontent.com/vindarel/bookshops/master/bookshops/frFR/dilicom/dilicomScraper.py), [README](https://raw.githubusercontent.com/vindarel/bookshops/master/README.md)) [V1] | As above | None: "Not available in FEL à la demande" (image and description) [V1] |
| **Dilicom — Distrimage** | Front-cover images for FEL titles | Members | File or URL service [M] | Free inside bookshop management software; "pour tout autre usage (internet, extranet), il convient de prendre contact avec le service clients" ([SLF, 2012](https://www.syndicat-librairie.fr/le_service_distrimage_de_dilicom)) [sib] | Yes, by contract |
| **Dilicom — Hub numérique** | Single connection between e-distributors and eBook resellers: catalogue notices (ONIX), availability, order, download link; also library lending (PNB) [M] | Resellers with a GLN, a Dilicom contract and an agreement with each distributor [M] | Web-service API; notices as ONIX 3.0 [M] | Data serves the sale of the titles by that reseller [M] | Cover URLs inside the notices [M] |
| **Electre** (Electre NG and its API) | Commercial bibliographic database of the Cercle de la Librairie group: French titles in print, forthcoming and out of print, summaries, tables of contents, author notes, covers, Rameau and Dewey indexing, own themes and identifiers [M; identifiers in ONIX lists V1] | Any organisation by annual subscription; customers are libraries, booksellers, publishers [M] | Web application and REST API [M] | Subscriber's internal use and display in its own catalogue; no redistribution or bulk reuse [M] | Yes, as a licensed option [M] |
| **Tite-Live** (Médiabase, ePagine, Place des Libraires) | Bookshop software vendor; Médiabase ships with it; ePagine is its eBook platform for independent bookshops (ONIX outlet EPA [V2]; its staff co-author Readium LCP notes, [note](https://raw.githubusercontent.com/readium/lcp-specs/master/notes/lcp-key-retrieval.md) [V1]) | Tite-Live customers only [M] | Bundled; web services for customers' sites [M] | Use within the customer bookshop's tools and site [M] | Yes, for customers [M] |
| **Decitre ORB** (Outil de Recherche Bibliographique) | "a french database of bibliographic information available by subscription" ([VuFind config](https://raw.githubusercontent.com/vufind-org/vufind/dev/config/vufind/config.ini)) [V1]; run by the Decitre bookseller group [M] | Subscribers: libraries, booksellers, institutions [M] | REST JSON ([VuFind Orb.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Orb.php), [Coce README](https://raw.githubusercontent.com/fredericd/coce/master/README.md)) [V2] | Subscription terms not read | `images.front.thumbnail.src` and `images.front.original.src` [V1] |

Request examples, for the day a contract exists:

```
Decitre ORB       GET https://api.base-orb.fr/v1/products?eans=9782070360024&sort=ean_asc
                  (HTTP basic authentication: user + key)

Dilicom FEL       POST http://websfel.centprod.com/v2/DemandeFicheProduit     (SOAP 1.1)
à la demande      body element demandeFicheProduit, namespace http://fel.ws.accelya.com/ :
                    <demandeur>{USER}</demandeur><motDePasse>{PASSWORD}</motDePasse>
                    <ean13s>9782070360024</ean13s><multiple>false</multiple>
```

The Dilicom answer carries title, author, publisher, price (integer, thousandths of a euro), publication date, availability code, thickness, height, width and weight; an unknown EAN returns the diagnostic `UNKNOWN_EAN` [V1]. The endpoint is plain `http://` in the 2020s client code read; whether an `https` endpoint exists is unknown.

The free software for independent bookshops, Abelujo, uses Dilicom when the shop has an account and otherwise scrapes a bookshop website ([README](https://raw.githubusercontent.com/vindarel/abelujo/master/README.md)) [V1]. That is the practical state of "open" French trade data: there is none.

### 4.3 France — eBook distributors and reseller platforms

All [M] unless marked. None publishes reseller terms that anyone could read.

| Actor | Role | Feed / API | Notes |
|---|---|---|---|
| **Immatériel.fr** | Distributor for independent publishers; operates the 7switch store | ONIX 3.0 (and 2.1) feed plus a reseller web service; also via the Dilicom Hub | Its open parser shows what it exchanges: front cover (largest version chosen), EPUB sample, description, CLIL and BISAC, GLNs, print-product link, sales rights, outlet restrictions, prices, protection type [V1] |
| **Eden Livres** | Distribution platform of Madrigall (Gallimard, Flammarion), Média-Participations (La Martinière, Seuil) and Actes Sud | Reseller API and ONIX; Dilicom Hub | Technology by De Marque (Cantook) [M, medium] |
| **e-Plateforme** | Editis / Interforum digital distribution | Reseller API and ONIX; Dilicom Hub | Also De Marque technology [M, medium] |
| **Hachette Livre** | Own digital distribution since 2012 | ONIX; Dilicom Hub | |
| **Numilog** | Aggregator-distributor, store, library offer | ONIX; Dilicom Hub; outlet code NUM [V2] | |
| **Izneo** | Digital comics platform (publishers plus Fnac Darty) | Own platform; feeds to partner stores | Outlet code IZN [V2] |
| **Vivlio (ex-TEA)** | Reseller platform behind Decitre, Furet du Nord, Cultura, E.Leclerc | Consumes distributor ONIX | Contributor to im_onix [V1]; outlet code TEA [V2] |
| **Kobo (for Fnac)** | Fnac's eBook catalogue is operated by Rakuten Kobo | Publishers and distributors feed Kobo ONIX; Fnac gets catalogue and sales through Kobo's private partner integration | Print rights would have to pass through Kobo as well |

### 4.4 Other EU countries, UK, US

All [M] unless marked; none checked against 2026 terms. The free national alternatives are in §2.1.

| Country | Source (operator) | Access | Interface | Covers |
|---|---|---|---|---|
| DE | **VLB** (MVB, Börsenverein group) | Paid licences for booksellers, libraries, vendors; publishers pay to list | REST API and ONIX exports; cover API `https://api.vlb.de/api/v1/cover/{isbn}/{size}` with a token ([VuFind Buchhandel.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Buchhandel.php)) [V1] | Token-gated; VuFind's [config](https://raw.githubusercontent.com/vufind-org/vufind/dev/config/vufind/config.ini) warns that templates may need changes "in order to comply with terms of service" [V1]; use limited to promoting the title [M] |
| DE | **Libri**, **Zeitfracht** (ex-KNV) | Wholesalers; data for their bookseller customers | Proprietary feeds | By contract |
| NL / BE | **CB** (Centraal Boekhuis); **Boekenbank** for Flanders (ONIX List 92 "Flemish supplier code" [V1]) | Trade customers | ONIX feeds, portal | By contract |
| ES | **DILVE** (FGEE) | Registered professional users; publishers load ONIX | Web service returning ONIX and resources | Publisher-supplied; tied to promoting and selling the book. Free fallback: the Ministry of Culture ISBN database ([isbnlib-mcues](https://pypi.org/pypi/isbnlib-mcues/json)) [V1] |
| IT | **Informazioni Editoriali** (Alice, Arianna+) | Trade subscribers | Data licences, portal | By contract |
| UK | **Nielsen BookData** (NielsenIQ); **BDS** | Subscription by quote | Feeds and web services | Licensed option |
| US | **Bowker** Books In Print; Syndetics | Subscription; Syndetics is sold to libraries ([Koha prefs](https://raw.githubusercontent.com/Koha-Community/Koha/main/koha-tmpl/intranet-tmpl/prog/en/modules/admin/preferences/enhanced_content.pref)) [V1] | Feeds and web services | Licensed for catalogue display |

### 4.5 Global commercial APIs, affiliate and partner programmes

| Source | Status as of 2026-11 | Constraints | Verdict |
|---|---|---|---|
| **ISBNdb** | Paid REST API, self-serve by credit card, no trade status needed [M]. Price list on [isbndb.com](https://isbndb.com/isbn-database), read through search on 2026-11-12 [FC]: Basic USD 14.99 per month (5,000 searches a day, 1 call per second, 7-day trial), Premium USD 35.99 (15,000 a day, 3 calls per second), Pro USD 99.99 (50,000 a day, 5 calls per second, the only tier with price data), Enterprise by quote. The figures recalled in the draft (USD 15 / 30 / 75) were out of date. Licence terms were not read. isbnlib documents an `isbndb` provider with an API key ([isbnlib](https://pypi.org/pypi/isbnlib/json)) [V1]; Open Library has imported ISBNdb bulk dumps ([importer](https://raw.githubusercontent.com/internetarchive/openlibrary/master/scripts/providers/isbndb.py)) [V1] | Use inside your own application; no resale as a competing dataset; cover images of unknown provenance, no rights warranty [M] | Possible for metadata pre-fill, not for covers. Test the hit rate on French eBook ISBNs first |
| **Amazon** | **PA-API 5.0 replaced by the Creators API**: Amazon's documentation reads "PA-API will be deprecated on May 15th, 2026. Please migrate to Creators API"; the endpoint has returned errors since that date ([PA-API docs](https://webservices.amazon.com/paapi5/documentation/), [Open Library issue](https://github.com/internetarchive/openlibrary/issues/12255)) [FC]. OAuth2 credentials from the Associates portal. python-amazon-paapi deprecated PA-API in 6.0.0 (2026-01-29) and removed it in 7.0.0 (2026-09-04) ([changelog](https://raw.githubusercontent.com/sergioteula/python-amazon-paapi/master/CHANGELOG.md), [migration guide](https://raw.githubusercontent.com/sergioteula/python-amazon-paapi/master/docs/pages/migration-guide-6.md), [PyPI](https://pypi.org/pypi/python-amazon-paapi/json), [Creators SDK on PyPI](https://pypi.org/pypi/amazon-creatorsapi-python-sdk/json)); Open Library added an `AmazonCreatorsAPI` class ([vendors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/core/vendors.py)) [V2] | Associates account with qualifying sales; content only to send traffic to Amazon, with links; short cache window; images not stored; **no offline or printed use** [M] | **Excluded** |
| **Goodreads** | No new API keys since 8 December 2020 [FC] | — | Excluded |
| **LibraryThing** | Cover service by developer key (`covers.librarything.com/devkey/{key}/…`, [VuFind](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/LibraryThing.php)) [V1]; thingISBN | Non-commercial, about 1,000 requests a day; the commercial offer is "LibraryThing for Libraries" [M, low] | Excluded for a commercial product |
| **Babelio** | No public API. Its B2B product "Babelthèque" injects reviews into library catalogues ([Koha prefs](https://raw.githubusercontent.com/Koha-Community/Koha/main/koha-tmpl/intranet-tmpl/prog/en/modules/admin/preferences/enhanced_content.pref)) [V1] | Site terms forbid extraction [M] | Excluded |
| **Kobo / Rakuten** | Affiliate programme with links and a product feed; no public catalogue API [M] | Feed only to promote Kobo links [M] | Not a data source; relevant only as Fnac's back end |
| **Fnac** | Affiliate programme (network not verified) and a marketplace seller API [M, low] | Affiliate content only to drive traffic to fnac.com [M] | Not a data source |
| **Bookshop.org** | Affiliate programme; not active in France; no public catalogue API [M]. Has an ONIX outlet code (BSO) [V1] | — | Irrelevant for France |
| **Google Play Books Partner Program** | Publisher-side channel to sell on Google Play; ingests ONIX (onixcheck ships a `google` profile [V1]) | Not a data service | Not a source |

### 4.6 What the reseller holds and what it may do

**Per title, a French eBook reseller already holds** (most fields verified in Immatériel's parser [V1]; contributors, extent and audience are standard ONIX [M]; that every distributor fills them is [M]):

- identifiers (EAN / ISBN-13, proprietary ids), publisher, imprint and distributor with GLNs;
- title, subtitle, collection, contributors with roles and biographies, language, extent;
- CLIL, Thema or BISAC subjects, audience;
- description texts; the **front cover** as `SupportingResource` 01 with URL, pixel size, format, MD5 and last-updated date; an EPUB extract (code 15); rarely a back cover; never, to the researcher's knowledge, a full cover or spine [M, low];
- the link to the print edition (`RelatedProduct`), publication and embargo dates;
- sales rights by country, outlet restrictions, fixed prices including tax per territory, protection type (List 144: none, watermark, Adobe DRM, Readium LCP) and usage constraints.

**What those contracts typically allow** [M; no reseller-facing contract was read by anyone]: display of metadata, cover and extract to present, promote and sell that eBook on the reseller's storefronts, apps, newsletters and advertising, in the licensed territory, while the title is on sale. Duties: faithful reproduction, prompt updates and takedowns, respect of embargo, price and territory, no sub-licensing, no transfer of the feed to third parties other than subcontractors acting for the reseller.

**What they do not obviously allow**: reproduction on a physical printed object, alteration (a generated spine and a back panel are an adaptation), and any use by a third party in its own name. The reason is ordinary contract interpretation and the limits of the publisher's own upstream grant (doc 07 §3.2; see §6.1 on the L. 131-3 correction).

**Consequences.**

1. Operate as the reseller's **subcontractor**: the reseller passes ONIX, or extracted fields, plus the cover bytes or URL. We keep no cross-reseller catalogue.
2. Evaluate per title: `ContentAudience` (reject 01 and trade-only audiences), `Territory`, `ContentDate` 14 and 15, `EpubUsageConstraint`, outlet restrictions, and a print permission flag.
3. Obtain that flag at distributor level (Immatériel, Eden Livres, e-Plateforme, Hachette, and Kobo for Fnac), as a contract rider or a named `EpubLicense`, rather than publisher by publisher. Default without the flag: typographic card without the cover image.
4. Always print the `ResourceFeature` 01 credit when present.

---

## 5. Legal framing of "allowed"

Everything in this section is [M] unless a marker says otherwise: no statute, directive or judgment could be opened in this session. Article numbers must be re-read at source before any external use. Doc 02 §6 and doc 07 treat the same rules for images in more depth.

### 5.1 Four separate questions

A source is "allowed" only when all four have an acceptable answer.

| Question | What it covers | Where the answer lives |
|---|---|---|
| 1. May we call it? | Access terms: API terms accepted with a key, site terms of use, subscription contract, robots rules | The operator's terms |
| 2. May we keep and reuse what it returns? | Copyright in records as written, and the database maker's sui generis right | The data licence, or the contract |
| 3. Does the answer carry someone else's work? | Cover images, summaries and back-cover texts, author biographies, reviews | Not granted by any metadata licence |
| 4. Is there personal data? | Author and contributor names, dates of birth | GDPR; the Licence Ouverte says reuse stays subject to data-protection law [V1] |

Two consequences shape the design:

- **Individual bibliographic facts are free; texts are not.** A title, an author's name, a publisher, a date, an ISBN and a page count are facts. A summary or biography is a text written by someone (the publisher, or Electre's editors). We store and print facts. We take descriptive text only from the reseller's ONIX under the reseller's licence, or from the user.
- **A metadata licence never clears a cover.** The BnF page that announces the Licence Ouverte for metadata restricts Gallica images to non-commercial use in the next sentence [V1].

### 5.2 Licences we rely on

| Licence | Sources | What it grants | Condition | Verified? |
|---|---|---|---|---|
| **Licence Ouverte 2.0** (Etalab) | BnF catalogue and data.bnf.fr [V1, 2016 statement]; SUDOC [M] | Free, non-exclusive, worldwide, unlimited reuse "for commercial or non-commercial purposes": copy, adapt, redistribute, include in a product. Covers the grantor's copyright and "sui generis rights of database producers". Governed by French law; stated compatible with CC BY, OGL and ODC-BY | Name the source (at least the grantor) and the date of the most recent update of the reused information; a hyperlink to the source can satisfy this. Do not suggest endorsement; do not mislead about content, source or date | Text [V1] ([English](https://raw.githubusercontent.com/etalab/licence-ouverte/master/open-licence.md), [French](https://raw.githubusercontent.com/etalab/licence-ouverte/master/LO.md)). Version applied by BnF today [M]: the 2016 page predates version 2.0 |
| **CC0 1.0** | Libris (Swedish national bibliography and authority records) [FC]; Open Library contributions [V1]; BookBrainz core [V1]; OpenAlex [V1]; DNB, lobid, Finna metadata [FC]; K10plus, BNE, Wikidata, Inventaire, DOAB/OAPEN [M] | Waiver of copyright and database rights | None. Credit the source anyway | As marked |
| **CC BY-SA 4.0** | BookBrainz non-core data [V1] | Reuse with attribution and share-alike | Share-alike would bind derived datasets | Not used |
| **CC BY-NC-ND** | Some KB (NL) datasets [V1] | Non-commercial, no derivatives | Incompatible with our use | Check which KB collection the SRU serves |
| **Crossref statement** | Crossref | No ownership claimed on individual items; may be cached and incorporated [V1] | Etiquette: contact address | [V1] |
| **Operator API terms** | Google Books [FC for the fee clause, M for caching]; Hardcover [V1]; Europeana, Trove [M] | A right to call and display | Google: no fee charged to users without written permission, results not altered, branding, no permanent copy. Hardcover: no user-owned data in a commercial product, token secrecy, no LLM training | Google wording unread |

### 5.3 Terms of use as contracts

- An API key is an acceptance step: creating a Google or Hardcover key binds us to their terms. A footer link without acceptance is a weak contract; an account or key is a strong one (doc 02 §6.5–6.6).
- *Ryanair v PR Aviation* (CJEU, C-30/14, 2015): when a database is protected neither by copyright nor by the sui generis right, the directive's mandatory user rights do not apply, and the owner may restrict use by contract.
- For anonymous, keyless services (Open Library, BnF, DNB) we still follow the documented rules — User-Agent, rate limits — because they are the good-faith evidence if a dispute arises.

### 5.4 The sui generis database right

- **Rule.** Directive 96/9/EC art. 7; in France CPI L. 341-1 to L. 343-7. The maker of a database who made a substantial investment in obtaining, verifying or presenting its contents may prohibit extraction or re-utilisation of all or a substantial part, and the **repeated and systematic** extraction of insubstantial parts that conflicts with normal exploitation (art. 7(5); CPI L. 342-2). Term: 15 years, renewed by new investment. France has criminal penalties (L. 343-4).
- **Who holds it.** Only makers established in the EU (art. 11): Electre, Dilicom, Decitre, Fnac Darty, Babelio, MVB and Amazon's EU entities can; the Internet Archive and Google LLC cannot, and rely on copyright and contract instead.
- **Public bodies.** The Open Data Directive (EU) 2019/1024 bars public-sector bodies from using the database right to prevent re-use of documents (art. 1(6)). In France re-use of public information is governed by the Code des relations entre le public et l'administration; the Licence Ouverte text itself cites CRPA L. 300-2, L. 321-1 and L. 323-2 [V1 for the citation]. For BnF and ABES the open licence settles the question.
- **A lawful user's minimum right.** For a protected database made available to the public, a lawful user may extract insubstantial parts for any purpose, and a contrary clause is void (Dir. art. 8(1) and 15; CPI L. 342-3). This does not license a site whose terms we have not accepted and whose catalogue we would query systematically.
- **Case law that maps onto ISBN lookup** (table in doc 02 §6.7): *Innoweb v Wegener* (C-202/12, 2013) — a service that forwards user queries in real time to someone else's database and shows the results re-utilises the whole database; *CV-Online Latvia* (C-762/19, 2021) — copying a freely accessible database is prohibited when it threatens the maker's investment; *Leboncoin v Entreparticuliers* (Cass. 1re civ., 5 Oct 2022) — French courts enforce this against scrapers.

**Applied to our sources.**

| Situation | Database right | Our position |
|---|---|---|
| One-ISBN lookups against an open-licence source (BnF, DNB, Libris, Wikidata, Open Library) | Waived or licensed | Allowed, storable |
| The same against a terms-bound API (Google Books) | Not the issue; the contract is | Allowed within the terms; no durable copy |
| A real-time ISBN relay over a French retailer's or aggregator's site (Fnac, Decitre, Babelio, Electre) | *Innoweb* structure; L. 342-2; *parasitisme* | **Forbidden** |
| Mirroring a dump (Open Library, data.bnf.fr, lobid) | Licensed | Allowed; not needed at our volume |
| Building our own catalogue from reseller feeds | Belongs to the distributors and the reseller by contract | Forbidden across resellers; allowed per reseller as its subcontractor |
| Using a dataset scraped from WorldCat or a retailer | Infringing source | **Forbidden**, whatever the dataset's own "licence" |

### 5.5 What each class lets us do

"Print" here means printing **text facts** (title, author, publisher) on the card. Images follow doc 07.

| Class | Sources | Query on a user action | Store the result | Show in the form | Print facts | Bulk reuse |
|---|---|---|---|---|---|---|
| **A — open** | BnF, data.bnf.fr, SUDOC, DNB, K10plus, lobid, Libris, Finna, BNE, Wikidata, Open Library, Inventaire, Crossref | Yes | Yes, with source and date | Yes | Yes | Yes (Licence Ouverte: with attribution) |
| **B — terms-bound** | Google Books; Hardcover | Yes, within quota; Google Books only in a flow that charges the user nothing, or with Google's written permission [FC] | No durable copy | Yes, with the required branding | Yes once the user has confirmed the form: what is printed is the user's input (my analysis) | No |
| **C — contract** | Reseller ONIX; ORB; Electre; Dilicom; ISBNdb; VLB | Per contract | Per contract, scoped to the tenant | Per contract | Per contract | No |
| **D — excluded** | Amazon; Goodreads; LibraryThing; Babelio; Fnac, Kobo and Decitre sites; WorldCat internal API; ISBN publisher register | No | No | No | No | No |

### 5.6 Attribution

- **Licence Ouverte sources**: show the source and a date. Proposed wording on the lookup result and on a "Sources des données" page: "Données bibliographiques : Bibliothèque nationale de France (catalogue général) — notice consultée le {date}", with a link to the record. The licence asks for the date of the most recent update of the information; the retrieval date is our proxy when the record carries none (my reading).
- **CC0 sources**: no obligation; list them on the same page.
- **Google Books**: "Powered by Google" next to the result and a link to the volume page [M].
- The printed card needs no attribution line for text facts (my analysis). Record source and date in the generation log and in the PDF's metadata, as doc 07 requires for rights data.

### 5.7 Reserved rights, personal data, AI

- A publisher can reserve text and data mining on covers in ONIX (List 145 code 11) [V1], and French trade publishers have begun to publish such reservations on their websites (doc 02). The lookup pipeline must carry that flag to the font and spine tools.
- Author names are personal data. Showing them to identify a book the user asked about is ordinary use; we add no profiling, and no user identifier is sent to a catalogue.
- Hardcover forbids use of its API to train language models [V1]. We train none on metadata.

---

## 6. Gaps and open questions

### 6.1 Contradictions resolved

| Topic | A says | B says | Kept, and why |
|---|---|---|---|
| OpenAlex key and credits | pyalex README: key required since 2026-02-13; 100 credits/day without a key, 100,000 with a free key; singleton requests free, list requests 1 credit ([pyalex](https://raw.githubusercontent.com/J535D165/pyalex/main/README.md)) [V1] | Official docs repository: "You don't need an API key"; 100,000 credits/day for free users; singleton 1 credit, list 10 credits ([rate limits](https://raw.githubusercontent.com/ourresearch/openalex-docs/main/how-to-use-the-api/rate-limits-and-authentication.md)) [V1] | Resolved by the fact-check: OpenAlex's own announcement and deprecations page say a key is required since 2026-02-13 and `mailto` is ignored [FC]; the GitHub docs page is stale. Still unused: OpenAlex has no ISBN filter [V1] |
| Open Library rate limit | Open note: 180 requests/min per IP on JSON and API paths, read in nginx [V1]; public docs 1 and 3 requests/s [M] | Doc 02: 400/min on the cover server [V1]; 100 per 5 minutes for ISBN-keyed cover requests [M, low] | Different endpoints, no conflict. For metadata, 180/min is the verified ceiling; we design for **1 request per second** per outgoing IP, which satisfies every figure quoted |
| ONIX spine code | Doc 02: List 158 has no spine-only code (im_onix's copy of the list) [V1] | Trade note: 56 "Spine image", 57 "Spine panorama image" (Jonix, Issue 75) [V1] | The trade note. Jonix tracks the October 2026 issue; im_onix ships an older list. Re-read today |
| Legal basis for reading a reseller's licence narrowly | Trade note: "French law reads grants narrowly (CPI L.131-3)" [M] | Doc 07, after fact-check: L. 131-3 governs authors' contracts only; the reseller-level limit is ordinary contract interpretation plus the publisher's upstream grant [M] | Doc 07. It governs where the documents meet, and its correction is reasoned. The practical conclusion is unchanged |
| Server-side relay | Open note: "plan a thin server-side relay with a cache" for SRU sources | Docs 02 and 07: "no image proxy"; the service never fetches a cover into a PDF | Both. The relay carries **text metadata** under Licence Ouverte or CC0. It never fetches, caches or forwards an image. Lookup returns text only (doc 07, implication 18) |
| Google Books `country=FR` on the list call | Open note: use it "if the live check confirms it" [M] | Discovery document: no such parameter on the list method [V1] | The discovery document. Use the two-step form or an EU-hosted relay |
| DNB licence | Open note: CC0 [M] | DNBLab README licence line: "CC BY 3.0" [V1] | CC0 1.0, confirmed by DNB's 2015 press release [FC]. The README line covers the tutorial repository, not the catalogue data |

### 6.2 Could not verify

Nothing below was checked against an operator's live site, a vendor's price list or a contract.

- **Every price**: Dilicom membership, FEL, FEL à la demande and Hub fees; Electre NG and its API; Decitre ORB; Tite-Live; VLB licences and cover token; CB; DILVE; Informazioni Editoriali; Nielsen BookData; BDS; Bowker; ISBNdb 2026 plans, daily caps and bulk licences.
- **Who may subscribe**: whether Dilicom, Electre, ORB and VLB accept a service provider that is neither bookseller nor library, and under which contract. Whether Dilicom logins are GLNs (inferred from one README example).
- **Every licence and terms statement marked [M]**: the Licence Ouverte version and attribution wording used by BnF and ABES; CC0 for DNB, K10plus, lobid, Finna, BNE, Wikidata, DOAB/OAPEN, Inventaire; the licences of OPAC SBN, swisscovery, OBV/ÖNB and KBR data; which KB collection is under which licence.
- **Google Books**: exact wording on caching and branding, the official quota page, whether keyless calls are still served, whether list results depend on the caller's IP, and — new — the full text and current date of the Books API clause on charging users (seen through search only).
- **CORS headers** of every service except Open Library (source) and Hardcover (operator statement).
- **Open Library**: the figures on its public API page; dump frequency.
- **BnF**: current SRU record schemas and any rate limit; eBook legal-deposit coverage; whether the catalogue général was re-platformed in 2024–2026 (the wiki read dates from 2016); terms of the cover endpoint.
- **SUDOC**: current endpoint scheme and the `{ppn}.rdf` / `.xml` record URLs.
- **Library of Congress** SRU after the 2025 platform change; **British Library / BNB** endpoints; **Trove** v3 terms; **NDL** commercial conditions; **Europeana** key policy; any current API for **ICCU/SBN** and **KBR**.
- **OCLC**: Classify shutdown date, what the free tier of WorldCat Entities returns, the status of the scraping litigation.
- **Crossref** December 2025 limits; **Wikidata** query-service limits and graph-split details; **DOAB/OAPEN** REST path; **BookBrainz** API base URL; the Hardcover ISBN query shape.
- **International ISBN Agency**: the licence or terms attached to `RangeMessage.xml` and to the Global Register of Publishers.
- **Dilicom Hub**: current API form, whether all distributors deliver ONIX 3.0, which distributors and resellers are connected in 2026. Current Dilicom terms for cover images (only a 2012 notice [sib]).
- **French e-distributors**: reseller-facing terms of Immatériel, Eden Livres, e-Plateforme, Hachette, Numilog, Izneo; whether Eden Livres and e-Plateforme still run on De Marque technology; Immatériel's current ownership and API; ePagine's 2026 status; how Kobo passes catalogue data to Fnac.
- **ONIX in practice**: whether any French feed carries List 158 codes 02, 29, 51, 56 or 57, a `ContentAudience` other than 00, `ResourceFeature` 01, or the 3.1.1 elements on resources; the issue in which codes 56 and 57 were added; the content of List 256; EDItEUR's sunset dates for ONIX 2.1 and its licence text; CLIL and Thema licence texts.
- **Amazon**: the PA-API shutdown date, Creators API eligibility and licence text. **Goodreads, LibraryThing, Babelio, Kobo / Rakuten, Fnac, Bookshop.org, Google Play Books Partner Program**: all terms; the Fnac affiliate network.
- **Company facts from memory**: ownership and structure of Electre, Tite-Live, Decitre / ORB, MVB, Libri, Zeitfracht, CB, DILVE / FGEE, Informazioni Editoriali, NielsenIQ BookData, BDS, Bowker.
- **All legal authorities in §5**: directive and CPI article numbers, case references, CRPA articles.
- **Not researched at all**: other EU national libraries with possible open APIs (Poland, Norway, Denmark, Czechia, Luxembourg); the Swiss National Library; BAnQ and Library and Archives Canada for French-language titles from Québec; OPDS catalogues published by French resellers or distributors.

### 6.3 Questions to put to vendors and partners

1. Dilicom: can a service provider without bookseller status contract "FEL à la demande"? Price per call or per year? Is there an `https` endpoint? May results be cached, and for how long?
2. Decitre ORB and Electre: subscription for a non-bookseller; price; whether pre-filling an end user's form counts as "internal use"; storage rights.
3. Immatériel, Eden Livres, e-Plateforme, Hachette, Kobo: does the reseller agreement allow a subcontractor to receive the title record and cover? Can a print permission be granted for all titles of consenting publishers, and in which field would it travel (`EpubLicense`, proprietary flag)?
4. Any distributor: do feeds ever carry List 158 codes 29 or 56?
5. BnF: rate limit and CORS policy of `catalogue.bnf.fr/api/SRU`; terms of the cover endpoint; share of eBook ISBNs catalogued.
6. Google: confirm the Books API terms on caching, branding and quota for a commercial service.

### 6.4 Measurement plan

Hit rates decide the provider order and whether any paid source is worth it. Nobody measured them.

- Sample: 200 ISBNs from a French reseller catalogue — proposed mix 100 print 978-2, 20 print 979-10, 60 eBook ISBNs, 20 other language groups.
- Per provider, record: found or not; title and first author present; publisher and date present; latency; whether the eBook ISBN or only the print ISBN matched.
- Proposed decision rules: keep the default order if steps 4 and 5 of §1.2 find title and author for at least 90 % of print ISBNs; switch Google Books on by default only if it raises the eBook-ISBN hit rate by at least 10 points; open a vendor discussion (ISBNdb, ORB) only if the open cascade stays below 70 % on eBook ISBNs after the print-ISBN fallback.
- Run it from a networked session, at 1 request per second, once.

### 6.5 Sources

All files were fetched on 2026-11-12 (HTTP 200) by the researchers unless marked [sib]. Files I re-opened are marked †; files new to this synthesis are marked ‡.

**Licences and BnF**

- Etalab Licence Ouverte 2.0: [English](https://raw.githubusercontent.com/etalab/licence-ouverte/master/open-licence.md) ‡, [French](https://raw.githubusercontent.com/etalab/licence-ouverte/master/LO.md) ‡
- BnF: [hackathon wiki home](https://raw.githubusercontent.com/wiki/hackathonBnF/hackathon2016/Home.md) †, [API-SRU (Gallica)](https://raw.githubusercontent.com/wiki/hackathonBnF/hackathon2016/API-SRU.md) ‡, [Z39.50](https://raw.githubusercontent.com/wiki/hackathonBnF/hackathon2016/API-Z39.50.md), [OAI-PMH](https://raw.githubusercontent.com/wiki/hackathonBnF/hackathon2016/API-OAI-PMH.md), [Zotero BnF ISBN](https://raw.githubusercontent.com/zotero/translators/master/BnF%20ISBN.js) †, [isbnlib-bnf on PyPI](https://pypi.org/pypi/isbnlib-bnf/json) †, [isbnlib-bnf source](https://raw.githubusercontent.com/xlcnd/isbnlib-bnf/master/isbnlib_bnf/_bnf.py) ‡, [Inventaire BnF resolver](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/bnf/get_bnf_entry_from_isbn.ts) †, [BELTRANS BnF notes](https://raw.githubusercontent.com/kbrbe/beltrans-data-integration/main/data-sources/bnf/README.md)
- SUDOC: [node-sudoc on npm](https://registry.npmjs.org/sudoc) †

**Open Library, Google Books**

- Open Library: [nginx.conf](https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/nginx.conf) †, [web_nginx.conf](https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/web_nginx.conf) †, [processors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/plugins/openlibrary/processors.py) †, [code.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/plugins/openlibrary/code.py) †, [oldump.sh](https://raw.githubusercontent.com/internetarchive/openlibrary/master/scripts/oldump.sh), [vendors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/core/vendors.py), [messages.pot](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/i18n/messages.pot), [ISBNdb importer](https://raw.githubusercontent.com/internetarchive/openlibrary/master/scripts/providers/isbndb.py)
- Google Books: [discovery document](https://raw.githubusercontent.com/googleapis/google-api-go-client/main/books/v1/books-api.json) †, [calibre-web provider](https://raw.githubusercontent.com/janeczku/calibre-web/master/cps/metadata_provider/google.py), [isbnlib](https://pypi.org/pypi/isbnlib/json)

**Germany and other libraries**

- [DNBLab README](https://raw.githubusercontent.com/deutsche-nationalbibliothek/dnblab/main/README.md) †, [DNB SRU tutorial](https://raw.githubusercontent.com/deutsche-nationalbibliothek/dnblab/main/DNB_SRU_Tutorial.ipynb) †, [calibre-dnb](https://raw.githubusercontent.com/citronalco/calibre-dnb/master/__init__.py), [Zotero K10plus ISBN](https://raw.githubusercontent.com/zotero/translators/master/K10plus%20ISBN.js) †, [lobid API page](https://raw.githubusercontent.com/hbz/lobid-resources/master/web/app/views/api.scala.html) †, [lobid README](https://raw.githubusercontent.com/hbz/lobid-resources/master/README.md) ‡
- [Zotero LoC ISBN](https://raw.githubusercontent.com/zotero/translators/master/Library%20of%20Congress%20ISBN.js) †, [isbnlib-loc](https://pypi.org/pypi/isbnlib-loc/json), [Koha sample servers](https://raw.githubusercontent.com/Koha-Community/Koha/main/installer/data/mysql/en/optional/sample_z3950_servers.yml)
- Libris: [terms](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/index.en.md) †, [User-Agent](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/howto/user_agent.en.md), [find](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/reference/find.en.md), [SRU](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/reference/sru.en.md), [XSearch placeholder](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/howto/xsearch_migration.sv.md)
- [finna-client](https://pypi.org/pypi/finna-client/json); [Inventaire BNE resolver](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/bne/get_bne_entry_from_isbn.ts); [isbnlib-sbn](https://pypi.org/pypi/isbnlib-sbn/json); [isbnlib-porbase](https://pypi.org/pypi/isbnlib-porbase/json); [isbnlib-mcues](https://pypi.org/pypi/isbnlib-mcues/json)
- KB: [README](https://raw.githubusercontent.com/KBNLresearch/KB-python-API/master/README.rst), [sru.py](https://raw.githubusercontent.com/KBNLresearch/KB-python-API/master/kb/nl/api/sru.py), [isbnlib-kb](https://pypi.org/pypi/isbnlib-kb/json), [BELTRANS KB](https://raw.githubusercontent.com/kbrbe/beltrans-data-integration/main/data-sources/kb/README.md), [BELTRANS KBR](https://raw.githubusercontent.com/kbrbe/beltrans-data-integration/main/data-sources/kbr/README.md)
- [almasru](https://pypi.org/pypi/almasru/json); [Catmandu-SRU](https://raw.githubusercontent.com/LibreCat/Catmandu-SRU/master/README.md); [pyeuropeana](https://pypi.org/pypi/pyeuropeana/json); [hathifiles](https://raw.githubusercontent.com/hathitrust/hathifiles/main/README.md); [VuFind preview.js](https://raw.githubusercontent.com/vufind-org/vufind/dev/themes/bootstrap5/js/preview.js); [Zotero NDL translator](https://raw.githubusercontent.com/zotero/translators/master/National%20Diet%20Library%20Catalogue.js)
- OCLC: [Zotero Open WorldCat](https://raw.githubusercontent.com/zotero/translators/master/Open%20WorldCat.js), [isbnlib-oclc](https://pypi.org/pypi/isbnlib-oclc/json)

**Open knowledge bases**

- [Inventaire Wikidata resolver](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/wikidata/get_wikidata_entry_from_isbn.ts), [Inventaire Open Library resolver](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/openlibrary/get_openlibrary_entry_from_isbn.ts), [Inventaire README](https://raw.githubusercontent.com/inventaire/inventaire/main/README.md) †, [wikibase-sdk README](https://raw.githubusercontent.com/maxlath/wikibase-sdk/main/README.md), [BookWyrm initdb](https://raw.githubusercontent.com/bookwyrm-social/bookwyrm/main/bookwyrm/management/commands/initdb.py), [BookWyrm Inventaire connector](https://raw.githubusercontent.com/bookwyrm-social/bookwyrm/main/bookwyrm/connectors/inventaire.py)
- [Crossref REST doc](https://raw.githubusercontent.com/CrossRef/rest-api-doc/master/README.md) †; OpenAlex [README](https://raw.githubusercontent.com/ourresearch/openalex-docs/main/README.md), [rate limits](https://raw.githubusercontent.com/ourresearch/openalex-docs/main/how-to-use-the-api/rate-limits-and-authentication.md) †, [filters](https://raw.githubusercontent.com/ourresearch/openalex-docs/main/api-entities/works/filter-works.md), [pyalex](https://raw.githubusercontent.com/J535D165/pyalex/main/README.md) †
- [BookBrainz licensing page](https://raw.githubusercontent.com/metabrainz/bookbrainz-site/master/src/client/components/pages/licensing.tsx), [Hardcover API guide](https://raw.githubusercontent.com/hardcoverapp/hardcover-docs/main/src/content/docs/api/Getting-Started.mdx) †, [regluit DOAB loader](https://raw.githubusercontent.com/EbookFoundation/regluit/master/core/loaders/doab.py), [Standard Ebooks feeds page](https://raw.githubusercontent.com/standardebooks/web/master/www/feeds/index.php), [public-apis list](https://raw.githubusercontent.com/public-apis/public-apis/master/README.md)

**ISBN ranges**

- [python-stdnum updater](https://raw.githubusercontent.com/arthurdejong/python-stdnum/master/update/isbn.py), [isbn.dat](https://raw.githubusercontent.com/arthurdejong/python-stdnum/master/stdnum/isbn.dat) †, [isbn3](https://raw.githubusercontent.com/inventaire/isbn3/main/README.md), [biblys/isbn](https://raw.githubusercontent.com/biblys/isbn/main/README.md)

**ONIX**

- Immatériel parser: [README](https://raw.githubusercontent.com/immateriel/im_onix/master/README.md), [gemspec](https://raw.githubusercontent.com/immateriel/im_onix/master/im_onix.gemspec), [supporting_resource.rb](https://raw.githubusercontent.com/immateriel/im_onix/master/lib/onix/supporting_resource.rb), [resource_version.rb](https://raw.githubusercontent.com/immateriel/im_onix/master/lib/onix/resource_version.rb), [collateral_detail.rb](https://raw.githubusercontent.com/immateriel/im_onix/master/lib/onix/collateral_detail.rb), [product.rb](https://raw.githubusercontent.com/immateriel/im_onix/master/lib/onix/product.rb), [price.rb](https://raw.githubusercontent.com/immateriel/im_onix/master/lib/onix/price.rb); code lists [27](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-27.yml), [139](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-139.yml), [144](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-144.yml), [145](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-145.yml), [154](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-154.yml), [155](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-155.yml), [158](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-158.yml), [160](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-160.yml), [161](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-161.yml), [162](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-162.yml), [218](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-218.yml)
- Jonix (Apache-2.0, generated from EDItEUR schema 3.1.3 and code lists Issue 75): [README](https://raw.githubusercontent.com/zach-m/jonix/master/README.md) †, [SupportingResource](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-onix3/src/main/java/com/tectonica/jonix/onix3/SupportingResource.java), [ResourceVersion](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-onix3/src/main/java/com/tectonica/jonix/onix3/ResourceVersion.java), [EpubUsageConstraint](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-onix3/src/main/java/com/tectonica/jonix/onix3/EpubUsageConstraint.java); lists [154](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ContentAudiences.java), [158](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ResourceContentTypes.java) †, [160](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ResourceFeatureTypes.java), [161](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ResourceForms.java), [155](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ContentDateRoles.java), [145](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/UsageTypes.java), [146](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/UsageStatuss.java), [27](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/SubjectSchemeIdentifiers.java), [44](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/NameIdentifierTypes.java), [92](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/SupplierIdentifierTypes.java), [139](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/OnixRetailSalesOutletIdss.java); [onixcheck](https://pypi.org/pypi/onixcheck/json)
- Promotional-use statement: [BISG webcast](https://es.slideshare.net/bisg/bisg-webcast-onix-for-books-v30-introduction-1787290) [sib]

**Trade sources**

- Dilicom: [bookshops README](https://raw.githubusercontent.com/vindarel/bookshops/master/README.md), [dilicomScraper.py](https://raw.githubusercontent.com/vindarel/bookshops/master/bookshops/frFR/dilicom/dilicomScraper.py) †, [Abelujo README](https://raw.githubusercontent.com/vindarel/abelujo/master/README.md); [SLF on Distrimage, 2012](https://www.syndicat-librairie.fr/le_service_distrimage_de_dilicom) [sib]; [Dilicom CGV 2021](http://dilicms.centprod.com/file/dilicom/file.do?path=/Conditions+g%C3%A9n%C3%A9rales/CGV+-+Dilicom.pdf) [sib]
- ORB, VLB, LibraryThing, Syndetics, Babelthèque: [VuFind config.ini](https://raw.githubusercontent.com/vufind-org/vufind/dev/config/vufind/config.ini), [Orb.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Orb.php) †, [Buchhandel.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Buchhandel.php), [LibraryThing.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/LibraryThing.php), [Coce README](https://raw.githubusercontent.com/fredericd/coce/master/README.md), [Koha enhanced_content.pref](https://raw.githubusercontent.com/Koha-Community/Koha/main/koha-tmpl/intranet-tmpl/prog/en/modules/admin/preferences/enhanced_content.pref)
- French platform actors: [Readium LCP key-retrieval note](https://raw.githubusercontent.com/readium/lcp-specs/master/notes/lcp-key-retrieval.md)
- Amazon: [python-amazon-paapi changelog](https://raw.githubusercontent.com/sergioteula/python-amazon-paapi/master/CHANGELOG.md), [migration guide](https://raw.githubusercontent.com/sergioteula/python-amazon-paapi/master/docs/pages/migration-guide-6.md), [python-amazon-paapi on PyPI](https://pypi.org/pypi/python-amazon-paapi/json), [amazon-creatorsapi-python-sdk on PyPI](https://pypi.org/pypi/amazon-creatorsapi-python-sdk/json)

---

## Implications for the implementation

Numbers marked "proposal" are design defaults chosen here, to be revised after the measurement in §6.4. Everything else follows from a finding above.

**ISBN handling**

1. Accept ISBN-10, ISBN-13 and EAN-13 input with hyphens or spaces. Validate the check digit. Reject EAN prefixes other than 978 and 979, and reject 979-0 (not an ISBN group).
2. Normalise to ISBN-13 internally. Derive an ISBN-10 only for 978 ISBNs. A 979 ISBN never has one; code that needs an ISBN-10 must handle its absence explicitly.
3. Generate the hyphenation and registration-group table from the Agency's `RangeMessage.xml` with our own script. Store the file's serial number and date beside the table. Refresh it monthly in the dependency-update job. Never copy python-stdnum's `isbn.dat`.
4. If an ISBN falls outside the known ranges, keep going with the unhyphenated forms only and flag `range_unknown`; do not fail the lookup.

**Lookup order and behaviour**

5. Lookup is assistive. The manual form is always available, every field stays editable, and no network failure blocks card creation.
6. Field precedence: typed input > EPUB OPF > uploaded or reseller ONIX > routed national library > Open Library > Google Books > others. A lower source never overwrites a higher one. Each field keeps its source.
7. Standalone mode runs the cascade of §1.2; reseller mode that of §1.3. In reseller mode a catalogue result may fill an empty field but never changes a field the reseller supplied.
8. Route the first network call by registration group using the table in §3.1. Query the routed library and Open Library in parallel; call Google Books only if both leave title or first author empty and a key is configured.
9. BnF SRU: send ISBN-10 and ISBN-13 in one CQL `or` query for 978 ISBNs (fact-check: `bib.fuzzyIsbn` is BnF's documented one-operand alternative, see item 47); request `recordSchema=dublincore` and `maximumRecords=5`. When several records match, prefer the record whose identifier equals the queried ISBN-13, then the most recent date. Clean titles and convert "Surname, Forename (dates)" headings to display order.
10. data.bnf.fr and Wikidata: send the hyphenated ISBN-13, the plain ISBN-13 and, for 978, the ISBN-10 forms, as in §3.3 and §3.9.
11. eBook ISBNs: on a miss, retry with the print ISBN when ONIX `RelatedProduct` gives one, and mark the result `matched_via = print_isbn`. Otherwise offer a title-and-author search and ask the user to confirm. Never substitute silently.

**Network etiquette**

12. Send one fixed User-Agent on every request: `eBookCadeauBot/{version} (+https://{site}/bot; mailto:contact@{site})`. It contains a URL and a contact address, as Open Library, Libris and Crossref ask, and must never contain "CloudFront" (doc 02).
13. Per upstream host, a token bucket of 1 request per second with a burst of 5 (proposal). Open Library's verified ceiling is 180 per minute per IP and its published policy is 1 request per second unidentified, 3 identified [FC]; Hardcover's is 60 per minute; Crossref allows 1 list request per second with no parallel request in the public pool [FC]; never exceed any of them.
14. Timeouts (proposal): 4 s per provider, 8 s for the whole lookup. On 429 or 503, honour `Retry-After`; otherwise back off 2 s, 4 s, 8 s, at most 2 retries. After 5 consecutive failures skip that provider for 10 minutes.
15. Google Books: count calls per day and stop at 900 (proposal, against a default quota of 1,000 confirmed by secondary sources). Never request it without a configured key in production. Fact-check: a key is not enough — see item 48.
16. One user action triggers at most one lookup per provider. No background enumeration, no crawling, no pre-warming of the cache with ISBN lists.

**Transport and architecture**

17. Providers are sans-IO: one function builds the HTTP request from an ISBN, another parses the response bytes into the common record. The same code runs in WebAssembly and on the server.
18. The hosted service sends all catalogue lookups through its own relay endpoint, which returns normalised JSON text. The relay never fetches, stores or forwards an image. Reasons: CORS is unverified everywhere but Open Library; keys must stay server-side; the user's IP address is not sent to third parties.
19. The static, server-less build may call Open Library directly from the browser (CORS verified) and nothing else; it tells the user that the request goes to the Internet Archive.
20. Host the relay in the EU: Google Books list results are believed to depend on the caller's location, and the `country` parameter exists only on `volumes/{volumeId}`.
21. Parse XML (SRU, ONIX, SOAP) with external entities and DTD loading disabled. Cap a lookup response at 2 MB (proposal) and stream ONIX files.
22. API keys and tokens (Google, Hardcover, ORB, Electre, Dilicom, ISBNdb) live in server configuration only and never reach the browser or the logs. A provider without credentials is disabled, not failing.

**Storage, provenance, attribution**

23. Every stored field carries: provider id, licence class (A to D), source record URL or identifier (ARK, PPN, OLID), and retrieval time.
24. Cache rules (TTLs are proposals): class A results 30 days, misses 24 hours. Class B (Google Books): in memory for the session, at most 1 hour, never written to disk or database. Class C: per contract, scoped to the tenant, purged when the contract ends.
25. Show the source next to pre-filled data. For BnF and SUDOC show source and date with a link to the record; list all sources on a "Sources des données" page. Show "Powered by Google" with a link beside Google results.
26. Write provider id and retrieval date into the generation log and the PDF metadata. Print no attribution line for text facts on the card.
27. Store and print facts only: title, subtitle, contributors, publisher, date, language, identifiers, page count, series. Do not store or print summaries, biographies or reviews from catalogues. Descriptive text comes only from reseller ONIX or from the user.
28. A lookup result is text. Cover URLs found on the way are passed to the cover tools as candidates with rights basis `none`, as doc 02 defines; they never enter a PDF produced by the hosted service.

**ONIX and reseller mode**

29. Ship one ONIX reader for 3.0 and 3.1, reference tags and short tags, tolerant of 2.1 for identifiers and titles. Read: identifiers, title elements, contributors, publisher and imprint, GLNs, `RelatedProduct`, subjects (CLIL, Thema), `SupportingResource` with content type, audience, territory, features, form, version features, dates, `EpubUsageConstraint`, `EpubLicense`, sales rights and outlet restrictions.
30. One pure function decides, per asset and order, among `allow`, `deny` and `needs_opt_in`. Deny when: `ContentAudience` is 01 or contains neither 00 nor 03; the order's country is outside `Territory`; today is outside `ContentDate` 14–15; the content type is 45 (holding image). Return `needs_opt_in` when no print permission is present. Default without a permission: typographic card without the cover image. Test this function exhaustively.
31. Recognise a print permission only from a configured source: a per-distributor contract flag, or a named `EpubLicense` whose name or link matches an allow-list. Nothing else in ONIX grants it.
32. Prefer cover assets in this order: List 158 code 29 (full cover); else 01 with 56 (spine) and 02 (back); else 01 alone with a generated spine.
33. When `ResourceFeature` 01 (required credit) is present, print it on the card; store 03 (copyright holder); reuse 07 as alternative text.
34. Check `ResourceVersionFeature` pixel size before downloading a cover, and MD5 or SHA-256 after. Re-fetch when `ContentDate` 17 changes. Fetch `ResourceForm` 02 resources server-side on the reseller's behalf, never from the buyer's browser.
35. When `EpubUsageConstraint` type 11 (text and data mining) has status 03 on a cover, set `tdm_reserved = true` on the asset; the font-identification and spine tools must skip it.
36. Keep reseller data tenant-scoped. Never merge records from two resellers, and never serve one reseller's metadata to another or to standalone users.

**Exclusions and optional providers**

37. Hard-code a deny-list of hosts that no fetcher, relay or provider may contact: Amazon and its image hosts, goodreads.com, librarything.com, babelio.com, fnac.com, kobo.com, decitre.fr, worldcat.org, and the ISBN Agency's and AFNIL's publisher-search pages. A request to them is refused in code and covered by a test.
38. Trade and paid providers (ORB, Electre, Dilicom, VLB, ISBNdb) are optional, server-side, off by default, and enabled only when configuration names a contract reference. Dilicom prices are integers in thousandths of a euro; batch at most 100 EANs per call.
39. Providers marked Hold in §2.1 (SBN, KBR, PORBASE, swisscovery, OBV, British Library, Library of Congress) and KB (NL) stay disabled until a live check confirms endpoint and licence. OpenAlex, Hardcover, HathiTrust, Europeana, Trove and NDL are not implemented.
40. Inventaire, Wikidata and SUDOC providers exist but stay off until the hit-rate test shows that each adds at least 2 points (proposal).

**Tests and fixtures**

41. Every provider has parser tests on stored responses. Real recorded responses may be committed only for class A sources, with source and retrieval date in a sidecar file. For class B and C sources, and for all ONIX, fixtures are synthetic: invented titles, invented GLNs, written by us.
42. No endpoint could be called from the research sandbox. Until a networked session records real class A responses, label fixtures `synthetic, schema unverified`, so that nobody mistakes them for recordings.
43. Use ISBN 9782070360024 as the canonical example in request-building tests: forms `978-2-07-036002-4`, `2070360024`, `2-07-036002-4`, group 978-2. Add a 979-10 ISBN to prove the no-ISBN-10 path, and an invalid check digit.
44. Live tests are opt-in and never run in the default CI job. A scheduled job may send one request per enabled provider per day to detect endpoint drift.
45. Provide the fake-reseller demos with synthetic ONIX: a "fake 7switch" in Immatériel-style ONIX 3.0, and a "fake Fnac" with a Kobo-style partner payload. Include one title with a print permission, one without, one with audience 01, one outside its territory, and one with a text-and-data-mining reservation.
46. Build the measurement harness of §6.4 as a command-line tool that takes a list of ISBNs and writes per-provider hit rate, field completeness and latency to CSV, at 1 request per second.

**Added by the fact-check (2026-11-12)**

47. BnF SRU: support `bib.fuzzyIsbn all "{isbn13}"` as a query form (one operand, matches EAN, ISBN-13 and ISBN-10 per BnF's criteria table). Until a live test, keep the exact two-operand `bib.isbn` query as the default and use `bib.fuzzyIsbn` as the retry on zero records. Request-builder tests cover both.
48. Google Books is gated by two settings, not one: an API key **and** a recorded permission basis (`free_flow` or a reference to Google's written permission). The Books API terms forbid charging users a fee for an application that uses the API without that permission. In any flow where the user pays, the provider is disabled unless the permission reference is set.
49. Expect eBook ISBNs to miss at BnF (2,606 digital books against 77,341 printed books deposited in 2025). The print-ISBN retry of item 11 is the normal path for eBooks, and the measurement of §6.4 must report eBook ISBNs separately.
50. Libris results are class A with a caveat: KB states CC0 for the Swedish national bibliography and authority records. Store the record's own licence or source field when present; do not label every Libris record CC0.
51. Crossref: never send two requests in parallel without `mailto`; a `filter=isbn:` call counts as a list request (1 per second public, 3 per second polite, since 1 December 2025).
52. Library of Congress: the `https://lx2.loc.gov/sru/lcdb` form is reported not to complete TLS in 2026. The provider stays disabled; do not downgrade to plain HTTP.
53. ISBNdb, if ever enabled: plan for daily caps (5,000 / 15,000 / 50,000 searches) as well as the per-second rate.

## Verification log

Fact-check of 2026-11-12. Method: 27 fresh web searches (10 extended), 6 primary files re-read on raw.githubusercontent.com. Vendor pages could not be opened directly; where noted, a quote comes from the search tool's rendering of the page. Full table with every source URL: `factcheck/isbn-databases.md`.

| # | Claim | Verdict | Change made |
|---|---|---|---|
| 1 | BnF SRU endpoint, version 1.2, `bib.isbn`, `dublincore` | CONFIRMED | Marked [FC]; BnF pages cited |
| 2 | Two-operand ISBN query is the way to reach ISBN-10-only records | PARTLY | Added `bib.fuzzyIsbn` and the 10 / 1000 `maximumRecords` values; implication 47 |
| 3 | BnF metadata under Licence Ouverte, commercial reuse allowed | CONFIRMED | Dated 1 January 2014; Dewey exception; one portal sentence flagged for re-reading |
| 4 | BnF has no published rate limit | UNSUPPORTED | Reworded "not found" |
| 5 | BnF eBook coverage partial | CONFIRMED, stronger | Deposit figures added; implication 49 |
| 6 | Open Library policy 1 / 3 requests per second | CONFIRMED | [M] replaced by the official page |
| 7 | Open Library nginx limits (180/min, 1/s, 400/min) | CONFIRMED | Re-read |
| 8 | Open Library CORS `*` | CONFIRMED | Re-read |
| 9 | Google Books 1,000 requests/day | CONFIRMED (secondary) | Marker updated |
| 10 | Google Books usable for display in our product | WRONG by omission | Books API fee clause added in findings, tables, §3.6, §5; implication 48 |
| 11 | `country` only on `volumes/{id}` | CONFIRMED | Re-read |
| 12 | DNB endpoint, no token, CC0 | CONFIRMED | [M] replaced by DNB sources |
| 13 | K10plus endpoint; CC0 | PARTLY | Endpoint confirmed; CC0 stays [M] |
| 14 | lobid CC0; `isbn:` query | CONFIRMED / UNSUPPORTED | Licence marker updated |
| 15 | Libris CC0 | PARTLY | Scope narrowed to national bibliography and authority records; implication 50 |
| 16 | Finna CC0 metadata; CORS | CONFIRMED / UNSUPPORTED | CORS set to unverified |
| 17 | SUDOC `isbn2ppn`, Licence Ouverte | CONFIRMED | Marker updated |
| 18 | Crossref lower limits since December 2025 | CONFIRMED | Figures added; implication 51 |
| 19 | Wikidata limits and graph split | PARTLY | "5 parallel queries" withdrawn; split dated 9 May 2025 |
| 20 | OpenAlex key policy "conflict" | Resolved | Key required since 2026-02-13 |
| 21 | Amazon PA-API replaced in 2026 | CONFIRMED | Shut-off date 15 May 2026 added |
| 22 | ISBNdb about USD 15 / 30 / 75 | OUTDATED | USD 14.99 / 35.99 / 99.99 with daily caps; implication 53 |
| 23 | ONIX Issue 75, List 158 codes 56 and 57; ONIX 3.1.3 | CONFIRMED / UNSUPPORTED | Issues of addition (70, 71) recorded; 3.1.3 marked unconfirmed |
| 24 | ISBN registration groups; 979 without ISBN-10 | CONFIRMED | Re-read |
| — | OCLC Classify 31 January 2024; Goodreads December 2020 | CONFIRMED | Dates sourced |
| — | Library of Congress "2025 platform change" | UNSUPPORTED | Reworded; implication 52 |

Not re-checked in this pass: Dilicom, Electre, Decitre ORB, Tite-Live, VLB and the other trade bases; e-distributor contracts; Hardcover, HathiTrust, Europeana, Trove, NDL; Inventaire and LibraryThing terms; every legal citation in §5; every CORS header other than Open Library's; every live response.
