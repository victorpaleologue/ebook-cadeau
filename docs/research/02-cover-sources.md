# Finding book covers (and spines) when absent — sources, scraping practice, legality of acquisition

_Status: draft — research date 2026-11-12_

> **Research to brief the implementation and a lawyer, not legal advice.** This document covers the **acquisition** step: finding, fetching, previewing and storing a cover or spine the user did not supply. Whether a cover may be **printed** is decided in `07-legal-covers-and-reseller-rights.md`. Where the two documents meet, doc 07 governs.

**Confidence markers.** [V2] = verified in at least two independent sources; [V1] = one source; [M] = from memory, not verified in this session. Qualifiers kept from the researcher notes: "[M, low]" = shaky recall, do not code or plead against it without a check; an asterisk ([V1\*], [V2\*]) = verified in a sibling researcher's search results, link copied from their notes and not re-opened here; [est.] = my arithmetic. Risk ratings are assessments, not sourced rulings.

**Method limits — read first.**

- **No web search ran**, for either researcher or for this synthesis. The shared budget of 200 searches was already spent; the five queries I sent were refused. WebFetch is blocked for almost every host. I did not work around either limit.
- The one reachable content host is `raw.githubusercontent.com`. "Source" in this document therefore means a primary file read there: the source code and server configuration of a service, the cover-provider code of library systems (calibre, VuFind, Koha, Coce, Inventaire), and specifications kept in Git (Readium LCP, W3C TDMRep, EPUB 3.3, Kobo's EPUB guidelines, Google's API discovery document). For [V2], "independent" means independent code bases or documents.
- The two researcher notes are `notes/cover-sources.sources.md` (about 45 files read) and `notes/cover-sources.scraping-law.md` (written entirely from memory). For this synthesis I read 26 files (19 new to the cover notes, 7 re-read; §9.3); they are the only new evidence.
- **Not verified by anyone**: every provider's terms of use and quotas, every legal authority (cases, CPI articles, CNIL guidance), every live HTTP behaviour, every French bookseller URL pattern. GPL/AGPL code (calibre, VuFind, Koha, Inventaire) was read for facts only; none of it may be copied into this BSD-3 repository.

**What this pass changed against the notes.**

