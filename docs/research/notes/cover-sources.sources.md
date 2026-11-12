# Finding book covers (and spines) when absent — sources, scraping practice, legality of acquisition

Research notes, angle "WHERE AND HOW". Date of research: 2026-11-12. Market: France first, then EU.

Confidence markers: **[V2]** verified in >= 2 independent sources this session, **[V1]** single source this session,
**[M]** from memory, not verified this session ("[M, low]" = I would not code against it without a live check),
**[est.]** my own arithmetic.

## 0. Method and limits (read first)

- **No web search could be run.** All five first queries were refused: the session's shared WebSearch budget (200 calls) was
  already exhausted by sibling agents. WebFetch is blocked. Nothing below comes from a search result.
- To avoid a purely from-memory note, I read **primary files on `raw.githubusercontent.com`** (the one content host the sandbox
  allows): the Open Library cover server source and nginx config, and the cover-provider code of calibre, VuFind, Koha, Coce and
  calibre-dnb, plus the READMEs/licences of the super-resolution projects. Every link in this note is a file I fetched (HTTP 200)
  and read today. "Independent sources" for [V2] therefore means independent code bases, not independent web pages.
- calibre (GPLv3), VuFind (GPLv2), Koha (GPL) and Inventaire (AGPL) were read **for facts only** (URL patterns, behaviours).
  No code from them may be copied into this BSD-3 repo.
- Not verified at all this session: any provider's terms of use, any quota, any actual pixel size returned by a live request, every
  French bookseller URL pattern, the BnF cover service, Amazon's current API programme. These are [M] and listed at the end.

## 1. Bottom line

1. **The best cover source is not a cover API.** For an eBook the highest-resolution, cleanest-rights image is (a) the cover file
   inside the EPUB or (b) the cover in the reseller's ONIX/distributor feed. Retail eBook covers are typically 1400–2560 px on the
   long side [M], i.e. 200–365 ppi on a 178 mm front [est.]. Public ISBN cover APIs are thumbnail services: about 500 px tall,
   i.e. about 71 ppi [est.]. Lookup APIs should be the fallback, not the primary path.
2. **Open Library is the only source that is open, keyless, and whose behaviour I could verify in source**: size suffixes,
   `?default=false`, an *original-size* URL (no suffix), a JSON endpoint giving width/height, bulk dumps on archive.org, and the
   nginx rate limits (section 2.1).
3. **Amazon, Goodreads-derived "bookcover" APIs, Syndetics, Content Cafe, VLB, Electre, ORB are not usable for a printed product**
   without a contract: either the terms forbid it [M] or they are paid B2B services whose licence is "promote the title in your
   catalogue" [M].
4. **A 500 px cover needs about 4.2x upscaling to reach 300 ppi on a 110 x 178 mm front** [est.]. Browser-side super-resolution is
   feasible with permissively licensed models (Real-ESRGAN BSD-3-Clause, SwinIR/Swin2SR Apache-2.0, waifu2x/nunif MIT, UpscalerJS
   MIT, ONNX Runtime MIT — licences [V1] each), but GAN models damage small type; the title must be re-typeset, not upscaled.
5. **No public source exposes spines by ISBN.** ONIX has a code for it (List 158 code 29 `FullCover`, 51 `CoverFlap`) [V1] but I
   know of no French feed that populates it [M, low]. Spine generation, as planned in the brief, is the only general solution;
   a flat-cover upload ("couverture à plat") is the only source of a real spine.

## 2. Source catalogue

### 2.0 Summary table

