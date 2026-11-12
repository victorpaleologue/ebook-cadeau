# ebc-catalog

Sans-IO clients for ISBN and book-metadata databases, with record merging.

Given an ISBN, the crate says which HTTP requests to make, reads the answers and returns
one normalised `ebc_model::BookMeta` with its sources. It performs no I/O by itself, so the
same code runs on a server, in WebAssembly and in tests.

> **Read this first.** No endpoint could be called while the crate was written: the
> sandbox blocks every catalogue host. Requests follow
> `docs/research/01-isbn-databases.md`; response parsing follows each API's documented
> format, from memory, and is tested against **hand-written fixtures, not recordings**.
> Licence, terms and limit statements below come from that research document, which was
> synthesised but **not fact-checked**; most are marked "from memory" there. They live in
> one `static ProviderInfo` per provider (`src/providers/*.rs`), so a correction is a
> one-line change — and `tests/providers.rs` checks that this README still agrees.
> Run the live tests (below) before relying on any provider.

## Use

```rust
use ebc_catalog::{Config, LookupOptions, lookup_all, providers_for};

let isbn = ebc_model::Isbn13::parse("978-2-07-036002-4")?;
let providers = providers_for(&isbn, &Config::default());   // BnF, then Open Library
let report = lookup_all(&providers, &isbn, |request| my_http(request), &LookupOptions::default());
if let Some(book) = report.merged { /* title, contributors, publisher…, sources */ }
```

- `Provider::lookup(isbn)` gives the first request(s); `Provider::parse(isbn, request,
  response)` gives records and follow-up requests. "Not found" is an empty result, not an
  error. `parse` never panics, whatever the bytes.
- `drive(provider, isbn, exec, max_requests)` runs one provider with a synchronous
  executor; `Session` is the same state machine for hosts that do their own (async) I/O.
- `lookup_all(providers, isbn, exec, options)` runs a cascade and merges. By default it
  always asks the first two providers and stops once the record has a title and an author.
- `merge(records)` reconciles records in priority order; `normalize` cleans ISBD and MARC
  artefacts (`"[Texte imprimé]"`, `" / statement"`, NSB/NSE and `<<…>>` marks,
  `"Camus, Albert (1913-1960)"`, `fre`, `"DL 2015"`, `"1 vol. (185 p.)"`,
  `"[Paris] : Gallimard"`, HTML in descriptions).
- `Config::with_base_url(id, url)` points a provider at another server (tests, the CLI's
  fixture server). For SRU and SPARQL providers the value is the full endpoint URL; for
  the others, the site root.
- Providers keep private context in the URL **fragment** of follow-up requests. HTTP
  never transmits fragments; key fixture maps by `wire_url(request)`.
- Feature `client` (off by default, never built for wasm32): `client::Client`, a
  `reqwest` + `rustls` executor with a per-host minimum interval, a timeout, a body-size
  cap, the `User-Agent` `ebook-cadeau/<version> (+https://github.com/victorpaleologue/ebook-cadeau)`,
  a deny-list and a `Cache` hook. Retries are left to the caller
  (`CatalogError::is_transient`, `retry_after_s`).

## Providers

Class: **A** open, **B** terms-bound, **C** contract (research §5.5). "On" = included by
`default_providers` / `providers_for` without configuration. Interval = minimum delay the
`client` leaves between two requests to the host; where no limit is published it is our
own default of one request per second. CORS "no" means "unverified or no": relay through
a server. Every row is unverified against the live service.