| Point | Notes said | Now | Evidence |
|---|---|---|---|
| Open Library cover server and CORS | "not verified; plan a server-side fetch proxy" | `Access-Control-Allow-Origin: *` is added to every response [V1]; the final hop is often archive.org, unverified | [asgi_app.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/asgi_app.py) |
| BnF cover URL and placeholder | [M] in the sources note, [V1] in `notes/isbn-databases.open.md` | [V1]: URL pattern, HEAD probe, placeholder bodies of 4566 or 4658 bytes | [Inventaire](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/bnf/get_bnf_entry_from_isbn.ts) |
| Google Books image sizes | [M] | [V1]: widths of about 80, 128, 300, 575, 800, 1280 px | [discovery document](https://raw.githubusercontent.com/googleapis/google-api-go-client/main/books/v1/books-api.json) |
| LCP leaves the cover unencrypted | [M, low] | [V1] | [Readium LCP spec](https://raw.githubusercontent.com/readium/lcp-specs/master/releases/lcp/latest.md) |
| TDMRep signal names and precedence | [M, low] | [V1] | [TDMRep draft spec](https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/spec/index.html) |
| TDMRep deployed by French trade publishers | belief, [M, low] | [V1]: Hachette Livre (most of its houses' websites) and Média-Participations (technique not stated) | [adopters list](https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/docs/adopters.md) |
| Wording of DSM art. 4 and recital 18 | [M] | [V1]; recital 18 itself names website terms and conditions | [extracts](https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/docs/eu-act-extracts.md) |
| Size of the cover inside an EPUB | "1400–2560 px, usually no upscaling" [M] | Contradicted: Kobo states a store average of 800 x 1224 px [V1] | [Kobo EPUB spec](https://raw.githubusercontent.com/kobolabs/epub-spec/master/README.md) |
| ONIX List 158 | codes 29 and 51 exist [V1] | Full list read: no spine-only code; 29 `FullCover` is the only one that includes a spine [V1] | [im_onix codelist](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-158.yml) |
| Amazon API status | "could not check" | PA-API 5.0 is being replaced by the "Creators API" [V1]; its image variants can include back cover and spine [V1] | [Open Library vendors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/core/vendors.py) |
| robots.txt fetch errors, 500 KiB cap | [M] | [V1] (secondary: a parser's README restating Google's guidance) | [texting_robots](https://raw.githubusercontent.com/Smerity/texting_robots/main/README.md) |

---

## 1. Executive summary and recommended acquisition cascade

### 1.1 Findings

1. **The best cover source is not a cover API.** In reseller mode it is the cover in the reseller's ONIX or distributor feed. In standalone mode it is the cover inside the user's own EPUB, read in the browser. Both come with the cleanest rights position and need no acquisition from a third party. Public ISBN cover APIs are thumbnail services: about 500 px on the long side, about 71 ppi on a 178 mm front [est.].
2. **No lookup source grants a print right, and none is print-grade.** Open Library, Google Books, BnF and DNB can locate and preview a cover. Under doc 07 their images never enter a PDF produced by the hosted service. The acquisition tools therefore produce two different things: **printable assets** (bytes with a recorded rights basis) and **candidates** (a URL plus facts, rights basis `none`).
3. **Open Library is the only open, keyless source whose behaviour is verified in its own source code** [V1/V2]: size suffixes, original-size URL, JSON endpoint with pixel dimensions, `?default=false` returning 404, rate limits, user-agent rules, CORS. It carries no licence on the images [M].
4. **Amazon, Goodreads-derived APIs, retailer pages and subscription services are out.** Either their terms forbid offline use [M] or they are paid B2B services licensed for catalogue display [M]. A real-time relay of a retailer's catalogue also has the structure condemned in *Innoweb* [M].
5. **"Publicly reachable" is not "free to take".** Three regimes apply separately: copyright in the image, the sui generis database right of the aggregator or retailer, and contract or unfair competition (terms of use, *parasitisme*) [M].
6. **robots.txt is not a licence, but a `Disallow` is a prohibition with legal weight**: it is the standard machine-readable opt-out under DSM art. 4(3) [M], and the directive's recital 18 also counts website terms and conditions [V1].
7. **The text-and-data-mining exception covers analysis only.** It allows a transient copy to derive facts (dominant colour, hash, font class) and requires the copy to go afterwards. It never covers display, caching or printing [V1 for the article text; application is my analysis]. French trade publishers have started to opt out: the TDMRep adopters list names Hachette Livre, on most of its houses' websites, and Média-Participations [V1].
8. **Linking and copying are treated oppositely.** A browser that loads an image from the source server makes no new communication if the source is authorised and unprotected (*Svensson* line [M]). Downloading it and serving it from our server does (*Renckhoff* [V2\*]). A server-side image proxy counts as a re-host.
9. **Resolution.** A 110 x 178 mm front needs 1299 x 2102 px at 300 ppi [est.]. A 500 px cover needs 4.2x upscaling; the cover inside an EPUB may be only about 800 x 1224 px (175 ppi) [V1], so it can also need help. In-browser super-resolution is feasible with permissively licensed, image-only models (Real-ESRGAN BSD-3, SwinIR and Swin2SR Apache-2.0, waifu2x MIT) [V1 each], but GAN models damage small type.
10. **Spines cannot be found.** No public source exposes spines by ISBN. ONIX has a full-cover code (List 158 code 29) [V1] that no known French feed fills [M, low]. Even a real spine rarely fits a 10 mm panel without adaptation. Generating the spine is the only general path; an optional flat-cover upload is the only source of a real one.

### 1.2 Recommended cascade

Risk is the exposure of the service, France first, on three acts: (a) locate and preview, (b) store image bytes on our side, (c) print. Ratings come from the scraping-law note (§6.10). Steps 1–6 yield printable assets under the rights basis named in doc 07. Steps 7–11 yield candidates and facts only.

| # | Step | Mode | Yields | (a) Locate, preview | (b) Store | (c) Print |
|---|---|---|---|---|---|---|
| 1 | Publisher or reseller feed cover **with an express print clause or per-title flag** (`print_licence`) | Reseller | Printable front; back and full cover if supplied | Low | Low | Low |
| 2 | Rights-holder upload: image, or flat full cover ("couverture à plat") from an author or publisher (`rights_holder_declared`) | Standalone | Printable front, real spine, back | Low | Low | Low |
| 3 | Cover extracted **in the browser** from the user's own EPUB (`user_private_copy`) | Standalone | Printable front for one home print | Low | Low if never uploaded; Medium if we host it | Home: Low–Medium. Pro: High unless the user is the rights holder |
| 4 | Local image file picked by the user (scan, photo, saved image) | Standalone | Same as 3 | Low | as 3 | as 3 |
| 5 | Public-domain artwork (artist dead more than 70 years) from a host without contractual restriction (`public_domain`) | Both | Printable | Low | Low | Low |
| 6 | Reseller feed cover under a "promotion and sale" licence only (`promo_all_media` if declared, else `none`) | Reseller | Preview on the reseller's pages; reduced-size print only with the reseller's written declaration | Low | Low | Medium |
| 7 | Open Library: ISBN → cover id and pixel size → preview by inline link, link to the original | Standalone | Candidate | Low–Medium | Medium–High | High |
| 8 | Google Books with an API key and attribution, used as its terms say | Standalone | Candidate, thumbnail only | Low–Medium | Medium–High | High |
| 9 | National catalogue cover: BnF for 978-2 and 979-10, DNB for 978-3 — **disabled until their image terms have been read** | Standalone | Candidate | Medium (terms unread; my rating) | High | High |
| 10 | Publisher's own page, URL pasted by the user: extract `og:image` or JSON-LD image URL, show it as a link | Standalone | Candidate | Low–Medium | High | High |
| 11 | Transient analysis of a candidate from 7–10: dominant colour, aspect ratio, perceptual hash, font class; the image is then discarded | Standalone | Facts only | Low–Medium, and only where no opt-out is expressed | Image: must go. Facts: Low | Colour and font choice on our own typographic card: Low |
| 12 | Typographic cover: title, author, publisher retyped in an open font | Both | Always available | Low | Low | Low |

**Never, in any mode** (High on all three acts, or contractually barred): Amazon image URLs, product pages and the PA-API or Creators API; Goodreads and Goodreads-derived APIs; pages or image CDNs of retailers and aggregators (Fnac, Decitre, Furet du Nord, Cultura, leslibraires.fr, Babelio); Electre, Dilicom, Tite-Live, ORB, VLB, Syndetics, Content Cafe without a contract; image-search results; bulk download or mirroring of any cover catalogue; any route that needs a login, a CAPTCHA, IP rotation, a spoofed user-agent or a forged Referer.

How the steps are used:

- **Hosted service, standalone.** Steps 2–5 feed the composer. Steps 7–10 only help the user find the cover: a small preview loaded by the browser from the source, the pixel size, and a plain link to the original so that a user who has the right can save it and drop the file (which is then step 4). Step 11 is optional and off by default.
- **Hosted service, reseller mode.** Steps 1 and 6 only. The lookup steps are not run: the reseller's feed is the source.
- **Command-line and self-hosted tool.** The operator may let the tool download bytes from steps 7, 9 and 10 by passing an explicit rights basis (doc 07, implication 4), for instance a publisher fetching its own covers. Without the argument the tool lists candidates and builds the typographic card.

### 1.3 Contradictions between the notes, and how they are resolved

| Topic | Sources note | Scraping-law note | Decision |
|---|---|---|---|
| Image proxy | "Plan for a small server-side fetch proxy" because CORS was unverified | "No image proxy": a proxy is a re-host (*Renckhoff*, *VCAST*) | No proxy. Doc 07 (implication 18) says the same. The need has also shrunk: Open Library sends `Access-Control-Allow-Origin: *` [V1]. |
| Open Library, Google, BnF and pasted URLs as cascade steps that feed the card | Steps 4–7 of its cascade | Preview by inline link at most; store and print rated High | Kept as **discovery** steps (7–10 above). Their output type cannot carry pixels into the composer. |
| Open Library "usable for print: technically yes" | Yes, with no licence on the images [M] | Print High: no licence, outside any display terms, too small | Not printable by the hosted service. |
| Open Library rate limit | nginx: 400 requests/min per IP [V1]; public documentation: 100 requests per 5 minutes for ISBN-keyed lookups [M] | 100 per 5 minutes [M, low] | Design for the stricter figure: at most one ISBN-keyed cover request every 3 s per IP. `notes/isbn-databases.open.md` adds a third figure for the JSON API (1 request/s) [M]. |
| BnF cover URL | [M] | not covered | [V1], from Inventaire's code, read twice (the ISBN note and this pass). |
| Opt-out by terms of use | not covered | "France is wider than the Directive" (CPI R122-28 [M]) | Recital 18 of the Directive already lists "terms and conditions of a website or a service" among machine-readable means [V1]. The rule "a terms-of-use clause is an opt-out" is applied EU-wide and no longer rests on R122-28, whose wording stays [M]. |
| Size of the cover in an EPUB | 1400–2560 px, "usually needs no upscaling" [M] | not covered | Kobo's guidelines give a store average of 800 x 1224 px [V1, undated text]. Plan for 800–2560 px; measure each file; do not assume print quality. |
| User-pasted product URL | Server-side extraction, "far lower risk than crawling by EAN" | Retailer page Medium–High; publisher page by inline link Low–Medium | Publisher pages only, subject to robots.txt, TDMRep and a denylist of retailer hosts. The result is shown as a link, not inlined (§7.3). |
| LCP and the cover | not covered | [M, low] | [V1]: the LCP specification forbids encrypting cover images. |

### 1.4 What "tools to scrape covers" means in this project

The brief asks for tools that scrape covers when absent. The lawful and useful scope is:

1. **Extractors that touch no third party**: EPUB cover extraction, flat-cover splitting, image sniffing, placeholder detection, print-quality rating. They run in the browser.
2. **Feed ingestion** for resellers: ONIX `SupportingResource` parsing and rights flags.
3. **Discovery adapters** for an allow-list of sources (Open Library, Google Books, BnF, DNB): they return candidates, never bytes, to the hosted composer.
4. **A page extractor** for a URL the user pastes (`og:image`, JSON-LD), behind robots.txt, TDMRep, a host denylist and SSRF protection.
5. **A compliant fetcher**: identification, rate limits, opt-out signals, evidence log.

There is **no generic crawler, no retailer adapter and no enumeration by ISBN range**. The same library serves the command-line tool, where an operator who holds the rights can download what the adapters find.

---

## 2. Source table

Terms are as of 2026-11 and, unless marked otherwise, from memory. Risk is given as preview / store / print (L = Low, M = Medium, H = High).

| Source | URL pattern | Typical resolution | Auth and limits | Terms | Risk |
|---|---|---|---|---|---|
| EPUB file (user's own, or reseller's) | none: manifest item with `properties="cover-image"` (EPUB 3) [V2]; `<meta name="cover">` (EPUB 2) [V2] | Kobo store average 800 x 1224 px [V1]; retail masters 1400–2560 px [M] | Local, in the browser. LCP leaves the cover unencrypted [V1] | The user's or reseller's own file | L / L if never uploaded / home L–M, pro H unless rights holder |
| Reseller ONIX or distributor feed | `SupportingResource`, List 158 code 01 `FrontCover`, 02 `BackCover`, 29 `FullCover`, 51 `CoverFlap` [V1] | Feed-dependent, often 1400 px or more [M] | Contract | "Offered for promotional use by supply chain partners" [V1\*]; no print right without a clause | L / L / M (L with a print clause) |
| Open Library Covers | `covers.openlibrary.org/b/isbn/{isbn}-L.jpg` [V2]; `/b/id/{coverid}-M.jpg`; no suffix = original upload [V1]; `/b/isbn/{isbn}.json` for dimensions [V1] | `-S` fits 116 x 58, `-M` 180 x 360, `-L` 500 x 500 px [V1]; originals vary [M] | Keyless. nginx: 400 requests/min per IP, burst 400 [V1]; documentation: 100 per 5 min for ISBN-keyed requests [M]. CORS `*` [V1] | No licence on the images (the data is CC0) [M]; a takedown list exists [V1] | L–M / M–H / H |
| Google Books | `books.google.com/books?jscmd=viewapi&bibkeys=ISBN:{isbn}` [V1]; API v1 `volumes?q=isbn:{isbn}` [M] with `imageLinks` [V1]; `books?id={vol}&printsec=frontcover&img=1&zoom=N` [V1] | `imageLinks` widths of about 80, 128, 300, 575, 800, 1280 px [V1]; larger sizes mainly for previewable titles [M] | Key recommended; 1,000 requests/day per project by default [M] | No durable caching, attribution and link back [M]; VuFind deliberately does not cache it [V1] | L–M / M–H / H |
| BnF catalogue | SRU by ISBN → ARK, then `catalogue.bnf.fr/couverture?appName=NE&idArk=ark:/12148/cb{id}&couverture=1` [V1] | Thumbnail [M, low] | Keyless; no published limit known [M] | Metadata under Licence Ouverte [V1\*]; that licence does not cover the images [M]; image terms unknown | M / H / H until the terms are read |
| DNB (Germany) | `portal.dnb.de/opac/mvb/cover?isbn={isbn}` [V1] | Unknown | Keyless | Images come from MVB (the path says so); reuse terms unknown | M / H / H until the terms are read |
| Publisher's own page | JSON-LD `image`, `og:image` | 300–1200 px typical [M] | Per-site CGU; TDMRep reservation on most Hachette Livre sites, and at Média-Participations [V1] | Per site | L–M / H / H |
| Publisher press area ("espace presse", "couverture HD") | Manual download | Print resolution [M] | Often an account | Press and bookseller use, per publisher [M] | As the publisher's terms state |
| Internet Archive scans | `covers.openlibrary.org/b/ia/{identifier}-L.jpg` → `archive.org/download/{id}/page/cover_w500_h500.jpg` [V1] | Scan resolution [M] | Keyless | A photograph of a library copy; same cover rights | as Open Library |
| LibraryThing | `covers.librarything.com/devkey/{KEY}/{size}/isbn/{isbn}` [V1] | Unknown | Developer key, about 1,000 requests/day [M, low] | Unclear [M, low] | Not used |
| ORB (France) | `api.base-orb.fr/v1/products?eans={ean}`, basic auth, `thumbnail` and `original` versions [V2] | "original" | Paid account [V1] | Contract | By contract only |
| VLB / MVB (Germany) | `api.vlb.de/api/v1/cover/{isbn}/{s,m,l}?access_token=` [V1] | Unknown | Paid token [V1] | Contract | By contract only |
| Electre, Dilicom (FEL, Hub), Tite-Live | Proprietary | Varies | Paid B2B [M] | Catalogue display for subscribers [M] | By contract only |
| Syndetics | `secure.syndetics.com/index.aspx?isbn={isbn}/{SC.GIF,MC.GIF,LC.JPG}&client={id}` [V2] | `LC` | Library subscription | Library catalogue display | Never |
| Content Cafe (Baker & Taylor) | `contentcafe2.btol.com/ContentCafe/Jacket.aspx?...&Type={S,M,L}&Value={isbn}` [V1] | `L` | Subscription [V1] | Library catalogue display | Never |
| Amazon legacy image URL | `images-na.ssl-images-amazon.com/images/P/{ISBN10}.01.LZZZZZZZ.jpg` [V1] | About 500 px [M] | Undocumented; needs an ISBN-10, which 979-10 ISBNs do not have [M] | Associates terms forbid offline use [M] | Never |
| Amazon product page; PA-API 5.0 and its successor, the Creators API [V1] | `"hiRes"`, `data-a-dynamic-image` [V1]; API `images.primary`, `images.variants` [V1] | Up to 1500–2560 px [M] | Anti-bot; Associates account with sales [M] | Images only to send traffic to Amazon, short cache, no print [M] | Never |
| bookcover-api (Goodreads-derived) | `bookcover.longitood.com/bookcover?isbn=` [V1] | Goodreads image [V1] | Free, HTTP 429 limits [V1] | Goodreads images are Amazon's | Never |
| French retailer pages and CDNs | Hints only, §2.6 | 475–600 px [M, low] | Per-site CGU; Fnac has strong anti-bot [M] | EU database makers; *parasitisme* exposure (§6.7–6.8) | M–H / H / H: never |
| Inventaire, Hardcover, BookWyrm | Their own APIs | User uploads | Varies | Not researched for images | Not used |

### 2.1 Open Library Covers

From the cover server ([code.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/code.py), [config.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/config.py)):

- **Routes**: `/{category}/{key}/{value}-{S|M|L}.jpg` and `/{category}/{key}/{value}.jpg`, category `b` for books, keys `isbn`, `id`, `olid`, `ia` and others [V1]. Hyphens in ISBNs are stripped server-side [V1].
- **Sizes** are bounding boxes (maximum width, maximum height): `S` (116, 58), `M` (180, 360), `L` (500, 500) [V1]. A portrait `-L` cover is at most 500 px tall.
- **No suffix returns the original upload** [V1]. This is the only way past 500 px; how large originals are depends on who uploaded them [M].
- **`?default=false` returns HTTP 404** instead of a placeholder [V2: Open Library source; hard-coded by [calibre](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/ebooks/metadata/sources/openlibrary.py) and [VuFind](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/OpenLibrary.php)]. `?default=<url>` redirects (303) only to vetted hosts since a security fix ([utils.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/utils.py)) [V1].
- **`/b/isbn/{isbn}.json`** redirects (302) to `/b/id/{coverid}.json`, whose record includes `width`, `height` and `source_url`; an unknown ISBN gets a 404 [V1]. Use it to learn the pixel size before any image request.
- **Redirects to archive.org** are common: `L` and original sizes of covers stored in the archive.org cluster, and every size of covers with an id of 8,000,000 or more once they have been archived, answer with a 302 [V1]. A client must follow redirects and allow `archive.org`.
- **Caching**: ISBN-keyed responses carry `Expires` +10 minutes; id-keyed responses "100 years" plus an ETag [V1]. Resolve the ISBN to a cover id once, then use `/b/id/`.
- **CORS**: the application adds `Access-Control-Allow-Origin: *`, methods GET, HEAD, OPTIONS, to every response, with or without an `Origin` header ([asgi_app.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/asgi_app.py)) [V1]. The JSON endpoints of `openlibrary.org` do the same [V1\*, `notes/isbn-databases.open.md`]. Browser code can therefore call the JSON endpoint and read image bytes served by `covers.openlibrary.org` directly. Whether archive.org sends the header on the redirect target is **not verified**.
- **Takedowns**: a `blocked_covers` list exists in the server configuration [V1], so removal requests do occur.
- **Other users**: BookWyrm reads the `covers` ids of an edition and builds `/b/id/{id}-L.jpg` ([connector](https://raw.githubusercontent.com/bookwyrm-social/bookwyrm/main/bookwyrm/connectors/openlibrary.py)) [V1]. An edition record already fetched by the catalogue lookup therefore gives the cover id with no extra request.

Rate limits and user-agent rules, from the deployed nginx configuration ([nginx.conf](https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/nginx.conf), [covers_nginx.conf](https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/covers_nginx.conf)):

- `cover_limit`: **400 requests per minute, burst 400, per client IP**, applied when there is no session cookie and the IP and user-agent are not allow-listed. Excess gets HTTP 429 with a JSON body asking to email `info@archive.org` [V1].
- Requests whose user-agent contains `CloudFront` or `Bytespider` are dropped (nginx 444) [V1]. Do not fetch Open Library covers through a CloudFront origin fetch.
- A user-agent containing "bot", "spider", "crawl", "google", "http" or "@" counts as self-identifying and stays out of the shared bucket for non-identifying crawlers [V1].
- The public documentation states 100 requests per IP per 5 minutes for ISBN-keyed lookups, with id and OLID lookups exempt [M]. The repository configuration is more generous. Design for the stricter figure.

Bulk: covers 0–7,139,999 are zips of 10,000 images in `olcovers{N}` items of the `ol_exports` collection on archive.org ([coverstore README](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/README.md)) [V1]. We do not mirror them (§6.7, §7).

### 2.2 Google Books

- Dynamic Links endpoint: `https://books.google.com/books?jscmd=viewapi&bibkeys=ISBN:{isbn13}&callback=x` returns JSONP with `thumbnail_url` ([VuFind Google.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Google.php)) [V1].
- Image by volume id: `https://books.google.com/books?id={volumeId}&printsec=frontcover&img=1&zoom=N` ([calibre google.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/ebooks/metadata/sources/google.py)) [V1]. VuFind rewrites `zoom=5` to `zoom=1`; calibre tries `zoom=0` then `zoom=1`: 5 is the smallest thumbnail, 0 and 1 are larger [V2].
- The API's `imageLinks` object has six keys: `smallThumbnail` (about 80 px wide), `thumbnail` (128), `small` (300), `medium` (575), `large` (800), `extraLarge` (1280) ([discovery document](https://raw.githubusercontent.com/googleapis/google-api-go-client/main/books/v1/books-api.json)) [V1]. That the larger keys are returned only by `volumes/{id}` and mostly for titles with a publisher preview is [M]. An `extraLarge` image would be about 296 ppi on a 110 mm panel [est.], but the terms do not allow that use.
- `&edge=curl` adds a page-curl effect; strip it [M].
- **Placeholder**: Google returns a dummy "image not available" picture, not a 404. calibre rejects two MD5 digests: `0de4383ebad0adad5eeb8975cd796657` and `a64fa89d7ebc97075c1d363fc5fea71f` [V1].
- **Terms**: VuFind enables local caching for Open Library, LibraryThing, Syndetics and Content Cafe but not for Google, and its configuration says "due to terms of service, some services will never have images cached" ([config.ini](https://raw.githubusercontent.com/vufind-org/vufind/dev/config/vufind/config.ini)) [V1]. Google's exact wording, the branding rules and the quota are [M].
- Results depend on the caller's country; server IPs can get location errors without a `country` parameter [M].

### 2.3 BnF and DNB

- **BnF**: resolve the ISBN to an ARK through the SRU API (`bib.isbn`), then send a HEAD request to `https://catalogue.bnf.fr/couverture?appName=NE&idArk=ark:/12148/cb{id}&couverture=1`. Inventaire accepts the URL when the status is 200 and the `Content-Length` is neither 4566 nor 4658, the two placeholder sizes it knows ([source](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/bnf/get_bnf_entry_from_isbn.ts)) [V1]. The returned size, the origin of the images (publisher deposit or a supplier) and their reuse terms are not verified. BnF metadata is under the Licence Ouverte ([BnF hackathon wiki](https://raw.githubusercontent.com/wiki/hackathonBnF/hackathon2016/Home.md)) [V1\*]; I believe the images are not [M, low]. Gallica holds public-domain scans only, so it does not help for in-print eBooks.
- **DNB**: `https://portal.dnb.de/opac/mvb/cover?isbn={isbn}`; calibre-dnb probes it with HEAD and treats an HTTP error as "no cover" ([calibre-dnb](https://raw.githubusercontent.com/citronalco/calibre-dnb/master/__init__.py)) [V1]. The path indicates that the images come from MVB, the German trade catalogue; reuse terms unknown.
- Both stay disabled in the hosted service until someone has read their terms (§8).

### 2.4 Amazon

- Legacy pattern still shipped in Koha: `https://images-na.ssl-images-amazon.com/images/P/{ISBN10}.01.{M|L}ZZZZZZZ.jpg` ([opac-detail.tt](https://raw.githubusercontent.com/Koha-Community/Koha/main/koha-tmpl/opac-tmpl/bootstrap/en/modules/opac-detail.tt)) [V1]. A missing image is a 1 x 1 GIF, not a 404 [M].
- Product pages: calibre extracts `"hiRes"`, `imageGalleryData[0].mainUrl` or the widest entry of `data-a-dynamic-image`, and strips the `._XX_.` size modifier ([amazon.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/ebooks/metadata/sources/amazon.py)) [V1].
- VuFind's `dev` branch no longer ships an Amazon cover provider ([Deprecated.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Deprecated.php), provider list in config.ini) [V1].
- Open Library's importer contains an `AmazonCreatorsAPI` class described as the "replacement for AmazonAPI (PA-API 5.0)", with OAuth 2.0 credentials, and maps `images.variants` to "alternate cover image URLs (back cover, spine, etc.)" ([vendors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/core/vendors.py)) [V1]. It also filters a known placeholder (`/01RmK+J4pJL.` in the image URL) [V1].
- Terms [M]: product images only through the official API, linked to Amazon, cached for a short window, no "offline" use including print; the API needs an Associates account with qualifying sales. Amazon data requires a link back ([LibraryThing blog](https://blog.librarything.com/category/covers/)) [V1\*]; Goodreads' own librarians were told that covers from Amazon or other booksellers could not legally be reused ([Goodreads thread](https://www.goodreads.com/topic/show/1277733-source-of-book-covers?page=1)) [V1\*].
- **Conclusion: Amazon is out**, including for spines, even though it is the only API seen that may return one.

### 2.5 Library and trade services

- **LibraryThing**: URL pattern in [VuFind](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/LibraryThing.php) [V1]. Developer key, daily cap, 1 x 1 GIF when absent, and whether keys are still issued: [M, low].
- **Syndetics**: pattern in Koha and [VuFind](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Syndetics.php) [V2]; files `SC.GIF`, `MC.GIF`, `LC.JPG`; `index.xml` lists the sizes that exist [V1].
- **Content Cafe**: "a subscription service" needing a user id and password ([ContentCafe.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/ContentCafe.php)) [V1].
- **VLB (MVB)**: `https://api.vlb.de/api/v1/cover/{isbn}/{s|m|l}?access_token=…` ([Buchhandel.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Buchhandel.php)) [V1].
- **ORB (France)**, `base-orb.fr`: REST `products?eans=` with basic auth, image versions `thumbnail` and `original` ([Orb.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Orb.php); also a provider in [Coce](https://raw.githubusercontent.com/fredericd/coce/master/README.md)) [V2]. Operator, price and terms not checked.
- **Koha** lists Amazon, Google, Open Library and Syndetics as cover sources, and Babelthèque, Novelist Select and LibraryThing for Libraries as other enriched content ([enhanced_content.pref](https://raw.githubusercontent.com/Koha-Community/Koha/main/koha-tmpl/intranet-tmpl/prog/en/modules/admin/preferences/enhanced_content.pref)) [V1].
- **Electre**, **Dilicom** (FEL notices; the Hub that carries eBook notices and cover links between French e-distributors and resellers) and **Tite-Live/ePagine**: all [M]. They are subscriber services. A reseller in mode 2 already receives covers through one of them, which is why reseller mode needs no lookup.

### 2.6 Publisher and bookseller pages

Extraction order for a page the user pasted (standard practice [M]): JSON-LD `Book` or `Product` `image` (string, array or `ImageObject.url`), then `og:image` with `og:image:width` and `og:image:height`, then `twitter:image`, then `link rel="image_src"`, then `itemprop="image"`. Then strip CDN resize hints to reach the original: WordPress `-300x450`, Shopify `_600x`, Cloudflare `/cdn-cgi/image/width=…/`, query `?w=&h=` [M]. Confirm that the page is about the right book by finding the ISBN in the page.

French and other bookseller patterns — **all [M, low], none checked live**. They are recorded here to build the denylist of retailer image hosts, not to write adapters:

| Site | Hint | Derivable from the EAN alone? |
|---|---|---|
| Decitre, Furet du Nord | `products-images.di-static.com/image/{slug}/{EAN}-475x500-1.jpg` | Probably not (slug) |
| leslibraires.fr, Place des Libraires, many independent shops (ePagine, Tite-Live) | `images.epagine.fr/{3 digits}/{EAN}_1_75.jpg` | Possibly |
| Cultura | `cdn.cultura.com/cdn-cgi/image/width=…/media/pim/TITELIVE/…{EAN}_1_75.jpg` | Possibly |
| Fnac | `static.fnac-static.com/multimedia/…/{internal id}/…` | No; strong anti-bot |
| IBS (Italy) | `www.ibs.it/images/{EAN}_0_536_0_75.jpg` | Possibly |
| Casa del Libro (Spain) | `imagessl{d}.casadellibro.com/a/l/t7/{dd}/{EAN}.jpg` | Possibly |

Publisher press areas sometimes offer a "couverture HD" download [M]. It is the only public place where print-resolution fronts are routinely found. It is per-publisher manual work, done by a person who then uploads the file under their own declaration.

### 2.7 Aggregators and prior art

- **Coce** (MIT): a Redis cache of cover URLs from Google Books, Amazon, Open Library and ORB, queried in a provider order chosen by the caller; it caches URLs by default, images optionally ([README](https://raw.githubusercontent.com/fredericd/coce/master/README.md)) [V1]. Good prior art for a URL-only cascade.
- **bookcover-api**: scrapes Goodreads and returns an `i.gr-assets.com` URL, rate-limited with 429 ([README](https://raw.githubusercontent.com/w3slley/bookcover-api/main/README.md)) [V1]. Not usable.
- **Inventaire** (French association, AGPL server, Wikidata-based) ([README](https://raw.githubusercontent.com/inventaire/inventaire/main/README.md)) [V1 for the licence]; its image coverage and API terms were not checked.

---

## 3. Resolution and upscaling

### 3.1 What the print needs

Default format: 110 x 178 mm front (French *poche*), 3 mm bleed, 10 mm spine (`notes/print-requirements.folding.md`). Printers ask for 300 dpi, with a 250 dpi floor at Saxoprint [V2\*]; the prepress note proposes preflight thresholds of 300 target, 250 warning, 150 error (`notes/print-requirements.prepress.md`).

| | 300 ppi | 250 ppi | 150 ppi |
|---|---|---|---|
| 110 x 178 mm panel | 1299 x 2102 px | 1083 x 1752 px | 650 x 1051 px |
| with 3 mm bleed (116 x 184 mm) | 1370 x 2173 px | 1142 x 1811 px | 685 x 1087 px |
| 148 x 210 mm (A5, professional print only) | 1748 x 2480 px | 1457 x 2067 px | 874 x 1240 px |

All [est.]. Doc 07 places the cover inset from the trim with a flat ground in the bleed, which lowers these figures by about 3 %.

### 3.2 What the sources deliver

| Source image, long side | ppi on a 178 mm front | ppi on a 210 mm front | Upscale to reach 300 / 250 / 150 ppi at 178 mm |
|---|---|---|---|
| 500 px: Open Library `-L` [V1]; Amazon legacy `L`, Decitre box [M] | 71 | 60 | 4.2x / 3.5x / 2.1x |
| 800 px: typical `og:image` [M] | 114 | 97 | 2.6x / 2.2x / 1.3x |
| 1200 px: good publisher site image [M] | 171 | 145 | 1.75x / 1.5x / none |
| 1224 px: average Kobo store cover, 800 x 1224 [V1] | 175 | 148 | 1.7x / 1.4x / none |
| 1500 px: Amazon `hiRes` [M] | 214 | 181 | 1.4x / 1.2x / none |
| about 2070 px: Google `extraLarge`, 1280 px wide [V1 for the width] | 295 | 250 | none |
| 2560 px: retail cover master [M] | 365 | 310 | none |

All ppi and factors are [est.]. Consequences:

1. Every public ISBN cover API result fails even the 150 ppi error threshold on its own.
2. A 2x model is not enough for 500 px inputs.
3. **The cover inside an EPUB is not automatically print-grade.** Kobo's guidelines say "the average size and dimensions for ebook covers in the Kobo store are 800:1224px" and that larger covers "will not improve the display" ([Kobo EPUB spec](https://raw.githubusercontent.com/kobolabs/epub-spec/master/README.md)) [V1]. The text is undated and probably several years old [M]; the sources note's 1400–2560 px figure is [M]. Until a sample is measured, assume 800–2560 px and let preflight decide per file.
4. Feed covers supplied by a publisher for print should be requested at 2102 px or more on the long side.
5. Actual sizes returned by live requests were not measured.

### 3.3 Reading the cover from an EPUB

1. Read `META-INF/container.xml`, open the package document.
2. EPUB 3: the manifest item whose `properties` contains `cover-image`. EPUB 3.3 says the property is recommended and that "in EPUB 3, the cover image must be identified using the cover-image property" ([EPUB 3.3](https://raw.githubusercontent.com/w3c/epub-specs/main/epub33/core/index.html)) [V1]; the LCP specification relies on the same property [V1] — together [V2].
3. EPUB 2: `<meta name="cover" content="{manifest id}"/>`, the form Kobo documents and reads for its thumbnails [V1]; EPUB 3.3 acknowledges it for EPUB 2 reading systems [V1].
4. Fallbacks [M]: `<guide><reference type="cover">` and the first image in that document; a manifest image whose id or path contains "cover"; the first image of the first spine document. Kobo notes that covers set through CSS `background-image` defeat automatic extraction [V1].
5. The spec's own examples use an SVG cover [V1], so SVG must be rasterised or refused with a clear message.
6. **DRM**: Readium LCP states that cover images "must not be encrypted", along with the navigation documents and the package document ([LCP spec](https://raw.githubusercontent.com/readium/lcp-specs/master/releases/lcp/latest.md)) [V1]. Reading the cover of an LCP-protected file therefore touches no protection measure. Adobe ADEPT and Kindle formats were not checked [M, low]. Rule: if `META-INF/encryption.xml` lists the cover resource, report "cover encrypted" and stop; never decrypt.
7. An EPUB may carry a TDM reservation in its package metadata (`tdm:reservation`) [V1]; Bookwire, Edigita, GEMS and Penguin Random House Spain put it in the files they distribute [V1]. See §8, question 9.

### 3.4 In-browser, image-only super-resolution

No language model is involved in any option below.

| Option | Licence (verified) | Model facts | Browser route | Fit for covers |
|---|---|---|---|---|
| Real-ESRGAN `realesr-general-x4v3` | BSD-3-Clause ([LICENSE](https://raw.githubusercontent.com/xinntao/Real-ESRGAN/master/LICENSE)) [V1] | "tiny small model", x4, also x1–x3, tunable denoise ([model zoo](https://raw.githubusercontent.com/xinntao/Real-ESRGAN/master/docs/model_zoo.md), [README](https://raw.githubusercontent.com/xinntao/Real-ESRGAN/master/README.md)) [V1]; about 5 MB [M] | Export to ONNX, run with ONNX Runtime Web | Best size-to-quality candidate |
| Real-ESRGAN `RealESRGAN_x4plus` | same | General x4, full RRDB network [V1]; about 64 MB [M] | ONNX; WebGPU realistically required | Strong but heavy; invents texture |
| `RealESRGAN_x4plus_anime_6B` | same | 6 RRDB blocks, "smaller network" [V1]; about 18 MB [M] | ONNX | Good on flat illustration covers |
| `RealESRNet_x4plus` | same | MSE loss, "over-smooth" [V1] | ONNX | Safer for type: no GAN artefacts |
| SwinIR | Apache-2.0 ([README](https://raw.githubusercontent.com/JingyunLiang/SwinIR/main/README.md), [LICENSE](https://raw.githubusercontent.com/JingyunLiang/SwinIR/main/LICENSE)) [V1] | Classical x4: 11.9 M parameters, 0.539 s per 256 px tile on an RTX 2080 Ti [V1] | ONNX; slow on wasm | Quality reference; too slow on CPU |
| Swin2SR | Apache-2.0 ([LICENSE](https://raw.githubusercontent.com/mv-lab/swin2sr/main/LICENSE)) [V1] | Has a compressed-input (JPEG) task and a lightweight x2 ([README](https://raw.githubusercontent.com/mv-lab/swin2sr/main/README.md)) [V1] | Listed in transformers.js, `image-to-image` pipeline, WebGPU option ([README](https://raw.githubusercontent.com/huggingface/transformers.js/main/README.md)) [V1] | Matches our input: small JPEGs |
| waifu2x (nunif) | MIT ([LICENSE](https://raw.githubusercontent.com/nagadomi/nunif/master/LICENSE)) [V1] | Art and photo models, 2x and 4x; ONNX files in releases ([README](https://raw.githubusercontent.com/nagadomi/nunif/master/README.md), [waifu2x](https://raw.githubusercontent.com/nagadomi/nunif/master/waifu2x/README.md)) [V1] | Already runs in the browser on onnxruntime-web ([unlimited:waifu2x](https://raw.githubusercontent.com/nagadomi/nunif/master/waifu2x/unlimited_waifu2x/README.md)) [V1] | Proven; the author calls wasm speed "very slow" [V1] |
| Real-CUGAN | MIT ([LICENSE](https://raw.githubusercontent.com/bilibili/ailab/main/Real-CUGAN/LICENSE), [README](https://raw.githubusercontent.com/bilibili/ailab/main/Real-CUGAN/README_EN.md)) [V1] | Anime-oriented | ONNX conversion needed [M] | Illustration covers only |
| UpscalerJS | MIT ([README](https://raw.githubusercontent.com/thekevinscott/UpscalerJS/main/README.md), [LICENSE](https://raw.githubusercontent.com/thekevinscott/UpscalerJS/main/LICENSE)) [V1] | TensorFlow.js; `esrgan-slim` 2x, 3x, 4x in the browser, 8x in Node only; built-in patch processing ([model README](https://raw.githubusercontent.com/thekevinscott/UpscalerJS/main/models/esrgan-slim/README.md)) [V1] | Turnkey JavaScript | Fastest to integrate; not Rust or wasm |
| Lanczos3 plus unsharp mask | n/a | Deterministic, a few kilobytes | Pure Rust in wasm | Mandatory baseline and fallback |

Notes:

- Runtime: ONNX Runtime is MIT ([LICENSE](https://raw.githubusercontent.com/microsoft/onnxruntime/main/LICENSE)) [V1]. nunif's README (note dated 2026-08) advises against fp16 weights on WebGPU because `shader-f16` support is limited, and reports fp16 on wasm as practically identical at half the download [V1]. Plan: fp32 for WebGPU, fp16 for the wasm fallback.
- Model sizes are [M]; none was downloaded. Timings on consumer hardware are unknown. Benchmark before promising anything in the interface.
- Licence traps [M]: community models are often CC BY-NC; the training sets behind the official weights (DIV2K, Flickr2K, OST) carry research-oriented terms although code and weights are BSD or Apache. If the project trains its own model, train on covers it has rights to or on synthetic covers (see `08-legal-fonts-and-genai.md`).
- Covers are type-heavy. GAN upscalers invent plausible but wrong glyph shapes on small text [M].

### 3.5 Decisions

1. **Resampling is not print quality.** Preflight rates the **native** resolution and records the upscale factor separately. An upscaled 500 px image is reported as "71 ppi native, upscaled 4x", never as 300 ppi.
2. **Default is Lanczos3** to the target size. It is plain resampling, which every print workflow does.
3. **AI super-resolution is opt-in**, in standalone mode, with a before-and-after view at print scale. The default model is PSNR-oriented or a small general model; GAN variants are a second choice the user must pick. Cap at 4x.
4. **Generative upscaling may count as altering the work** (doc 07: covers are never modified; only uniform scaling and ICC conversion). This is my reading, not a sourced rule. Until counsel says otherwise, AI upscaling is disabled for licensed covers in reseller mode. Licensed covers should arrive at print size anyway.
5. Professional print needs at least 150 ppi native, and warns below 250. Home print is allowed at any resolution with a visible quality label.
6. The title is never recovered by upscaling. Where a title must be legible on our own panels, it is re-typeset.

---

## 4. Placeholder detection heuristics

| Layer | Check | Evidence |
|---|---|---|
| Protocol | Open Library: always send `?default=false`; 404 = absent | [V2] |
| Protocol | BnF: HEAD request; status 200 and `Content-Length` not 4566 or 4658 | [V1] |
| Protocol | DNB: HEAD request; an HTTP error = absent | [V1] |
| Protocol | Syndetics: read `index.xml` for the sizes that exist | [V1] |
| Bytes | Hash blocklist: the two Google dummy images (MD5 digests in §2.2) | [V1] |
| Bytes | Known asset URLs: Amazon `…/images/G/01/x-site/icons/no-img-sm.gif` (referenced by Koha); `/01RmK+J4pJL.` in an Amazon image URL (filtered by Open Library) | [V1] |
| Bytes | 1 x 1 GIF or a body under about 100 bytes (Amazon legacy, LibraryThing, Content Cafe with `erroroverride=1`) | [M] |
| Bytes | The response is not an image: check magic bytes, not the `Content-Type` header alone | [M] |
| Pixels | Reject when either side is under 100 px; when width/height is outside about 0.55–0.85; when the luminance standard deviation is near zero; when fewer than about 16 distinct colours remain after quantisation | my heuristics [M] |
| Pixels | Perceptual hash (dHash or pHash) against a gallery of collected placeholders per provider, with a Hamming-distance threshold | standard practice [M] |
| Semantics | 3D packshots and photographs: uniform white corners plus non-rectangular content; a promotional "bandeau" across the cover; retailer watermarks | my heuristics [M] |
| Edition | The ISBN on the page or record equals the requested one, or a known sibling (print ISBN for an eBook); otherwise flag "other edition" | [M] |
| Cross-check | Two providers returning near-identical perceptual hashes raises confidence that the edition is right | [M] |

Design notes:

- Keep the blocklist as test fixtures: hashes and byte lengths only, never the images.
- The pixel heuristics apply to files the user supplies and to transient analysis. They cannot apply to an inline preview, whose pixels the page does not read (§7.3). For previews, the protocol and length checks must be enough.
- Two strategies exist: query every provider, discard dummies and keep the largest (calibre, as I remember it [M]); or query in a fixed order and stop at the first hit (Coce [V1]). The hosted service uses the second, because each user action should cost the sources as few requests as possible. The command-line tool may offer the first behind a flag.
- A lookup on an eBook ISBN often misses where the print ISBN hits [M] (`notes/isbn-databases.open.md`). In reseller mode take the print ISBN from the ONIX `RelatedProduct` block; in standalone mode let the user confirm the match.

---

## 5. Spine and wraparound availability

| Candidate source | Finding | Confidence |
|---|---|---|
| ONIX 3.0 List 158 | 56 codes read. Cover-related: 01 `FrontCover`, 02 `BackCover`, 03 `CoverPack`, 27 `CoverThumbnail`, **29 `FullCover`**, 45 `CoverHoldingImage`, **51 `CoverFlap`**, 54 `PageEdgeDecoImage`, 55 `EndpaperDecoImage`. There is **no spine-only code** ([im_onix codelist](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-158.yml)). This answers an open point of doc 07 | [V1] |
| French feeds (Dilicom, Electre, e-distributors) | Front cover, sometimes back cover ("4e de couverture"); no feed known to carry code 29 | [M, low] |
| Publisher print files ("couverture à plat": plat 1 + dos + plat 4) | Exist for every printed book; held by publisher and printer; not public | [M] |
| Self-publishing and print-on-demand authors (KDP, BoD, Lulu, Bookelis) | The author owns the wraparound PDF: an upload path gives a real spine | [M] |
| Publisher press kits | HD front common; flat full cover rare | [M, low] |
| Amazon Creators API | `images.variants` described as "alternate cover image URLs (back cover, spine, etc.)" by Open Library's importer | [V1]; barred by terms [M] |
| Internet Archive scans | Front and often back photographed; spine not systematically captured | [M, low] |
| BnF Gallica | Binding views ("plat supérieur", "dos", "plat inférieur") for heritage items; public domain only | [M] |
| Retailer "3D" packshots | Synthesised from the front with a generic spine; not real spine data | [M] |
| Second-hand marketplaces | Real photos with spines; per listing, unstructured, the seller's copyright on top of the cover's | [M] |
| Research datasets | Shelf and spine sets for detection and OCR; small, not keyed by ISBN | [M] |
| Cover datasets | Amazon-derived cover corpora at about 224 px; fronts only | [M] |

Conclusions:

1. **There is nothing to scrape.** The only API that may return a spine is Amazon's, which we cannot use. "Finding" a spine otherwise means shelf photographs or printer files: routes rated High, with a photographer's right on top.
2. Many eBooks are digital-only and never had a spine.
3. **A real spine rarely fits.** Our spine panel is 10 mm wide and as tall as the front. A supplied spine scaled uniformly to the panel height keeps its own proportions: a *poche* spine fits only if the book's spine was 10 mm or less (roughly 150–200 pages [M]); a 20 mm spine of a 140 x 205 mm book becomes 17.4 mm at 178 mm height and does not fit [est.]. Re-fitting it is an adaptation that needs consent (doc 07, implication 13).
4. So: **generate the spine by default** (flat colour, title and author retyped; doc 07, implications 11–12). Accept an optional flat-cover upload from a rights holder; use its spine only when it fits.
5. Shelf datasets are useful only to learn conventions for the generator: order of title, author and publisher mark; colour continuity with the front.
6. Reading direction: French and most continental spines read bottom-to-top, English-language spines top-to-bottom [M]. Make it a per-locale default with an override.
7. A feed that does carry code 29 or 51 should be ingested: it is the clean way for a publisher to supply a real spine.

---

## 6. Legal analysis of acquisition

Everything in this section is [M] unless marked. It is a map for French IP counsel, not advice. Authorities are cited by identifier because no legal database could be opened. Printing is analysed in doc 07.

### 6.1 Which act engages which right

| Step in a "find the missing cover" pipeline | Right or rule engaged | Usable basis |
|---|---|---|
| Fetch and parse an HTML page or API response to find an image URL | Reproduction of the page (InfoSoc art. 2); extraction from a database (Dir. 96/9 art. 7); contract (terms of use) | TDM art. 4 if access is lawful and no opt-out; insubstantial-part rule for databases; API licence |
| Keep the URL, ISBN, dimensions, hash | None: facts are not protected (the database right can apply at scale) | — |
| Fetch the image bytes on our server | Reproduction of the cover | Licence, or TDM (temporary, analysis only) |
| Keep the bytes (cache, CDN) | Continuing reproduction | Licence only; TDM copies must go |
| Show the image from our server | Communication to the public (InfoSoc art. 3): *Renckhoff* | Licence only |
| Show it by inline link to the source server | No new public if the source is authorised and unprotected: *Svensson* line | Case law, plus the source's terms |
| The user's browser renders and caches it | Temporary copy, InfoSoc art. 5(1) (*PRCA v NLA*, C-360/13) | Statutory exception |
| Embed in the PDF, print | Reproduction on a new medium; distribution if we ship | Licence, or the user's own private copy (doc 07) |

### 6.2 robots.txt and RFC 9309

**The standard.** The Robots Exclusion Protocol is specified in RFC 9309 ([TDMRep primer](https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/docs/robots.md); Google's parser [README](https://raw.githubusercontent.com/google/robotstxt/master/README.md)) [V2], published by the IETF in September 2022 [V1, the primer]. Points that matter for a compliant fetcher:

- The rules "are not a form of access authorization" and the protocol "is not a substitute for valid content security measures" [M].
- Longest match wins; on a tie `Allow` wins; user-agent matching is case-insensitive [M].
- A 4xx on `/robots.txt` means no restrictions, except 429, which means retry later; a 5xx means do not crawl until it is fixed [V1, a parser's README restating Google's guidance: [texting_robots](https://raw.githubusercontent.com/Smerity/texting_robots/main/README.md)]. The RFC says the same in its own words [M].
- Parse at least the first 500 KiB [V1 for the 500 kibibyte figure as Google's recommendation, same README]; follow at least five redirects; do not use a cached copy for more than 24 hours unless the file is unreachable [M].

**Legal weight in the EU.**

| Question | Answer | Confidence |
|---|---|---|
| Does "no Disallow" grant permission to copy or display? | No. France: assignments must be express and are read narrowly (CPI L131-3). Belgium: *Copiepresse v Google* (CA Bruxelles, 5 May 2011) rejected an opt-out theory of copyright. EU: implied consent needs prior, effective, individual information of the author (*Soulier and Doke*, C-301/15). | [M] |
| German exception | BGH *Vorschaubilder I* (I ZR 69/08, 29 Apr 2010): putting images online without blocking crawlers is "schlichte Einwilligung" to the usual thumbnail display by search engines. Limited to search engines; not transferable to product imagery. | [M] |
| Does a `Disallow` bind a crawler as such? | No EU statute makes robots.txt mandatory. | [M] |
| Is it an opt-out from TDM? | Yes: the paradigm machine-readable means of DSM art. 4(3). The GPAI Code of Practice (10 July 2025, under AI Act art. 53(1)(c)) commits signatories to crawlers that follow robots.txt as specified in RFC 9309. | [M] |
| German and Dutch case law | LG Hamburg, 27 Sep 2024, 310 O 227/23 (*Kneschke v LAION*): obiter, a natural-language reservation may count as machine-readable. OLG Hamburg, 5 U 104/24, 10 Dec 2025: as recalled, dismissal upheld and the natural-language reservation held not machine-readable at the 2021 copying date. Rb. Amsterdam, 30 Oct 2024 (*DPG Media v HowardsHome*): a robots.txt naming only some bots was not an effective reservation against another bot. | [M, low] |
| Is robots.txt an "effective technological measure" (InfoSoc art. 6), so that ignoring it is circumvention under *VG Bild-Kunst*? | Probably not: it restricts nothing technically. My analysis; no authority known. | [M] |
| Other weight | Ignoring it is evidence of bad faith for *parasitisme* and removes the "reasonable expectations" limb of GDPR legitimate interest (CNIL, §6.8). | [M] |

### 6.3 The text-and-data-mining exception

**Text.** DSM Directive art. 4, as extracted in the W3C TDMRep group's repository ([extracts](https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/docs/eu-act-extracts.md)) [V1]:

- (1) an exception "for reproductions and extractions of lawfully accessible works and other subject matter for the purposes of text and data mining";
- (2) copies "may be retained for as long as is necessary for the purposes of text and data mining";
- (3) it applies "on condition that the use … has not been expressly reserved by their rightsholders in an appropriate manner, such as machine-readable means in the case of content made publicly available online".
- Recital 18: for content publicly available online, "it should only be considered appropriate to reserve those rights by the use of machine-readable means, **including metadata and terms and conditions of a website or a service**"; and "rightsholders should be able to apply measures to ensure that their reservations in this regard are respected".

Whether terms written in natural language count as "machine-readable" is disputed (the Hamburg decisions in §6.2 [M, low]). We take the cautious reading: any reservation in a site's terms is treated as effective, in every Member State.

| | DSM art. 3 / CPI L122-5-3 II | DSM art. 4 / CPI L122-5-3 III |
|---|---|---|
| Beneficiary | Research organisations, cultural heritage institutions | Anyone, any purpose including commercial |
| Condition | Lawful access; scientific research | Lawful access **and** no reservation by the rightholder |
| Opt-out | None; contract cannot override (art. 7(1)) | Yes (art. 4(3)); in France by any means (R122-28 [M]) |
| Retention | Secure storage; may keep for verification | Only as long as needed for the mining; French text: copies "détruites à l'issue de la fouille" [M] |

France transposed by ordonnance n° 2021-1518 of 24 Nov 2021; the opt-out provision is CPI R122-28 (décret n° 2022-928 of 23 June 2022), which as recalled lets the opposition be expressed "par tout moyen", including a site's terms of use [M]. Both sibling legal documents cite the same article number; none verified it.

**Opt-out signals in use.** The W3C Community Group protocol TDMRep defines `tdm-reservation` (1 = rights reserved, 0 = not reserved) and an optional `tdm-policy` URL, carried by four techniques ([README](https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/README.md), [draft spec](https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/spec/index.html)) [V1]:

- a site-wide file `/.well-known/tdmrep.json`: an array of rules with `location`, `tdm-reservation`, `tdm-policy`; patterns use `*` and `$` as in robots.txt; the most specific match is the first in sequence;
- HTTP response headers `tdm-reservation` and `tdm-policy`, described as the preferred technique;
- HTML `<meta name="tdm-reservation" content="1">`. This covers the HTML document only, **not the images it references** ([technique note](https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/techniques/technique-html-meta.md)) [V1], so an image's own response headers and its origin's `tdmrep.json` must be checked separately;
- metadata inside EPUB and PDF files (`tdm:reservation`).
- Precedence: check the file first; a header supersedes the file; HTML or in-file metadata supersedes both; the absence of a value at a later step does not reset an earlier one [V1].

**Adoption.** The group's adopters list names, among trade publishers, **Hachette Livre** ("on most websites of the group's publishing houses": Hachette.fr, Livre de Poche, Grasset, Fayard, Larousse and others) and **Média-Participations** ("in its different publishing houses, including La Martinière, Seuil, Fleurus"; the technique is not stated), plus Mondadori, Aufbau and Diogenes; among platforms, Cairn, Bookwire and Edigita ([adopters](https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/docs/adopters.md)) [V1]. That the SNE recommends it to French publishers is [M, low]. Other signals a fetcher should read: `X-Robots-Tag` and meta `noai`, `noimageai` (non-standard); IPTC "Data Mining" photo metadata; the IETF "AI Preferences" work [M, low].

**What TDM allows us to do** (my application): fetch a cover from a lawfully accessible location, analyse it automatically to generate information — dominant colours, title bounding box, typeface class, a perceptual hash to check that a user-supplied image matches the ISBN — then discard the copy. The derived facts are not protected and may be stored and used to build our own typographic card. Training a font-identification model on such images is also TDM, under the same opt-outs.

**What TDM does not allow:**

- Showing the fetched image to anyone: communication to the public is outside arts. 3–4.
- Keeping it as a cache or asset library.
- Printing it or embedding it in the PDF.
- Output that reproduces the expression: LG München I, 11 Nov 2025, 42 O 14139/24 (*GEMA v OpenAI*) held that memorised and regurgitated works fall outside the exception [M, low]. Generative "outpainting" of a cover onto the spine is therefore not sheltered.
- Ignoring robots.txt, TDMRep or a terms-of-use clause.
- Bypassing a paywall or login: access would not be "lawful".

**Practical effect.** For the websites of Hachette Livre's houses at least, the TDM route is closed [V1 for the reservation]; expect the same wherever a French publisher follows them. The route remains open only where no reservation is expressed, which must be checked per request.

### 6.4 Linking versus copying

| Case | Holding | Confidence |
|---|---|---|
| *Svensson*, C-466/12, 13 Feb 2014 | A clickable link to a work freely available with the rightholder's consent reaches no new public. Different if the link bypasses access restrictions. | [M] |
| *BestWater*, C-348/13, order of 21 Oct 2014 | Same for framing and embedding. | [M] |
| *GS Media*, C-160/15, 8 Sep 2016 | A link to an unauthorised copy infringes if the linker knew or should have known; **knowledge is presumed when the link is posted for profit**. | [M] |
| *Renckhoff*, C-161/17, 7 Aug 2018 | Copying a freely available photograph and posting it on another site is a new communication to a new public. Sources: [IRIS Merlin](https://merlin.obs.coe.int/article/8373), [Columbia GFoE](https://globalfreedomofexpression.columbia.edu/cases/land-nordrhein-westfalen-v-dirk-renckhoff/), [Mondaq](https://www.mondaq.com/uk/copyright/730320/the-risks-of-reposting-online-content-cjeu-rules-on-the-unauthorised-reposting-of-copyright-protected-works-on-the-internet). | [V2\*] |
| *VG Bild-Kunst*, C-392/19, 9 Mar 2021 | Framing that circumvents effective technological measures against framing is a communication to a new public. The rightholder can limit its consent only through such measures, not by contract alone. | [M] |

Reading for covers:

- **Inline display from the source server is the only unlicensed display route with CJEU support**, and only if (i) the source posted the cover with the rightholder's consent — a publisher's own site yes, a retailer under licence yes, an unknown blog no, and as a commercial service we carry the *GS Media* presumption; (ii) the source uses no hotlink protection (Referer checks, signed or expiring URLs, tokens). If it does, stop.
- Copyright is not the only constraint on hotlinking: the source's terms, bandwidth free-riding as *parasitisme*, and data protection — the visitor's IP address goes to the third-party host (compare LG München I, 20 Jan 2022, 3 O 17493/20, Google Fonts).
- France has treated image-search thumbnails as licensable: CPI L136-1 to L136-4 set up collective management for automated image-referencing services [M, low]. There is no French thumbnail exception.
- **A server-side image proxy is a re-host.** Working around a missing CORS header with our own proxy means we fetch, copy and transmit a file we selected: *Renckhoff* and *VCAST* territory, with no "mere conduit" shelter under DSA art. 4 because we select the content (my application; *VCAST* [V2\*] via [Kluwer](https://legalblogs.wolterskluwer.com/copyright-blog/up-in-the-cloud-some-reflections-on-the-cjeu-judgment-in-vcast/)).

### 6.5 Terms of use — *Ryanair v PR Aviation*

- C-30/14, 15 Jan 2015: Directive 96/9 does not apply to a database protected neither by copyright nor by the sui generis right, so its mandatory user rights do not stop the owner of such a database from restricting use **by contract**.
- The paradox: for a protected database the lawful user may extract insubstantial parts for any purpose and contrary clauses are void (Dir. art. 8(1), art. 15; CPI L342-3); for an unprotected one the terms can forbid everything.
- Whether the terms bind is national contract law. In France general conditions bind only if brought to the other party's knowledge and accepted (C. civ. art. 1119): a footer link with no acceptance step is weak; a logged-in account or an API key is strong.
- France, same claimant: Cass. com., 10 Feb 2015, n° 12-26.023 (*Ryanair v Opodo*) rejected the database claim because the investment went into creating the data [M, low]. Germany: BGH, 30 Apr 2014, I ZR 224/12: screen-scraping in breach of ticked terms is not unfair hindrance unless a technical barrier is overcome.
- **API terms are contracts we sign.** As recalled [M, low]: Open Library Covers — display on public pages, no bulk download, courtesy link back, no rights warranty; Google Books — thumbnails with attribution and link, limits on caching; Amazon — images only to send traffic to Amazon. None grants print rights, and none can grant more than the provider holds.

### 6.6 What the terms route means for our sources

| Source | Acceptance step | Binding strength (my assessment) |
|---|---|---|
| Open Library, anonymous | None | Weak contract; US operator without EU database right. Follow its documented rules anyway: they are also the rate-limit and good-faith evidence |
| Google Books with an API key | Key creation accepts the API terms | Strong |
| BnF, DNB catalogue pages | None | Unknown terms; public bodies |
| Retailer sites | Footer CGU; often an account | Weak to strong, but database right and *parasitisme* apply regardless |

### 6.7 Sui generis database right

**Rule.** Dir. 96/9 art. 7 (France: CPI L341-1 to L343-7): the maker who made a substantial investment in obtaining, verifying or presenting the contents may prohibit extraction or re-utilisation of all or a substantial part, and the **repeated and systematic** extraction of insubstantial parts that conflicts with normal exploitation (art. 7(5); CPI L342-2). Term 15 years, renewed by new investment. Criminal penalties exist in France (L343-4). Only EU-based makers qualify (art. 11): the Internet Archive and Google LLC do not; Fnac Darty, Decitre, Babelio, Electre, Dilicom, MVB and Amazon's EU entities can.

| Case | Holding | Meaning for cover lookup |
|---|---|---|
| *BHB v William Hill*, C-203/02, 9 Nov 2004 | Investment in creating data does not count; only investment in collecting, checking and presenting it. Art. 7(5) targets cumulative reconstitution of a substantial part. | A publisher gets no database right in its own covers by producing them. Aggregators and retailers that collect from many publishers qualify. |
| *Directmedia*, C-304/07, 9 Oct 2008 | Extraction needs no technical copy; transfer after consultation suffices. | Manual or semi-manual harvesting is still extraction. |
| *Football Dataco v Sportradar*, C-173/11, 18 Oct 2012 | Re-utilisation is located where the targeted public is. | A service aimed at France is judged in France. |
| *Innoweb v Wegener*, C-202/12, 19 Dec 2013 | A dedicated meta search engine that forwards user queries in real time to a database and shows the results in its own interface re-utilises the whole database. | A real-time ISBN-to-cover relay over a retailer's catalogue has the same structure. |
| *CV-Online Latvia v Melons*, C-762/19, 3 Jun 2021 | Indexing and copying a freely accessible database is prohibited if it risks the maker's ability to recoup its investment; courts balance this against competitors' and users' interests. | Link-back and no substitution lower the risk; they do not remove it. |
| *Leboncoin v Entreparticuliers*, CA Paris, 2 Feb 2021; Cass. 1re civ., 5 Oct 2022, n° 21-16.307 | A classified-ads site is a database maker; scraping its listings was extraction of a qualitatively substantial part; damages upheld. | French courts do enforce this against scrapers. |

Public bodies: BnF bibliographic metadata is under the Licence Ouverte [V1\*], and the Open Data Directive 2019/1024 bars public bodies from using the database right to block re-use. That covers metadata, not cover images, which remain third-party works [M, low].

### 6.8 CNIL, unfair competition, criminal law

- **CNIL.** Guidance of 30 April 2020 on re-using publicly accessible data: public data remain personal data; check the source's terms. AI how-to sheets finalised 19 June 2025, including one on legitimate interest and *moissonnage*: define collection criteria in advance, delete irrelevant data at once, and **do not collect from sites that object by robots.txt or CAPTCHA** [M, low]. Sanctions: Clearview AI, EUR 20 million (2022); KASPR, EUR 240,000 (2024) [M, low]. Relevance to covers is limited: author names are personal data; retailer pages carry customer reviews, which we must not ingest; some covers show identifiable persons. The institutional takeaway is that the French regulator treats robots.txt as binding on a good-faith scraper.
- **Parasitisme** (C. civ. art. 1240): placing oneself in another's wake to profit, without paying, from its efforts or investment; no competition relationship or risk of confusion needed. Cass. com., 26 June 2024, n° 23-13.535 tightened proof: an identified, individualised economic value and intent to ride on it [M, low]. It fills gaps left by IP: a cover held non-original still attracted a *parasitisme* award (CA Versailles, 22 Mar 2022, *Zulma v Elior* — [Fournol](https://www.fournol-avocat.fr/actualite/2022/8/18/parasitisme-reconnu-pour-la-couverture-dun-livre)) [V1\*]; systematic reuse of auction catalogues led to infringement plus parasitic competition against Artprice ([UGGC](https://www.uggc.com/condamnation-dartprice-com-pour-contrefacon-et-concurrence-parasitaire/)) [V1\*]. Lifting a retailer's catalogue imagery, which it paid to license, normalise and host, to power a paid gift-card product is the textbook pattern, even where copyright and database right both fail.
- **Criminal law.** Code pénal art. 323-1 (fraudulent access to or remaining in an automated data-processing system) and 323-3 (fraudulent extraction of data). Cass. crim., 20 May 2015, n° 14-81.336 ("Bluetouff") upheld a conviction for remaining in, and taking files from, an extranet reachable through a search engine once the defendant knew access was meant to be restricted. Public pages are outside this; logging in under false pretences, defeating CAPTCHAs or rotating IPs after a block moves towards it.

### 6.9 United States, for comparison

| Topic | Position | Confidence |
|---|---|---|
| Scraping public pages | *hiQ Labs v LinkedIn* (9th Cir. 2019, reaffirmed 2022): not "without authorization" under the CFAA. On remand hiQ was held in breach of LinkedIn's User Agreement and accepted a USD 500,000 judgment and injunction. | [M] |
| Contract and preemption | *Meta v Bright Data* (N.D. Cal., Jan 2024): terms did not reach logged-out scraping. *X Corp v Bright Data* (May 2024): state-law claims over public data preempted by the Copyright Act. | [M, low] |
| Database right | None (*Feist v Rural*, 1991). | [M] |
| Thumbnails | *Kelly v Arriba Soft* and *Perfect 10 v Amazon*: search thumbnails are transformative fair use — [opinion](https://m.openjurist.org/336/f3d/811), [CRS RL33810](https://www.everycrsreport.com/reports/RL33810.html). | [V2\*] |
| Inline linking | *Perfect 10* "server test": no display by the site that only links; reaffirmed in *Hunley v Instagram* (9th Cir. 2023), rejected in *Goldman v Breitbart* (S.D.N.Y. 2018). | [M] |
| Limits | *Warhol v Goldsmith* (2023): same-purpose commercial use weighs against fair use. A full-size printed cover is not a thumbnail. | [M] |

This explains why US-hosted services (Open Library, Goodreads, LibraryThing) show covers freely. It gives a service aimed at France nothing: the applicable law follows the country of protection (Rome II art. 8) and jurisdiction follows accessibility (*Pinckney*, C-170/12; *Hejduk*, C-441/13) [M].

### 6.10 Risk by acquisition route

Exposure of the service, France first. My assessment (taken from the scraping-law note), not a sourced rating.

| # | Route | (a) Locate and preview | (b) Store | (c) Print |
|---|---|---|---|---|
| 1 | Rightholder upload, or publisher or reseller feed **with an explicit print clause** | Low | Low | Low |
| 2 | Reseller's ONIX feed cover, "promotional use" licence only | Low | Low (inside the reseller's licence, as its processor) | Medium until a print clause exists (CPI L131-3) |
| 3 | User's own file: cover extracted **in the browser** from the user's EPUB, or own scan | Low | Low if never uploaded; Medium if we host it (DSA notice duties) | Home print: Low–Medium (the user's private copy). Pro print: High unless the user is the rightholder |
| 4 | Inline link to the publisher's own page image, with link back, no technical protection, robots and TDMRep clean | Low–Medium | High (*Renckhoff*): keep the URL and derived facts only | High through us |
| 5 | Cover API with display terms (Open Library, Google Books), used as the terms say | Low–Medium (third-party rights not cleared; IP address goes to the host) | Medium–High (caching limits; *Renckhoff*) | High (outside the terms, no licence, too small anyway) |
| 6 | On-demand scrape of a retailer or aggregator page (Amazon, Fnac, Decitre, Babelio, Goodreads) | Medium–High (terms, L342-2, *Innoweb*, *parasitisme*) | High | High |
| 7 | Bulk crawl or mirror of a cover catalogue | High | High | High |
| 8 | Image-search results of unknown provenance | Medium–High (*GS Media* presumption) | High | High |
| 9 | TDM-only fetch: derive palette, font class, hash, then discard | No display. Low–Medium if access is lawful and no opt-out | Image: must go. Derived facts: Low | Derived colour and font choice on our own spine: Low (watch series look-alikes) |
| 10 | Public-domain cover (artist dead more than 70 years) from an open-licence host | Low | Low | Low (check the host's terms; moral rights against distortion) |
| 11 | Any route that defeats a block (login, CAPTCHA, IP rotation, hotlink protection, spoofed user-agent) | High | High | High |

Opt-out and takedown lists are mitigation only: after *Soulier and Doke*, an offer to opt out does not legalise an unlicensed use.

---

## 7. Scraper design rules

### 7.1 Scope

1. **Allow-list, not open web.** Each source has a recorded basis (contract, open licence, documented public API, or statutory exception) and the date its terms were last read. A source whose review is older than 180 days (proposal) is disabled until re-read.
2. **Hosts never contacted**: retailer and aggregator sites and their image CDNs (list seeded from §2.4 and §2.6: Amazon, Goodreads, Fnac, Decitre, Furet du Nord, Cultura, ePagine/leslibraires.fr, Babelio, IBS, Casa del Libro), image search engines, any host that answered with a block.
3. **On demand, no enumeration.** One ISBN per user action. No background crawling, no warm-up of a cover cache, no relay of a retailer's search.
4. **Never defeat a barrier.** Stop on 401, 403, CAPTCHA or hotlink protection. No retry through another IP, no forged Referer, no browser impersonation.

### 7.2 Identification, rate, caching

| Rule | Default | Basis |
|---|---|---|
| User-agent | Fixed string with product name, version, a URL describing the bot and a contact address, for example `eBookCadeauBot/1.0 (+https://{site}/bot; contact@{site})`. Never contains "CloudFront" | Open Library treats "bot", "http", "@" as self-identifying and drops "CloudFront" [V1] |
| Open Library, ISBN-keyed cover requests | At most 1 every 3 s per IP (100 per 5 minutes) | Documentation [M]; hard limit 400/min [V1] |
| Open Library, id-keyed images and JSON | At most 1 per second | Documentation figure [M]; nginx 180/min on JSON [V1\*] |
| Google Books API | At most 1 per second; daily cap 1,000 per project | [M] |
| BnF, DNB | At most 1 every 2 s | No published limit known [M] |
| Any other host (pasted URL) | At most 3 requests per user action (robots.txt, tdmrep.json, the page); at least 5 s between actions on the same host | proposal |
| `Crawl-delay` in robots.txt | Honoured when longer than our default | Non-standard but polite |
| HTTP 429 | Honour `Retry-After`; otherwise back off 60 s, doubling, three attempts; then stop for 24 h | [V1 for the 429 handling advice] |
| Timeouts and sizes | 5 s connect, 10 s total; HTML 2 MiB; image 15 MiB; 5 redirects; HTTPS only | proposal |
| robots.txt and tdmrep.json cache | 24 hours per origin | RFC 9309 [M] |
| Candidate facts (URL, id, dimensions) | 7 days; negative results 24 hours | proposal |
| Google Books results | Not kept beyond the session | Terms [M]; VuFind's practice [V1] |
| Image bytes on our servers | None in the hosted service, outside licensed feed covers (doc 07, implication 26) | §6.4 |

### 7.3 Where each request runs

| Request | Runs in | Why |
|---|---|---|
| Open Library JSON (`/b/isbn/{isbn}.json`, edition records) | The user's browser | CORS `*` [V1]; the rate limit then applies to the user's IP, and our server relays nothing |
| Preview image from an allow-listed API host | The user's browser, as an inline link (`<img>`), without reading the pixels | *Svensson* line; doc 07, implication 19 |
| Google Books, BnF SRU, DNB metadata | Browser where CORS allows, else a server relay that returns **text metadata only** | Metadata is not the cover |
| The HTML of a pasted publisher page | Our server (browsers cannot read cross-origin HTML), after robots.txt, TDMRep, denylist and SSRF checks; returns URLs and declared dimensions only | §6.1, row 1 |
| Transient analysis of a candidate image | The user's browser, by a separate CORS fetch, in a code path that returns facts only | §6.3; no proxy |
| Download of image bytes | Command-line or self-hosted tool only, with an explicit rights basis | doc 07, implication 4 |

Display rules for candidates:

- Inline preview only for allow-listed API hosts, in the smallest useful size: Open Library `-M` (fits 180 x 360 px), Google `thumbnail` (about 128 px wide). This keeps the preview under the 400 px ceiling proposed in doc 07.
- For a pasted publisher page, the result is shown as a **plain link** ("open the image on the publisher's site") with the pixel size, not inlined. This is stricter than the scraping-law note, which rated an inline link Low–Medium. Reasons: we cannot detect hotlink protection reliably, a strict image-source policy is not possible with arbitrary hosts, and the *GS Media* presumption applies to a commercial service. It can be relaxed per publisher later.
- Every preview carries the source name and a link back.
- Third-party previews load only after the user asked for a lookup, and the interface names the hosts contacted. The privacy notice lists them (Internet Archive and Google are US-based; BnF and DNB are EU public bodies).

### 7.4 Opt-out signals

For every server-side fetch, and for every transient analysis:

1. **robots.txt** of the origin: fetch, cache 24 h, parse up to 500 KiB. Match our product token, then `*`. 4xx other than 429 = no restriction; 5xx or unreachable = disallow.
2. **TDMRep**: `/.well-known/tdmrep.json` of the origin first; then the `tdm-reservation` header of the response; then HTML meta tags, each superseding the previous one as the protocol specifies. An effective value of 1 closes the analysis route for that resource. An image is checked on its own response and its own origin, because the HTML tag does not cover it [V1].
3. **`X-Robots-Tag` and meta `noai`, `noimageai`**: treated as a reservation.
4. **Terms of use**: recorded per allow-listed host at review time. For a host that is not reviewed, there is no server-side analysis.
5. Any reservation means: no fetch for analysis, no facts derived. An inline link by the user's browser is not TDM and stays possible for allow-listed API hosts.
6. Each decision is logged with the fetch: time, URL, user-agent, robots verdict with a hash of the file, TDM signal state, status, bytes, purpose, and the fact that nothing was retained.

### 7.5 Product-side rules that complete the fetcher

- **Store facts, not pixels**: URL, timestamp, hash, dimensions, dominant colour, font class.
- **Publisher opt-out list** by ISBN registrant prefix, publisher name or host: listed titles get no lookup and no preview, only the typographic card. Requests are handled the same day. This is mitigation, not a legal basis.
- **No personal data** from pages: no reviews, ratings or reviewer names.
- **Abuse limits**: lookups are capped per session so the service cannot be used as an enumeration relay.

---

## 8. Open questions

Ordered by how much the design depends on them. None could be checked in this session.

1. **Terms of each candidate source, as of 2026-11**: Open Library Covers (display rule, link-back, the 100-per-5-minutes figure, stance on reuse); Google Books API (caching, attribution, quota, whether a browser key is acceptable); BnF catalogue cover service (origin of the images, reuse terms); DNB/MVB cover service; LibraryThing. Rows 7–9 of the cascade depend on them.
2. **Hit rate and real pixel sizes** for French eBook ISBNs. The missing experiment is a run on 200–500 French eBook EANs against Open Library, Google Books and BnF, recording hit, dimensions and placeholder behaviour. It also decides whether BnF should precede Open Library for 978-2 and 979-10.
3. **CORS on archive.org** for the redirect targets of Open Library covers, and CORS of Google Books, BnF and DNB image and metadata responses.
4. **Actual size of covers inside French EPUBs** (the Kobo average is old), and the minimums that distributors impose.
5. **Counsel**: (a) is a browser-direct import of a candidate image acceptable when the user declares to be the rights holder, or must the user always save and re-select the file? Doc 07 currently requires a local file. (b) Is an inline preview by link acceptable for a commercial service given the *GS Media* presumption? (c) Is transient in-browser analysis of a third-party cover covered by art. 4 or art. 5(1)? (d) Does AI upscaling of a licensed cover count as a modification?
6. **Legal authorities cited from memory**: CPI R122-28 wording and the TDM items of L342-3; OLG Hamburg 5 U 104/24; Rb. Amsterdam *DPG Media v HowardsHome*; LG München I *GEMA v OpenAI*; the pending CJEU reference C-250/25; the CNIL sheet of 19 June 2025 and whether it frames robots.txt as mandatory; French decision numbers (n° 12-26.023, n° 21-16.307, n° 23-13.535, n° 14-81.336); CPI L136-1 to L136-4; penalties in L343-4 and Code pénal 323-1 and 323-3; RFC 9309 quotations and numeric limits; US district-court outcomes.
7. **robots.txt, TDMRep and CGU state of French sites** we might touch: publisher sites beyond the two groups on the adopters list, and whether the SNE recommendation is as widely followed as believed. A French trade outlet is said to track this daily (linked from the adopters list; not opened).
8. **Spines**: whether any French ONIX feed fills List 158 code 29 or 51; whether 7switch's distributor does (the ONIX library read for the code list is published by "immateriel", which I take to be that distributor [M]); the spine reading-direction convention by country.
9. **TDM reservation inside EPUB files**: should in-browser analysis of the user's own EPUB cover (colour, font class) be skipped when the package carries `tdm:reservation = 1`? Proposal: read and record the flag, keep all results on the device, never use such files for training, and ask counsel.
10. **Super-resolution**: model file sizes, availability and licence of ready-made ONNX exports, browser timings on mid-range hardware, licence status of pretrained weights against their training data.
11. **No authority known** for: robots.txt as an "effective technological measure"; a French judgment treating a robots.txt violation as a fault in itself; a French judgment on image hotlinking as *parasitisme*; any EU decision on cover-lookup services.
12. **Not researched for lack of search budget**: Inventaire's cover API and terms, Babelio, Goodreads' own terms, publisher press-kit practices, Dutch and German retailer patterns, Adobe ADEPT and Kindle handling of cover resources, Amazon Creators API terms.

---

## 9. Sources

No URL in this document was invented. Every link below is either a file fetched with HTTP 200 and read on 2026-11-12 (§9.1–9.3), or a link copied from a sibling note where it had appeared in that researcher's search results (§9.4). Legal authorities are cited by identifier only (§9.5).

### 9.1 Read by the sources researcher — services and provider code

GPL and AGPL code read for facts only; do not copy.

- Open Library: [code.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/code.py), [config.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/config.py), [utils.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/utils.py), [coverstore README](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/README.md), [nginx.conf](https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/nginx.conf), [covers_nginx.conf](https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/covers_nginx.conf)
- calibre: [openlibrary.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/ebooks/metadata/sources/openlibrary.py), [google.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/ebooks/metadata/sources/google.py), [amazon.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/ebooks/metadata/sources/amazon.py); [calibre-dnb](https://raw.githubusercontent.com/citronalco/calibre-dnb/master/__init__.py)
- VuFind: [OpenLibrary.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/OpenLibrary.php), [Google.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Google.php), [LibraryThing.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/LibraryThing.php), [Syndetics.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Syndetics.php), [ContentCafe.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/ContentCafe.php), [Buchhandel.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Buchhandel.php), [Orb.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Orb.php), [Deprecated.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Deprecated.php), [config.ini](https://raw.githubusercontent.com/vufind-org/vufind/dev/config/vufind/config.ini)
- Koha: [opac-detail.tt](https://raw.githubusercontent.com/Koha-Community/Koha/main/koha-tmpl/opac-tmpl/bootstrap/en/modules/opac-detail.tt), [enhanced_content.pref](https://raw.githubusercontent.com/Koha-Community/Koha/main/koha-tmpl/intranet-tmpl/prog/en/modules/admin/preferences/enhanced_content.pref)
- Others: [Coce README](https://raw.githubusercontent.com/fredericd/coce/master/README.md), [bookcover-api README](https://raw.githubusercontent.com/w3slley/bookcover-api/main/README.md), [BookWyrm connector](https://raw.githubusercontent.com/bookwyrm-social/bookwyrm/main/bookwyrm/connectors/openlibrary.py), [Inventaire README](https://raw.githubusercontent.com/inventaire/inventaire/main/README.md), [im_onix codelist 158](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-158.yml)

### 9.2 Read by the sources researcher — super-resolution

- Real-ESRGAN: [README](https://raw.githubusercontent.com/xinntao/Real-ESRGAN/master/README.md), [LICENSE](https://raw.githubusercontent.com/xinntao/Real-ESRGAN/master/LICENSE), [model zoo](https://raw.githubusercontent.com/xinntao/Real-ESRGAN/master/docs/model_zoo.md)
- SwinIR: [README](https://raw.githubusercontent.com/JingyunLiang/SwinIR/main/README.md), [LICENSE](https://raw.githubusercontent.com/JingyunLiang/SwinIR/main/LICENSE)
- Swin2SR: [README](https://raw.githubusercontent.com/mv-lab/swin2sr/main/README.md), [LICENSE](https://raw.githubusercontent.com/mv-lab/swin2sr/main/LICENSE)
- nunif / waifu2x: [README](https://raw.githubusercontent.com/nagadomi/nunif/master/README.md), [LICENSE](https://raw.githubusercontent.com/nagadomi/nunif/master/LICENSE), [waifu2x README](https://raw.githubusercontent.com/nagadomi/nunif/master/waifu2x/README.md), [unlimited:waifu2x README](https://raw.githubusercontent.com/nagadomi/nunif/master/waifu2x/unlimited_waifu2x/README.md)
- UpscalerJS: [README](https://raw.githubusercontent.com/thekevinscott/UpscalerJS/main/README.md), [LICENSE](https://raw.githubusercontent.com/thekevinscott/UpscalerJS/main/LICENSE), [esrgan-slim README](https://raw.githubusercontent.com/thekevinscott/UpscalerJS/main/models/esrgan-slim/README.md)
- [transformers.js README](https://raw.githubusercontent.com/huggingface/transformers.js/main/README.md), [ONNX Runtime LICENSE](https://raw.githubusercontent.com/microsoft/onnxruntime/main/LICENSE)
- Real-CUGAN: [README](https://raw.githubusercontent.com/bilibili/ailab/main/Real-CUGAN/README_EN.md), [LICENSE](https://raw.githubusercontent.com/bilibili/ailab/main/Real-CUGAN/LICENSE)

### 9.3 Read for this synthesis

- Open Library: [asgi_app.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/asgi_app.py) (CORS), [vendors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/core/vendors.py) (Amazon APIs); re-read: code.py, config.py, utils.py, coverstore README, nginx.conf, covers_nginx.conf, and [server.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/server.py)
- BnF cover probe: [Inventaire get_bnf_entry_from_isbn.ts](https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/bnf/get_bnf_entry_from_isbn.ts)
- Google Books: [API discovery document](https://raw.githubusercontent.com/googleapis/google-api-go-client/main/books/v1/books-api.json)
- EPUB and DRM: [EPUB 3.3 core](https://raw.githubusercontent.com/w3c/epub-specs/main/epub33/core/index.html), [Kobo EPUB spec](https://raw.githubusercontent.com/kobolabs/epub-spec/master/README.md), [Readium LCP specification](https://raw.githubusercontent.com/readium/lcp-specs/master/releases/lcp/latest.md)
- TDMRep (W3C Community Group): [README](https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/README.md), [draft specification](https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/spec/index.html), [HTTP header technique](https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/techniques/technique-http-headers.md), [well-known file technique](https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/techniques/technique-file-at-origin.md), [HTML meta technique](https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/techniques/technique-html-meta.md), [adopters](https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/docs/adopters.md), [robots.txt primer](https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/docs/robots.md), [extracts of the CDSM Directive and AI Act](https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/docs/eu-act-extracts.md)
- robots.txt handling: [texting_robots README](https://raw.githubusercontent.com/Smerity/texting_robots/main/README.md), [google/robotstxt README](https://raw.githubusercontent.com/google/robotstxt/master/README.md)
- ONIX: [im_onix codelist 158](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-158.yml) (full list), [im_onix README](https://raw.githubusercontent.com/immateriel/im_onix/master/README.md)

### 9.4 Copied from sibling notes (appeared in those researchers' results; not re-opened)

- [IRIS Merlin — Renckhoff](https://merlin.obs.coe.int/article/8373)
- [Columbia Global Freedom of Expression — Renckhoff](https://globalfreedomofexpression.columbia.edu/cases/land-nordrhein-westfalen-v-dirk-renckhoff/)
- [Mondaq — Renckhoff](https://www.mondaq.com/uk/copyright/730320/the-risks-of-reposting-online-content-cjeu-rules-on-the-unauthorised-reposting-of-copyright-protected-works-on-the-internet)
- [Kluwer Copyright Blog — VCAST](https://legalblogs.wolterskluwer.com/copyright-blog/up-in-the-cloud-some-reflections-on-the-cjeu-judgment-in-vcast/)
- [Fournol — parasitisme pour la couverture d'un livre (Zulma v Elior)](https://www.fournol-avocat.fr/actualite/2022/8/18/parasitisme-reconnu-pour-la-couverture-dun-livre)
- [UGGC — Artprice](https://www.uggc.com/condamnation-dartprice-com-pour-contrefacon-et-concurrence-parasitaire/)
- [Kelly v Arriba Soft, 336 F.3d 811](https://m.openjurist.org/336/f3d/811)
- [CRS RL33810 — search engines and fair use](https://www.everycrsreport.com/reports/RL33810.html)
- [LibraryThing covers blog](https://blog.librarything.com/category/covers/), [Goodreads — source of book covers](https://www.goodreads.com/topic/show/1277733-source-of-book-covers?page=1) (from `07-legal-covers-and-reseller-rights.md`)
- [BnF hackathon wiki](https://raw.githubusercontent.com/wiki/hackathonBnF/hackathon2016/Home.md), [Open Library processors.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/plugins/openlibrary/processors.py) (CORS on JSON), [Open Library web_nginx.conf](https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/web_nginx.conf) (API rate limit) — files read by the ISBN researcher, from `notes/isbn-databases.open.md`

### 9.5 Authorities cited from memory, by identifier

- **EU legislation**: Directive 96/9/EC (arts. 7, 8, 10, 11, 15); Directive 2001/29/EC (arts. 2, 3, 5(1), 6); Directive (EU) 2019/790 (arts. 2(2), 3, 4, 7; recitals 14, 18 — art. 4 and recital 18 read in extract, §6.3); Regulation (EU) 2024/1689 art. 53(1)(c) and the GPAI Code of Practice of 10 July 2025; Regulation (EU) 2022/2065 art. 4; Directive (EU) 2019/1024; Rome II Regulation art. 8.
- **CJEU**: C-203/02 *BHB v William Hill*; C-304/07 *Directmedia*; C-5/08 *Infopaq*; C-173/11 *Football Dataco v Sportradar*; C-202/12 *Innoweb*; C-466/12 *Svensson*; C-360/13 *PRCA v NLA*; C-348/13 *BestWater*; C-30/14 *Ryanair v PR Aviation*; C-160/15 *GS Media*; C-301/15 *Soulier and Doke*; C-161/17 *Renckhoff*; C-265/16 *VCAST*; C-392/19 *VG Bild-Kunst*; C-762/19 *CV-Online Latvia v Melons*; C-170/12 *Pinckney*; C-441/13 *Hejduk*.
- **France**: CPI L122-5-3, R122-28, L131-3, L136-1 to L136-4, L341-1 to L343-7; ordonnance n° 2021-1518; décret n° 2022-928; loi n° 98-536; C. civ. arts. 9, 1119, 1240; Code pénal arts. 323-1, 323-3; Cass. com. 10 Feb 2015 n° 12-26.023; Cass. crim. 20 May 2015 n° 14-81.336; CA Paris 2 Feb 2021 and Cass. 1re civ. 5 Oct 2022 n° 21-16.307; Cass. com. 26 June 2024 n° 23-13.535; CNIL guidance of 30 Apr 2020 and AI recommendations of 19 June 2025; CNIL sanctions Clearview (2022) and KASPR (2024).
- **Germany, Netherlands, Belgium**: BGH I ZR 69/08; BGH I ZR 224/12; LG Hamburg 310 O 227/23; OLG Hamburg 5 U 104/24; LG München I 42 O 14139/24 and 3 O 17493/20; Rb. Amsterdam 30 Oct 2024; CA Bruxelles 5 May 2011.
- **US**: *hiQ v LinkedIn* (938 F.3d 985; 31 F.4th 1180); *Van Buren v US* (2021); *Meta v Bright Data* (2024); *X Corp v Bright Data* (2024); *Perfect 10 v Amazon* (508 F.3d 1146); *Hunley v Instagram* (2023); *Goldman v Breitbart* (2018); *Feist v Rural* (1991); *Warhol v Goldsmith* (2023).
- **Standards**: RFC 9309 (September 2022).

Internal cross-references: `07-legal-covers-and-reseller-rights.md` (print rights, rights-basis model), `08-legal-fonts-and-genai.md` (training data, generated spines), `notes/print-requirements.prepress.md` and `notes/print-requirements.folding.md` (resolution and format), `notes/isbn-databases.open.md` (catalogue providers, CORS, ISBN routing).

---

## Implications for the implementation

Numbers marked (proposal) are design choices, not legal thresholds or provider-stated limits. Rights-basis names are those of doc 07.

**Two output types, kept apart by construction**

1. The acquisition code produces either a **printable asset** (image bytes plus a rights basis other than `none`) or a **candidate** (source id, URL, declared width and height, fetch time, the date the source's terms were last read, rights basis `none`). A candidate has no field that can hold pixels, and the composer accepts printable assets only.
2. Printable assets enter in exactly four ways: a reseller feed (bytes or URL supplied by the reseller under its per-title flag), a local file picked or dropped by the user, a cover extracted in the browser from a local EPUB, a flat full cover uploaded by a declared rights holder. Nothing fetched by a lookup becomes a printable asset in the hosted service.
3. The command-line and self-hosted tools may download a candidate's bytes only when the operator passes an explicit rights basis; they print one line stating that the operator is responsible for the rights. Without the argument they list candidates and build the typographic card.
4. Every job records provenance for its cover: step number of the cascade, source, URL if any, native pixel size, rights basis, and for lookups the opt-out signals seen.

**Cascade and policy**

5. Candidate lookup runs in this order and stops at the first accepted hit: Open Library, Google Books (only if a key is configured), then national catalogues (BnF for 978-2 and 979-10, DNB for 978-3). BnF and DNB ship disabled behind a configuration flag until their image terms are recorded. The order is configurable so that the hit-rate test can change it.
6. Each source has a static descriptor: id, homepage, terms URL, date terms were last read, whether a key is needed, whether cross-origin reads are allowed, minimum interval between requests, allowed uses (`preview_inline`, `link_only`, `analyse_transient`, `download_cli`), and a risk level for preview (Low, Medium, High; "Low–Medium" is stored as Medium). The hosted service refuses any source whose preview risk is High, and any source whose terms review is older than 180 days (proposal).
7. A built-in denylist of hosts is checked before any request and against every extracted image URL: Amazon image and site hosts, Goodreads (`gr-assets.com`), Fnac, Decitre and Furet du Nord (`di-static.com`), Cultura, ePagine/Tite-Live, leslibraires.fr, Babelio, IBS, Casa del Libro, and image search engines. A denylisted URL pasted by the user is refused with a message explaining that the user may supply the file directly.
8. Reseller mode never runs lookups. If the feed has no cover or the title has `cover_rights = none`, the output is the typographic card.
9. Where the catalogue lookup already returned a cover id or image link (Open Library edition `covers`, Google `imageLinks`), the candidate is built from that response without a second request.
10. When an eBook ISBN has no hit, the lookup is retried once with the print ISBN if one is known (ONIX `RelatedProduct`, or a catalogue record), and the candidate is flagged "other edition" for the user to confirm.

**Open Library adapter**

11. Request `https://covers.openlibrary.org/b/isbn/{isbn13}.json` from the browser, with the ISBN written without hyphens (the JSON route does not strip them), follow the 302 to the id record, read `width` and `height`. A 404 means no cover. Then use id-keyed URLs only: `/b/id/{id}-M.jpg` for the preview, `/b/id/{id}.jpg` (original) as the "open the image" link.
12. Every image URL built for Open Library carries `?default=false`. Clients follow up to 5 redirects and allow `archive.org` and its subdomains as redirect targets.
13. ISBN-keyed requests are limited to 1 every 3 seconds per client; id-keyed requests to 1 per second. No request is ever routed through a CloudFront origin fetch.

**Google Books adapter**

14. Disabled unless an API key is configured. Results are held in memory for the session only and never written to a cache or database. The preview uses the `thumbnail` link with `edge=curl` removed, and shows "Google Books" attribution with a link to the volume page.
15. An image whose MD5 is `0de4383ebad0adad5eeb8975cd796657` or `a64fa89d7ebc97075c1d363fc5fea71f` is a Google placeholder. Since preview pixels are not read in the hosted service, the adapter rejects volumes whose record has no `imageLinks`; the two digests stay in the fixture list for any code path that does read bytes. Google images are never downloaded by the command-line tool.

**BnF and DNB adapters**

16. BnF: resolve the ARK through SRU, then send a HEAD request to the cover URL; accept only status 200 with a `Content-Length` other than 4566 and 4658. DNB: HEAD request; any HTTP error means no cover. Both return a candidate with `link_only` until their terms are recorded.

**Pasted page extractor**

17. Accept only `https` URLs. Resolve the host and refuse private, loopback, link-local and metadata addresses, at connection time as well as before it; re-check after every redirect; at most 5 redirects; 2 MiB body cap; 10 s total timeout.
18. Before fetching the page: check the denylist, then robots.txt for our user-agent, then `/.well-known/tdmrep.json`. After fetching: check the `tdm-reservation` header, `X-Robots-Tag`, and the HTML meta tags. A disallow or a reservation means the page is not parsed and the user is told to supply the image file directly.
19. Extraction order: JSON-LD `Book` or `Product` image, `og:image`, `twitter:image`, `link rel="image_src"`, `itemprop="image"`. The page must contain the requested ISBN (or a known sibling ISBN); otherwise the result is flagged "ISBN not found on page".
20. The extractor returns image URLs and declared dimensions only. The interface shows them as plain links with the source name. No image from a non-allow-listed host is inlined: the `img-src` directive of the Content-Security-Policy lists only our own origin, `data:`/`blob:` for local files, and the allow-listed preview hosts (`covers.openlibrary.org`, `archive.org` and its subdomains, the Google Books image host, and BnF or DNB once enabled).

**Fetcher behaviour (all server-side and command-line requests)**

21. User-agent: fixed product token, version, a URL describing the bot and a contact address; it contains "bot" and never contains "CloudFront". It is never changed to imitate a browser.
22. robots.txt: fetched per origin, cached 24 hours, parsed up to 500 KiB; our product token is matched first, then `*`; longest match wins and `Allow` wins ties; a 4xx other than 429 means no restriction; a 5xx, a timeout or an unreachable host means disallow. `Crawl-delay` is honoured when longer than the configured interval.
23. TDMRep: `tdmrep.json` is cached 24 hours per origin; precedence is file, then response header, then HTML meta; an absent value never resets an earlier one; any value other than 0 or 1 is treated as unset. `noai` and `noimageai` in `X-Robots-Tag` or meta robots count as a reservation.
24. On 401, 403, a CAPTCHA page or a hotlink refusal: stop, mark the host blocked for 24 hours, do not retry by any other route. On 429: honour `Retry-After`, otherwise wait 60 s and double, at most 3 attempts.
25. Default interval for a host with no specific rule: 5 seconds, and at most 3 requests per user action (proposal).
26. Each request writes one log record: time, job id, URL, user-agent, robots verdict and a hash of the robots.txt used, TDM signal state, HTTP status, byte count, purpose (`locate`, `analyse`, `download_cli`) and retention outcome. The log holds no image data and no end-user personal data. Retention 12 months (proposal).
27. Caches hold facts only: candidate records 7 days, negative results 24 hours (proposal). The hosted service has no endpoint that returns third-party image bytes, and an integration test asserts this for every route.

**Preview**

28. A candidate from an allow-listed API host is shown as an image element pointing at the source, at most 400 CSS px on its long edge, with `referrerpolicy="no-referrer"`, the source name and a link back. The element is never drawn to a canvas and never passed to WebAssembly.
29. Third-party previews load only after the user starts a lookup; the lookup control names the hosts that will be contacted, and the privacy notice lists them.

**Transient analysis (optional, off by default)**

30. When enabled, analysis runs in the browser through a separate cross-origin fetch. If the fetch fails for lack of CORS headers, analysis is skipped; there is no server fallback. The function's return type holds only: one dominant colour and up to four secondary colours, aspect ratio, a 64-bit perceptual hash, and a font-class label. The image buffer is zeroed and dropped before the function returns.
31. Analysis is skipped when the image response or its origin expresses a TDM reservation, or when the source descriptor lacks `analyse_transient`.

**EPUB cover extraction (browser)**

32. Order: manifest item with `cover-image` in `properties`; else the item referenced by `<meta name="cover">`; else `guide` reference of type `cover` and the first image in that document; else a manifest image whose id or path contains "cover"; else the first image of the first spine document. The method used is recorded.
33. If `META-INF/encryption.xml` lists the chosen resource, the extractor reports "cover encrypted" and stops. No decryption code exists in the project.
34. Raster covers (JPEG, PNG, GIF, WebP) are decoded with a cap of 10,000 px per side and 64 megapixels (proposal). An SVG cover is refused with a message asking for a raster file, unless a sandboxed rasteriser without external resource loading is available.
35. The package's `tdm:reservation` value, if present, is read and stored with the job.

**Placeholder and quality checks**

36. `is_placeholder` returns true for: a body under 100 bytes; a 1 x 1 image; either side under 100 px; width/height outside 0.55–0.85; luminance standard deviation under 2 on a 0–255 scale; fewer than 16 distinct colours after quantisation to 4 bits per channel; an MD5 or perceptual-hash match (Hamming distance of 6 or less on 64 bits) with the fixture list (thresholds are proposals to tune on the sample run). Image type is decided from magic bytes.
37. The fixture list stores hashes and byte lengths only: the two Google MD5 digests, BnF lengths 4566 and 4658, the Amazon placeholder markers.
38. Print quality is computed from native pixels against the placed size: 300 ppi and above is "good", 250–299 "acceptable", 150–249 "low" (warning), under 150 "insufficient". A professional print order is refused under 150 ppi native; home print proceeds with the label shown on the preview and in the preflight report.
39. The preflight report states native ppi and the upscale factor as two separate values.

**Upscaling**

40. Default resampling is Lanczos3 to the placed size at 300 ppi, with no sharpening beyond a mild unsharp mask.
41. AI super-resolution is an opt-in step in standalone mode: maximum factor 4; processed in tiles of 256 px or less with a 16 px overlap; before-and-after shown at print scale; the model file is downloaded on first use from the project's own origin, cached by the browser, and is 20 MB or less for the default model (proposal). WebGPU uses fp32 weights; the wasm fallback uses fp16.
42. The default model is non-GAN or a small general model; GAN variants are selectable but never default. AI upscaling is unavailable for covers with `print_licence` or `promo_all_media` unless the reseller configuration sets `enhance = true`.

**Spine and full cover**

43. Reseller feeds: ONIX `SupportingResource` with List 158 code 01 is the front, 02 the back, 29 a full cover, 51 a flap. Codes 29 and 51 are ingested when present and treated as publisher-supplied files under the same rights flags.
44. A flat full cover is split by measured panel widths supplied by the uploader (back, spine, front), not guessed from the image. Its spine is used only if, scaled uniformly to the panel height, its width is between 6 and 10 mm (proposal); it is then centred on the 10 mm panel with the flat ground colour either side. Otherwise the typographic spine is generated.
45. Spine text direction defaults to bottom-to-top for French, German, Spanish, Italian and Dutch locales and top-to-bottom for English, with a per-card override.
46. No adapter, test or tool queries Amazon for spine or variant images.

**Opt-out, abuse, tests**

47. A publisher opt-out list (ISBN registrant prefix, publisher name, host) is consulted before any lookup; a listed title gets no lookup and no preview. The bot information page gives the contact address and the handling time (same working day).
48. Lookups are capped at 30 per session per hour and 1 per ISBN per user action (proposal). The extractor never stores review text, ratings or reviewer names.
49. Integration tests run the fetcher against a fake publisher site and a fake cover API that serve, in turn: a robots.txt `Disallow`, a robots.txt returning 503, `tdm-reservation: 1` as a header and as a `tdmrep.json` rule, an HTML meta reservation with an unreserved image, a 429 with `Retry-After`, a 403, a redirect to a private address, a placeholder image of each known kind, and an oversized body. Each case asserts the decision and the log record.
50. A scheduled, opt-in live test (never in the default CI run) samples a fixed list of ISBNs against Open Library and reports hit rate and pixel sizes, at 1 request every 3 seconds, to feed open question 2.