| Source | Lookup by ISBN | Largest image | "No cover" signal | Access | Usable for print? |
|---|---|---|---|---|---|
| EPUB file (user/reseller upload) | n/a — parse OPF `cover-image` | 1400–2560 px long side [M] | absent manifest item | local, in-browser | Best: rights follow the reseller/publisher chain |
| Reseller ONIX / distributor feed | `SupportingResource`, List 158 code 01 [V1 for code] | feed-dependent, often >= 1400 px [M] | element absent | contract | Best in reseller mode |
| Open Library Covers | `covers.openlibrary.org/b/isbn/{isbn}-L.jpg` [V2] | `-L` = 500 px box; no suffix = original upload [V1] | HTTP 404 with `?default=false` [V2] | keyless; nginx 400 req/min per IP [V1] | Technically yes; no licence on the images [M] |
| Google Books | `books.google.com/books?jscmd=viewapi&bibkeys=ISBN:{isbn}` [V1]; Books API v1 `q=isbn:` [M] | `zoom=0/1` variants [V2]; larger sizes patchy [M] | dummy image (MD5 check) [V1] | keyless/low quota [M] | ToS-restricted [M]; VuFind does not cache it [V1] |
| Amazon legacy image URL | `images-na.ssl-images-amazon.com/images/P/{ISBN10}.01.LZZZZZZZ.jpg` [V1] | about 500 px [M] | 1x1 GIF [M] | undocumented | No: Associates terms forbid offline use [M] |
| Amazon product page | scrape `"hiRes"` / `data-a-dynamic-image` [V1] | up to about 1500–2560 px [M] | n/a | anti-bot, ToS [M] | No |
| LibraryThing | `covers.librarything.com/devkey/{KEY}/{small,medium,large}/isbn/{isbn}` [V1] | unknown | 1x1 GIF [M] | dev key, daily cap [M] | Unclear [M, low] |
| BnF catalogue cover | `catalogue.bnf.fr/couverture?&appName=NE&idArk={ark}&couverture=1` [M] | thumbnail [M, low] | generic image [M, low] | keyless [M] | Terms unknown |
| DNB (Germany) | `https://portal.dnb.de/opac/mvb/cover?isbn={isbn}` [V1] | unknown | HTTP error on HEAD [V1] | keyless | MVB-sourced; reuse terms unknown |
| VLB / MVB (Germany) | `https://api.vlb.de/api/v1/cover/{isbn}/{s,m,l}?access_token=` [V1] | unknown | unknown | paid token [V1] | By contract only |
| ORB (France) | `api.base-orb.fr/v1/products?eans={ean}` basic auth, `thumbnail`/`original` [V2] | "original" | product absent | paid account [V1] | By contract only |
| Syndetics | `secure.syndetics.com/index.aspx?isbn={isbn}/{SC,MC}.GIF or LC.JPG&client={id}` [V2] | LC [V1] | via `index.xml` metadata [V1] | library subscription | No |
| Content Cafe (Baker & Taylor) | `contentcafe2.btol.com/ContentCafe/Jacket.aspx?UserID=&Password=&Return=1&Type={S,M,L}&Value={isbn}&erroroverride=1` [V1] | L | `erroroverride=1` [V1 param] | subscription [V1] | No |
| Electre, Dilicom (FEL / Hub), Tite-Live | proprietary | varies | varies | paid B2B [M] | By contract only |
| bookcover-api (longitood) | `bookcover.longitood.com/bookcover?isbn=` [V1] | Goodreads image [V1] | 404 [V1] | free, 429 limits [V1] | No (Goodreads/Amazon origin) |
| Publisher / bookseller pages | og:image, JSON-LD `image` | 300–1200 px typical [M] | n/a | per-site CGU | Case by case |
| Internet Archive scans | `archive.org/download/{id}/page/cover_w{W}_h{H}.jpg` [V1] | scan resolution [M] | item not `texts` / no imagecount [V1] | keyless | Photo of a library copy; same cover rights |

### 2.1 Open Library Covers (verified in source)

From the cover server ([code.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/code.py),
[config.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/config.py)):

- Routes: `/{category}/{key}/{value}-{S|M|L}.jpg` and `/{category}/{key}/{value}.jpg` (category `b` = books; keys `isbn`, `id`,
  `olid`, `ia`, and others resolved by a query) [V1]. Hyphens in ISBNs are stripped server-side [V1].
- Sizes are bounding boxes `(max width, max height)`: `S (116, 58)`, `M (180, 360)`, `L (500, 500)` [V1]. A portrait `-L` cover is
  therefore at most 500 px tall.
- **No suffix = the original upload** (`""` "being the original upload" in the source) [V1]. This is the only way to get more
  than 500 px from Open Library; how large originals are depends on who uploaded them [M].
- `?default=false` returns **HTTP 404** instead of a placeholder [V2: OL source; also hard-coded by
  [calibre](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/ebooks/metadata/sources/openlibrary.py) and
  [VuFind](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/OpenLibrary.php)].
  `?default=<url>` redirects (303) only to vetted OL/IA/cover hosts since a security fix
  ([utils.py](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/utils.py)) [V1].
- `/b/isbn/{isbn}.json` redirects (302) to `/b/id/{coverid}.json`, whose record includes `width`, `height`, `source_url` [V1].
  **Use this to learn the original pixel size before downloading.**
- Many requests answer with a **302 to archive.org** (`archive.org/download/olcovers{N}/olcovers{N}-L.zip/{id}-L.jpg`, or the
  `covers_0008` items) for `L` and original sizes [V1]. A fetcher must follow redirects and allow `archive.org`.
- Caching: ISBN-keyed responses carry `Expires` +10 minutes; id-keyed responses "100 years" plus ETag [V1]. Resolve ISBN to cover
  id once, then use `/b/id/`.
- `/b/ia/{identifier}-L.jpg` redirects to `archive.org/download/{identifier}/page/cover_w500_h500.jpg` when the IA item is a
  finished `texts` scan [V1].
