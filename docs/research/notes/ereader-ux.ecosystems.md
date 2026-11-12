# From paper to e-reader — reseller ecosystems, existing gift flows and the technical rails

Research notes, 2026-11-12. Angle: how eBook sellers deliver today, whether they let a buyer gift a *specific* eBook, and which open rails (OPDS, LCP, ACSM, watermark, tokens, deep links) a printed "eBook cadeau" card can ride on.

## Method caveat — read first

**No web search succeeded in this session.** The shared WebSearch budget (200 calls per turn, all agents) was already spent: 6 attempts, 6 refusals. WebFetch is blocked by the sandbox allowlist. The only documentation host reachable was `raw.githubusercontent.com`, so I fetched primary documents there (20 files: the OPDS and Readium LCP specifications, EDRLab / KOReader / calibre / Standard Ebooks documentation). Consequences:

- **Section 4 (technical rails) is verified against primary specifications** — tagged [V1], or [V2] where two separate documents agree.
- **Sections 2 and 3 (what each retailer does) are almost entirely from memory** — tagged [M], with my own recall confidence (high / medium / low). Retail gift features change often; treat every [M] row as a hypothesis to re-check, not a finding.
- Four facts are reused from sibling notes in this repo, tagged **[V1-sib]**. Their URLs came from another agent's search results, not mine, and were not re-opened.
- The 25+ search target and the two-source rule could not be met. A follow-up run with search budget is needed for sections 2–3 (see "Could not verify").

---

## 0. Bottom line

1. **Gifting a specific eBook exists, but only as an e-mail** with a redeem link, inside each retailer's own account system. No current retailer flow I know of produces a physical object [M]; the printed title-specific cards that existed (Enthrill, Livrada, Walmart×Kobo, Kindle cards, Seebook) were retailer- or publisher-issued and are gone or marginal [V1-sib]/[M].
2. **The redemption target is nearly always an account library, not a file.** Kindle, Kobo/Fnac, Apple, Google, tolino and Vivlio deliver by cloud sync. For these, "QR → claim into account → book appears on the e-reader at next sync" is the most immediate path, and it never touches the e-reader's browser [M, high].
3. **A third party cannot inject a title into Kindle or Kobo libraries.** For those, a card can only wrap a redemption link the platform itself issued, or the retailer must integrate the card into its own gift entitlement [M, high].
4. **Open-file retailers (DRM-free / watermark — the 7switch type) are the easiest integration**: the card's token is a claim on a download; the watermark must be stamped at claim time with the *recipient's* identity [analysis].
5. **LCP is the only DRM whose design fits a gift card**: a licence file (`.lcpl`) that points to the publication, opened with a passphrase that "can be anything at all", including a provider-defined one [V1]. But browser apps cannot implement LCP [V1], and Kindle/Kobo do not support it [M].
6. **OPDS already defines the post-purchase handoff**: an `opds://` callback link to a publication or to the user's bookshelf, and a `http://opds-spec.org/shelf` relation [V1, draft best practice].
7. **Kindle remains reachable for unprotected files only**: Amazon accepts EPUB e-mailed to `@kindle.com` (with a flawed intake), and the Kindle's built-in browser can download a file directly [V1 each, different documents].
8. **France is fragmented by platform, not by brand**: Fnac = Kobo; Cultura / Decitre / Furet du Nord / E.Leclerc = Vivlio (ex-TEA); independents = ePagine and others; 7switch = Immatériel [M, medium]. Integrating one platform reaches several storefronts.

---

## 1. Vocabulary seen in the ecosystems