| id | Source | Endpoint used | Class | Data licence | Terms URL | Key | Interval | CORS | On |
|---|---|---|---|---|---|---|---|---|---|
| `bnf` | BnF catalogue général | SRU 1.2 `https://catalogue.bnf.fr/api/SRU`, `recordSchema=unimarcXchange` (Dublin Core also parsed) | A | Licence Ouverte / Open Licence (Etalab) | https://www.bnf.fr/fr/conditions-de-reutilisation-des-donnees-de-la-bnf | no | 1 s | no | yes |
| `dnb` | Deutsche Nationalbibliothek | SRU 1.1 `https://services.dnb.de/sru/dnb`, `MARC21-xml`, `query=num=` | A | CC0 1.0 | https://www.dnb.de/EN/Professionell/Metadatendienste/Datenbezug/SRU/sru_node.html | no | 1 s | no | yes |
| `k10plus` | K10plus (GBV + BSZ) | SRU 1.1 `https://sru.k10plus.de/opac-de-627`, `query=pica.isb=` | A | CC0 1.0 | https://wiki.k10plus.de/display/K10PLUS/Open+Data | no | 1 s | no | yes |
| `lobid` | lobid-resources (hbz) | `https://lobid.org/resources/search?q=isbn:…&format=json` | A | CC0 1.0 | https://lobid.org/usage-policy/ | no | 1 s | no | yes |
| `libris` | Libris (KB, Sweden) | `https://libris.kb.se/find.jsonld?q=…` | A | CC0 1.0 | https://libris.kb.se/api-docs/ | no (User-Agent required) | 1 s | no | yes |
| `finna` | Finna (Finland) | `https://api.finna.fi/api/v1/search?lookfor=…` | A | CC0 1.0 (metadata; images excluded) | https://www.finna.fi/Content/terms | no | 1 s | no | yes |
| `openlibrary` | Open Library | `https://openlibrary.org/api/books?bibkeys=ISBN:…&format=json&jscmd=details`; follow-ups `/authors/OL…A.json`, `/works/OL…W.json` only when names are missing | A | CC0 1.0 (contributions to Open Library) | https://openlibrary.org/developers/api | no (User-Agent with contact) | 1 s (verified ceiling 180/min per IP) | **yes** (verified in source) | yes |
| `googlebooks` | Google Books | `https://www.googleapis.com/books/v1/volumes?q=isbn:…&key=…` | **B** | Google APIs Terms of Service and Books API terms (proprietary) | https://developers.google.com/books/terms | **yes** | 1 s; about 1,000/day, stop at 900 | yes | only with a key |
| `sudoc` | SUDOC (ABES) | `https://www.sudoc.fr/services/isbn2ppn/…`, then `https://www.sudoc.fr/{ppn}.xml` (UNIMARC) | A | Licence Ouverte / Open Licence (Etalab) | https://abes.fr/reseau-sudoc/reutiliser-les-donnees-sudoc/ | no | 1 s | no | no |
| `loc` | Library of Congress | SRU 1.1 `https://lx2.loc.gov/sru/lcdb`, `query=bath.isbn=` | A | US government work, no known copyright restriction | https://www.loc.gov/legal/ | no | 3 s | no | no |
| `inventaire` | Inventaire.io | `https://inventaire.io/api/entities?action=by-uris&uris=isbn:…`, then the same endpoint for work, publisher, collection and author entities | A | CC0 1.0 | https://wiki.inventaire.io/wiki/Terms_of_Use | no | 2 s | no | no |
| `wikidata` | Wikidata Query Service | `https://query.wikidata.org/sparql?format=json&query=…` (one SPARQL query on P212 / P957) | A | CC0 1.0 | https://www.mediawiki.org/wiki/Wikidata_Query_Service/User_Manual | no (User-Agent policy) | 2 s | yes | no |
| `crossref` | Crossref | `https://api.crossref.org/works?filter=isbn:…&rows=5&mailto=…` | A | No ownership claimed on individual items of bibliographic metadata | https://www.crossref.org/documentation/retrieve-metadata/rest-api/ | no (`mailto` for the polite pool) | 1 s | yes | no |

Obligations the host must honour (not enforced by this crate):

- **Attribution.** Licence Ouverte sources (`bnf`, `sudoc`) require naming the source and
  a date: show `SourceRef::attribution` with the retrieval date and a link to
  `SourceRef::record_url`. Google Books requires "Powered by Google" and a link to the
  volume. CC0 sources ask for nothing; list them anyway.
- **Storage.** Class A results may be cached with source and date. Class B (Google
  Books, `permanent_storage = Forbidden`) may be kept in memory for the session only —
  the `Cache` hook receives the `ProviderInfo` to decide. `LookupReport::records()` tells
  which provider supplied what.
- **Descriptions.** Open Library and Google Books return publisher descriptions. The
  research recommends storing and printing facts only (implication 27): drop
  `BookMeta::description` before persisting unless the text comes from the reseller or the
  user. Library catalogues' summaries are not read at all.
- **Covers.** `BookMeta::covers` are references found on the way (Open Library, Google
  Books, Inventaire). No metadata licence clears an image: they are preview candidates
  for `ebc-covers`, never printable by themselves.
- **Etiquette.** One lookup per provider per user action; no crawling.

### Routing (`providers_for`)

978-2 and 979-10 → `bnf`; 978-3 → `dnb`, `k10plus`, `lobid`; 978-91 → `libris`; 978-951
and 978-952 → `finna`; then always `openlibrary`, then `googlebooks` if a key is
configured, then the optional sources that were enabled (`sudoc` for French groups, `loc`
for 978-0, 978-1 and 979-8, `inventaire`, `wikidata`, `crossref`).

### Not implemented, and why