- Blocked covers: a server-side `blocked_covers` list exists for takedowns [V1] — a hint that takedown requests do occur.

Rate limits, from the deployed nginx configuration
([nginx.conf](https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/nginx.conf),
[covers_nginx.conf](https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/covers_nginx.conf)):

- `cover_limit`: **400 requests/minute, burst 400, keyed by client IP** when there is no session cookie and the IP/UA is not
  allow-listed; excess gets HTTP 429 with a JSON body asking to email `info@archive.org` [V1].
- Requests whose User-Agent contains `CloudFront` or `Bytespider` are dropped (nginx 444) [V1]. **Do not fetch OL covers through
  a CloudFront origin fetch.** UAs containing "bot", "http" or "@" count as self-identifying and escape the shared
  "non-identifying crawler" bucket [V1].
- The public API page states "100 requests per IP per 5 minutes" for ISBN-keyed lookups, with id/OLID lookups exempt [M]. The
  repo config is more generous. I could not reconcile the two; **design for the stricter figure.**

Bulk: covers 0–7,139,999 are zips of 10,000 images in `olcovers{N}` items of the `ol_exports` collection on archive.org
([coverstore README](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/README.md)) [V1].
Open Library puts no licence on cover images (its *data* is CC0) [M].

### 2.2 Google Books

- Dynamic Links endpoint: `https://books.google.com/books?jscmd=viewapi&bibkeys=ISBN:{isbn13}[,OCLC:{n}]&callback=x` returns
  JSONP with `thumbnail_url`
  ([VuFind Google.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Google.php)) [V1].
- Image URL by volume id: `https://books.google.com/books?id={volumeId}&printsec=frontcover&img=1` plus `&zoom=N`
  ([calibre google.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/ebooks/metadata/sources/google.py)) [V1].
- `zoom`: VuFind rewrites `zoom=5` to `zoom=1` for medium/large [V1]; calibre tries `zoom=0` then `zoom=1` [V1]. So `5` is the
  smallest thumbnail and `0`/`1` are larger [V2]. The commonly cited ladder (5 about 80 px, 1 about 128 px wide, 2, 3, 4, 6 up to
  about 1280 px wide) and the `imageLinks.small/medium/large/extraLarge` keys returned only by `volumes/{id}` are [M]; larger
  sizes exist mainly for titles with a publisher-supplied preview [M].
- `&edge=curl` adds a page-curl effect; strip it [M]. `fife=w800-h1200` works on `books.google.com/books/publisher/content/...`
  URLs for Partner Program titles [M, low].
- **Placeholder**: Google returns a dummy "image not available" picture rather than a 404; calibre rejects two known MD5 digests
  (`0de4383ebad0adad5eeb8975cd796657`, `a64fa89d7ebc97075c1d363fc5fea71f`) [V1].
- Terms: VuFind enables local caching for Open Library, LibraryThing, Syndetics and Content Cafe but **not** for Google
  (`cacheAllowed` is not set in Google.php), and its config says "due to terms of service, some services will never have images
  cached" ([config.ini](https://raw.githubusercontent.com/vufind-org/vufind/dev/config/vufind/config.ini)) [V1]. The exact Google
  wording and the default quota (1,000 requests/day/project) are [M].
- Results are geo-dependent; server IPs can get "cannot determine user location" errors without a `country` parameter [M].

### 2.3 Amazon

