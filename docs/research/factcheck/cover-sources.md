# Fact-check report — cover-sources

_Produced by an adversarial fact-checker on 2026-11-12. Read this next to the matching document: where that document's status is still "draft", these corrections were NOT applied._

# Fact-check of `/home/user/ebook-cadeau/docs/research/02-cover-sources.md`

## Method and its limit

**No web search ran.** All 8 queries I sent were refused: the shared budget of 200 searches is spent. I did not work around it.

What I could check:
- **31 primary files on `raw.githubusercontent.com`**, the only host WebFetch reaches. Most are the document's own sources, re-read for misreadings. Four are independent of it: Jonix, the python-amazon-paapi changelog, VuFind's `OpenLibrary.php`, Open Library's compose file.
- **Sibling notes in the repo**, for contradictions.

What nobody could check: provider terms, quotas, prices and every legal authority. `www.rfc-editor.org`, `openlibrary.org`, `developers.google.com`, `en.wikipedia.org` and `webservices.amazon.com` do not resolve; they would have to be allowed in the session's network settings. Those claims are UNSUPPORTED below. Where I add "matches my recall", that is my memory, not evidence.

Source keys ([J1], [O2], …) are listed at the end.

## Result in one paragraph

Three claims are wrong, and one of them drives a design decision: ONIX does have a spine-image code. One claim is outdated (Amazon). The technical facts read from source code hold up, with a few over-readings. Every quota, every provider's terms and all case law stay unverified.

## The 20 load-bearing claims

1. **"ONIX List 158 has no spine-only code; 29 is the only code with a spine"** (finding 10, §5, opening table) — **WRONG.**
   - Code-list Issue 75 has 56 "Spine image" ("2D, portrait orientation") and 57 "Spine panorama image" [J1].
   - The im_onix file the document used stops at 55 plus 99; it is a stale copy [J2].
   - Doc 07 §3.3 and `notes/isbn-databases.trade.md` already say 56 exists, so doc 02 contradicts the document it defers to.
   - Corrected: no public source exposes spines by ISBN, but ONIX has slots for them (29, 56, 57); whether any French feed fills them is unknown. Implication 43 must ingest 56 and 57.

2. **Spine direction "bottom-to-top for French, German, Spanish, Italian and Dutch"** (implication 45) — **WRONG for Dutch.**
   - `notes/spine-design.conventions.md` lists the Netherlands and Flanders as top-to-bottom, and German as majority bottom-to-top but not uniform.
   - That note also says to choose from the book's language or country, not the site locale.
   - Both notes are from memory; mine agrees with the conventions note. Remove Dutch and mark the rest unverified.

3. **"Every public ISBN cover API result fails even the 150 ppi threshold"** (§3.2) and "about 500 px on the long side" (finding 1) — **WRONG; contradicted by the document's own table.**
   - 500 px is only the Open Library `-L` box [O3]; the no-suffix URL returns the uncapped original [O2].
   - Google documents `extraLarge` at about 1280 px wide [G1], which the document itself rates at 295 ppi.
   - Corrected: default sizes are thumbnails; larger originals exist for some titles, hit rate unmeasured.

4. **Open Library URL grammar and behaviour** — **CONFIRMED** [O2][O3][V2].
   - Sizes, no-suffix original, `default=false` giving 404, the `.json` 302 to the id record, the two cache lifetimes and the archive.org redirects all match the code.
   - One error: implication 11 says the JSON route does not strip hyphens. It does, through `_query`, for `b/isbn`.

5. **"`Access-Control-Allow-Origin: *` on every response [V1]"** — **PARTLY.**
   - The code does this [O1].
   - Not shown: that production runs this ASGI app. The compose file names no app module [O6], and the nginx in front adds no CORS header [O4].
   - The 302 targets on archive.org are unverified, and they cover every size of archived covers with an id of 8,000,000 or more. Browser-side byte reads (implication 30) may fail for most recent titles. An `<img>` preview is unaffected.

6. **Open Library rate limits** — nginx figures **CONFIRMED**; documentation figure **UNSUPPORTED.**
   - `cover_limit` is 400 r/m [O5] with burst 400, the 429 body, and 444 for Bytespider and CloudFront [O4]; the identifying-user-agent map is as described [O5].
   - Missing from the document: the limit applies only without a session cookie, and non-identifying crawlers share a separate 150 r/s bucket [O5].
   - "100 requests per 5 minutes for ISBN lookups" matches my recall only.