| Source | Status |
|---|---|
| data.bnf.fr (SPARQL) | Same catalogue as the BnF SRU; adds work and author identifiers we do not need yet |
| BNE datos (Spain, SPARQL) | Ranked tier 1 for 978-84 / 979-13, but only the ISBN property is documented in the research; the rest of the ontology would be guesswork |
| KB (Netherlands, SRU) | Hold: some KB datasets are CC BY-NC-ND; needs a licence check per collection first |
| swisscovery, OBV, ICCU/SBN, KBR/Unicat, PORBASE, British Library | Hold: endpoint or licence unknown |
| OpenAlex | No ISBN filter: not usable |
| Hardcover, HathiTrust, Europeana, Trove, NDL, DOAB/OAPEN, BookBrainz, Internet Archive | Ranked "skip" |
| ONIX files (publisher or reseller) | The best source in reseller mode, but not an ISBN database: out of this crate's contract |

### Contract sources (class C) — documented, not implemented

None could be priced or its terms read. They would be optional, server-side, off by
default, enabled only when configuration names a contract.

| Source | What is known | Status |
|---|---|---|
| Dilicom FEL / "FEL à la demande" (FR) | SOAP lookup by EAN, up to 100 EANs per call, user + password; price as an integer in thousandths of a euro; no covers | Not implemented: members only, `http://` endpoint seen in 2020s client code |
| Decitre ORB (FR) | `GET https://api.base-orb.fr/v1/products?eans=…`, HTTP basic auth (user + key); best-documented shape | Not implemented: subscription terms unread |
| Electre NG (FR) | Subscription REST API | Not implemented: API shape not public |
| Tite-Live Médiabase (FR) | Bundled with its bookshop software | Not available |
| VLB (MVB, DE) | REST + ONIX, token | Not implemented: paid licence |
| ISBNdb | Self-serve paid REST API | Not implemented: test the hit rate on French eBook ISBNs first |

### Excluded — never contact

Amazon (all retail and image domains), Goodreads, LibraryThing, Babelio, Fnac, Kobo,
decitre.fr, WorldCat, the ISBN Agency's publisher register, AFNIL. `DENIED_HOSTS` /
`is_denied_host` encode the list; `Session` refuses to issue a request to them (even
through a base-URL override) and `client::Client` refuses requests and redirects to them.

## Tests and fixtures

```sh
cargo test -p ebc-catalog                     # offline: unit, provider, merge, driver, robustness
cargo test -p ebc-catalog --features client   # adds the reqwest client against a loopback server
```

`tests/fixtures/<provider>/` holds the hand-written responses; `tests/fixtures/README.md`
says what is invented. `tests/robustness.rs` feeds every fixture truncated at many
offsets, corrupted copies and pseudo-random garbage to every request shape of every
provider.

### Live tests

Opt-in twice — `#[ignore]` and `EBC_LIVE=1` — so that no CI job sends a request by
accident. They have never been run.

```sh
EBC_LIVE=1 cargo test -p ebc-catalog --features client --test live -- --ignored --nocapture
EBC_LIVE=1 EBC_GOOGLE_BOOKS_KEY=… cargo test -p ebc-catalog --features client --test live -- --ignored live_googlebooks
```

A failure means a fixture, and the parser built on it, encodes a wrong assumption. Things
most likely to need a fix, in order: the BnF `recordSchema` name and its `bib.isbn`
query; the K10plus `recordSchema` parameter; the lobid `isbn:` field; the SUDOC `https`
scheme and `{ppn}.xml` record URL; the Libris and Finna field names; the Inventaire claim
shapes; whether the Library of Congress SRU endpoint still exists.

### Re-recording fixtures

```sh
EBC_LIVE=1 EBC_CONTACT=you@example.org cargo run -p ebc-catalog --features client \
    --example record-fixtures -- target/recorded 9782070360024 9783257229530 9780141439518
```

The example runs every open (class A) provider for each ISBN at the provider's minimum
interval and writes each response body to `target/recorded/<provider>/<isbn>-<n>.<ext>`
plus `target/recorded/index.json` (URL, status, content type, retrieval time, licence).
Google Books is skipped: class B responses must not be committed.

To replace a fixture: compare the recording with the hand-written file, fix the parser if
they differ in shape, copy the recording under `tests/fixtures/<provider>/` with a line in
`tests/fixtures/README.md` giving its URL and retrieval date (the Licence Ouverte asks for
both), and update the expectations in `tests/providers.rs`. Then correct the "unverified"
wording here and in the provider's `ProviderInfo::notes`.

## Updating licence or terms data

Edit the `static` at the top of the provider's module (`src/providers/sru.rs` holds four),
then this table. `tests/providers.rs::readme_table_matches_provider_info` fails when the
two disagree.
