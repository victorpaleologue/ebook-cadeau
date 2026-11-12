# ISBN and book-metadata databases we are allowed to access — open / public / library sources

Research notes, angle "OPEN / PUBLIC / LIBRARY". Date of research: 2026-11-12. Market: France first, then EU.
Commercial B2B feeds (Electre, Dilicom FEL, ORB, VLB, Nielsen, ISBNdb, ONIX from distributors) are out of scope here.

Confidence markers: **[V2]** verified in >= 2 independent sources this session, **[V1]** single source this session,
**[M]** from memory, not verified this session ("[M, low]" = do not code against it without a live check).

## 0. Method and limits (read first)

- **No web search could be run.** The session's shared WebSearch budget (200 calls) was already spent by sibling agents; all
  five first queries were refused. WebFetch and every live API host (`openlibrary.org`, `catalogue.bnf.fr`, ...) are blocked by
  the sandbox proxy. Nothing below comes from a search result, and **no endpoint was called live**.
- To avoid a from-memory note, I read **primary files on the three hosts the sandbox allows**: `raw.githubusercontent.com`
  (operators' own repositories: Open Library, DNB, KB Sweden, KBR, BnF hackathon wiki, Crossref, OpenAlex, Hardcover,
  HathiTrust, hbz, SLSP, NatLibFi; plus client code that hard-codes endpoints: Zotero translators, calibre, Inventaire,
  BookWyrm, Koha, VuFind), `pypi.org` and `registry.npmjs.org` (package sources). Every link in this note is a file I fetched
  (HTTP 200) and read today. "Independent sources" for [V2] means independent code bases or documents, not web pages.
- GPL/AGPL/LGPL projects (Zotero translators, calibre, Inventaire, BookWyrm, Koha, VuFind, python-stdnum) were read **for facts
  only** (URL patterns). No code or data file from them may be copied into this BSD-3 repository.
- Consequence: **endpoint patterns are mostly [V1]/[V2]; terms of use, quotas and licences are mostly [M]** unless the operator
  states them in a repository. Section 9 lists what a human or a later session with network access must re-check.

## 1. Bottom line

1. **For a French eBook the catalogue is not the best source.** The EPUB's own OPF metadata (standalone mode) and the reseller's
   ONIX feed (reseller mode) are authoritative and rights-clean. Open catalogues are the fallback, mainly to pre-fill a form
   from a typed or scanned ISBN.
2. **Fully open, no key, commercial reuse allowed** (attribution at most): BnF catalogue général and data.bnf.fr (Licence
   Ouverte) [V1 for the licence], SUDOC web services (Licence Ouverte) [M], DNB (CC0) [M], K10plus and lobid (CC0) [M],
   Libris (CC0) [V1], Finna (CC0) [M], BNE (CC0) [M], Wikidata (CC0) [M], Open Library (CC0 contributions) [V1], Crossref
   [V1], DOAB/OAPEN (CC0) [M], BookBrainz core data (CC0) [V1], Inventaire (CC0) [M].
3. **Usable with strings attached**: Google Books (API key, about 1,000 requests/day by default, no permanent copy, branding)
   [M]; OpenAlex (free API key mandatory since 2026-02-13) [V1]; Hardcover (token, 5,000 requests/day, token must stay
   server-side) [V1]; Europeana and Trove (API key) [V1 / M]; NDL Japan (attribution; commercial conditions) [M].
4. **Not open**: OCLC WorldCat (Classify is gone; the Search API needs a subscription; the website encrypts its internal
   API) [V1/M]; the International ISBN Agency's publisher register (search only) [M]; Amazon, Goodreads [M].