- Legacy pattern, still shipped in Koha's OPAC: `https://images-na.ssl-images-amazon.com/images/P/{ISBN10}.01.{M|L}ZZZZZZZ.jpg`
  ([opac-detail.tt](https://raw.githubusercontent.com/Koha-Community/Koha/main/koha-tmpl/opac-tmpl/bootstrap/en/modules/opac-detail.tt)) [V1]
  (a `T` thumbnail variant also exists [M]). It needs an ISBN-10 (978-prefixed ISBNs only; 979-10 French ISBNs have no
  ISBN-10) [M]. A missing image is a 1x1 GIF, not a 404 [M].
- Product pages: calibre extracts `"hiRes":"..."`, `imageGalleryData[0].mainUrl`, or the widest entry of
  `data-a-dynamic-image`, and strips the `._XX_.` size modifier from `images/I/` URLs to get the original
  ([amazon.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/ebooks/metadata/sources/amazon.py)) [V1].
- VuFind's `dev` branch no longer ships an Amazon cover provider (the file is gone and the provider list omits Amazon) [V1].
- Terms [M]: Associates Program policies allow product images only through the official API, linked to Amazon, cached at most
  24 hours, and prohibit "offline" use including printed material. The Product Advertising API needs an Associates account with
  qualifying sales. I could not check the 2026 status of PA-API 5.0 versus its announced successor.
  **Conclusion: Amazon is out for a printed card.**

### 2.4 Library and trade services

- **LibraryThing**: URL pattern verified in
  [VuFind](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/LibraryThing.php) [V1].
  Developer key, about 1,000 requests/day, 1x1 GIF when absent, and whether new keys are still issued: all [M, low].
- **Syndetics** (library subscription): pattern verified in Koha and
  [VuFind](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Syndetics.php) [V2];
  files `SC.GIF`, `MC.GIF`, `LC.JPG`; `…/index.xml` lists which sizes exist [V1].
- **Content Cafe** (Baker & Taylor): "a subscription service" needing user id and password
  ([ContentCafe.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/ContentCafe.php), config.ini) [V1].
- **VLB (MVB, Germany)**: `https://api.vlb.de/api/v1/cover/{isbn}/{s|m|l}?access_token=…`, token required
  ([Buchhandel.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Buchhandel.php), config.ini) [V1].
- **DNB**: `https://portal.dnb.de/opac/mvb/cover?isbn={isbn}`; calibre-dnb probes it with HEAD and treats an HTTP error as "no
  cover" ([calibre-dnb](https://raw.githubusercontent.com/citronalco/calibre-dnb/master/__init__.py)) [V1]. The path says the
  images come from MVB; reuse terms unknown.
- **ORB (France)**, `base-orb.fr`: REST `products?eans=` with HTTP basic user/key, image versions `thumbnail` and `original`
  ([Orb.php](https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Orb.php);
  also a provider in [Coce](https://raw.githubusercontent.com/fredericd/coce/master/README.md)) [V2]. Operator, price and terms not checked.
- **Koha**'s enriched-content settings list Amazon, Google ("GoogleJackets"), Open Library and Syndetics as cover sources, and
  Babelthèque (French; reviews and citations, not covers), Novelist Select and LibraryThing for Libraries as other content
  ([enhanced_content.pref](https://raw.githubusercontent.com/Koha-Community/Koha/main/koha-tmpl/intranet-tmpl/prog/en/modules/admin/preferences/enhanced_content.pref)) [V1].
- **Electre** (`GetBlob.ashx?Ean=…&Size=Original` style URLs), **Dilicom** (FEL notices; the Hub that carries eBook notices and
  cover links between French e-distributors and resellers) and **Tite-Live/ePagine** (file naming `{EAN}_1_75.jpg`): all [M].
  They are subscriber services; a reseller in mode 2 already receives covers through one of them.

### 2.5 BnF

All [M]: resolve the ISBN to an ARK with the SRU API (`catalogue.bnf.fr/api/SRU?...query=bib.isbn adj "{isbn}"`), then request
`https://catalogue.bnf.fr/couverture?&appName=NE&idArk={ark}&couverture=1`. Returned size, the placeholder behaviour, the origin of
the images (publisher deposit versus a supplier) and whether the Licence Ouverte that covers BnF metadata extends to these images
are **not verified** — I believe it does not [M, low]. Gallica holds public-domain scans only, so it does not help for in-print eBooks.

### 2.6 Publisher and bookseller pages

Generic extraction order (my recommendation, standard practice [M]): JSON-LD `Book`/`Product` `image` (string, array or
`ImageObject.url`) > `og:image` (+ `og:image:width/height`) > `twitter:image` > `link rel="image_src"` > `itemprop="image"` >
largest `<img>` whose `alt` matches the title. Then strip CDN resize hints to reach the original: Amazon `._SX300_.`, WordPress
`-300x450`, Shopify `_600x`, Cloudflare `/cdn-cgi/image/width=…/`, query `?w=&h=` [M].

French bookseller patterns — **all [M, low], none checked live; treat as hints, not specifications**:

| Site | Hint | Derivable from EAN alone? |
|---|---|---|
| Decitre, Furet du Nord (same group/platform) | `products-images.di-static.com/image/{slug}/{EAN}-475x500-1.jpg` | Probably not (slug) |
| Leslibraires.fr, Place des Libraires, many indie shops (ePagine/Tite-Live) | `images.epagine.fr/{3 digits}/{EAN}_1_75.jpg` | Possibly |
| Cultura | `cdn.cultura.com/cdn-cgi/image/width=…/media/pim/TITELIVE/…{EAN}_1_75.jpg` | Possibly |
| Fnac | `static.fnac-static.com/multimedia/…/{internal id}/…` | No — product page needed; strong anti-bot |
| IBS (Italy) | `www.ibs.it/images/{EAN}_0_536_0_75.jpg` | Possibly |
| Casa del Libro (Spain) | `imagessl{d}.casadellibro.com/a/l/t7/{dd}/{EAN}.jpg` | Possibly |

Publisher press areas ("espace presse", "espace libraires") sometimes offer a "couverture HD" download [M]; this is the only
public place where print-resolution front covers are routinely found, and it is per-publisher manual work.

### 2.7 Aggregators and open projects

- **bookcover-api**: scrapes Goodreads and returns an `i.gr-assets.com` URL; rate-limited with 429
  ([README](https://raw.githubusercontent.com/w3slley/bookcover-api/main/README.md)) [V1]. Goodreads images are Amazon's. Not usable.
- **Coce** (MIT): a Redis cache of cover *URLs* from Google Books, Amazon, Open Library and ORB, queried in a caller-given provider
  order [V1]. Good prior art for our provider cascade; it caches URLs by default, with image caching as an option [V1].
- **Inventaire** (French association, AGPL server, Wikidata-based) [V1 for licence]; its image coverage and API terms were not checked.
- **BookWyrm** uses Open Library `covers` ids with `/b/id/{id}-L.jpg`
  ([connector](https://raw.githubusercontent.com/bookwyrm-social/bookwyrm/main/bookwyrm/connectors/openlibrary.py)) [V1].

## 3. Resolution reality check

Required pixels [est.]: 110 x 178 mm front = 1299 x 2102 px at 300 ppi (1370 x 2173 with 3 mm bleed); 148 x 210 mm = 1748 x 2480 px.
The sibling note `docs/research/notes/print-requirements.prepress.md` found printers asking for 300 dpi, with a 250 dpi floor at
Saxoprint, and proposes preflight thresholds 300 target / 250 warning / 150 error.

| Source image (long side) | ppi on 178 mm front | ppi on 210 mm front | Upscale to reach 300 / 250 / 150 ppi (178 mm) |
|---|---|---|---|
| 500 px (OL `-L` [V1]; Amazon legacy `L`, Decitre box [M]) | 71 | 60 | 4.2x / 3.5x / 2.1x |
| 800 px (typical og:image [M]) | 114 | 97 | 2.6x / 2.2x / 1.3x |
| 1200 px (good publisher site image [M]) | 171 | 145 | 1.75x / 1.5x / none |
| 1500 px (Amazon `hiRes` [M]) | 214 | 181 | 1.4x / 1.2x / none |
| 2560 px (retail eBook cover master [M]) | 365 | 310 | none |

Consequences: (1) every public ISBN API result fails even a 150 ppi error threshold on its own; (2) a 2x model is not enough for
500 px inputs — 4x is needed, or 2x plus an honest "print-at-home quality" label; (3) the EPUB/feed cover usually needs no upscaling.
Actual sizes returned by live requests were **not measured** this session.

## 4. In-browser, image-only super-resolution

| Option | Licence (verified) | Model facts | Browser route | Fit for covers |
|---|---|---|---|---|
| Real-ESRGAN `realesr-general-x4v3` | BSD-3-Clause ([LICENSE](https://raw.githubusercontent.com/xinntao/Real-ESRGAN/master/LICENSE)) [V1] | "tiny small model", x4, also x1–x3, tunable denoise ([model zoo](https://raw.githubusercontent.com/xinntao/Real-ESRGAN/master/docs/model_zoo.md)) [V1]; about 5 MB [M] | export to ONNX, run with ONNX Runtime Web | Best size/quality candidate |
| Real-ESRGAN `RealESRGAN_x4plus` | same | general x4, full RRDB net [V1]; about 64 MB [M] | ONNX; WebGPU realistically required | Strong but heavy; hallucinates texture |
| `RealESRGAN_x4plus_anime_6B` | same | 6 RRDB blocks, "smaller network" [V1]; about 18 MB [M] | ONNX | Good on flat illustration covers |
| `RealESRNet_x4plus` | same | MSE loss, "over-smooth" [V1] | ONNX | Safer for type: no GAN artefacts |
| SwinIR | Apache-2.0 ([README](https://raw.githubusercontent.com/JingyunLiang/SwinIR/main/README.md)) [V1] | classical x4: 11.9 M params, 0.539 s per 256 px tile on an RTX 2080 Ti [V1]; real-world M and L variants | ONNX; transformer, slow on wasm | Quality reference, too slow for CPU |
| Swin2SR | Apache-2.0 ([LICENSE](https://raw.githubusercontent.com/mv-lab/swin2sr/main/LICENSE)) [V1] | has a **compressed-input (JPEG) SR** task and lightweight x2 ([README](https://raw.githubusercontent.com/mv-lab/swin2sr/main/README.md)) [V1] | listed in transformers.js, `image-to-image` pipeline, WebGPU option ([README](https://raw.githubusercontent.com/huggingface/transformers.js/main/README.md)) [V1] | Matches our input (small JPEGs) |
| waifu2x (nunif) | MIT ([LICENSE](https://raw.githubusercontent.com/nagadomi/nunif/master/LICENSE)) [V1] | art and photo models, 2x/4x; ONNX zips in releases [V1] | **already runs in-browser** on onnxruntime-web ([unlimited:waifu2x](https://raw.githubusercontent.com/nagadomi/nunif/master/waifu2x/unlimited_waifu2x/README.md)) [V1] | Proven; author calls wasm speed "very slow" [V1] |
| Real-CUGAN | MIT ([LICENSE](https://raw.githubusercontent.com/bilibili/ailab/main/Real-CUGAN/LICENSE)) [V1] | anime-oriented | ONNX conversion needed [M] | Illustration covers only |
| UpscalerJS | MIT ([README](https://raw.githubusercontent.com/thekevinscott/UpscalerJS/main/README.md)) [V1] | TensorFlow.js; `esrgan-slim` 2x/3x/4x in browser, 8x Node only; built-in patch processing ([model README](https://raw.githubusercontent.com/thekevinscott/UpscalerJS/main/models/esrgan-slim/README.md)) [V1] | turnkey JS | Fastest to integrate; not Rust/wasm |
| Lanczos3 + unsharp (no AI) | n/a | deterministic, a few KB | pure Rust in wasm | Mandatory fallback and baseline |

Notes:
- Runtime: ONNX Runtime is MIT ([LICENSE](https://raw.githubusercontent.com/microsoft/onnxruntime/main/LICENSE)) [V1]. nunif's
  README (note dated 2026-08) advises against fp16 weights on WebGPU because `shader-f16` support is limited, and reports fp16 on
  wasm as practically identical (PSNR 65+) at half the download [V1]. Plan: fp32 for WebGPU, fp16 for the wasm fallback.
- Model file sizes above are [M]; none was downloaded. Timings on consumer hardware are unknown — **benchmark before promising UX**.
- Licence traps [M]: community models (OpenModelDB etc.) are often CC BY-NC; the training sets behind the official weights
  (DIV2K, Flickr2K, OST) carry research-oriented terms even though code and weights are BSD/Apache. If the project trains its own
  model ("that can be arranged" in the brief), train on covers it has rights to or on synthetic covers.
- Covers are type-heavy. GAN upscalers invent plausible but wrong glyph shapes on small text (author lines, blurbs) [M]. Prefer
  PSNR-oriented variants, cap AI upscaling at 4x, and show the user a before/after at print scale.

## 5. Detecting placeholder / "image not available" covers

| Layer | Check | Evidence |
|---|---|---|
| Protocol | Open Library: always send `?default=false`, treat 404 as absent | [V2] |
| Protocol | DNB: HEAD request, HTTP error = absent | [V1] |
| Protocol | Syndetics: read `index.xml` for available sizes before fetching | [V1] |
| Bytes | Hash blocklist: Google dummy images (two MD5 digests in calibre) | [V1] |
| Bytes | Known asset URLs: Amazon `…/images/G/01/x-site/icons/no-img-sm.gif` (referenced by Koha) | [V1] |
| Bytes | 1x1 GIF / body under about 100 bytes (Amazon legacy, LibraryThing, Content Cafe with `erroroverride=1`) | [M] |
| Pixels | Decode and reject: either side under 100 px; aspect ratio outside about 0.55–0.85 (w/h); luminance standard deviation near zero; fewer than about 16 distinct colours after quantisation | my heuristics [M] |
| Pixels | Perceptual hash (dHash/pHash) against a gallery of collected placeholders per provider, Hamming distance threshold | standard practice [M] |
| Semantics | 3D packshots and photos: uniform white corners plus non-rectangular content; promotional "bandeau" on French covers; retailer watermarks | my heuristics [M] |
| Cross-check | Two providers returning near-identical pHash raises confidence that it is the right edition | [M] |

Keep the blocklist as test fixtures (hashes only, not the images). Two designs exist in the wild: query every provider, discard
dummies and keep the largest (calibre's approach, as I remember it [M]), or query providers in a fixed order and stop at the first
hit (Coce [V1]; VuFind's `coverimages` list works the same way as far as I remember [M]). The first gives better resolution;
the second costs fewer requests.

## 6. Spines and full wraparound covers

| Candidate source | Finding | Confidence |
|---|---|---|
| ONIX 3.0 List 158 | Codes exist: 01 `FrontCover`, 02 `BackCover`, 03 `CoverPack`, 27 `CoverThumbnail`, **29 `FullCover`**, 45 `CoverHoldingImage`, **51 `CoverFlap`**, 54 `PageEdgeDecoImage`, 55 `EndpaperDecoImage` ([im_onix codelist](https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-158.yml)) | [V1] |
| French feeds (Dilicom, Electre, e-distributors) | Front cover, sometimes back cover ("4e de couverture"); I know of no feed that carries code 29 | [M, low] |
| Publisher print files ("couverture à plat": plat 1 + dos + plat 4) | Exist for every printed book, held by publisher and printer, not public | [M] |
| Self-publishing / POD authors (KDP, BoD, Lulu, Bookelis…) | The author owns the wraparound PDF — an **upload path** in standalone mode gives a real spine | [M] |
| Publisher press kits | Front cover HD common; flat full cover rare | [M, low] |
| Internet Archive scans | Front (and often back) cover photographed; spine not systematically captured | [M, low] |
| BnF Gallica | Binding views ("plat supérieur", "dos", "plat inférieur") exist for heritage items; public domain only | [M] |
| Retailer "3D" packshots | Synthesised from the front cover with a generic spine; not real spine data | [M] |
| Second-hand marketplaces | Real photos including spines, per listing, unstructured, seller's copyright on top of the cover's | [M] |
| Research datasets | Shelf/spine sets exist for detection and OCR (Stanford mobile visual search book spines; "Smart Library" shelf reading; Roboflow/Kaggle spine sets); small, not keyed by ISBN | [M] |
| Cover datasets | Amazon-derived cover corpora (e.g. the Uchida-lab book cover dataset, about 207k covers at 224 px) — fronts only, low resolution | [M] |

Conclusions: (1) there is nothing to scrape; (2) many eBooks are digital-only and never had a spine; (3) the shelf datasets are
useful only to **learn conventions** for the generator (title, author, publisher mark order; colour continuity with the front);
(4) reading direction matters: French and most continental spines read bottom-to-top, English-language spines top-to-bottom [M] —
make it a per-locale default; (5) accept an optional flat-cover upload and crop the spine by measured width.

## 7. Legality of acquisition (summary; see `legal-covers-reseller.reseller.md` for the rights analysis)

All [M] — no legal source was opened this session.

- A printed card **reproduces** the cover; hotlinking doctrines (Svensson, GS Media) do not help. Acquisition legality and
  reproduction rights are separate questions; solving the first does not solve the second.
- Scraping risk in France/EU has three layers: site terms (CJEU *Ryanair v PR Aviation*, C-30/14: contract terms bind even
  where no database right exists), the sui generis database right (CPI L. 342-1: substantial or repeated systematic extraction),
  and technical measures/anti-bot circumvention. The text-and-data-mining exception (CPI L. 122-5-3) covers analysis, not
  republishing the image.
- Per-request retrieval of one image at a user's explicit instruction (user pastes a product URL) is far lower risk than
  crawling by EAN, but it does not grant a print right.
- API terms that matter: Amazon (no offline use), Google (no durable caching — consistent with VuFind's behaviour [V1]),
  subscription services (catalogue display only).
- Practical rule: in reseller mode the reseller supplies the image and warrants the right; in standalone mode the user supplies
  the EPUB/image and declares the right; lookups are a convenience with an explicit confirmation step.

## 8. Recommended acquisition cascade for the `covers` crate

1. Reseller-supplied cover (ONIX/feed URL or bytes) — mode 2.
2. EPUB upload, cover extracted in the browser (zip + OPF `properties="cover-image"` / EPUB 2 `<meta name="cover">`) — mode 1.
3. User-supplied image file, or flat full cover (gives the spine too).
4. Open Library: `/b/isbn/{isbn}.json` → dimensions → `/b/id/{id}.jpg` (original), `?default=false`, follow redirects,
   identifying User-Agent, at most a few requests per second, cache by cover id.
5. Google Books `viewapi` → `zoom=0` then `zoom=1`, MD5 dummy check, no durable cache.
6. BnF and DNB as national fallbacks once verified live.
7. User-pasted product page URL → JSON-LD/og:image extraction, server-side, one request.
8. Never: Amazon, Goodreads-derived APIs, subscription services without a contract.

Score candidates by pixel area, aspect ratio, placeholder checks and cross-provider pHash agreement; record provenance (provider,
URL, fetch time, dimensions) in the job so the preflight report can state the effective ppi and its origin. Browser-side fetches
need CORS headers to read pixels; I did not verify any provider's CORS behaviour, so plan for a small server-side fetch proxy
(not behind CloudFront for Open Library).

## Sources

All fetched and read on 2026-11-12 (HTTP 200). No web-search result was available.

Open Library (internetarchive/openlibrary):
- https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/code.py
- https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/config.py
- https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/utils.py
- https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/README.md
- https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/nginx.conf
- https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/covers_nginx.conf

Cover-provider implementations (read for facts only; GPL/AGPL code, do not copy):
- https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/ebooks/metadata/sources/openlibrary.py
- https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/ebooks/metadata/sources/google.py
- https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/ebooks/metadata/sources/amazon.py
- https://raw.githubusercontent.com/citronalco/calibre-dnb/master/__init__.py
- https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/OpenLibrary.php
- https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Google.php
- https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/LibraryThing.php
- https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Syndetics.php
- https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/ContentCafe.php
- https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Buchhandel.php
- https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Orb.php
- https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Deprecated.php
- https://raw.githubusercontent.com/vufind-org/vufind/dev/config/vufind/config.ini
- https://raw.githubusercontent.com/Koha-Community/Koha/main/koha-tmpl/opac-tmpl/bootstrap/en/modules/opac-detail.tt
- https://raw.githubusercontent.com/Koha-Community/Koha/main/koha-tmpl/intranet-tmpl/prog/en/modules/admin/preferences/enhanced_content.pref
- https://raw.githubusercontent.com/fredericd/coce/master/README.md
- https://raw.githubusercontent.com/w3slley/bookcover-api/main/README.md
- https://raw.githubusercontent.com/bookwyrm-social/bookwyrm/main/bookwyrm/connectors/openlibrary.py
- https://raw.githubusercontent.com/inventaire/inventaire/main/README.md
- https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-158.yml

Super-resolution:
- https://raw.githubusercontent.com/xinntao/Real-ESRGAN/master/README.md
- https://raw.githubusercontent.com/xinntao/Real-ESRGAN/master/LICENSE
- https://raw.githubusercontent.com/xinntao/Real-ESRGAN/master/docs/model_zoo.md
- https://raw.githubusercontent.com/JingyunLiang/SwinIR/main/README.md
- https://raw.githubusercontent.com/JingyunLiang/SwinIR/main/LICENSE
- https://raw.githubusercontent.com/mv-lab/swin2sr/main/README.md
- https://raw.githubusercontent.com/mv-lab/swin2sr/main/LICENSE
- https://raw.githubusercontent.com/nagadomi/nunif/master/README.md
- https://raw.githubusercontent.com/nagadomi/nunif/master/LICENSE
- https://raw.githubusercontent.com/nagadomi/nunif/master/waifu2x/README.md
- https://raw.githubusercontent.com/nagadomi/nunif/master/waifu2x/unlimited_waifu2x/README.md
- https://raw.githubusercontent.com/thekevinscott/UpscalerJS/main/README.md
- https://raw.githubusercontent.com/thekevinscott/UpscalerJS/main/LICENSE
- https://raw.githubusercontent.com/thekevinscott/UpscalerJS/main/models/esrgan-slim/README.md
- https://raw.githubusercontent.com/huggingface/transformers.js/main/README.md
- https://raw.githubusercontent.com/microsoft/onnxruntime/main/LICENSE
- https://raw.githubusercontent.com/bilibili/ailab/main/Real-CUGAN/README_EN.md
- https://raw.githubusercontent.com/bilibili/ailab/main/Real-CUGAN/LICENSE

Internal cross-reference: `docs/research/notes/print-requirements.prepress.md` (printer resolution minimums),
`docs/research/notes/legal-covers-reseller.reseller.md` (rights analysis).

## Could not verify

Nothing in this list was checked this session; each item needs a web search or a live request before it drives code or a decision.

1. **Every terms-of-use statement**: Open Library's stance on cover reuse and its documented "100 requests / 5 minutes" rule;
   Google Books API terms, branding rules and quota; Amazon Associates / Product Advertising API terms and the API's 2026 status;
   LibraryThing key availability, daily cap and commercial-use rule; MVB/VLB, Electre, Dilicom, Tite-Live, ORB, Syndetics and
   Content Cafe licence scopes and prices.
2. **BnF cover service**: URL pattern, returned size, placeholder behaviour, image origin, reuse licence.
3. **Every French / Italian / Spanish bookseller URL pattern** in section 2.6, and their robots.txt / CGU / anti-bot posture.
   Dutch and German retailer patterns were not researched at all.
4. **Actual pixel dimensions** returned by any provider for real French ISBNs, and the share of French eBook ISBNs each provider
   covers. A hit-rate test on a sample of 200–500 French eBook EANs is the missing experiment.
5. **CORS headers** of Open Library, archive.org, Google Books, BnF and DNB image responses.
6. **Typical retail eBook cover sizes** (1400–2560 px) and distributor minimums (Apple, Kobo, Amazon).
7. **Super-resolution model file sizes, ONNX availability and browser timings**; licence status of pretrained weights versus
   their training data; existence and licence of ready-made ONNX exports on Hugging Face.
8. **Spines**: whether any French ONIX feed carries List 158 code 29; whether Internet Archive scan metadata marks spine images;
   names, sizes and licences of the spine/shelf research datasets; the spine reading-direction convention by country.
9. **Legal points** in section 7 (case references and CPI article numbers are from memory).
10. Services named in the brief but not researched for lack of search budget: Inventaire's cover API, Babelio, Goodreads' own
    terms, publisher-specific press-kit practices, "3D cover" retailer features.
