# ISBN and book-metadata databases we are allowed to access — commercial / book-trade sources

Research notes, angle "COMMERCIAL / BOOK-TRADE", France and EU first. Date of research: 2026-11-12.
Companions: `isbn-databases.open.md` (open / public / library sources), `cover-sources.sources.md`,
`legal-covers-reseller.reseller.md`.

Confidence markers: **[V2]** verified in >= 2 independent sources this session, **[V1]** single source this session,
**[M]** from memory, not verified this session ("[M, low]" = do not act on it without a check), **[sib]** = link and
fact taken from a sibling note in this folder, whose author got it from a search result; not re-read by me.

## 0. Method and limits (read first)

- **No web search could be run.** The session's shared WebSearch budget (200 calls) was already spent; my first three
  queries were refused. WebFetch and vendor sites are blocked by the sandbox proxy.
- Instead I read **primary files on hosts the sandbox allows** (`raw.githubusercontent.com`, `pypi.org`): the ONIX parser
  published by the French e-distributor Immatériel, an Apache-2.0 Java library generated from EDItEUR's schema and code lists
  (Issue 75), a free bookshop tool that calls Dilicom's web service, VuFind / Koha / Coce (library software with trade
  cover providers), Open Library, and Amazon API client packages. Every link below is a file I fetched (HTTP 200) on
  2026-11-12, or is marked [sib]. "Independent sources" for [V2] means independent code bases or documents.
- **Consequence.** ONIX facts are well verified. **Access conditions, prices and contract terms of the trade databases are
  almost all [M]**: no vendor site, price list or contract was read. **No price could be confirmed as of 2026-11.** Treat
  sections 3 to 6 as a briefing for vendor calls, not as settled fact. "Could not verify" at the end is the to-do list.
- GPL/LGPL code (VuFind, Koha, bookshops) was read for facts only; nothing was copied.

## 1. Bottom line

1. **ONIX 3.0 / 3.1 is the canonical trade feed**, and it is what a French reseller already receives from its
   e-distributors. Current release is ONIX 3.1.3 with code lists Issue 75 (October 2026) [V1]. Build an ONIX reader first;
   it serves the reseller mode and any publisher opt-in.
2. **ONIX can now carry a real spine and full jacket**: List 158 has 29 "Full cover" (front, back, spine, flaps) [V2] and,
   new in recent issues, **56 "Spine image"** and 57 "Spine panorama image" [V1]. This answers a question left open in two
   sibling notes. Whether any French feed populates them is unknown.
3. **ONIX carries terms for assets but no print licence.** Audience (List 154), territory, dates, required credit and
   copyright holder (List 160) exist [V2]. ONIX 3.1.1 added `<EpubUsageConstraint>` and `<EpubLicense>` to each
   `<ResourceVersion>` [V1]: a standard slot for a per-image licence. No code means "may be printed on a physical object".
4. **No French trade database is open.** Dilicom (FEL, Hub), Electre, Tite-Live and Decitre ORB are subscriber services
   built for booksellers and libraries [V1 for ORB's subscription and Dilicom's credentials; M for the rest]. A company that is neither needs a
   negotiated contract. None of them grants the right to print a cover [M; sib for Dilicom images].
5. **Dilicom's cheap lookup has no cover and no description**: "FEL à la demande" is a SOAP service, EAN-only, 100 EANs per
   call, returning title, author, publisher, price, availability and dimensions [V1].
6. **Amazon is out**: the Product Advertising API was replaced by the Creators API in 2026 [V2], and the Associates terms
   bar offline and printed use [M]. Goodreads issues no keys [M]. Babelio, Fnac and Kobo have no public catalogue API [M].
7. **ISBNdb is the only self-serve paid API** in the list (credit card, no trade status needed) [M]; 2026 plan prices and
   limits are unverified.
8. **Standalone mode cannot be solved by buying a database.** The rights-clean sources are the EPUB itself, the user, the
   publisher's own ONIX, and the open catalogues. Trade databases are worth a contract only for metadata pre-fill and EAN
   validation, once volume justifies it.

## 2. ONIX for Books — verified facts

### 2.1 Versions and who uses it