5. **Route by ISBN registration group.** 978-2 = French language, 979-10 = France, 979-12 = Italy, 979-8 = United States
   ([range data dated 2026-01-04](https://raw.githubusercontent.com/arthurdejong/python-stdnum/master/stdnum/isbn.dat)) [V1].
   Query BnF first for 978-2 / 979-10, DNB for 978-3, and so on (section 7). **979-prefixed ISBNs have no ISBN-10**
   ([isbn3 README](https://raw.githubusercontent.com/inventaire/isbn3/main/README.md),
   [Open Library vendors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/core/vendors.py))
   [V2] — any provider keyed on ISBN-10 (Amazon ASIN, old Open Library records) silently fails for recent French titles.
6. **Browser-direct (CORS) is verified in source only for Open Library** (`.json` and `/api/` paths) and stated by the operator
   for Hardcover. For SRU endpoints (BnF, DNB, K10plus, LoC, KB, SLSP) it is unverified: plan a thin server-side relay with a
   cache, and keep providers sans-IO so the same code runs in WASM or on the server.

## 2. Catalogue — access

`{isbn}` = 13 digits without hyphens unless noted. Patterns are code, not citations.

| Source (operator) | ISBN lookup | Format | Key | Evidence |
|---|---|---|---|---|
| **Open Library** (Internet Archive, US) | `https://openlibrary.org/isbn/{isbn}.json` (302 to `/books/OL…M.json`); `https://openlibrary.org/api/books?bibkeys=ISBN:{isbn}&format=json&jscmd=data`; `https://openlibrary.org/search.json?q={isbn}` | JSON | none | [V2] [OL code.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/plugins/openlibrary/code.py), [Inventaire](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/openlibrary/get_openlibrary_entry_from_isbn.ts), [BookWyrm](https://raw.githubusercontent.com/bookwyrm-social/bookwyrm/main/bookwyrm/management/commands/initdb.py) |
| Open Library dumps | `ol_dump_YYYY-MM-DD` items in the `ol_exports` collection on archive.org (editions, works, authors, covers metadata, wikidata) | TSV + JSON, gzip | none | [V1] [oldump.sh](https://raw.githubusercontent.com/internetarchive/openlibrary/master/scripts/oldump.sh): 192 M records, 29 GB (Nov 2021) |
| **Internet Archive** | `https://archive.org/advancedsearch.php?q=isbn:{isbn}&fl[]=identifier&output=json`, then `https://archive.org/metadata/{identifier}` | JSON | none | [M] |
| **Google Books** (Google) | `https://www.googleapis.com/books/v1/volumes?q=isbn:{isbn}&key={KEY}` (also `books.googleapis.com`); `country=FR` is documented on `volumes/{id}` only, its use on the list call is [M] | JSON | API key (optional for light use [M]) | [V2] [discovery doc rev. 20240214](https://raw.githubusercontent.com/googleapis/google-api-go-client/main/books/v1/books-api.json), [calibre-web](https://raw.githubusercontent.com/janeczku/calibre-web/master/cps/metadata_provider/google.py), [isbnlib](https://pypi.org/pypi/isbnlib/json) |
| **BnF catalogue général** (FR) | `https://catalogue.bnf.fr/api/SRU?version=1.2&operation=searchRetrieve&query=bib.isbn%20all%20%22{isbn}%22&recordSchema=dublincore&maximumRecords=1` (`unimarcXchange`, `intermarcXchange` also exist [M]) | SRU 1.2 XML | none | [V2] [Zotero BnF ISBN](https://raw.githubusercontent.com/zotero/translators/master/BnF%20ISBN.js), [isbnlib-bnf](https://pypi.org/pypi/isbnlib-bnf/json) |
| BnF Z39.50 / OAI-PMH | `z3950.bnf.fr:2211`, db `TOUT-UTF8`, user `Z3950`/`Z3950_BNF`; OAI `catoai.bnf.fr/oai2/OAIHandler` | MARC / DC | public login | [V2] [BnF wiki](https://raw.githubusercontent.com/wiki/hackathonBnF/hackathon2016/API-Z39.50.md), [Koha](https://raw.githubusercontent.com/Koha-Community/Koha/main/installer/data/mysql/en/optional/sample_z3950_servers.yml); OAI [V1] [BnF wiki](https://raw.githubusercontent.com/wiki/hackathonBnF/hackathon2016/API-OAI-PMH.md) (2016 text) |
| **data.bnf.fr** (BnF) | `https://data.bnf.fr/sparql?format=json&query=…` with `?e bnf-onto:isbn "978-2-…"` (hyphenated) or `?e bnf-onto:ean "{isbn}"`; `bibo:isbn13` / `bibo:isbn10` also present | SPARQL JSON; RDF dumps | none | [V2] [Inventaire](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/bnf/get_bnf_entry_from_isbn.ts), [KBR BELTRANS](https://raw.githubusercontent.com/kbrbe/beltrans-data-integration/main/data-sources/bnf/README.md) (editions dump 39 GB RDF/XML) |
| **SUDOC** (ABES, FR) | `https://www.sudoc.fr/services/isbn2ppn/{isbn}` with `Accept: text/json` (also `ean2ppn`, `issn2ppn`; comma-separated batch); then `https://www.sudoc.fr/{ppn}.rdf` or `.xml` [M] | XML / JSON | none | [V1] [node-sudoc 1.0.4, Inist-CNRS](https://registry.npmjs.org/sudoc) |
| **DNB** (DE) | `https://services.dnb.de/sru/dnb?version=1.1&operation=searchRetrieve&recordSchema=MARC21-xml&maximumRecords=1&query=num%3D{isbn}` (schemas: MARC21-xml, oai_dc, RDFxml; the tutorial pages by 100 records) | SRU 1.1 XML | none | [V2] [DNBLab tutorial](https://raw.githubusercontent.com/deutsche-nationalbibliothek/dnblab/main/DNB_SRU_Tutorial.ipynb), [calibre-dnb](https://raw.githubusercontent.com/citronalco/calibre-dnb/master/__init__.py) |
| **K10plus** (GBV + BSZ, DE) | `https://sru.k10plus.de/opac-de-627?version=1.1&operation=searchRetrieve&query=pica.isb={isbn}&maximumRecords=1` (MARCXML) | SRU XML | none | [V1] [Zotero K10plus ISBN](https://raw.githubusercontent.com/zotero/translators/master/K10plus%20ISBN.js) (updated 2024-10) |
| **lobid-resources** (hbz, DE) | `https://lobid.org/resources/search?q=isbn:{isbn}&format=json` (field name [M]); JSONP; weekly JSON-lines dump | JSON-LD | none | [V1] [lobid API page source](https://raw.githubusercontent.com/hbz/lobid-resources/master/web/app/views/api.scala.html) |
| **Library of Congress** (US) | `https://lx2.loc.gov/sru/lcdb?operation=searchRetrieve&version=1.1&query=bath.ISBN=^{isbn}&maximumRecords=1`; JSON: `https://www.loc.gov/books/?q={isbn}&fo=json` [M] | SRU XML (MARCXML, MODS, DC) / JSON | none | [V2] for SRU: [Zotero](https://raw.githubusercontent.com/zotero/translators/master/Library%20of%20Congress%20ISBN.js), [isbnlib-loc](https://pypi.org/pypi/isbnlib-loc/json), [Koha](https://raw.githubusercontent.com/Koha-Community/Koha/main/installer/data/mysql/en/optional/sample_z3950_servers.yml) — all predate the 2025 ILS change, see section 5 |
| **British Library / BNB** (UK) | none I can vouch for; `bnb.data.bl.uk` is still what [public-apis](https://raw.githubusercontent.com/public-apis/public-apis/master/README.md) lists | — | — | [M, low] |
| **Libris** (KB, SE) | `https://libris.kb.se/find.jsonld?q={isbn}`; SRU `https://libris.kb.se/sru/libris?operation=searchRetrieve&version=1.2&query=…`; OAI-PMH; SPARQL | JSON-LD / XML | none (User-Agent required) | [V1] [find](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/reference/find.en.md), [SRU](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/reference/sru.en.md) |
| **Finna** (National Library of Finland) | `https://api.finna.fi/api/v1/search?lookfor={isbn}&type=AllFields` (an `isbn:` field query is [M]) | JSON | none | [V1] [finna-client, NatLibFi](https://pypi.org/pypi/finna-client/json) |
| **BNE datos** (ES) | `https://datos.bne.es/sparql?format=json&query=…` with `?e <https://datos.bne.es/def/P3013> "{isbn}"` (try plain and hyphenated, 10 and 13) | SPARQL JSON | none | [V1] [Inventaire](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/bne/get_bne_entry_from_isbn.ts) |
| **ICCU / OPAC SBN** (IT) | 2020 HTML-form URL on `opac.sbn.it/sbn3.0/opaclib` returning UNIMARC | HTML/UNIMARC | none | [V1, stale] [isbnlib-sbn](https://pypi.org/pypi/isbnlib-sbn/json); current API unknown |
| **KB** (NL) | `https://jsru.kb.nl/sru/sru?version=1.2&operation=searchRetrieve&startRecord=0&x-collection=GGC&recordSchema=dc&maximumRecords=1&query=identifier.ISBN%20exact%20{isbn}` (the plugin literally sends `query=query=identifier.ISBN…` and retries with the ISBN-10); dumps at `data.bibliotheken.nl/files/` | SRU XML; RDF dumps | none | [V2] [KB python API](https://raw.githubusercontent.com/KBNLresearch/KB-python-API/master/kb/nl/api/sru.py), [isbnlib-kb](https://pypi.org/pypi/isbnlib-kb/json); dumps [V1] [BELTRANS](https://raw.githubusercontent.com/kbrbe/beltrans-data-integration/main/data-sources/kb/README.md) |
| **swisscovery** (SLSP, CH) | `https://swisscovery.slsp.ch/view/sru/41SLSP_NETWORK?version=1.2&operation=searchRetrieve&recordSchema=marcxml&query=alma.isbn={isbn}` (index name [M]) | SRU XML | none | [V1] base URL: [almasru, SLSP](https://pypi.org/pypi/almasru/json) |
| **OBV / ÖNB** (AT) | Alma network-zone SRU, same shape as swisscovery | SRU XML | none | [M, low] |
| **KBR** (BE) | no public ISBN API found; KBR's own project obtained data by catalogue export; union catalogue SRU `http://www.unicat.be/sru` | — / SRU | — | [V1] [BELTRANS KBR](https://raw.githubusercontent.com/kbrbe/beltrans-data-integration/main/data-sources/kbr/README.md), [Catmandu-SRU](https://raw.githubusercontent.com/LibreCat/Catmandu-SRU/master/README.md) (age unknown) |
| **PORBASE** (BNP, PT) | `http://urn.porbase.org/isbn/dc/xml?id={isbn}` | DC XML | none | [V1, 2021] [isbnlib-porbase](https://pypi.org/pypi/isbnlib-porbase/json) |
| **NDL Search** (JP) | `https://ndlsearch.ndl.go.jp/api/opensearch?isbn={isbn}`; SRU `…/api/sru?operation=searchRetrieve&query=isbn%3D{isbn}` | XML (DC-NDL) | none | [M] |
| **Trove** (NLA, AU) | API v3, `X-API-KEY` header | JSON | API key | [M]; key requirement [V1] [public-apis](https://raw.githubusercontent.com/public-apis/public-apis/master/README.md) |
| **Europeana** | `https://api.europeana.eu/record/v2/search.json?wskey={KEY}&query={isbn}` | JSON | API key | key [V1] [pyeuropeana](https://pypi.org/pypi/pyeuropeana/json); pattern [M] |
| **HathiTrust Bib API** | `https://catalog.hathitrust.org/api/volumes/brief/isbn/{isbn}.json`; batch `…/brief/json/isbn:{a}\|isbn:{b}` (max 20) | JSON | none | batch form [V1] [VuFind preview.js](https://raw.githubusercontent.com/vufind-org/vufind/dev/themes/bootstrap5/js/preview.js); bulk "hathifiles" have an ISBN column [V1] [hathifiles](https://raw.githubusercontent.com/hathitrust/hathifiles/main/README.md) |
| **OCLC WorldCat** | nothing open by ISBN (section 5) | — | WSKey + subscription | [V1]/[M] |
| **Wikidata** (WMF) | `https://query.wikidata.org/sparql?format=json&query=SELECT ?item WHERE { { ?item wdt:P212 "{isbn13 hyphenated}" } UNION { ?item wdt:P212 "{isbn13}" } UNION { ?item wdt:P957 "{isbn10…}" } }` | SPARQL JSON | none (User-Agent policy) | [V2] [Inventaire](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/wikidata/get_wikidata_entry_from_isbn.ts), [BookWyrm mappings](https://raw.githubusercontent.com/bookwyrm-social/bookwyrm/main/bookwyrm/connectors/inventaire.py); endpoint in [wikibase-sdk](https://raw.githubusercontent.com/maxlath/wikibase-sdk/main/README.md) |
| **Crossref** | `https://api.crossref.org/works?filter=isbn:{isbn}&mailto={contact}` | JSON | none | [V1] [REST API doc](https://raw.githubusercontent.com/CrossRef/rest-api-doc/master/README.md) (deprecated copy) |
| **OpenAlex** (OurResearch) | **no ISBN filter** in the works filter list | JSON | API key | [V1] [filter-works](https://raw.githubusercontent.com/ourresearch/openalex-docs/main/api-entities/works/filter-works.md) |
| **DOAB / OAPEN** | `https://directory.doabooks.org/rest/search?query=isbn:{isbn}&expand=metadata`; OAI-PMH | JSON / XML | none | [M]; older OAI endpoint in [regluit](https://raw.githubusercontent.com/EbookFoundation/regluit/master/core/loaders/doab.py) [V1] |
| **BookBrainz** (MetaBrainz) | REST API by BBID and search; JSON dumps | JSON | none | [M] |
| **Inventaire.io** (association, FR) | `https://inventaire.io/api/entities?action=by-uris&uris=isbn:{isbn}` | JSON | none | [V1] [BookWyrm](https://raw.githubusercontent.com/bookwyrm-social/bookwyrm/main/bookwyrm/management/commands/initdb.py) |
| **Hardcover** | GraphQL `https://api.hardcover.app/v1/graphql`, `authorization: Bearer …` | JSON | personal token / OAuth | [V1] [Hardcover docs](https://raw.githubusercontent.com/hardcoverapp/hardcover-docs/main/src/content/docs/api/Getting-Started.mdx) |
| **Project Gutenberg / Standard Ebooks** | no ISBNs (public-domain editions); PG catalogue CSV/RDF, SE OPDS/Atom feeds | CSV, RDF, OPDS | none / SE account | [M]; SE feeds page [V1] [feeds/index.php](https://raw.githubusercontent.com/standardebooks/web/master/www/feeds/index.php) |
| **International ISBN Agency** | range message `https://www.isbn-international.org/export_rangemessage.xml`; publisher register: web search only | XML | none | [V2] [python-stdnum updater](https://raw.githubusercontent.com/arthurdejong/python-stdnum/master/update/isbn.py), [isbn3](https://raw.githubusercontent.com/inventaire/isbn3/main/README.md) |

## 3. Catalogue — rights, limits, covers, CORS

| Source | Data licence / ToS | Rate limits | Covers | Browser (CORS) | Value for FR eBooks |
|---|---|---|---|---|---|
| Open Library | Edits are contributed under CC0 ([i18n strings](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/i18n/messages.pot)) [V1]; no licence on cover images [M] | nginx: **180 requests/min per IP** (burst 25) on `/api/*` and `*.json`, 1 request/s on HTML pages; 429 body points to the dumps; applies when there is no session cookie and the IP/UA is not allow-listed ([nginx.conf](https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/nginx.conf), [web_nginx.conf](https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/web_nginx.conf)) [V1]. The public docs ask for a User-Agent with app name and contact and quote 1 request/s unidentified, 3 identified [M] | yes, see `cover-sources.sources.md` | **Yes**: `Access-Control-Allow-Origin: *` on any `.json` path and `/api/` ([processors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/plugins/openlibrary/processors.py)) [V1]; public-apis still says "No" | Medium: print-heavy, patchy for French eBook ISBNs [M] |
| Google Books | Google APIs ToS: no scraping, no permanent database, cache only as long as cache headers allow; Books branding rules ("Powered by Google", link back) [M] | 1,000 requests/day default per project [M] | thumbnails 80–1280 px wide per discovery doc [V1]; no reuse right | Yes [M] | **High**: `saleInfo.isEbook`, many eBook ISBNs; results vary by `country` [V1 for fields] |
| BnF (SRU, data.bnf.fr, dumps) | "L'ensemble des métadonnées descriptives est disponible sous Licence Ouverte" ([BnF wiki](https://raw.githubusercontent.com/wiki/hackathonBnF/hackathon2016/Home.md), 2016) [V1]; commercial reuse and caching allowed, attribution (source + date) required [M] | none published that I know; be polite [M] | `catalogue.bnf.fr/couverture?appName=NE&idArk=ark:/12148/cb{id}&couverture=1`, placeholder detected by content length ([Inventaire](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/bnf/get_bnf_entry_from_isbn.ts)) [V1]; cover rights not granted by the metadata licence [M] | unknown | **High for print**, partial for eBooks (section 4) |
| SUDOC | Licence Ouverte [M] | unknown | no | unknown | Low–medium: academic holdings |
| DNB | CC0 1.0 for bibliographic data [M]; no token in current client code [V2] | unknown | `portal.dnb.de/opac/mvb/cover?isbn=` (MVB-sourced, rights unclear) [V1] | unknown | High for German-language eBooks (legal deposit of online publications) [M] |
| K10plus, lobid | CC0 [M] | "Richtlinien zur API-Nutzung" exist for lobid [V1], content not read | no | lobid: JSONP documented [V1] | Medium (DE academic) |
| LoC | US government work, no copyright in the US [M] | loc.gov JSON API has published per-endpoint limits and blocks for about an hour on excess [M] | no | unknown | Low |
| Libris | "The Swedish national bibliography and Swedish authority records are freely available without restrictions", CC0; KB may rate-limit or block; anonymous clients may be blocked without warning ([terms](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/index.en.md), [User-Agent](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/howto/user_agent.en.md)) [V1] | none numeric | no | docs show a browser `fetch()` example [V1], header not verified | Low |
| Finna | metadata CC0, images excluded [M] | unknown | `api.finna.fi/Cover/Show?id=…` [V1], rights per record | [M, low] yes | Low |
| BNE | CC0 [M] | unknown | no | unknown | Medium for ES |
| KB NL | bibliographic sets open; some KB datasets are CC BY-NC-ND ([KB python API](https://raw.githubusercontent.com/KBNLresearch/KB-python-API/master/README.rst)) [V1] — check per collection | unknown | no | unknown | Medium for NL |
| Wikidata | CC0 [M] | WDQS: 60 s query timeout, 5 parallel queries per IP, User-Agent policy enforced [M] | via P18 / Commons, rarely a cover | Yes [M] | Low hit rate for editions |
| Crossref | "Crossref asserts no claims of ownership to individual items of bibliographic metadata … may be cached and incorporated into the user's content and systems" [V1] | advertised in `X-Rate-Limit-*` headers; polite pool with `mailto` [V1]; lower limits since Dec 2025 [M] | no | Yes [M] | Low: scholarly books with DOIs |
| OpenAlex | CC0 ([README](https://raw.githubusercontent.com/ourresearch/openalex-docs/main/README.md)) [V1] | 100,000 credits/day with a free key, 100 without ([pyalex](https://raw.githubusercontent.com/J535D165/pyalex/main/README.md)) [V1] | no | Yes [V1] | None (no ISBN filter) |
| BookBrainz | core data (author, work, edition group, edition, publisher, relationship) CC0; other public data CC BY-SA 4.0 ([licensing page source](https://raw.githubusercontent.com/metabrainz/bookbrainz-site/master/src/client/components/pages/licensing.tsx)) [V1] | unknown | no | unknown | Very low |
| Inventaire | data CC0 [M]; server code AGPL-3.0 [V1] | unknown | user-uploaded | [M, low] | Medium: it re-queries BnF, BNE, Open Library and Wikidata by ISBN [V1] |
| Hardcover | "we make no copyright or proprietary rights over this data"; user-owned data (reviews, lists, …) barred for commercial products; no LLM training; images are user uploads, DMCA policy advised [V1] | Free: 5,000/day, burst 10, 60/min; Supporter: 50,000/day; 30 s query timeout; API "in beta", tokens may be reset [V1] | user uploads | "CORS is open" but "queries are not allowed to run in the browser" (token secrecy) [V1] | Low |
| HathiTrust | bib API for lookups, hathifiles for bulk [M/V1] | max 20 ids per call [V1] | no | VuFind calls it with a browser `fetch()` [V1], so cross-origin reads presumably work | None |
| Europeana | metadata CC0 [M] | per key [M] | heritage objects | [M] | None (not a trade-book catalogue) |
| NDL | NDL-created records CC BY 4.0; continuous commercial API use may need an application [M, low] | unknown | thumbnail API with third-party conditions [M, low] | unknown | None |

## 4. eBook ISBNs specifically

- Each format of a title has its own ISBN (print, EPUB, PDF) [M]. A lookup on the eBook ISBN often misses where the print
  ISBN hits. In reseller mode take the print ISBN from the ONIX `RelatedProduct` block; in standalone mode fall back to a
  title + author search and let the user confirm.
- BnF: digital legal deposit of eBooks is recent and its cataloguing coverage is partial [M, low]. Expect misses on eBook-only
  ISBNs; `bnf-onto:ean` / `bib.isbn` do work on what is catalogued [V2 for the fields].
- Google Books exposes `saleInfo.isEbook`, `accessInfo.epub` and per-country sale data
  ([discovery doc](https://raw.githubusercontent.com/googleapis/google-api-go-client/main/books/v1/books-api.json)) [V1]; it is
  the open-ish source most likely to know an eBook ISBN [M].
- DNB has collected online publications under legal deposit for years, so German eBook ISBNs are well covered [M].
- Open Library can import a missing ISBN on demand from Amazon and Better World Books
  ([vendors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/core/vendors.py)) [V1]; this is
  US-centric and disabled for plain `/isbn/` lookups (`allow_import=False` in
  [code.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/plugins/openlibrary/code.py)) [V1].

## 5. Changed terms or shut down, 2024–2026

| Service | What happened | Confidence |
|---|---|---|
| OCLC Classify | Discontinued (31 Jan 2024 [M]); the isbnlib plugin for it is marked "DISCONTINUED" on [PyPI](https://pypi.org/pypi/isbnlib-oclc/json) | [V1] + [M] date |
| OCLC WorldCat website | Zotero's translator (updated 2026-08-28) calls an internal `search.worldcat.org/api/search` with a cookie and a response-decryption step ([source](https://raw.githubusercontent.com/zotero/translators/master/Open%20WorldCat.js)): not a public API. OCLC also litigates against bulk scraping [M]. **Do not replicate.** | [V1] |
| OpenAlex | "Starting February 13, 2026, an API key is required", credit-based limits ([pyalex](https://raw.githubusercontent.com/J535D165/pyalex/main/README.md)); the [official docs repo](https://raw.githubusercontent.com/ourresearch/openalex-docs/main/how-to-use-the-api/rate-limits-and-authentication.md) describes credits but still says no key is needed, with different credit costs — the two disagree | [V1], conflict |
| Hardcover | Scoped personal tokens replaced the old ones in August 2026; published rate-limit table; LLM-training ban | [V1] |
| Open Library | Amazon PA-API 5 being replaced by the "Creators API" in its importer ([vendors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/core/vendors.py)); explicit nginx rate limits and bot rules; Internet Archive breach and outage (Oct 2024) and the end of *Hachette v. Internet Archive* (2024) affected lending, not metadata [M] | [V1] / [M] |
| Crossref | GitHub REST docs marked deprecated [V1]; public and polite pool limits reduced on 2025-12-01 [M] | [V1] / [M] |
| Libris | Official docs keep a placeholder page "Migrera från XSearch" ([file](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/howto/xsearch_migration.sv.md)): the legacy `xsearch` API is on its way out; use `find.jsonld` or SRU | [V1], date unknown |
| British Library / BNB | Services crippled by the October 2023 cyber-attack; BNB linked-data access was offline through 2024 and relaunched on a new platform afterwards | [M, low] |
| NDL Search | Relaunched January 2024 on `ndlsearch.ndl.go.jp`; old `iss.ndl.go.jp` URLs are dead (Zotero's NDL translator still targets them, last updated 2015 [V1]) | [M] |
| Trove | API v2 retired in favour of v3; keys reissued under new terms | [M, low] |
| Library of Congress | New library platform replaced the Voyager catalogue in 2025; whether `lx2.loc.gov` SRU kept working is unknown | [M, low] |
| Wikidata Query Service | 2025 graph split: scholarly articles moved to a separate endpoint; book editions stay on the main one | [M] |
| Standard Ebooks | Bulk feeds tied to a Patrons Circle account; the feed URL embeds the member's e-mail ([feeds page](https://raw.githubusercontent.com/standardebooks/web/master/www/feeds/index.php)) | [V1] weak / [M] |

## 6. International ISBN Agency: range message and publisher register

- The hyphenation table is published as `RangeMessage.xml`; open-source libraries regenerate their tables from
  `https://www.isbn-international.org/export_rangemessage.xml` on a schedule (python-stdnum, isbn3 monthly via CI,
  [biblys/isbn](https://raw.githubusercontent.com/biblys/isbn/main/README.md)) [V2]. The XML carries a serial number and a
  date (python-stdnum prints both) [V1]; I know of no licence element in it [M].
- **Licence: not found.** None of the three projects states one; they ship derived tables under MIT or LGPL. From memory the
  Agency offers the file free of charge for implementing ISBN handling and keeps its copyright [M, low]. For this repository:
  generate the Rust table directly from the Agency's XML with our own script (do **not** copy python-stdnum's LGPL
  `isbn.dat`), record serial and date, refresh it in the dependency-update job, and treat prefix ranges as facts.
- Global Register of Publishers: free web search of publisher prefixes, no API, no bulk download, all rights reserved [M].
  AFNIL (French agency) has no public API [M]. Do not scrape either.

## 7. Recommended lookup order for `ebc-catalog`

1. EPUB OPF metadata or reseller ONIX — no network, no rights question.
2. By registration group: 978-2 / 979-10 → BnF SRU (then data.bnf.fr SPARQL); 978-3 → DNB, then K10plus / lobid;
   978-84 → BNE; 978-88 / 979-12 → SBN (API to be re-found); 978-90 / 978-94 → KB; 978-91 → Libris; 978-951 / 978-952 →
   Finna; 978-0 / 978-1 / 979-8 → Open Library, LoC. Group numbers other than those in section 1 are [M].
3. Open Library (identify with a User-Agent carrying a contact address; stay under 1 request/s).
4. Google Books with a key (and `country=FR` if the live check confirms it) — display-only, do not store beyond the session unless the ToS check in section 9
   says otherwise.
5. Inventaire or Wikidata as a last resort; SUDOC `isbn2ppn` for academic titles.

Store what Licence Ouverte and CC0 sources return (with source name and retrieval date for BnF and SUDOC). Keep one provider
trait, sans-IO, with recorded fixtures per provider; live tests opt-in. Send a descriptive User-Agent everywhere: Open
Library, Libris, Crossref, Wikimedia and Hardcover all ask for it [V1 for OL, Libris, Crossref, Hardcover].

## Sources

All fetched 2026-11-12 (HTTP 200).

- Open Library: [nginx.conf](https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/nginx.conf),
  [web_nginx.conf](https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/web_nginx.conf),
  [processors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/plugins/openlibrary/processors.py),
  [code.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/plugins/openlibrary/code.py),
  [oldump.sh](https://raw.githubusercontent.com/internetarchive/openlibrary/master/scripts/oldump.sh),
  [vendors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/core/vendors.py),
  [messages.pot](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/i18n/messages.pot)
- BnF: [hackathon wiki home](https://raw.githubusercontent.com/wiki/hackathonBnF/hackathon2016/Home.md),
  [Z39.50](https://raw.githubusercontent.com/wiki/hackathonBnF/hackathon2016/API-Z39.50.md),
  [OAI-PMH](https://raw.githubusercontent.com/wiki/hackathonBnF/hackathon2016/API-OAI-PMH.md),
  [Zotero BnF ISBN](https://raw.githubusercontent.com/zotero/translators/master/BnF%20ISBN.js),
  [isbnlib-bnf](https://pypi.org/pypi/isbnlib-bnf/json),
  [Inventaire BnF resolver](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/bnf/get_bnf_entry_from_isbn.ts),
  [BELTRANS BnF notes](https://raw.githubusercontent.com/kbrbe/beltrans-data-integration/main/data-sources/bnf/README.md)
- SUDOC: [node-sudoc on npm](https://registry.npmjs.org/sudoc)
- Germany: [DNBLab README](https://raw.githubusercontent.com/deutsche-nationalbibliothek/dnblab/main/README.md),
  [DNB SRU tutorial](https://raw.githubusercontent.com/deutsche-nationalbibliothek/dnblab/main/DNB_SRU_Tutorial.ipynb),
  [calibre-dnb](https://raw.githubusercontent.com/citronalco/calibre-dnb/master/__init__.py),
  [Zotero K10plus ISBN](https://raw.githubusercontent.com/zotero/translators/master/K10plus%20ISBN.js),
  [lobid API page](https://raw.githubusercontent.com/hbz/lobid-resources/master/web/app/views/api.scala.html)
- Other libraries: [Zotero LoC ISBN](https://raw.githubusercontent.com/zotero/translators/master/Library%20of%20Congress%20ISBN.js),
  [isbnlib-loc](https://pypi.org/pypi/isbnlib-loc/json),
  [Koha sample servers](https://raw.githubusercontent.com/Koha-Community/Koha/main/installer/data/mysql/en/optional/sample_z3950_servers.yml),
  Libris [terms](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/index.en.md),
  [User-Agent](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/howto/user_agent.en.md),
  [find](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/reference/find.en.md),
  [SRU](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/reference/sru.en.md),
  [XSearch placeholder](https://raw.githubusercontent.com/libris/libris-api-docs/master/docs/howto/xsearch_migration.sv.md);
  [finna-client](https://pypi.org/pypi/finna-client/json);
  [Inventaire BNE resolver](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/bne/get_bne_entry_from_isbn.ts);
  [isbnlib-sbn](https://pypi.org/pypi/isbnlib-sbn/json); [isbnlib-porbase](https://pypi.org/pypi/isbnlib-porbase/json);
  KB [README](https://raw.githubusercontent.com/KBNLresearch/KB-python-API/master/README.rst),
  [sru.py](https://raw.githubusercontent.com/KBNLresearch/KB-python-API/master/kb/nl/api/sru.py),
  [isbnlib-kb](https://pypi.org/pypi/isbnlib-kb/json),
  [BELTRANS KB](https://raw.githubusercontent.com/kbrbe/beltrans-data-integration/main/data-sources/kb/README.md),
  [BELTRANS KBR](https://raw.githubusercontent.com/kbrbe/beltrans-data-integration/main/data-sources/kbr/README.md);
  [almasru](https://pypi.org/pypi/almasru/json); [Catmandu-SRU](https://raw.githubusercontent.com/LibreCat/Catmandu-SRU/master/README.md);
  [pyeuropeana](https://pypi.org/pypi/pyeuropeana/json);
  [hathifiles](https://raw.githubusercontent.com/hathitrust/hathifiles/main/README.md),
  [VuFind preview.js](https://raw.githubusercontent.com/vufind-org/vufind/dev/themes/bootstrap5/js/preview.js);
  [Zotero NDL translator](https://raw.githubusercontent.com/zotero/translators/master/National%20Diet%20Library%20Catalogue.js)
- Google Books: [discovery document](https://raw.githubusercontent.com/googleapis/google-api-go-client/main/books/v1/books-api.json),
  [calibre-web provider](https://raw.githubusercontent.com/janeczku/calibre-web/master/cps/metadata_provider/google.py),
  [isbnlib](https://pypi.org/pypi/isbnlib/json)
- OCLC: [Zotero Open WorldCat](https://raw.githubusercontent.com/zotero/translators/master/Open%20WorldCat.js),
  [isbnlib-oclc](https://pypi.org/pypi/isbnlib-oclc/json)
- Open knowledge bases: [Inventaire Wikidata resolver](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/wikidata/get_wikidata_entry_from_isbn.ts),
  [Inventaire Open Library resolver](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/openlibrary/get_openlibrary_entry_from_isbn.ts),
  [Inventaire README](https://raw.githubusercontent.com/inventaire/inventaire/main/README.md),
  [wikibase-sdk README](https://raw.githubusercontent.com/maxlath/wikibase-sdk/main/README.md),
  [BookWyrm initdb](https://raw.githubusercontent.com/bookwyrm-social/bookwyrm/main/bookwyrm/management/commands/initdb.py),
  [BookWyrm Inventaire connector](https://raw.githubusercontent.com/bookwyrm-social/bookwyrm/main/bookwyrm/connectors/inventaire.py),
  [Crossref REST doc](https://raw.githubusercontent.com/CrossRef/rest-api-doc/master/README.md),
  OpenAlex [README](https://raw.githubusercontent.com/ourresearch/openalex-docs/main/README.md),
  [rate limits](https://raw.githubusercontent.com/ourresearch/openalex-docs/main/how-to-use-the-api/rate-limits-and-authentication.md),
  [filters](https://raw.githubusercontent.com/ourresearch/openalex-docs/main/api-entities/works/filter-works.md),
  [pyalex](https://raw.githubusercontent.com/J535D165/pyalex/main/README.md),
  [BookBrainz licensing page](https://raw.githubusercontent.com/metabrainz/bookbrainz-site/master/src/client/components/pages/licensing.tsx),
  [Hardcover API guide](https://raw.githubusercontent.com/hardcoverapp/hardcover-docs/main/src/content/docs/api/Getting-Started.mdx),
  [regluit DOAB loader](https://raw.githubusercontent.com/EbookFoundation/regluit/master/core/loaders/doab.py),
  [Standard Ebooks feeds page](https://raw.githubusercontent.com/standardebooks/web/master/www/feeds/index.php),
  [public-apis list](https://raw.githubusercontent.com/public-apis/public-apis/master/README.md)
- ISBN ranges: [python-stdnum updater](https://raw.githubusercontent.com/arthurdejong/python-stdnum/master/update/isbn.py),
  [isbn.dat](https://raw.githubusercontent.com/arthurdejong/python-stdnum/master/stdnum/isbn.dat),
  [isbn3](https://raw.githubusercontent.com/inventaire/isbn3/main/README.md),
  [biblys/isbn](https://raw.githubusercontent.com/biblys/isbn/main/README.md)

## Could not verify

Nothing in this list was checked against the operator's live site today.

- **Every licence and ToS statement marked [M]**: Licence Ouverte version and attribution wording for BnF and ABES; CC0 for DNB,
  K10plus, lobid, Finna, BNE, Wikidata, DOAB/OAPEN, Inventaire; licence of OPAC SBN, swisscovery, OBV/ÖNB and KBR data.
- **Google Books**: exact ToS wording on caching and branding, the default quota, whether keyless calls are still served.
- **CORS headers** of every service except Open Library (source) and Hardcover (operator statement). No live request was made.
- **Open Library**: the figures on its public API page (1 and 3 requests/s); only the nginx config (180/min) was read. Dump
  frequency (monthly [M]).
- **BnF**: current SRU record schemas and any rate limit; eBook legal-deposit coverage; whether the catalogue général was
  re-platformed in 2024–2026. The BnF wiki read here dates from 2016.
- **SUDOC**: current SRU endpoint and the `{ppn}.rdf` / `.xml` record URLs; node-sudoc dates from 2022 and uses `http://`.
- **Library of Congress**: whether `lx2.loc.gov` SRU survived the 2025 platform change; loc.gov JSON rate limits.
- **British Library / BNB**: current endpoints and status. **Trove**: v3 terms on commercial use. **NDL**: commercial-use and
  thumbnail conditions. **Europeana**: current key policy. **ICCU/SBN**: any current API. **KBR**: any public API.
- **OCLC**: Classify shutdown date, what the free tier of WorldCat Entities returns, the status of the scraping litigation.
- **Crossref** December 2025 limits; **OpenAlex** credit costs (two sources disagree); **Wikidata** WDQS limits and graph-split
  details; **DOAB/OAPEN** REST path after any platform migration; **BookBrainz** API base URL.
- **International ISBN Agency**: the licence or terms attached to `RangeMessage.xml` and to the Global Register of Publishers.
- Registration-group routing beyond 978-2, 979-10, 979-12, 979-8 (read today) is from memory.
- Hit rates ("high", "medium", "low" for French eBooks) are judgement, not measurements: measure them on a sample of 200
  ISBNs from a French reseller catalogue before fixing the provider order.