| Language | "Gift this eBook" | "Redeem" | Conf. |
|---|---|---|---|
| FR | "Offrir" / "Offrir cet ebook" (Decitre: bouton « Offrir » [V1-sib]); Amazon.fr "Acheter pour d'autres" | "Utiliser / saisir le code", "Récupérer mon livre" | [M] except Decitre |
| DE | "eBook verschenken", "Als Geschenk kaufen"; Amazon.de "Für andere kaufen" | "Geschenkcode / Gutschein einlösen" | [M] |
| NL | "ebook cadeau geven / cadeau doen" | "code verzilveren" | [M] |
| ES | "regalar (un) eBook" | "canjear código" | [M] |
| IT | "regala un eBook" | "riscatta il codice" | [M] |
| EN | Amazon "Buy for others"; Apple "Gift This Book"; Google "Buy as gift"; B&N "Buy as Gift" | "Redeem" | [M] |

---

## 2. How sellers deliver and gift today

### 2.1 Closed global platforms

| Seller | Delivery rail | Gift a specific eBook? | Redemption flow | Conf. |
|---|---|---|---|---|
| **Amazon Kindle** | Account library, pushed to device/app. Kindle format, Amazon DRM | Yes on Amazon.com (since 2010). **Amazon.fr: yes** [V1-sib] ([idboox](https://www.idboox.com/news-livres/amazon-on-peut-enfin-offrir-un-livre-numerique-kindle/), via `market-business.market.md`). Amazon.de: I believe yes; launch year not recalled | Buyer enters recipient e-mail, date, message — or sends the gift to themself. E-mail carries a redeem link; recipient signs in to an Amazon account **of the same marketplace**; the book lands in the library. US: quantity > 1 yields copyable "redemption links" | [M, medium] beyond the idboox fact |
| **Kobo (kobo.com)** | Account library; KEPUB sync to Kobo devices/apps; Adobe DRM or DRM-free download | **No native gifting**; value gift cards only [V1-sib, low-authority source] ([ebook-enligne.fr](https://ebook-enligne.fr/blog/offrir-ebook-guide-cadeau-numerique/)) | Title-specific codes have existed only through partners: Walmart "digital book cards" 2018 [V1-sib]; some Humble Bundle book bundles redeemed as Kobo codes [M, medium] | [M] |
| **Apple Books** | Apple Account library | Yes — "Gift This Book" in the share menu, scheduled send date | E-mail with "Redeem"; recipient needs an Apple Account on the **same country storefront** | [M, medium-high] |
| **Google Play Books** | Google account library; many titles also downloadable as EPUB/ACSM | Yes — "Buy as gift" for eBooks in a list of countries (France's presence not recalled) | E-mail with a code, redeemed at Play's redeem page; same country required | [M, medium] |
| **B&N Nook** | B&N account; US only | Yes — "Buy as Gift" (instant e-mail gift) | Redeem into a B&N account | [M, medium] |

Two side rails matter for the *standalone* mode (author or publisher makes the card):

- **Apple Books and Google Play Books give publishers promo codes** for their own titles (Apple: a capped number per book, 250 in my recollection; Google: Partner Center campaigns) [M, medium]. These are platform-issued, title-specific, printable codes — exactly what a card needs — but each is limited to one platform.
- **Amazon gift links sent to oneself** can in principle be forwarded or printed [M, medium]. A buyer could paste such a link into eBook Cadeau and get a QR for it. Whether Amazon's terms allow re-wrapping, and whether Amazon.fr exposes the link the way Amazon.com does, was not verified.

### 2.2 France

| Seller | Platform / rail | Protection | Gift a specific eBook? | Conf. |
|---|---|---|---|---|
| **Fnac ("Kobo by Fnac")** | Fnac account linked to a Kobo account; purchases sync to Kobo devices and apps | Adobe DRM / watermark / none, per publisher | Not known today. Recollection of a Fnac "offrir un ebook" launch around 2013 is weak | [M, high] for the rail; [M, low] for gifting |
| **Decitre** | Vivlio network (ex-TEA) | CARE / Adobe / LCP / watermark | **Yes — "Offrir" button, personal message, chosen send date, delivered by e-mail** [V1-sib] ([Decitre FAQ](https://www.decitre.fr/faq/numerique/offrir-un-ebook)) | [V1-sib] |
| **Cultura, Furet du Nord, E.Leclerc, Vivlio store, (former) Bookeen store** | Same Vivlio back end; Vivlio e-readers sign in with the bookseller account and sync | as above | Unknown. If Decitre's feature is a platform feature, the others could switch it on [inference] | [M, medium] |
| **7switch** | Immatériel's own store; account library with re-download | Mostly none or watermark ("tatouage numérique"); some DRM titles | Unknown. "Envoyer vers Kindle/Kobo" not confirmed | [M, medium] / unknown |
| **leslibraires.fr, Mollat, independents** | ePagine, Numilog or in-house, by shop | mixed | Unknown | [M, low] |
| **Nextory (ex-Youboox), Storytel, BookBeat** | Subscription apps | in-app only | Subscription gift cards (a code for N months), not titles | [M, medium-high] |

What the LCP specifications themselves show about French actors: the "LCP Automatic Key Retrieval" note is co-authored by staff of **EDRLab, De Marque, Vivlio and ePagine** [V1] ([note](https://raw.githubusercontent.com/readium/lcp-specs/master/notes/lcp-key-retrieval.md)), and OPDS 2.0 is edited by Hadrien Gardeur (De Marque at the time of that note) [V1] ([OPDS 2.0](https://raw.githubusercontent.com/opds-community/specs/master/opds-2.0.md)). So the Vivlio network, ePagine-powered independents and Cantook/De Marque are inside the LCP/OPDS community. Which storefronts actually ship LCP to consumers today is from memory: Vivlio network yes; French library lending (PNB) migrated to LCP; Fnac/Kobo, Amazon, Apple, Google no [M, medium].

TEA's **CARE** is a proprietary account-bound DRM used inside the Vivlio apps and e-readers [M, medium]; nothing public was retrieved on it.

**Watermark prevalence in France**: my recollection is a split market — independent, genre and technical publishers use watermark or nothing (Bragelonne and Publie.net were early DRM-free), while several large trade groups kept Adobe DRM and added LCP; Germany's trade moved to watermark in 2015–16 [M, medium]. No figure was retrieved.

### 2.3 Germany, Netherlands, Spain, Italy

| Seller | Rail | Gift a specific eBook? | Conf. |
|---|---|---|---|
| **Thalia, Hugendubel, ebook.de (Libri), buecher.de — tolino alliance** | Shop account + tolino cloud; libraries of several alliance shops can be linked ("Bibliothek-Verknüpfung"); mostly watermark | "eBook verschenken" exists at several alliance shops: e-mail with a code or link, redeemed into a shop account. I recall at least one shop offering a **print-it-yourself voucher** — the closest precedent to this project's print-at-home mode | [M, medium]; print voucher [M, low] |
| **Weltbild** | — | Insolvent in 2024, online shop closed | [M, medium-high] |
| **bol.com** (NL/BE) | Kobo partner since 2014; purchases land in a Kobo library | "ebook cadeau geven" existed (e-mail + code); current status unknown | [M, low-medium] |
| **Casa del Libro** (ES) | Own ecosystem (Tagus) | "regalar eBook": unknown | [M, low] |
| **IBS / laFeltrinelli** (IT) | Own library; Kobo and tolino partnerships at different times | "Regala un eBook" (e-mail + code) recalled | [M, low] |

### 2.4 Direct-sales and indie tools (DRM-free, link or code based)

| Tool | Token type | Redemption | Conf. |
|---|---|---|---|
| **Smashwords (Draft2Digital's store)** | Author coupons up to 100 % off; "Give as a gift" | Code at checkout, or e-mail link; free account; multi-format download | [M, high] / gift [M, medium] |
| **Leanpub** | Coupon URLs, including 100 % off | Link → account library → PDF/EPUB; Send to Kindle | [M, medium] |
| **Gumroad** | "Give as a gift" at checkout; offer codes; PDF stamping | E-mail link to a download page | [M, medium-high] |
| **Payhip** | Coupons; download limits; PDF stamping | E-mail link | [M, medium] |
| **Humble Bundle** | Gift links; keys | Claim into a Humble account; DRM-free multi-format, or partner key | [M, medium-high] |
| **ebooks.com** | Gift purchase | Unknown | [M, low] |

### 2.5 Delivery specialists: BookFunnel and StoryOrigin

These are the reference for "get a file onto whatever the reader owns" [M, medium-high for the shape, low for wording]:

- **BookFunnel**: the link opens a landing page with the cover → "Get my book" → the reader **chooses a device or app** → device-specific steps. Options: its own app (iOS, Android, Fire), an in-browser reader, raw EPUB/PDF download, and Send-to-Kindle by e-mail with guided approval of the sender address. A visible "Need help?" link leads to human support, which is their reputation. It also offers **printable single-use codes** for in-person events, typed at a short URL or in the app, and watermarked files for review copies.
- **StoryOrigin**: similar download page with Send-to-Kindle instructions; watermarking for review copies.

Lessons [analysis]: ask "what do you read on?" first; keep a reading option that needs nothing installed (browser reader); never assume the scanning device is the reading device; pair every QR with a short typed code.

---

## 3. Physical eBook gift cards, past and present

| Product | What it was | Outcome | Conf. |
|---|---|---|---|
| **Enthrill** (CA, 2013–16) | Title-specific cards in Walmart Canada and others, 3,000+ locations; publishers paid ~5 % of list per code | Ceased | [V1-sib] ([Publishers Weekly](https://www.publishersweekly.com/pw/print/20140922/64046-enthrill-partners-with-walmart-canadian-publishing-2014.html), via `legal-covers-reseller.reseller.md`) |
| **Livrada** (US, 2012) | Title-specific cards at Target | Ceased | [V1-sib] ([The Book Designer](https://www.thebookdesigner.com/e-books-for-real-e-book-gift-cards-from-livrada-and-enthrill/)) |
| **Walmart × Kobo digital book cards** (US, 2018) | ~40 titles as cards | Status unknown | [V1-sib] ([Boing Boing](https://boingboing.net/2018/08/22/walmarts-selling-ebooks-now.html/amp)) |
| **Amazon book-specific Kindle cards** (2016) | Existence only | Unknown | [V1-sib] ([The eBook Reader](https://blog.the-ebook-reader.com/2016/01/22/kindle-cards-new-book-specific-kindle-gift-cards)) |
| **Seebook** (ES, ~2012–) | Card with cover, QR and typed code, sold in bookshops and used for signings; download EPUB/PDF/MOBI or send by e-mail | Unknown | [M, medium] — closest design precedent; not verified |
| **Storytel / BookBeat / Nextory cards** | Subscription time, not a title | Current | [M, medium-high] |
| **Livrio, Epubli cards, tolino "eBook-Geschenkkarte", French "carte cadeau ebook"** | Named in the brief | **Nothing recalled with confidence; not verified** | — |

Pattern [V1-sib + analysis]: every title-specific card was produced by, or with, the rightsholder or the platform. None was a reseller-agnostic wrapper. All used "code typed on a website → account → download or library", and all predate the general habit of scanning QR codes.

---

## 4. Technical rails (verified against the specifications)

### 4.1 OPDS

| Fact | Source | Conf. |
|---|---|---|
| OPDS 1.2 (Atom) was released 11 Nov 2018; OPDS 2.0 (JSON, on the Readium Web Publication Manifest) is a "living standard" whose core has been stable since 2017 | [specs README](https://raw.githubusercontent.com/opds-community/specs/master/README.md), [OPDS 2.0](https://raw.githubusercontent.com/opds-community/specs/master/opds-2.0.md) | [V2] |
| Media types: `application/opds+json` (feed), `application/opds-publication+json` (publication); 1.2 uses `application/atom+xml;profile=opds-catalog;kind=acquisition` | [OPDS 2.0](https://raw.githubusercontent.com/opds-community/specs/master/opds-2.0.md), [OPDS 1.2](https://raw.githubusercontent.com/opds-community/specs/master/opds-1.2.md) | [V1] |
| Acquisition relations: generic, `open-access`, `borrow`, `buy`, `sample`, `subscribe`. **There is no "gift" or "redeem" relation** | same | [V1] |
| `indirectAcquisition` describes what the user finally gets when the link goes through an HTML page or a licence, e.g. `application/vnd.readium.lcp.license.v1.0+json` → `application/epub+zip` | [OPDS 1.2](https://raw.githubusercontent.com/opds-community/specs/master/opds-1.2.md), [buying best practice](https://raw.githubusercontent.com/opds-community/drafts/master/best-practices/buying.md) | [V2] |
| **Authentication for OPDS 1.0**: a protected resource answers `401` with an Authentication Document (`application/opds-authentication+json`). Flows: HTTP Basic, OAuth 2.0 implicit, OAuth 2.0 password. All OPDS clients share one client id with callback `opds://authorize/` | [Auth spec](https://raw.githubusercontent.com/opds-community/drafts/master/authentication-for-opds-1.0.md), [LCP key-retrieval note](https://raw.githubusercontent.com/readium/lcp-specs/master/notes/lcp-key-retrieval.md) | [V2] |
| **"Best Practices: Buying Publications with OPDS 2.0"** (draft, partly "TBD"): every purchase must appear in an OPDS **bookshelf** (`rel="http://opds-spec.org/shelf"`) advertised in the Authentication Document; after payment a store "may also display a callback using the `opds://` URI scheme" pointing to a publication or to the shelf, e.g. `<a href="opds://example.com/user/shelf">Download in your reading app</a>` | [buying best practice](https://raw.githubusercontent.com/opds-community/drafts/master/best-practices/buying.md) | [V1] |
| The same draft states why OPDS buying faded (mobile platforms restricting third-party payment) and why it is returning (e-reader vendors' interest, legal pressure on Apple/Google) | same | [V1] |
| Clients: Thorium Reader registers the `opds` and `thorium` URI schemes and the `.lcpl` file type. The community list names only **KOReader** under "eReaders"; apps include PocketBook Reader, FBReader, Moon+ Reader, Palace | [Thorium package.json](https://raw.githubusercontent.com/edrlab/thorium-reader/develop/package.json), [awesome-opds](https://raw.githubusercontent.com/opds-community/awesome-opds/master/README.md) | [V1] |
| KOReader runs on Kindle, Kobo and PocketBook devices and can add OPDS catalogs — but it is a user-installed replacement reader, not stock firmware | [KOReader README](https://raw.githubusercontent.com/koreader/koreader/master/README.md) | [V1] |

Reading: OPDS is a clean rail for a claim ("QR → `https://…/claim/<token>` → page offering an `opds://` link"), but **no stock Kindle or Kobo firmware speaks OPDS** [M, high], so it serves app users and the Vivlio/PocketBook family [M, medium].

### 4.2 Readium LCP

| Fact | Source | Conf. |
|---|---|---|
| LCP is passphrase-based, managed by EDRLab, has no per-transaction cost; the only mandatory cost is a yearly certification fee per server or client application | [LCP README](https://raw.githubusercontent.com/readium/lcp-specs/master/README.md) | [V1] |
| "Browser based applications cannot integrate LCP" | same | [V1] |
| The licence document has extension `.lcpl`, media type `application/vnd.readium.lcp.license.v1.0+json`, and sits at `META-INF/license.lcpl` once embedded | [LCP 1.0](https://raw.githubusercontent.com/readium/lcp-specs/master/releases/lcp/latest.md) | [V1] |
| Required links: `publication` (where to download the protected file) and `hint` (where to learn about the passphrase). A reading system given a bare licence **must** download the publication from that link and **must** show the text hint and URL when asking for the passphrase | same | [V1] |
| The passphrase "can be anything at all: a User-defined password, a Provider-defined password, an e-mail address, a library card number" | same | [V1] |
| Constraints: the provider **should** use one user key for all licences of one user; passphrases **must not** be transmitted in plaintext; reading systems store only the hashed key and try stored keys before prompting | same | [V1] |
| Optional `user` object (id, e-mail, name; may be encrypted) and `rights` (print, copy, start, end) | same | [V1] |
| Licence Status Document: status `ready` ("user hasn't accessed the License") becomes `active` on first device registration; also `revoked`, `returned`, `cancelled` ("cancelled prior to activation"), `expired` | [LSD 1.0](https://raw.githubusercontent.com/readium/lcp-specs/master/releases/lsd/latest.md) | [V1] |
| Automatic key retrieval: an authenticated OPDS shelf may carry `lcp_hashed_passphrase` on the acquisition link, so the user never types the passphrase | [key-retrieval note](https://raw.githubusercontent.com/readium/lcp-specs/master/notes/lcp-key-retrieval.md) | [V1] |
| The open-source server and Thorium build handle only the "basic" test profile; production needs confidential material and a certificate from EDRLab | [readium-lcp-server README](https://raw.githubusercontent.com/readium/readium-lcp-server/master/README.md), [Thorium README](https://raw.githubusercontent.com/edrlab/thorium-reader/develop/README.md) | [V2] |
| EDRLab publishes a demo store wired to an LCP server ("pubstore", demonstration only) | [pubstore README](https://raw.githubusercontent.com/edrlab/pubstore/main/README.md) | [V1] |

Consequences for a gift [analysis on verified facts]:

- **Issue the licence at claim time, to the recipient.** The card's token is a claim on a future licence, not a licence. This keeps the "one user key per user" rule and lets the recipient use their usual passphrase.
- **LSD gives a free gift state machine**: `ready` = claimed but not opened, `active` = opened on a device, `cancelled` = refunded before activation.
- **A card-printed passphrase is technically allowed** ("provider-defined") with hint text such as "the code printed inside your card". It must not be inside the QR URL (plaintext rule), and it weakens the per-user key model. Treat it as an option for recipients who have no account, not as the default.
- **Our browser/WASM stack cannot open LCP books**; the redemption page can only hand the `.lcpl` to an LCP app or e-reader.

### 4.3 Adobe ACSM

From memory [M, high]: an `.acsm` file is a short-lived fulfilment token, not the book. An Adobe-RMSDK reader (Adobe Digital Editions, or a compatible e-reader/app) authorised with an Adobe ID exchanges it for an encrypted EPUB/PDF bound to that ID. Kobo, PocketBook/Vivlio and tolino devices support Adobe DRM; Kindle does not. The only in-session evidence is indirect: a calibre plugin exists whose stated purpose is to "turn ACSM files into EPUB or PDF files without the need for Adobe Digital Editions" [V1] ([README](https://raw.githubusercontent.com/Leseratte10/acsm-calibre-plugin/master/README.md)). The fetched OPDS and LCP specifications do not mention Adobe at all. For a gift, ACSM is the worst rail: a third account (Adobe ID), desktop software or on-device activation, and expiring tokens.

### 4.4 Unprotected and watermarked files: getting them onto devices

| Fact | Source | Conf. |
|---|---|---|
| Amazon stopped accepting MOBI by e-mail and accepts EPUB at `@kindle.com`; its "EPUB intake is very flawed" and rejects files that work elsewhere | [calibre FAQ](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/faq.rst) | [V1] |
| A Kindle's built-in browser can download a book file directly (Standard Ebooks: tap the azw3 link; it "won't add the ebook to your Kindle library" and has no cover) | [Standard Ebooks help](https://raw.githubusercontent.com/standardebooks/web/master/www/help/how-to-use-our-ebooks.php), [calibre FAQ](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/faq.rst) | [V2] |
| Per-device formats in practice: Kindle → azw3 (or EPUB via Send to Kindle); Kobo → **kepub** (file name must end `.kepub.epub`); others → EPUB; iPad → tap the EPUB link, "open in iBooks" | [Standard Ebooks help](https://raw.githubusercontent.com/standardebooks/web/master/www/help/how-to-use-our-ebooks.php) | [V1] |
| Some e-reader browsers cannot run app-like JavaScript pages; calibre keeps a no-JavaScript `/mobile` interface for them | [calibre FAQ](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/faq.rst) | [V1] |
| "Magic Link" login exists as a pattern "for easy access on eReaders" (approve on another device rather than type a password on e-ink) | [calibre-web README](https://raw.githubusercontent.com/janeczku/calibre-web/master/README.md) | [V1] |
| Send to Kindle requires the sender address on the user's approved list; size limits apply | — | [M, high] |

Watermarking ("tatouage numérique", social DRM) itself was not researched in session. Design consequence [analysis]: the mark must name the **recipient**, so the file is generated at claim time, after the recipient gives a name or e-mail.

### 4.5 Tokens, account linking, deep links

- **One-time claim tokens**: every flow in section 2 is "opaque token → sign in or create account → entitlement added". None delivers a file straight from the token [M]. Keeping to that pattern protects the reseller (no anonymous file URL on paper) and makes the watermark and the LCP licence personal.
- **URL-based account linking**: Fnac↔Kobo and tolino's cross-shop library linking are browser redirects [M, medium]. Standard, discoverable version: Authentication for OPDS [V2].
- **Deep links**: `opds://` and `thorium://` are registered by Thorium [V1]. Platform links (Apple Books universal links, Play Store links, `kindle://`) are from memory and platform-specific [M]. No cross-vendor "add this book to my e-reader" link exists [M, high].

---

## 5. Implications for the eBook Cadeau redemption UX

Analysis; it rests on the [M] rows above wherever it concerns a named retailer.

| Reseller type | Example | What the QR / code should do | Friction |
|---|---|---|---|
| A. Account-sync platform with its own gifting | Amazon, Apple, Google | Wrap the platform's own redemption link; the card adds only the object | Low for the recipient; needs the platform's link |
| B. Account-sync platform without gifting | Fnac/Kobo, bol.com | Reseller claim page → sign in or create account → title added to library → appears on device at sync. Needs the reseller (and Kobo) to create the entitlement | Low once integrated; impossible without them |
| C. Open-file store | 7switch type, publishers, authors | Claim page → "what do you read on?" → EPUB / KEPUB download, Send to Kindle, `opds://` shelf link, browser reader; watermark at claim | Medium; depends on device |
| D. LCP store | Vivlio network, ePagine shops | Claim → licence issued to the recipient → `.lcpl` or OPDS shelf; passphrase or automatic key retrieval | Medium; LCP devices only |
| E. Adobe DRM store | legacy | Claim → `.acsm` | High |

Rules that follow:

1. **The token is a claim, never a file URL.** States: issued → claimed → delivered (→ opened, where LSD or sync reports it).
2. **Assume the phone scans and the e-reader reads.** The account library is the bridge; a file download on the phone is the fallback, not the goal.
3. **Always print a short typed code** next to the QR for e-ink browsers and desktops; the landing page must work without JavaScript.
4. **Ask for the reading device first**, then show one path.
5. **Same-country restrictions** (Amazon, Apple, Google [M]) must be printed on a wrapped-platform card.
6. **Demo fixtures to build**: a "fake 7switch" of type C (plus one LCP title, basic profile), a "fake fnac" of type B, and fake e-readers of four kinds — sync-only, browser-download, e-mail-ingest (Kindle-like), and OPDS/LCP (handles `opds://`, `.lcpl`, passphrase prompt, LSD `register`).

---

## Sources

Fetched in this session (HTTP 200, primary documents):

- https://raw.githubusercontent.com/opds-community/specs/master/README.md
- https://raw.githubusercontent.com/opds-community/specs/master/opds-1.2.md
- https://raw.githubusercontent.com/opds-community/specs/master/opds-2.0.md
- https://raw.githubusercontent.com/opds-community/drafts/master/README.md
- https://raw.githubusercontent.com/opds-community/drafts/master/authentication-for-opds-1.0.md
- https://raw.githubusercontent.com/opds-community/drafts/master/best-practices/buying.md
- https://raw.githubusercontent.com/opds-community/awesome-opds/master/README.md
- https://raw.githubusercontent.com/readium/lcp-specs/master/README.md
- https://raw.githubusercontent.com/readium/lcp-specs/master/releases/lcp/latest.md
- https://raw.githubusercontent.com/readium/lcp-specs/master/releases/lsd/latest.md
- https://raw.githubusercontent.com/readium/lcp-specs/master/notes/lcp-key-retrieval.md
- https://raw.githubusercontent.com/readium/readium-lcp-server/master/README.md
- https://raw.githubusercontent.com/edrlab/thorium-reader/develop/README.md
- https://raw.githubusercontent.com/edrlab/thorium-reader/develop/package.json
- https://raw.githubusercontent.com/edrlab/pubstore/main/README.md
- https://raw.githubusercontent.com/koreader/koreader/master/README.md
- https://raw.githubusercontent.com/janeczku/calibre-web/master/README.md
- https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/faq.rst
- https://raw.githubusercontent.com/standardebooks/web/master/www/help/how-to-use-our-ebooks.php
- https://raw.githubusercontent.com/Leseratte10/acsm-calibre-plugin/master/README.md

Reused from sibling notes (found by other agents' searches; not re-opened here):

- `docs/research/notes/market-business.market.md` §5.2 — idboox (Amazon.fr Kindle gifting), Decitre FAQ ("Offrir"), ebook-enligne.fr (Kobo has no gifting)
- `docs/research/notes/legal-covers-reseller.reseller.md` §6 — Publishers Weekly, The Book Designer, Boing Boing, The eBook Reader (Enthrill, Livrada, Walmart×Kobo, Kindle cards)

## Could not verify

Everything tagged [M]. In order of importance for the project:

1. **Current gift flows of French resellers**: Fnac/Kobo, 7switch, Cultura, Furet du Nord, Leslibraires.fr, Mollat, ePagine shops, Vivlio store — whether a "gift this eBook" option exists, and its redemption steps. Only Decitre is evidenced (sibling note).
2. **7switch specifics**: protection mix, account library, any "envoyer vers Kindle/Kobo" feature.
3. **Amazon**: countries where Kindle gifting is live (Amazon.de, .es, .it, .nl), whether Amazon.fr exposes a copyable redemption link, and whether terms allow wrapping it in a third-party card.
4. **Kobo**: whether any title-specific code mechanism is open to partner retailers today; Kobo gift card availability in France.
5. **Apple Books and Google Play Books**: gifting availability in France; publisher promo-code quotas and cross-storefront validity.
6. **tolino alliance**: which shops offer "eBook verschenken" in 2026, exact flow, and whether a printable voucher exists; alliance membership after Weltbild; LCP/CARE support on tolino devices.
7. **bol.com, Casa del Libro, IBS/laFeltrinelli, ebooks.com** gifting features.
8. **Which French storefronts ship LCP to consumers**, which e-readers and apps are LCP-certified (EDRLab's list), and the certification fee amounts.
9. **TEA "CARE"** DRM: nothing retrieved.
10. **Watermark vs DRM share** among French publishers (SNE / KPMG baromètre).
11. **BookFunnel / StoryOrigin** exact current flows, the print-code redemption URL, and pricing.
12. **Physical cards**: Seebook, Livrio, Epubli cards, tolino "eBook-Geschenkkarte", any French "carte cadeau ebook" — existence, dates, why they stopped.
13. **Stock-firmware OPDS and LCP support** on Kobo, Kindle, PocketBook, Vivlio and tolino devices (the community list is not exhaustive).
14. **Send to Kindle** current limits and approved-sender rules; ACSM details.