7. **"No licence on Open Library images"; its terms; preview risk "Low–Medium"** — **UNSUPPORTED, and the rating is under-argued.**
   - §6.4 requires that the source posted the cover with the rightholder's consent.
   - Open Library covers are uploads and imports with no licence and a takedown list [O3]. On the document's own test that is closer to "unknown provenance" with the *GS Media* presumption.
   - Rate it Medium until counsel answers question 5(b).

8. **Google Books** — technical facts **CONFIRMED**; quota and terms **UNSUPPORTED**; one **PARTLY**.
   - Confirmed: six `imageLinks` sizes [G1], the `jscmd=viewapi` endpoint and zoom rewrite [V1], calibre's two MD5 digests and zoom order [C1].
   - Unsupported: 1,000 requests a day, caching and attribution terms, "larger sizes only from `volumes/{id}`".
   - "VuFind deliberately does not cache Google [V1]" is an inference: `Google.php` does not set `cacheAllowed`, `OpenLibrary.php` sets it true [V1][V2], and the config comment names no service [V3].

9. **BnF cover URL and placeholder lengths 4566 and 4658** — **CONFIRMED** in one code base [B1]. Thumbnail size, origin of the images and reuse terms: **UNSUPPORTED**.

10. **DNB cover URL and HEAD probe** — **CONFIRMED** in one code base [D1]. "Images come from MVB" is read off the path only; the file says nothing about origin or terms: **UNSUPPORTED**.

11. **"PA-API 5.0 is being replaced by the Creators API"** — **CONFIRMED, wording OUTDATED.**
    - Open Library's importer has `AmazonCreatorsAPI`, "replacement for AmazonAPI (PA-API 5.0)" [O7].
    - Independently, python-amazon-paapi deprecated PA-API on 2026-01-29 and removed it on 2026-09-04 [A1].
    - Corrected: "replaced during 2026". Amazon's shutdown date is in neither file.
    - "Variants can include a spine [V1]" is a code comment in Open Library's importer, not Amazon documentation.
    - Associates terms (offline use, 24-hour cache): UNSUPPORTED.

12. **Kobo "average 800:1224px"** — quote **CONFIRMED** [K1]; inference **PARTLY.**
    - The text is undated and is a store-wide average, not a measurement of French EPUBs.
    - "(175 ppi)" should not drive design before the sample run.

13. **"LCP forbids encrypting cover images"** — **CONFIRMED**, LCP 1.0 revision 5, §2.1 [L1].
    - The rule covers only resources carrying the `cover-image` property. A cover marked only by `<meta name="cover">` is not exempt.
    - It says nothing about Adobe, Kobo or Kindle protection.

14. **TDMRep mechanics** — **CONFIRMED** [T1][T2], with two corrections.
    - Confirmed: values 0 and 1, anything else unset; file, then header, then HTML meta, then EPUB or PDF metadata; an absent value never resets; HTML meta does not reach images.
    - "HTTP headers, described as the preferred technique" — **UNSUPPORTED**: not in the spec, the README or the header technique note [T1][T3][T4].
    - The spec is a Community Group Final Report (`CG-FINAL`), not a "draft".

15. **TDMRep adopted by Hachette Livre and Média-Participations; "the TDM route is closed [V1]"** — list **CONFIRMED** [T5]; conclusion **PARTLY.**
    - The list is self-declared and undated, says "most websites", and no site was probed.
    - Corrected: "reported reserved; check per request".
    - The house names the document adds (Hachette.fr, Grasset and others) were not re-confirmed.

16. **DSM art. 4 and recital 18 wording** — **CONFIRMED** against the W3C extract [T6]. CPI R122-28, décret 2022-928 and ordonnance 2021-1518 — **UNSUPPORTED**; matches my recall, and no one in the project has read the article.