- Jonix tracks EDItEUR releases: ONIX 3.1.3 + code lists Issue 75 in its `2026-10` release; it moved to 3.1.0 in 2023-04,
  3.1.1 in 2024-04, 3.1.2 in 2025-01 ([Jonix README](https://raw.githubusercontent.com/zach-m/jonix/master/README.md)) [V1].
  onixcheck validates "ONIX versions 2.1, 3.0, and 3.1 against the official XML Schema" and also offers a `google` schema
  profile, presumably Google's ONIX requirements ([onixcheck on PyPI](https://pypi.org/pypi/onixcheck/json)) [V1]. So
  ONIX 3.1 is in production tooling [V2].
- Immatériel publishes its "ONIX 3.0 & 2.1 parser for Ruby" under MIT; contributors are TEA "The Ebook Alternative" (now
  Vivlio) and Bookmate, i.e. resellers on the receiving end
  ([README](https://raw.githubusercontent.com/immateriel/im_onix/master/README.md),
  [gemspec](https://raw.githubusercontent.com/immateriel/im_onix/master/im_onix.gemspec)) [V1]. French eBook feeds are
  ONIX 3.0, with 2.1 still handled.
- ONIX itself and its code lists are free to implement under EDItEUR's licence [M].

### 2.2 Where a cover sits, and which fields carry its terms

Path: `Product / CollateralDetail / SupportingResource / ResourceVersion / ResourceLink`. The schema describes
`SupportingResource` as "a supporting resource used for marketing and promotional purposes, eg a cover image"
([Jonix SupportingResource](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-onix3/src/main/java/com/tectonica/jonix/onix3/SupportingResource.java)) [V1].

| Element | List | Content (quotes are EDItEUR text relayed by Jonix) | Use for the card | Conf. |
|---|---|---|---|---|
| `ResourceContentType` | 158 | 01 Front cover (2D); 02 Back cover (2D); 03 Cover / pack (may be 3D); 27 thumbnail (transitional only); **29 Full cover** "Includes cover, back cover, spine and - where appropriate - any flaps"; 45 Cover holding image (placeholder, must be replaced); 51 Cover flap image; **56 Spine image** "2D, portrait orientation"; 57 Spine panorama image | Prefer 29, or 01 + 56 + 02; never print 45; fall back to a generated spine | 01–55 [V2] ([im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-158.yml), [Jonix](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ResourceContentTypes.java)); 56–57 [V1] Jonix |
| `ContentAudience` | 154 | 00 Unrestricted; 01 Restricted = "Distribution by agreement between the parties to the ONIX exchange … material which is not for general distribution"; 02 Booktrade; 03 End customers; 07 Press; 09 Search engine index; 10 Social media; 13 LLM training | Use only 00 or 03; treat 01 and trade-only as "not for us" | 00–10 [V2] ([im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-154.yml), [Jonix](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ContentAudiences.java)); 11–13 [V1] |
| `Territory` in `SupportingResource` | — | Added at 3.0.3. If present, "the resource should be used by recipients in the specified territory only" | Filter on the country of the order | [V2] (Jonix; [im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/lib/onix/supporting_resource.rb)) |
| `ResourceFeature` | 160 | 01 Required credit: "Credit that must be displayed when a resource is used"; 02 Caption; 03 Copyright holder "(indicative only, as the resource can be used without consultation)"; 07 alternative text; 08 background colour; 09 image attribute (List 256) | Print 01 on the card; store 03; reuse 07 for accessibility | [V2] ([im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-160.yml), [Jonix](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ResourceFeatureTypes.java)) |
| `ResourceForm` | 161 | 01 Linkable: recipient "may embed the URL … and need not host an independent copy". 02 Downloadable: hosted by the sender "only for the ONIX recipient's direct use"; the recipient "must host an independent copy" | Printing needs the bytes: fetch server-side, in the reseller's name; no hotlinking from the buyer's browser for 02 | codes [V2] ([im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-161.yml)); text [V1] ([Jonix](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ResourceForms.java)) |
| `ResourceVersionFeature` | 162 | 01 file format; 02 / 03 image height / width in pixels; 04 filename; 06 MD5; 07 exact size; 08 SHA-256 | Resolution preflight before download; integrity check | [V2] ([im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-162.yml), Jonix) |
| `ContentDate` in `ResourceVersion` | 155 | 14 From date ("embargoed until this date"); 15 Until date ("may be referenced or used"); 17 Last updated; 27 / 28 Available from / until (download window only) | Refuse outside 14–15; re-fetch when 17 changes | codes [V2] ([im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-155.yml)); text [V1] ([Jonix](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ContentDateRoles.java)) |
| `EpubUsageConstraint` in `ResourceVersion` | 145 / 146 | "Added `<EpubUsageConstraint>` and `<EpubLicense>` at revision 3.1.1": "a usage constraint specific to a particular version of the resource offered". Status 01 permitted, 02 permitted with limit, 03 prohibited. Type 11 Text and data mining explicitly covers "the product metadata or supporting resources"; type 02 Print means "Make physical copy of extract" (of the book, not of the cover) | Read and honour it; today it mainly expresses the TDM / AI opt-out, which matters for our font and spine models | [V1] ([ResourceVersion](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-onix3/src/main/java/com/tectonica/jonix/onix3/ResourceVersion.java), [EpubUsageConstraint](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-onix3/src/main/java/com/tectonica/jonix/onix3/EpubUsageConstraint.java), [List 145](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/UsageTypes.java)) |
| `EpubLicense` in `ResourceVersion` | 218 | Licence name plus a link to an expression: 01 human-readable, 02 professional-readable, 10 ONIX-PL, 20 ODRL | The standard slot for a publisher's "print on gift card allowed" licence | [V1] Jonix; List 218 codes [V1] ([im_onix](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-218.yml)) |

What ONIX does **not** say: there is no audience, usage type or feature meaning "may be reproduced on a physical object".
The default remains "promotional use by supply-chain partners" ([BISG](https://es.slideshare.net/bisg/bisg-webcast-onix-for-books-v30-introduction-1787290)) [sib],
bounded by the trading contract. List 158 code 99 "License" concerns the licence of the *product content* and is deprecated
in favour of `<EpubLicense>` [V1]. A print permission therefore needs either a named `EpubLicense` on the cover's
`ResourceVersion` (3.1.1+) or a proprietary flag agreed with the distributor.

**Caution on the AI angle**: List 145 code 11 lets a publisher prohibit text and data mining of "supporting resources",
which includes covers [V1]. Font identification on a cover, or training a spine generator on covers, is TDM; under the EU
DSM Directive a machine-readable reservation removes the commercial TDM exception [M]. Honour status 03.

### 2.3 Subjects, identifiers, outlets

- Subject schemes (List 27): **29 CLIL** "France. A four-digit number, see https://clil.centprod.com/listeActive.html" (a
  Dilicom-hosted domain); **28 Thèmes Electre**; **93–99 Thema** (subject plus place, language, time, education, age,
  style qualifiers); A5 Rameau; 26 Warengruppen-Systematik (Germany, documented on vlb.de); 32 NUR (Netherlands [M]);
  53–58 CCE (Italian-language scheme); 51 Spanish ISBN UDC subset ([im_onix List 27](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-27.yml),
  [Jonix](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/SubjectSchemeIdentifiers.java)) [V2].
  Map CLIL and Thema to our own genre list; both are free to use [M].
- Electre is wired into the standard: List 44 codes 28 "Identifiant Editeur Electre" and 30 "Identifiant Marque Electre"
  ([Jonix List 44](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/NameIdentifierTypes.java)) [V1].
  Parties are otherwise identified by GLN (List 44 / 92 code 06) [V2]; im_onix exposes `publisher_gln` and
  `distributor_gln` ([product.rb](https://raw.githubusercontent.com/immateriel/im_onix/master/lib/onix/product.rb)) [V1].
- French actors have ONIX retail outlet codes (List 139): DIL Dilicom, ELE Electre, EPA E-Pagine, FNC Fnac, IMM
  Immatériel.fr, IZN Izneo, NUM Numilog, TEA Vivlio ("Formerly The Ebook Alternative"), YOU Youboox, YSC Youscribe, KBO Kobo,
  DEM De Marque ([im_onix List 139](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-139.yml),
  [Jonix](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/OnixRetailSalesOutletIdss.java)) [V2].
  Publishers use them in `SalesRestriction / SalesOutlet` to include or exclude named retailers; im_onix reads them
  (`onix_outlets_values`) [V1]. We have no code: a publisher would name us as a proprietary outlet.

## 3. France — trade databases

Vendor-side facts are [M] unless a link is given. Prices: not found.

| Source | What it is | Who can get access | Interface | Licence constraints | Covers |
|---|---|---|---|---|---|
| **Dilicom — FEL** (Fichier Exhaustif du Livre) | Interprofessional EDI hub of the French book trade; FEL is the reference file of titles commercially available, fed by distributors: EAN, title, publisher, distributor, fixed retail price, availability code, dimensions, CLIL [M] | Dilicom members: booksellers, distributors, publishers; software vendors and other third parties by specific contract [M]. The member portal URL in the README example carries a 13-digit, GLN-like `emet` number [V1] | Bulk file with updates [M] | Members' own trade use; no redistribution [M]. General terms: [CGV 2021](http://dilicms.centprod.com/file/dilicom/file.do?path=/Conditions+g%C3%A9n%C3%A9rales/CGV+-+Dilicom.pdf) [sib] | Separate service, see Distrimage |
| **Dilicom — FEL à la demande** | Per-EAN lookup of the FEL | Same; needs user + password | **SOAP**, `POST http://websfel.centprod.com/v2/DemandeFicheProduit`, namespace `fel.ws.accelya.com`; **EAN only, no free-text search; up to 100 EANs per request**; returns title, author, publisher, price, publication date, availability code, thickness / height / width / weight ([bookshops scraper](https://raw.githubusercontent.com/vindarel/bookshops/master/bookshops/frFR/dilicom/dilicomScraper.py), [README](https://raw.githubusercontent.com/vindarel/bookshops/master/README.md)) [V1] | As above | **None**: "Not available in FEL à la demande" (image and description) [V1] |
| **Dilicom — Distrimage** | Front-cover images for FEL titles | Members | File / URL service [M] | Free inside bookshop management software; "pour tout autre usage (internet, extranet), il convient de prendre contact avec le service clients" ([SLF, 2012](https://www.syndicat-librairie.fr/le_service_distrimage_de_dilicom)) [sib]. Licensed per use; print is not a default use | Yes, by contract |
| **Dilicom — Hub numérique** | Single connection between e-distributors' warehouses and eBook resellers: catalogue notices (ONIX), availability, order, download link; also carries library lending (PNB) [M] | Resellers with a GLN, a Dilicom contract and a commercial agreement with each distributor [M] | Web-service API; notices delivered as ONIX 3.0 [M] | Data serves the sale of the titles by that reseller [M] | Cover URLs inside the ONIX notices [M] |
| **Electre** (Electre NG, Electre NG API) | Commercial bibliographic database of the Cercle de la Librairie group: in-print, forthcoming and out-of-print French titles, summaries, tables of contents, author notes, covers, Rameau / Dewey indexing, own themes and identifiers (in ONIX lists [V1]) | Any organisation by annual subscription; customers are libraries, booksellers, publishers [M] | Web application plus REST API for integration into library and bookshop systems [M] | Subscriber's internal use and display in its own catalogue, by option; no redistribution, no bulk reuse [M] | Yes, as a licensed option for catalogue display [M] |
| **Tite-Live** (Médiabase, ePagine, Place des Libraires) | Bookshop software vendor; Médiabase is the bibliographic base shipped to its bookshop customers; ePagine is its eBook platform for independent bookshops (ONIX outlet code EPA [V2]; its staff co-author Readium LCP notes with De Marque and Vivlio, [note](https://raw.githubusercontent.com/readium/lcp-specs/master/notes/lcp-key-retrieval.md) [V1]) | Tite-Live customers only [M] | Bundled in the software; web services for customers' sites [M] | Use within the customer bookshop's tools and site [M] | Yes, for customers [M] |
| **Decitre ORB** (Outil de Recherche Bibliographique) | "a french database of bibliographic information available by subscription" ([VuFind config](https://raw.githubusercontent.com/vufind-org/vufind/dev/config/vufind/config.ini)) [V1]; operated by the Decitre bookseller group [M] | Subscribers: libraries, booksellers, institutions [M] | **REST**, `https://{user}:{key}@api.base-orb.fr/v1/products?eans=…`, JSON ([VuFind Orb.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Orb.php), [Coce README](https://raw.githubusercontent.com/fredericd/coce/master/README.md)) [V2] | Subscription terms not read | `images.front.{thumbnail,original}.src` [V1]; used by libraries for catalogue display |

Reading: the free software for independent bookshops, Abelujo, uses Dilicom when the shop has an account and otherwise
**scrapes a bookshop website** ([Abelujo README](https://raw.githubusercontent.com/vindarel/abelujo/master/README.md)) [V1].
That is the practical state of "open" French trade data: there is none.

## 4. France — eBook distributors and reseller platforms

All [M] unless marked. None publishes reseller terms that I could read.

| Actor | Role | Feed / API | Notes |
|---|---|---|---|
| **Immatériel.fr** | Distributor for independent publishers; operates the 7switch store | ONIX 3.0 (and 2.1) feed plus a reseller web service; also reachable through the Dilicom Hub | Its open parser shows what it exchanges: front cover (largest version chosen), EPUB sample, description, CLIL and BISAC, GLNs, print-product link, sales rights, outlet restrictions, prices, protection type [V1] |
| **Eden Livres** | Distribution platform of Madrigall (Gallimard, Flammarion), Média-Participations (La Martinière, Seuil) and Actes Sud | Reseller API and ONIX; Dilicom Hub | Technology by De Marque (Cantook) [M, medium] |
| **e-Plateforme** | Editis / Interforum digital distribution | Reseller API and ONIX; Dilicom Hub | Also De Marque technology [M, medium] |
| **Hachette Livre** | Own digital distribution since 2012 | ONIX; Dilicom Hub | |
| **Numilog** | Aggregator-distributor, store, library offer | ONIX; Dilicom Hub; outlet code NUM [V2] | |
| **Izneo** | Digital comics platform (publishers plus Fnac Darty) | Own platform; feeds to partner stores | Outlet code IZN [V2] |
| **Vivlio (ex-TEA)** | Reseller platform behind Decitre, Furet du Nord, Cultura, E.Leclerc | Consumes distributor ONIX | Contributor to im_onix [V1]; outlet code TEA [V2] |
| **Kobo (for Fnac)** | Fnac's eBook catalogue is operated by Rakuten Kobo | Publishers and distributors feed Kobo ONIX; Fnac gets catalogue and sales through Kobo's private partner integration | Print rights would have to pass through Kobo as well |

## 5. Other EU countries, UK, US

All [M] unless marked; none checked against 2026 terms. Free national alternatives are in `isbn-databases.open.md`.

| Country | Source (operator) | Access | Interface | Covers |
|---|---|---|---|---|
| DE | **VLB** (MVB, Börsenverein group) | Paid licences for booksellers, libraries, vendors; publishers pay to list | REST API and ONIX exports; cover API `https://api.vlb.de/api/v1/cover/{isbn}/{size}` with a token ([VuFind Buchhandel.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Buchhandel.php)) [V1] | Token-gated; VuFind's [config](https://raw.githubusercontent.com/vufind-org/vufind/dev/config/vufind/config.ini) warns that templates may have to be changed "in order to comply with terms of service" [V1]; use limited to promoting the title [M] |
| DE | **Libri**, **Zeitfracht** (ex-KNV) | Wholesalers; catalogue data for their bookseller customers | Proprietary feeds | By contract |
| NL / BE | **CB** (Centraal Boekhuis); **Boekenbank** for Flanders (ONIX List 92 "Flemish supplier code" [V1]) | Trade customers | ONIX feeds, portal | By contract |
| ES | **DILVE** (FGEE) | Registered professional users; publishers load ONIX | Web service returning ONIX and resources | Publisher-supplied; use tied to promoting and selling the book. Free fallback: Ministry of Culture ISBN database ([isbnlib-mcues](https://pypi.org/pypi/isbnlib-mcues/json)) [V1] |
| IT | **Informazioni Editoriali** (Alice catalogue, Arianna+) | Trade subscribers | Data licences, portal | By contract |
| UK | **Nielsen BookData** (NielsenIQ); **BDS** | Subscription by quote | Feeds and web services | Licensed option |
| US | **Bowker** Books In Print; Syndetics | Subscription; Syndetics is sold to libraries ([Koha prefs](https://raw.githubusercontent.com/Koha-Community/Koha/main/koha-tmpl/intranet-tmpl/prog/en/modules/admin/preferences/enhanced_content.pref)) [V1] | Feeds and web services | Licensed for catalogue display |

## 6. Global commercial APIs, affiliate and partner programmes

| Source | Status as of 2026-11 | Constraints | Verdict |
|---|---|---|---|
| **ISBNdb** | Paid REST API, self-serve. Plans as I remember them (2025): Basic about USD 15, Premium about USD 30, Pro about USD 75 per month, with 1 / 3 / 5 requests per second, plus bulk data licences by quote [M, low]. isbnlib's documentation shows an API key being configured for an `isbndb` provider ([isbnlib](https://pypi.org/pypi/isbnlib/json)) [V1]; Open Library has imported ISBNdb bulk dumps ([importer](https://raw.githubusercontent.com/internetarchive/openlibrary/master/scripts/providers/isbndb.py)) [V1] | Use inside your own application; no resale as a competing dataset; cover images of unknown provenance, no rights warranty [M] | Possible for metadata pre-fill; not for covers. Test the hit rate on French eBook ISBNs first |
| **Amazon** | **PA-API 5.0 replaced by the Creators API**: OAuth2 credentials from the Associates portal; python-amazon-paapi deprecated PA-API in 6.0.0 (2026-01-29) and removed it in 7.0.0 (2026-09-04) ([changelog](https://raw.githubusercontent.com/sergioteula/python-amazon-paapi/master/CHANGELOG.md), [migration guide](https://raw.githubusercontent.com/sergioteula/python-amazon-paapi/master/docs/pages/migration-guide-6.md)); Open Library added an `AmazonCreatorsAPI` class ([vendors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/core/vendors.py)) [V2] | Associates account with qualifying sales; content only to send traffic to Amazon, with links; short cache window, images not stored; **no offline or printed use** [M] | **Excluded** |
| **Goodreads** | No new API keys since December 2020 [M] | — | Excluded |
| **LibraryThing** | Cover service by developer key (`covers.librarything.com/devkey/{key}/…`, [VuFind](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/LibraryThing.php)) [V1]; thingISBN | Non-commercial, about 1,000 requests a day; commercial offer is "LibraryThing for Libraries" [M, low] | Excluded for a commercial product |
| **Babelio** | No public API. B2B product "Babelthèque" injects reviews and citations into library catalogues ([Koha prefs](https://raw.githubusercontent.com/Koha-Community/Koha/main/koha-tmpl/intranet-tmpl/prog/en/modules/admin/preferences/enhanced_content.pref)) [V1] | Site terms forbid extraction [M] | Excluded |
| **Kobo / Rakuten** | Affiliate programme with links and product feed; no public catalogue API [M] | Feed only to promote Kobo links [M] | Not a data source; relevant only as Fnac's back end |
| **Fnac** | Affiliate programme (network not verified) and a marketplace seller API [M, low] | Affiliate content only to drive traffic to fnac.com [M] | Not a data source |
| **Bookshop.org** | Affiliate programme; not active in France; no public catalogue API [M]. Has an ONIX outlet code (BSO) [V1] | — | Irrelevant for France |
| **Google Play Books Partner Program** | Publisher-side channel to sell on Google Play; ingests ONIX [V1 for the ONIX profile] | Not a data service | Not a source. The Google Books API is covered in the open note |

## 7. Answer (a) — standalone mode: what a small French company may use, in order

1. **The EPUB and the user.** OPF metadata and the embedded cover, or typed fields and an uploaded image. No database
   licence is involved; the cover question becomes the user's declaration (see the legal notes).
2. **Publisher or author opt-in through ONIX.** Accept an ONIX 3.0 / 3.1 file or feed from publishers, self-publishing
   platforms and e-distributors. It is the only channel that can carry a print permission (section 2.2), and Immatériel and
   De Marque can address many publishers at once.
3. **Open catalogues** by ISBN prefix (BnF, DNB, Open Library and others): facts under Licence Ouverte / CC0, storable.
   See `isbn-databases.open.md`.
4. **Google Books API**: display-only pre-fill, no permanent copy [M].
5. **ISBNdb** (paid, self-serve): only if a measured hit rate on French eBook ISBNs justifies the fee; metadata only.
6. **One contracted French database, later, for pre-fill and validation only.** Decitre ORB has a documented REST API and
   a subscription model [V2]; Electre NG is the richest and probably the most expensive [M]; Dilicom FEL à la demande
   answers "is this EAN a real, available book, and at what fixed price" but has neither cover nor description [V1] and
   presupposes Dilicom membership [M]. Ask each vendor whether a non-bookseller service provider can subscribe, and get
   the "internal use" clause in writing before storing anything.
7. **Abroad**: open national libraries first; VLB (Germany), CB, DILVE, Informazioni Editoriali by contract when a
   country justifies it.

Never: Amazon, Goodreads, LibraryThing (commercial use), or scraping Babelio, Fnac, Decitre or Kobo. These are EU database
makers with ToS and sui generis rights (`cover-sources.scraping-law.md`).

**No commercial source in this note grants print rights on covers to a third party.** Buying access solves pre-fill, not
the cover.

## 8. Answer (b) — reseller mode: what the reseller holds and what it may do

What a French eBook reseller already holds, per title, through its distributor contracts and the Dilicom Hub
(most fields verified in Immatériel's parser [V1]; contributors, extent and audience are standard ONIX [M]; that every
distributor fills them is [M]):

- identifiers (EAN / ISBN-13, proprietary ids), publisher, imprint and distributor with GLNs;
- title, subtitle, collection, contributors with roles and biographies, language, extent;
- CLIL, Thema or BISAC subjects, audience;
- description texts; **front cover** as `SupportingResource` 01 with URL, pixel size, format, MD5 and last-updated date;
  an EPUB extract (code 15); rarely back cover, never (to my knowledge) full cover or spine [M, low];
- link to the print edition (`RelatedProduct`), publication and embargo dates;
- sales rights by country, outlet restrictions, fixed prices incl. tax per territory, protection type (List 144: none,
  watermark, Adobe DRM, Readium LCP) and usage constraints.

What those contracts typically allow [M; no reseller-facing contract could be read by me or by the sibling legal note]:

- display of metadata, cover and extract **to present, promote and sell that eBook** on the reseller's storefronts, apps,
  newsletters and advertising, in the licensed territory, for as long as the title is on sale;
- with duties: faithful reproduction (no cropping or alteration), prompt updates and takedowns, respect of embargo,
  price and territory, no sub-licensing, no transfer of the feed to third parties other than subcontractors acting for the
  reseller.

What they do not obviously allow: reproduction on a **physical printed object**, alteration (adding a generated spine and
a back panel is an adaptation), and any use by a third party in its own name. French law reads grants narrowly
(CPI L.131-3) [M]; see `legal-covers-reseller.reseller.md`.

Consequences:

1. Operate as the reseller's **subcontractor**: the reseller passes ONIX or extracted fields plus the cover bytes or URL;
   we keep no cross-reseller catalogue.
2. Evaluate, per title: `ContentAudience` (reject 01), `Territory`, `ContentDate` 14 / 15, `EpubUsageConstraint`,
   outlet restrictions, and a **print permission flag**.
3. Obtain that flag at distributor level (Immatériel, Eden Livres, e-Plateforme, Hachette, and Kobo for Fnac) as a contract
   rider or as a named `EpubLicense` on the cover resource, rather than publisher by publisher. Default without the flag:
   typographic card without the cover image.
4. Always print the `ResourceFeature` 01 credit when present.

## 9. Engineering consequences for `ebc-catalog`

- One sans-IO ONIX 3.0 / 3.1 reader (reference and short tags) feeding the same model as the open providers.
- Per asset, store provenance and terms: source, audience, territory, dates, credit, copyright holder, constraints,
  licence name and URL, pixel size, hash. One pure function decides allow / deny / needs opt-in; test it exhaustively.
- Test fixtures must be **synthetic ONIX written by us**, with invented titles and GLNs: a "fake 7switch" in
  Immatériel-style ONIX 3.0, a "fake Fnac" with a Kobo-style partner feed. Do not commit real publisher records.
- Trade providers (ORB, Electre, Dilicom, VLB, ISBNdb) are optional, server-side only (credentials), off by default.

## Sources

All fetched 2026-11-12 (HTTP 200) unless marked [sib].

- ONIX, Immatériel parser: [README](https://raw.githubusercontent.com/immateriel/im_onix/master/README.md),
  [gemspec](https://raw.githubusercontent.com/immateriel/im_onix/master/im_onix.gemspec),
  [supporting_resource.rb](https://raw.githubusercontent.com/immateriel/im_onix/master/lib/onix/supporting_resource.rb),
  [resource_version.rb](https://raw.githubusercontent.com/immateriel/im_onix/master/lib/onix/resource_version.rb),
  [collateral_detail.rb](https://raw.githubusercontent.com/immateriel/im_onix/master/lib/onix/collateral_detail.rb),
  [product.rb](https://raw.githubusercontent.com/immateriel/im_onix/master/lib/onix/product.rb),
  [price.rb](https://raw.githubusercontent.com/immateriel/im_onix/master/lib/onix/price.rb); code lists
  [27](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-27.yml),
  [139](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-139.yml),
  [144](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-144.yml),
  [145](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-145.yml),
  [154](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-154.yml),
  [155](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-155.yml),
  [158](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-158.yml),
  [160](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-160.yml),
  [161](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-161.yml),
  [162](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-162.yml),
  [218](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-218.yml)
- ONIX, Jonix (Apache-2.0, generated from EDItEUR schema 3.1.3 and code lists Issue 75):
  [README](https://raw.githubusercontent.com/zach-m/jonix/master/README.md),
  [SupportingResource](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-onix3/src/main/java/com/tectonica/jonix/onix3/SupportingResource.java),
  [ResourceVersion](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-onix3/src/main/java/com/tectonica/jonix/onix3/ResourceVersion.java),
  [EpubUsageConstraint](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-onix3/src/main/java/com/tectonica/jonix/onix3/EpubUsageConstraint.java),
  lists [154](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ContentAudiences.java),
  [158](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ResourceContentTypes.java),
  [160](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ResourceFeatureTypes.java),
  [161](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ResourceForms.java),
  [155](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ContentDateRoles.java),
  [145](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/UsageTypes.java),
  [146](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/UsageStatuss.java),
  [27](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/SubjectSchemeIdentifiers.java),
  [44](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/NameIdentifierTypes.java),
  [92](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/SupplierIdentifierTypes.java),
  [139](https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/OnixRetailSalesOutletIdss.java);
  [onixcheck](https://pypi.org/pypi/onixcheck/json)
- Dilicom: [bookshops README](https://raw.githubusercontent.com/vindarel/bookshops/master/README.md),
  [dilicomScraper.py](https://raw.githubusercontent.com/vindarel/bookshops/master/bookshops/frFR/dilicom/dilicomScraper.py),
  [Abelujo README](https://raw.githubusercontent.com/vindarel/abelujo/master/README.md);
  [SLF on Distrimage, 2012](https://www.syndicat-librairie.fr/le_service_distrimage_de_dilicom) [sib];
  [Dilicom CGV 2021](http://dilicms.centprod.com/file/dilicom/file.do?path=/Conditions+g%C3%A9n%C3%A9rales/CGV+-+Dilicom.pdf) [sib]
- ORB, VLB, LibraryThing, Syndetics, Babelthèque:
  [VuFind config.ini](https://raw.githubusercontent.com/vufind-org/vufind/dev/config/vufind/config.ini),
  [Orb.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Orb.php),
  [Buchhandel.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Buchhandel.php),
  [LibraryThing.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/LibraryThing.php),
  [Coce README](https://raw.githubusercontent.com/fredericd/coce/master/README.md),
  [Koha enhanced_content.pref](https://raw.githubusercontent.com/Koha-Community/Koha/main/koha-tmpl/intranet-tmpl/prog/en/modules/admin/preferences/enhanced_content.pref)
- French platform actors: [Readium LCP key-retrieval note](https://raw.githubusercontent.com/readium/lcp-specs/master/notes/lcp-key-retrieval.md)
- Amazon: [python-amazon-paapi changelog](https://raw.githubusercontent.com/sergioteula/python-amazon-paapi/master/CHANGELOG.md),
  [migration guide](https://raw.githubusercontent.com/sergioteula/python-amazon-paapi/master/docs/pages/migration-guide-6.md),
  [python-amazon-paapi on PyPI](https://pypi.org/pypi/python-amazon-paapi/json),
  [amazon-creatorsapi-python-sdk on PyPI](https://pypi.org/pypi/amazon-creatorsapi-python-sdk/json),
  [Open Library vendors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/core/vendors.py)
- ISBNdb, Spain: [Open Library ISBNdb importer](https://raw.githubusercontent.com/internetarchive/openlibrary/master/scripts/providers/isbndb.py),
  [isbnlib](https://pypi.org/pypi/isbnlib/json), [isbnlib-mcues](https://pypi.org/pypi/isbnlib-mcues/json)
- ONIX promotional-use statement: [BISG webcast](https://es.slideshare.net/bisg/bisg-webcast-onix-for-books-v30-introduction-1787290) [sib]

## Could not verify

Nothing below was checked against a vendor site, price list or contract. No search ran in this session.

- **Every price.** Dilicom membership, FEL, FEL à la demande and Hub fees; Electre NG and its API; Decitre ORB; Tite-Live;
  VLB licences and cover token; CB; DILVE; Informazioni Editoriali; Nielsen BookData; BDS; Bowker; ISBNdb 2026 plans, daily
  caps and bulk licences.
- **Who may subscribe**: whether Dilicom, Electre, ORB and VLB accept a service provider that is neither bookseller nor
  library, and under which contract type. Whether Dilicom logins are GLNs (inferred from an example in a README).
- **Licence wording** on redistribution, caching, display and cover reuse for every source in sections 3 to 6. In particular
  current Dilicom terms for cover images (only a 2012 notice [sib]), Electre's image option, VLB's cover terms, ISBNdb's
  terms and the origin of its cover images.
- **Dilicom Hub**: current API form, whether notices are ONIX 3.0 for all distributors, which distributors and resellers
  are connected in 2026.
- **French e-distributors**: reseller-facing terms of Immatériel, Eden Livres, e-Plateforme, Hachette, Numilog, Izneo;
  whether Eden Livres and e-Plateforme still run on De Marque technology; Immatériel's current ownership and API; ePagine's
  2026 status; how Kobo passes catalogue data to Fnac.
- **ONIX in practice**: whether any French feed carries List 158 codes 02, 29, 51, 56 or 57, `ContentAudience` other than
  00, `ResourceFeature` 01, or the 3.1.1 `EpubUsageConstraint` / `EpubLicense` on resources; the issue in which codes 56
  and 57 were added; the content of List 256; EDItEUR's sunset dates for ONIX 2.1 and its licence text; CLIL and Thema
  licence texts.
- **Amazon**: the PA-API shutdown date, Creators API eligibility rules and licence text on caching, images and offline use.
- **Goodreads, LibraryThing, Babelio, Kobo / Rakuten, Fnac, Bookshop.org, Google Play Books Partner Program**: all terms and
  programme details; the Fnac affiliate network.
- **Company facts from memory**: ownership and structure of Electre, Tite-Live, Decitre / ORB, MVB, Libri, Zeitfracht, CB,
  DILVE / FGEE, Informazioni Editoriali, NielsenIQ BookData, BDS, Bowker.
- **Hit rates**: no source was queried. Measure on about 200 ISBNs from a French reseller catalogue before paying anyone.
