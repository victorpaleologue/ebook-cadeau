File edited (only this one; no git): /home/user/ebook-cadeau/docs/research/07-legal-covers-and-reseller-rights.md — status line now "_Status: fact-checked — research date 2026-11-12_"; "## Verification log" appended at the end.

Nothing in the document was confirmed at source. My two WebSearch calls were refused (shared 200-search budget spent), and direct fetches failed because `eur-lex.europa.eu`, `www.legifrance.gouv.fr` and `ns.editeur.org` do not resolve from this session; they must be allowed in the session's network settings before a real verification pass. Corrections rest on repository notes (`notes/legal-gifting-commerce.commerce.md`, `notes/isbn-databases.trade.md`, the font notes) and on recall tagged [M]. Added authorities are cited by name and number, without URL; the only links added are ONIX code-list links already present in `notes/isbn-databases.trade.md`.

## (1) Final conclusions, in 12 lines

1. Default output in every mode is the typographic card (retyped title, author, publisher; own layout; open fonts); artwork needs a recorded rights basis.
2. A reseller owns nothing in a cover; its licence is read under ordinary contract law (C. civ. arts. 1188 ff.), not CPI L. 131-3 (Cass. 1re civ., 13 Oct 1993, "Perrier", from recall). The limit is the publisher's own upstream grant. No reseller contract was seen.
3. Printing a cover on a card is a reproduction needing its own basis. For eBooks there is no exhaustion, so the contract is the only basis; printed covers in reseller mode need a per-title flag, written publisher authorisation and a reseller warranty.
4. Standalone use has no exception in the EU; the only foothold is private copying by the user. Browser composition makes the user arguably the copier, only on the user's own printer; it lowers risk and is not a proven shield. T3 stays off in France until counsel's opinion.
5. The service never fetches a cover into a printable file, never decrypts an eBook to reach one, and never sends an unlicensed cover to a printer.
6. Covers are never modified; the 10 mm spine is typeset on a separate panel. The publisher alone cannot clear moral rights; the artist must consent to a specific alteration.
7. Public-domain artwork is a per-country gate: death + 70 in France, Germany and most of the EU; Spain 80 years for deaths before 7 Dec 1987; US publication-based (before 1931, as of 2026); last surviving co-author.
8. Price law: a free printed and mailed card is plausibly a "prime" (loi 2011-590 art. 4); a free print-at-home PDF probably is not; a paid optional card is not. The médiateur du livre can settle it.
9. VAT, conservative position: a paid optional card is a distinct supply at 20 %, the eBook stays at 5.5 %; the code is a single-purpose voucher only if redemption is limited to one Member State.
10. New layers added: EU unregistered design right (3 years, against copying), collective management (ADAGP, SAIF; warranty clause X.2(e)), reprography regime, consumer unfair-terms risk, Ocilion, Filmspeler/Ziggo, Louboutin v Amazon; damages under L. 331-1-3 corrected (lump sum higher than the royalties due, plus moral prejudice).
11. Trade marks: naming "Fnac" or "7switch" descriptively is referential use; the exposure is in logos, site look and look-alike names. Demo stores keep invented brands (the repo's "Septante" and "Grande Enseigne" fit).
12. Still unverified: every risk rating, all of §6 and §7.6, the reseller-facing contract terms, eight decisions known from a snippet or one secondary source, and all statute wordings. §10.3 lists 21 downgraded statements; the Verification log lists 15 checks for the next pass.

## (2) Full text of "Implications for the implementation"

## Implications for the implementation

Numbers marked (proposal) are design choices derived from the analysis, not legal thresholds; counsel should confirm them.

Items 1 to 36 keep the numbers of the draft, because sibling documents cite them. The fact-check changed items 3, 14, 19, 20, 21, 22, 24, 28, 29 and 36 in place and added items 37 to 43. None of the legal points behind these items was verified at source (see the Verification log); they are conservative defaults, to be confirmed by counsel.

**Rights model**

1. Every cover asset carries a mandatory rights basis with exactly these values: `none` (default), `print_licence` (T1), `promo_all_media` (T1r), `rights_holder_declared` (T2), `user_private_copy` (T3), `public_domain` (T4). No code path may place a raster or vector cover in a print file when the basis is `none`.
2. The default output of every entry point (web, command line, reseller API) is the typographic card: title, subtitle, author names, publisher name and ISBN as plain text, set in fonts from the project's own openly licensed catalogue. No logo, no blurb, no barcode.
3. Print-route gate: home-print PDF accepts every basis except `none`; a professional print order accepts only `print_licence`, `rights_holder_declared` and `public_domain`; `promo_all_media` yields the reduced-size layout only; `user_private_copy` never reaches a print order. `public_domain` passes a route only for the countries in which the test of item 22 passes.
4. The command-line and self-hosted tools require an explicit rights-basis argument to embed a cover and print a one-line notice that the operator is responsible for the rights; without the argument they produce the typographic card.

**Cover integrity (enforced in layout and again in preflight)**

5. A cover is placed with uniform scaling only: horizontal and vertical scale factors equal within 0.1 %; rotation 0°; the clip box equals the image box (no crop); no colour change other than ICC conversion to the output profile.
6. Aspect-ratio mismatch is resolved by letterboxing on a flat neutral ground, never by cropping or stretching.
7. No object may intersect the cover's bounding box: QR code, redemption code, ribbons, text, fold marks and crop marks are placed on other panels or outside the trim. Preflight fails the file otherwise.
8. Bleed is 3 mm and is filled with the flat ground colour, not with mirrored or extended artwork. By default the cover is inset at least 3 mm inside the trim on its outer edges so that trimming tolerance never cuts the artwork. Full-bleed placement (trimming up to 3 mm of artwork) is allowed only when the title carries `trim_authorised = true`.
9. Reduced-size layout (`promo_all_media`): the cover occupies at most 60 % of the front panel's width and height (at most 36 % of its area) (proposal), on a neutral ground, with the title retyped beside it.
10. When a cover is printed, a credit line is printed on the back or inside panel at 6 pt minimum (proposal): the feed's credit text when supplied (ONIX List 160 codes 01 and 03), otherwise "Couverture : © {publisher}" with `credit_missing = true` recorded in the generation log.

**Spine (10 mm) and other panels**

11. The spine is always a separate panel. Default: flat colour, title and author typeset from metadata text in an open font. The colour may be a single dominant colour computed from the cover; no pixels, textures or gradients are sampled from the cover.
12. Outpainting, mirroring or stretching the artwork onto the spine, cutting out or regenerating the cover's title lettering, and any image-model output derived from the cover are disabled unless the title carries `adaptation_authorised = true` and the basis is `print_licence` or `rights_holder_declared`. The flag defaults to false.
13. A publisher-supplied spine or flat full-cover file is used as supplied with uniform scaling. Re-fitting it to 10 mm requires `adaptation_authorised = true`; otherwise the typeset spine is used.
14. Templates are original designs. No template may reproduce a publisher's collection layout or the look of an existing cover (parasitisme; EU design right, including the unregistered right for designs less than three years old); no template takes a publisher logo. Publisher, imprint and collection logos are accepted only inside the unaltered cover image, or as separate elements when `logo_use = true`.
15. Blurbs, extracts, review quotes and author photographs are printed only when the corresponding per-title flag is true (`blurb_use`, `extract_use`, `author_photo_use`); all default to false.
16. Every card carries an imprint line naming who produced it (reseller or eBook Cadeau) and the mention "carte cadeau eBook" in the card's language. The book's EAN barcode is never reproduced. Where "QR Code" is printed as a term, the Denso Wave acknowledgement line is added.

**Standalone mode**

17. A cover enters the standalone flow only through a local file picked or dropped in the browser. Decoding, analysis and PDF composition run client-side in WebAssembly. There is no upload endpoint for cover images in the home-print flow, and an integration test asserts that no network request carries the image bytes.
18. The service has no server-side proxy or fetcher that inserts a cover into a PDF. ISBN lookup returns text metadata only.
19. A preview thumbnail of a cover the service has no rights in is at most 400 px on its long edge (proposal), is hotlinked by the browser from an allow-listed source, is not cached or re-served by our servers, is not read into a canvas, and is never embedded in the PDF. A source is allow-listed only if it shows the cover with the rights holder's consent, its terms allow display, and it applies no anti-hotlinking measure; the browser never works around a referrer check, a signed URL or any other such measure, and a failed load shows a placeholder. Covers found by the acquisition tools are stored as `none` with their source URL and the date the source's terms were last read.
20. On cover import the user must choose one declaration, stored with the session: "I am the author or publisher, or hold a licence" (`rights_holder_declared`); "copy for my private use, from a book I bought, printed on my own printer" (`user_private_copy`); "the artwork is in the public domain" (`public_domain`, requires the data of item 22). No choice means no cover.
21. `user_private_copy` produces exactly one jacket per gift code: one jacket per PDF, no multi-up imposition, no professional-print button, no link or hint to a copy shop, photo kiosk or print service, and the generation counter for that code blocks a second cover-bearing PDF. The download screen states that the file is for the user's own printer.
22. `public_domain` is a per-country gate, not one rule. Inputs: the death year of every author of the artwork (the latest one counts), the year of first publication, the user's country and, for a print order, the delivery country. With Y = current year: France, Germany and other EU Member States except Spain: Y minus death year at least 71. Spain: at least 81 if the death was before 7 December 1987, otherwise 71. United Kingdom: at least 71. United States: Y minus first-publication year at least 96 (published before 1931, as of 2026); the death year is ignored. Any other country, or any missing input: refused, typographic card. For France the interface warns about wartime extensions and authors "morts pour la France" and asks the user to confirm that neither applies. The thresholds are stored as data, with the date they were last checked, so that counsel's corrections need no code change.
23. A professional print order with `rights_holder_declared` requires an account, the declarant's name, role and email, and acceptance of the rights warranty; the declaration is logged with the order.

**Reseller API**

24. Per-title fields, all defaulting to the restrictive value: `cover_rights` (`none`, `promo_all_media`, `print_licence`), `adaptation_authorised`, `trim_authorised`, `logo_use`, `blurb_use`, `extract_use`, `author_photo_use`, `credit_line`, `excluded_reason` (`tie_in`, `licensed_character`, `rights_managed`, `fine_art`), `cleared_per_title`, `territories` (ISO country list for delivery), `licence_reference`, `valid_until`. When the reseller's data comes from an ONIX feed, `territories` is filled from the `<Territory>` of the supporting resource and `valid_until` from its `<ContentDate>` with role 15 "Until date" (role 14 "From date" is honoured as an embargo); a resource with `<ContentAudience>` 01 "Restricted" is never used.
25. A title with an `excluded_reason`, or flagged as a media tie-in edition in ONIX, is forced to the typographic card unless `cleared_per_title = true`.
26. When `cover_rights = none` the API ignores any image payload and does not store it. Cover files received under a licence are cached for the life of the order plus 30 days (proposal) and purged on takedown.
27. A cover-bearing card is not shipped to a country outside the title's `territories`; the order falls back to the typographic card.
28. `promo_all_media` is disabled per reseller until the reseller's written declaration is recorded in its account configuration. `print_licence` is disabled per reseller until its acceptance of the warranty (article X, including point X.2(e) on artwork under collective management) is recorded there.
29. In reseller checkout flows (including the demo stores) a **printed** card is an optional line item, never pre-ticked, with its own price and the standard VAT rate (20 % in France); a price of zero for a printed card is rejected unless the reseller configuration carries `publisher_premium = true`. A **print-at-home PDF** may be offered at no charge and adds no line. The eBook line keeps its own price and rate; the two are never merged into one bundle price. The VAT treatment of the code itself depends on its redemption territory and is specified in `09-legal-gifting-and-commerce.md`.

**Takedown, logging, metadata**

30. A blocklist keyed by ISBN and by ISBN registrant prefix is checked at every generation: enforced server-side for reseller and print orders, and fetched as a signed list by the browser for home print. A blocked title yields the typographic card.
31. Takedown: new generation stops within 24 business hours of a valid notice (target: same business day); cached covers are purged; unprinted orders are regenerated as typographic cards; delivered cards are not recalled.
32. A public notice form collects the DSA art. 16 elements (reasons, ISBN or URL, notifier's name and email, good-faith statement), acknowledges receipt, and records the decision with its reasons. A contact point is published.
33. Server-side generations are logged per card: ISBN, rights basis, flags, reseller identifier, licence reference, template, printer, timestamp. Retention: 6 years (proposal, to cover civil and criminal limitation). Client-side home-print generations log nothing about the cover.
34. Each PDF embeds in its XMP metadata the rights basis, credit line, licence reference and generator version, so that a printed card can be traced to its authorisation.

**Tests and fixtures**

35. Integration tests assert the gates: a reseller feed with `cover_rights = none` yields a PDF with no image on the front panel; a crop, overlay or non-uniform scale fails preflight; a blocked ISBN yields the typographic card; a `user_private_copy` session cannot create a print order.
36. Demo stores, fixtures and screenshots use invented retailer brands with an original design, and invented or public-domain books with own-made or public-domain covers. No real retailer or publisher logo, colour scheme, page layout or look-alike name (a near-miss spelling of a real brand), and no real in-copyright cover, is committed to the repository. A real retailer may be named in plain text in documentation to say which archetype a demo is modelled on ("a store of the 7switch type", "a retailer of the Fnac type"), with a line stating that the project is not affiliated with it; the demo's user interface itself shows only the invented brand. Public-domain covers in fixtures pass the gate of item 22 for every country in which the demo is published.

**Added by the fact-check**

37. No decryption. The cover of an eBook file is read only when the package carries no encryption: for an EPUB, no `META-INF/encryption.xml` entry other than font obfuscation, and no `rights.xml` or licence document. If the package is encrypted in any way the tool reads nothing from it, says why, and offers an image-file picker or the typographic card. No code path removes, bypasses or keys a protection scheme, and the interface gives no instruction for doing so. (The stricter of two readings is chosen until counsel answers question 20.)
38. Own printer. Every screen that delivers a PDF produced under `user_private_copy` carries the sentence that it is to be printed on the user's own printer. A typographic card, and a card produced under any other rights basis, carries no such restriction and may be taken to any printer.
39. Consumer wording. The standalone declaration of item 20 is a condition of use, not an indemnity: no clause makes a consumer indemnify the service. The warranty-and-indemnity text of item 23 is shown only to professional orderers.
40. Country inputs. The standalone flow records the user's country (chosen, not inferred silently) and, for a print order, the delivery country; both feed items 22 and 27. A card is not generated under `public_domain` or `user_private_copy` for a country where the corresponding rule is not implemented. `user_private_copy` is unavailable in the United Kingdom, which has no private-copy exception (proposal).
41. ONIX ingestion. A publisher-supplied flat cover (List 158 code 29) or spine image (code 56) is accepted as the "real spine" of item 13; a cover holding image (code 45) is never printed.
42. Marketing and interface copy never present the tool as a way to print a book's cover. The landing page, the store listings and the default flow show the typographic card; the cover options appear only after a rights basis is chosen. An automated check fails the build if marketing strings match a deny-list of such phrasings (proposal).
43. Feature switches. `user_private_copy` (T3), `promo_all_media` (T1r) and free printed cards are behind per-country configuration switches, off by default. A country is switched on once its rule has been read at source and the reading is recorded in the configuration; for France this requires counsel's written opinion (rule 11 of §8.2). The demo environment may switch them on, with invented titles and own-made covers only.