17. **robots.txt handling "as RFC 9309 says"** — **PARTLY.**
    - The cited source attributes everything to Google's guidance and never mentions RFC 9309 [R1].
    - From my recall, the RFC has no 429 carve-out (that is Google's practice), and the 500 KiB minimum is in the RFC itself.
    - Delete "The RFC says the same in its own words". The implemented behaviour is the conservative one and can stay.

18. **Linking versus copying; "a server-side proxy is a re-host"** — **UNSUPPORTED** in this session.
    - Case identifiers and holdings match my recall (*Svensson*, *BestWater*, *GS Media*, *Renckhoff*, *VG Bild-Kunst*, *VCAST*).
    - The proxy conclusion and "no DSA art. 4 shelter" are the author's application, with no authority.
    - The *Renckhoff* and *VCAST* links are copied from sibling notes, unopened.

19. **Database-right, French, German and Dutch case law, CNIL, GPAI Code** — **UNSUPPORTED**, all of it.
    - Identifiers and dates match my recall for the CJEU database cases, *Leboncoin*, *Ryanair v Opodo*, *Bluetouff*, Cass. com. 26 June 2024, LG and OLG Hamburg, LG München I, the GPAI Code date and the CNIL sanctions.
    - Do not rely on the holdings of OLG Hamburg, *DPG Media v HowardsHome* or the CNIL sheet's robots.txt wording without reading them.
    - *Zulma v Elior* rests on one law-firm blog (doc 07 says so).

20. **Super-resolution licences and print arithmetic** — **CONFIRMED** where checkable.
    - Real-ESRGAN BSD-3 [X1], SwinIR and Swin2SR Apache-2.0 [X2][X3], nunif MIT [X4]; nunif's 2026-08 fp16 note and "very slow" [X5].
    - Those licence files cover code; none mentions weights.
    - Model sizes, timings and "GAN models damage small type": UNSUPPORTED.
    - I recomputed the pixel and ppi tables; they are correct.

## Omissions

1. **ONIX carries usage data the document ignores** (`notes/isbn-databases.trade.md` §2, Jonix links there; [J1]).
   - List 161: "linkable" lets the recipient embed the URL; "downloadable" requires it to host its own copy. This decides link-versus-store in reseller mode.
   - `EpubUsageConstraint` type 11 (text and data mining) on a resource version, since ONIX 3.1.1: an opt-out channel missing from §6.3 and §7.4.
   - Also List 160 code 01 (required credit), List 154 audience, territory, embargo dates, and List 158 code 60 "LLM training data".
2. **Dilicom Distrimage**, the French trade cover service: free inside bookshop software, "pour tout autre usage … prendre contact" (doc 07 §3.3, https://www.syndicat-librairie.fr/le_service_distrimage_de_dilicom). "FEL à la demande" returns no image. Doc 02 names only FEL and the Hub.
3. **ORB is run by Decitre** (trade note, from memory). Doc 02 says "operator not checked" while denylisting Decitre's CDN.
4. **The gift buyer usually holds no EPUB.** An ACSM is a token, and Kindle and Kobo deliver into an account (`notes/ereader-ux.ecosystems.md` §4.3). Cascade step 3 mostly serves authors, publishers and DRM-free buyers, not the typical gifter.
5. **Feed-level TDM reservations exist**: Penguin Random House Germany in ONIX records, Bookwire and Edigita in EPUB and ONIX [T5]. Reseller-mode analysis (palette, font class) needs the same check as web fetches.
6. **The adopters list links a tracker of French publishers** (Mind Media, https://www.mind.eu.com/media/data/ia-generative-quels-editeurs-francais-bloquent-les-robots-dopenai-et-google-lesquels-ont-adopte-le-protocole-tdmrep/). The document says it was not opened; it answers open question 7.
7. **"Publisher pages only" is enforced only by a retailer denylist.** Any host not listed passes, including blogs and unlisted retailers. It needs an allow-list of publisher domains.
8. **Sources missing from the table** (my recall, unverified):
   - Apple Books / iTunes Search artwork and Kobo's image CDN. Both are absent from the denylist although Fnac/Kobo is a target reseller.
   - Amazon's current media hosts.
   - OPDS feeds from e-distributors.
   - ISBNdb, DILVE, CB, Informazioni Editoriali and Nielsen appear in the trade note but not in doc 02.
9. **Newer opt-out signals a fetcher will meet** (my recall, unverified): Cloudflare "Content Signals" lines in robots.txt, RSL licence directives, IETF AIPREF drafts, the Commission's consultation on opt-out protocols.
10. **"Public domain = artist dead more than 70 years"** (step 5) is too simple (my recall, unverified): collective works run from publication, a cover combines several contributions, and French moral rights do not expire.
11. **No contrary court decision was found because no search could run.** Still untracked: C-250/25, any further appeal in *LAION* (OLG Hamburg) or *GEMA v OpenAI*.

## Statements to downgrade to "unverified"

- "Open Library sends `Access-Control-Allow-Origin: *` [V1]" → in repository code; production and the archive.org hop unverified.
- "The TDM route is closed for Hachette Livre's sites [V1]" → self-declared list, no live check.
- "HTTP headers are the preferred TDMRep technique".
- "Amazon variants can include back cover and spine [V1]" → a third party's code comment.
- "VuFind deliberately does not cache Google [V1]" → inference.
- "DNB images come from MVB".
- "The RFC says the same in its own words" → delete.
- "Kobo covers are about 800 x 1224 px, 175 ppi" as a planning figure.
- Every [V1\*] and [V2\*] item: *Renckhoff*, *VCAST*, *Zulma v Elior*, Artprice, *Kelly v Arriba*, the LibraryThing and Goodreads links, Saxoprint 250 dpi, BnF Licence Ouverte, the ONIX "promotional use" quote (a BISG slide deck).
- All quotas: Open Library 100 per 5 minutes, Google 1,000 a day, LibraryThing 1,000 a day.
- All provider terms in §2 and §6.5; "no French feed carries code 29".
- The retailer URL patterns behind the denylist.
- Model sizes; "10 mm spine is roughly 150–200 pages"; spine direction by country.
- Everything in §6 that design rules rest on: R122-28, the CNIL robots.txt and CAPTCHA rule, OLG Hamburg, *DPG Media*, *GEMA v OpenAI*, CPI L136-1 to L136-4, Cass. com. 26 June 2024.

## Sources fetched this session

- [J1] https://raw.githubusercontent.com/zach-m/jonix/master/jonix-common/src/main/java/com/tectonica/jonix/common/codelist/ResourceContentTypes.java
- [J2] https://raw.githubusercontent.com/immateriel/im_onix/master/data/codelists/codelist-158.yml
- [O1] https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/asgi_app.py
- [O2] https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/code.py
- [O3] https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/coverstore/config.py
- [O4] https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/covers_nginx.conf
- [O5] https://raw.githubusercontent.com/internetarchive/openlibrary/master/docker/nginx.conf
- [O6] https://raw.githubusercontent.com/internetarchive/openlibrary/master/compose.production.yaml
- [O7] https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/core/vendors.py
- [A1] https://raw.githubusercontent.com/sergioteula/python-amazon-paapi/master/CHANGELOG.md
- [G1] https://raw.githubusercontent.com/googleapis/google-api-go-client/main/books/v1/books-api.json
- [C1] https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/ebooks/metadata/sources/google.py
- [V1] https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/Google.php
- [V2] https://raw.githubusercontent.com/vufind-org/vufind/dev/module/VuFind/src/VuFind/Content/Covers/OpenLibrary.php
- [V3] https://raw.githubusercontent.com/vufind-org/vufind/dev/config/vufind/config.ini
- [B1] https://raw.githubusercontent.com/inventaire/inventaire/main/server/data/bnf/get_bnf_entry_from_isbn.ts
- [D1] https://raw.githubusercontent.com/citronalco/calibre-dnb/master/__init__.py
- [K1] https://raw.githubusercontent.com/kobolabs/epub-spec/master/README.md
- [L1] https://raw.githubusercontent.com/readium/lcp-specs/master/releases/lcp/latest.md
- [T1] https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/spec/index.html
- [T2] https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/techniques/technique-html-meta.md
- [T3] https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/README.md
- [T4] https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/techniques/technique-http-headers.md
- [T5] https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/docs/adopters.md
- [T6] https://raw.githubusercontent.com/w3c/tdm-reservation-protocol/main/docs/eu-act-extracts.md
- [R1] https://raw.githubusercontent.com/Smerity/texting_robots/main/README.md
- [X1] https://raw.githubusercontent.com/xinntao/Real-ESRGAN/master/LICENSE
- [X2] https://raw.githubusercontent.com/JingyunLiang/SwinIR/main/LICENSE
- [X3] https://raw.githubusercontent.com/mv-lab/swin2sr/main/LICENSE
- [X4] https://raw.githubusercontent.com/nagadomi/nunif/master/LICENSE
- [X5] https://raw.githubusercontent.com/nagadomi/nunif/master/waifu2x/unlimited_waifu2x/README.md

Repo files used for contradictions: `/home/user/ebook-cadeau/docs/research/notes/isbn-databases.trade.md`, `/home/user/ebook-cadeau/docs/research/notes/spine-design.conventions.md`, `/home/user/ebook-cadeau/docs/research/notes/ereader-ux.ecosystems.md`, `/home/user/ebook-cadeau/docs/research/07-legal-covers-and-reseller-rights.md`.

The document was not edited.
