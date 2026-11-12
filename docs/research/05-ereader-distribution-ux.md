# From paper to e-reader: what devices can do, and the most immediate redemption UX

_Status: fact-checked — research date 2026-11-12_

## How to read this document

**Inputs.** Three researcher notes: `notes/ereader-ux.devices.md` (devices and community tools), `notes/ereader-ux.ecosystems.md` (resellers and rails), `notes/ereader-ux.ux.md` (codes, QR, pairing, card text). Project contracts read alongside: `docs/api.md` v0.1, `docs/architecture.md`, `docs/research/09-legal-gifting-and-commerce.md` (doc 09), `docs/research/12-market-and-business-inputs.md` (doc 12), and the existing simulator in `demo/ereaders/`.

**Method limit — read before relying on anything here.**

- **No web search ran, in the notes or in this consolidation.** The shared search budget was spent before the three researchers started, and it was still spent on 2026-11-12 when this document was written (4 queries sent, 4 refused). There are no search results behind this document.
- The notes rest on about 115 cited primary documents fetched directly from GitHub (specifications, source code, issue trackers, manuals). Everything about what a vendor or a retailer does today (Amazon, Kobo, Fnac, PocketBook, Vivlio, tolino, Apple, Google) is from the researchers' memory.
- In this pass, seven primary documents were read or re-read on GitHub (send2ereader `index.js` and issues #28, #63, #70, #76; ZXing's Android `ResultHandler.java`; Android's `Uri.java`), and QR sizes and the check-symbol construction were recomputed locally.
- Facts that other agents found by search are reused from sibling documents and marked "via doc 12" or "via doc 09". Their URLs were not re-opened here.
- **Fact-check pass (2026-11-12, `factcheck/ereader-distribution-ux.md`).** Twelve fresh web searches (eight in extended mode) re-verified the load-bearing claims; those are marked **[FC]**. The main change: **Kobo OS 6.0 now reads Readium LCP** on four current models, released first outside France. Corrections are written in place and listed in the Verification log at the end. PocketBook/Vivlio browser behaviour, `.acsm` handling and the OPDS/LCP specification details were not re-checked.

**Markers.**

| Marker | Meaning |
|---|---|
| [V2] | Two independent sources read in this run |
| [V1] | One source read in this run |
| [M] | From a researcher's memory; not verified. Treat as a hypothesis |
| [calc] | Computed in this run (arithmetic or script) |
| (analysis) | Reasoning on the facts above |
| (proposal) | A design number chosen here, not a measured or legal threshold |

No marker was upgraded without new evidence. New evidence changed two things compared with the notes: the case of the QR payload's scheme (§6.3) and some details of the send2ereader issues (§2.5).

**Consequence.** Every device-specific promise ("works on Kobo", "arrives on your Vivlio") must be tested on hardware before it is printed on a card. §9 lists the tests.

---

## 1. Executive summary

### Verdict on the current scheme, "file upload → QR code → download"

1. **It aims at the wrong device.** A QR code is read by a phone camera, and no mainstream e-ink reader has a camera [M]. A scan puts the file on the phone. Among French digital readers, 48 % read on a smartphone, 34 % on a liseuse and 33 % on a tablet (Baromètre SGDL-Sofia-SNE 2026, Médiamétrie, 14.4 million digital readers) [V2, FC: [SGDL](https://www.sgdl.org/sgdl-accueil/le-guide-pratique/ressources-documentaires/barometres-et-observatoire/4867-barometre-sgdl-sofia-sne-2026-des-usages-dachat-et-de-lecture-des-livres-imprimes-numeriques-et-audio), [Bookelis](https://www.bookelis.com/barometre-lecture-france-2026-autoedition/)]. The phone is the right first stop, but one reader in three then needs a second hop.
2. **A raw file is the wrong payload for most titles.** Kindle does not open EPUB natively [V2]. No third party can add a title to a Kindle or Kobo account library [M, high]. A title sold with DRM cannot be handed over as a file at all. Doc 09 also limits file hosting to files the uploader may distribute (own work, public domain in France, open licence, licensed distributor).
3. **It is the only scheme that needs no partner, and it works** in the browsers of Kobo [V2], Kobo-based tolino [V1] and, by belief only, PocketBook and Vivlio [M]. It stays in the product as one lane of three, always behind a gift page. The QR code never points at a file.

### Recommended UX in ten lines

1. **One card, one code, two entry points.** The QR code is for a phone. A short host name and a 12-letter code, printed in clear beside it, are for an e-reader or a computer.
2. **The scan opens the gift, not a download.** Cover, giver, message first; then one primary action chosen for the gift's lane and the device; "Autres façons de lire" always visible.
3. **Reseller-integrated mode: the book arrives by account sync.** Scan → "Ajouter à ma bibliothèque" → sign in to the reseller account → the e-reader syncs. Three to four actions, nothing typed on e-ink. This is the most immediate path for Kobo/Fnac, Vivlio-network and tolino devices (analysis on [M] facts).
4. **Standalone mode, title sold in a store: the card carries the store's own gift link** (Amazon.fr, Decitre, Furet du Nord today [V2, via doc 12]). No file, three actions, the book lands in the recipient's library.
5. **Standalone mode, rights-cleared file:** on a phone or tablet, download and open (2–3 actions). On an e-reader with a browser, type the host and the card's code on the e-reader itself (4 actions, about 18 characters, no phone).
6. **Kindle is the exception.** For an open EPUB: from the phone, through the Kindle app or Amazon's Send to Kindle [M]; never promise a direct download on the Kindle. For Amazon titles: the gift-link lane.
7. **The typed code is 12 consonants, `XXXX-XXXX-XXXX`,** with a check symbol that catches every single typo and every swap of neighbours [calc]. No digits: no keyboard layer switch, no 0/O confusion, no accidental word.
8. **Pairing a phone with an e-reader is a secondary tool.** It is offered only when the phone holds something the card does not (a reseller sign-in). The card's own code already is the pairing code.
9. **Pages for e-readers are plain HTML:** server-rendered, no JavaScript, under 30 kB, port 443, the URL path ending in an ASCII file name. The user agent is a hint, never a gate.
10. **Claim rules:** hosted file = the card is the key, 10 download sessions, history shown; reseller library = one claim, bound to the account; no expiry; the giver can cancel and reissue.

---

## 2. Device capability matrix

Sources for each [V1]/[V2] cell are in §2.5. Line-ups are as recalled for 2026 [M].

### 2.1 Dedicated devices: how something gets in

| Family | Camera / QR | Built-in browser | Download from the browser | Other wireless routes | USB |
|---|---|---|---|---|---|
| **Amazon Kindle** (Kindle, Paperwhite, Colorsoft, Scribe) | None [M] | Yes, "Web Browser" in the ⋮ menu [V1]; Chromium-based from firmware 5.16.4 (2023), not on every device at once [V2, FC: [MobileRead](https://www.mobileread.com/forums/showthread.php?t=357173&highlight=chromium), [Liliputing](https://liliputing.com/amazons-kindle-web-browser-is-actually-usable-now/)] | Kindle formats only: azw3, mobi [V2]; azw, prc, txt [M]. **Not EPUB** [V2]. PDF download reported failing [V1] | Send to Kindle: e-mail, web, apps; EPUB accepted and converted [V2] | MTP on 2024+ models [V1]; mass storage before |
| **Rakuten Kobo** (Clara BW/Colour, Libra Colour, Sage, Elipsa 2E) | None [M] | Yes, Nickel browser, a QtWebKit fork [V1]; labelled beta [M] | Yes: EPUB, KEPUB, PDF are added to the library [V2] | Dropbox (firmware 4.18+), Google Drive (4.36+) on selected models [V1 for firmware]; Libra Colour yes, Clara BW and Clara Colour no (officially) [FC: [Kobo US](https://us.kobobooks.com/products/kobo-libra-colour), [review](https://whatsyourtech.ca/?p=51220)]; Instapaper (4.43+), OverDrive [V1] | Mass storage [V2] |
| **tolino** shine 5, shine color, vision color (2024+, Kobo firmware) | None [M] | Yes; user agent contains `eReader` [V1] | Yes, confirmed by a user in 2024-08 [V1] | tolino cloud: upload through the bookseller's web reader [V1, old sources] | Mass storage, detected as Kobo [V1] |
| **tolino**, older Android models (Vision 4 HD) | None [M] | Yes | Download completes, file reported "damaged" [V1] | tolino cloud [V1, old sources] | Mass storage [M] |
| **PocketBook** (Verse, Era, InkPad) | None [M] | Yes, WebKit-based [M] | Yes [M] | Send-to-PocketBook e-mail (`@pbsync.com`) [V1]; PocketBook Cloud, Dropbox [M] | Mass storage [M] |
| **Vivlio** (France; PocketBook hardware) | None [M] | As PocketBook [M] | As PocketBook [M] | Vivlio / bookseller account sync; PocketBook channels [M] | Mass storage [M] |
| **Onyx Boox** and other Android e-ink | **Yes on Palma and Tab Ultra families** [M] | Chromium-class Android browser [M] | Any file [M] | Send2Boox cloud push (account + 6-digit code by e-mail) [V1]; local Wi-Fi transfer [M] | MTP [V1, generic Android] |
| **reMarkable** (2, Paper Pro, Paper Pro Move) | None [M] | **None** [M] | Not applicable | reMarkable cloud apps, browser extension, USB web interface, third-party uploaders [V1] | USB web interface [V1] |
| **Bookeen** (Diva, Notea; legacy) | None [M] | Basic [M] | Unknown | Store sync only [M] | Mass storage [M] |

### 2.2 Dedicated devices: what they open

| Family | Formats without DRM | DRM and licences | Account library sync | OPDS in stock firmware |
|---|---|---|---|---|
| **Kindle** | KFX, AZW3, AZW, MOBI, PDF, TXT [V1]; **no EPUB**, not even by USB [V2] | Amazon only [M] | Amazon account [M] | No; KOReader only on jailbroken units [V1] |
| **Kobo** | EPUB 2/3, KEPUB, PDF, TXT, CBZ/CBR [V1]; MOBI, RTF, HTML [M]. Plain `.epub` uses the older Adobe engine, `.kepub.epub` the Kobo WebKit engine [V2] | Kobo DRM [V1]; Adobe via a computer [M]; **Readium LCP since Kobo OS 6.0** on Clara BW, Clara Colour, Libra Colour, Elipsa 2E — optional update, first released in Canada, the UK, Ireland, Australia, New Zealand and Spain, **not yet France** [FC; the draft's "no LCP" is outdated] | Kobo account; in France through a linked Fnac account [M] | No [M]; KOReader [V1] |
| **tolino** 2024+ | EPUB, PDF, TXT [V1 for the older driver] | Adobe; LCP or tolino DRM [M, low] | Bookseller account + tolino cloud [M] | No [M] |
| **PocketBook** | EPUB, PDF, FB2, MOBI, DJVU, RTF, CHM, DOC(X), TXT [V1, older drivers] | Adobe on the device, LCP [M] | PocketBook Cloud [M] | Unclear [M]; KOReader without jailbreak [V1] |
| **Vivlio** | As PocketBook [M] | Adobe, LCP [M]; Vivlio staff co-authored an LCP note [V1] | Bookseller account (Vivlio network) [M] | Unknown |
| **Onyx Boox** | EPUB, PDF, MOBI/AZW3 in NeoReader [M]; any Android app | Whatever the installed app supports [M] | Per app | Through apps (KOReader, Librera, Moon+) [V1] |
| **reMarkable** | PDF, EPUB [M] | None [M] | reMarkable cloud | No; KOReader possible [V1] |
| **Bookeen** | EPUB, PDF, HTML, FB2, TXT [V1] | Adobe; LCP on Diva and Notea [M] | Store sync [M] | No [M] |

### 2.3 Reading apps (phone, tablet, desktop)

| App | Gets a DRM-free EPUB by | DRM | OPDS | Note |
|---|---|---|---|---|
| **Apple Books** | Safari download → "Open in Books" [V2] | Apple only [M] | No [M] | Lowest friction on iPhone and iPad |
| **Google Play Books** | "Upload files" on the web, or open-with on Android [M] | Google; Adobe export [M] | No [M] | Uploads sync across devices [M] |
| **Kindle app** | Share sheet → Send to Kindle, with conversion [M]; calibre still targets MOBI/AZW3 for Kindle apps [V1] | Amazon [M] | No | Same account as the e-ink Kindle, so the book reaches it too [M] |
| **Kobo app** | iOS: Share → open in Kobo Books; Android: local file [V1] | Kobo [V1] | No [M] | Side-loaded files stay local to the app [M] |
| **Thorium Reader** (EDRLab) | Import a file or an OPDS feed; opens `.epub` and `.lcpl` [V1] | **LCP** in production builds [V1]; no Adobe [M] | Yes [V1] | Windows, macOS, Linux; no mobile repository visible in EDRLab's GitHub organisation, only `thorium-web` [V1] |
| **Vivlio app** | Account sync, file import [M] | LCP, Adobe [M] | Unknown | French reseller ecosystem |
| **Aldiko Next / Cantook** (De Marque) | File import, catalogues [M] | Adobe + LCP [M]; De Marque co-authored LCP notes [V1] | Yes [M] | |
| **PocketBook Reader app** | File import, cloud [M] | Adobe + LCP [M] | Yes [M] | |
| **KOReader** | OPDS, calibre wireless, Dropbox/FTP/WebDAV, SSH [V1] | None [M] | Yes: HTTP Basic only, no Digest [V1] | Runs on Kindle (jailbreak required [V1]), Kobo, PocketBook, reMarkable, Android. AGPL-3: patterns only, no code copied into this BSD-3 project |

### 2.4 What an e-reader can do on its own

This is the answer to "what is actually available directly from e-readers", in order of immediacy (analysis on the cells above).

| # | Capability | Kindle | Kobo | tolino 2024+ | PocketBook / Vivlio | Use for the card |
|---|---|---|---|---|---|---|
| 1 | **Sync the account library** | Yes [M] | Yes [M] | Yes [M] | Yes [M] | The best rail. Needs the store to create the entitlement (lanes R1, S1) |
| 2 | Buy in the built-in store | Yes | Yes | Yes | Yes | None: a gift is already paid |
| 3 | **Open a typed URL and download a file** | Kindle formats only [V2] | Yes [V2] | Yes [V1] | Yes [M] | The no-partner rail (lanes S2, S3, R3) |
| 4 | Read from a cloud folder | No | Selected models [V1/M] | tolino cloud [V1] | Cloud, Dropbox [M] | Help-page tip only |
| 5 | Receive by a device e-mail address | Yes, sender must be approved [V1] | No | No | Yes [V1] | Secondary option |
| 6 | Open a licence file (`.lcpl`, `.acsm`) fetched by the browser | No | `.lcpl`: possible on Kobo OS 6.0 models where released (library loans as `.lcpl` load directly, per reports) [FC], untested from the browser; not in France yet | Unknown | Believed yes; Vivlio/PocketBook read LCP [FC, secondary] | Lane R2, untested |
| 7 | USB transfer from a computer | Not EPUB [V2] | Yes [V2] | Yes [V1] | Yes [M] | Last fallback |

What no stock e-reader does: scan a QR code [M]; redeem a third party's code on the device (believed: account website only [M]); follow a link sent from another device; speak OPDS [M].

### 2.5 Evidence notes per family

**Kindle**

- No native EPUB: "Kindle devices still do not natively support epub files… Send to Kindle… converts the epub file to an Amazon format… It's still not possible to use a USB cable to transfer an epub" [V2: [Standard Ebooks](https://raw.githubusercontent.com/standardebooks/web/master/www/help/how-to-use-our-ebooks.php), [calibre FAQ](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/faq.rst)]. "Amazon converts that epub into azw3" [V1: [kindle-send](https://raw.githubusercontent.com/nikhil1raghav/kindle-send/main/README.md)].
- Send to Kindle by e-mail: the sender must be on the account's approved list; AZW3/KF8 are refused; "Amazon recently started sending confirmation emails that you have to click on… before the book is actually delivered" [V1: calibre FAQ; approved list also in the [calibre wizard](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/gui2/wizard/send_email.py)]. Standard Ebooks confirms the AZW3 refusal [V2]. calibre warns that "Amazon's EPUB intake is very flawed, they will reject a number of EPUB files that work everywhere else" [V1]; Calibre-Web-Automated ships an "EPUB Fixer" for this [V1: [CWA README](https://raw.githubusercontent.com/crocodilestick/Calibre-Web-Automated/main/README.md)]. Limits: 50 MB per e-mail (up to 25 attachments) and 200 MB per file by web or app; up to 15 approved senders [FC, third-party guides]. Amazon's help page says a document e-mailed to a Send to Kindle address can trigger a request to **verify within 48 hours**, avoidable with a harder-to-guess Send to Kindle address; a 2025 forum answer says the verification is expected for e-mail, not for the Send to Kindle app [FC: [Amazon help](https://www.amazon.com/gp/help/customer/display.html?nodeId=G98WYFZBRHWGZACA), [Amazon forum](https://www.amazonforum.com/s/question/0D56Q0000DyRrDgSQK/how-to-stop-getting-email-verify-request)].
- **Since 20 January 2026**, verified purchasers of DRM-free Kindle titles can download EPUB and PDF files from Manage Your Content and Devices; publishers opt in; borrowed titles excluded [FC: [Good e-Reader](https://goodereader.com/blog/kindle/amazon-will-let-you-download-your-e-books-in-epub-in-2026), [Kindlepreneur](https://kindlepreneur.com/amazon-drm-epub-downloads/)]. A recipient on another device may thus receive an Amazon-bought DRM-free EPUB; the redemption design is unchanged, the upload gate of doc 09 is affected.
- Browser download: tap an `.azw3` link, then "OK"; this "won't add the ebook to your Kindle library, but your Kindle will remember your reading position", and shows no cover [V1: Standard Ebooks]. send2ereader converts EPUB to MOBI with KindleGen for Kindle and strips every file-name character outside `[.\w\-"'()]` [V1: [index.js](https://raw.githubusercontent.com/daniel-j/send2ereader/master/index.js), [upload.html](https://raw.githubusercontent.com/daniel-j/send2ereader/master/static/upload.html)]. calibre drops `Content-Disposition` for `Kindle/3` user agents because non-ASCII file names break the download [V1: [legacy.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/srv/legacy.py)]. KindleGen is discontinued [V1].
- 2024+ hardware uses MTP; Colorsoft and newer hide covers of side-loaded books unless marked as personal documents [V1: [calibre changelog](https://raw.githubusercontent.com/kovidgoyal/calibre/master/Changelog.txt), FAQ]. Format list from the [calibre Kindle driver](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/kindle/driver.py) [V1].
- User-agent regression: a "Kindle 11th gen 2024, firmware 5.18.6" sees "the same web as in a laptop" on send.djazz.se (issue opened 2025-12-29); a second report on 2026-03-24; the workaround is the explicit `/receive` URL [V1: [#63](https://github.com/daniel-j/send2ereader/issues/63), [#70](https://github.com/daniel-j/send2ereader/issues/70), both re-read in this pass]. **Neither issue quotes the user-agent string**, so the string of current Kindles is unknown; the fact-check found no source for it either, only a community tool that changes the Kindle browser's **desktop** user agent, which fits the "same web as a laptop" report [FC].
- Old Kindles (1, 2, DX) cannot reach non-standard ports such as 8080 [V1: calibre FAQ].
- Code redemption on the device: none; Amazon codes are redeemed on the account website [M].

**Kobo**

- Kobo's own guidelines: EPUB 2.0.1 to 3.3; a side-loaded `.epub` uses the "Adobe Digital Editions" engine; renaming to `.kepub.epub` triggers "the Kobo WebKit"; store files carry Kobo DRM; JavaScript inside books has "limited support" on eInk [V1: [kobolabs/epub-spec](https://raw.githubusercontent.com/kobolabs/epub-spec/master/README.md)]. Standard Ebooks and [kepubify](https://raw.githubusercontent.com/pgaskin/kepubify/master/README.md) confirm the two renderers [V2].
- Browser: Kobo publishes `qtwebkit` and `qtbase` forks [V1: [kobolabs](https://github.com/kobolabs); see also the [Kobo-Reader README](https://raw.githubusercontent.com/kobolabs/Kobo-Reader/master/README)]. NickelMenu documents `nickel_browser` [V1: [NickelMenu doc](https://raw.githubusercontent.com/pgaskin/NickelMenu/master/res/doc)]. The send2ereader receiver page runs there with ES5 JavaScript and `XMLHttpRequest` [V1]. "Kobo readers [have] problems using the internal web browser on non-standard ports" [V1: [Calibre-Web FAQ](https://raw.githubusercontent.com/wiki/janeczku/calibre-web/FAQ.md)].
- Download quirks: calibre appends `.epub` to KEPUB file names and replaces `!` when the user agent contains `Kobo Touch`, and keeps a URL shape ending in the file name "needed for Kobo browsers" [V1: [content.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/srv/content.py), legacy.py].
- User-agent change: on a Kobo Libra Colour (issue opened 2026-08-29) the user agent carries a generic `eReader` and no longer `Kobo`. send2ereader then skipped its KEPUB conversion, and the reporter says the device downloaded the `.epub` file instead of a KEPUB [V1: [#76](https://github.com/daniel-j/send2ereader/issues/76), re-read]. Two things follow: detection by `Kobo` is dead on new firmware, and a plain EPUB download works on a 2026 Kobo.
- Cloud: Dropbox from firmware 4.18 ("the Forma only at this point"), Google Drive from 4.36, Pocket removed after 4.38/4.42, Instapaper from 4.43, OverDrive from 4.10 [V1: NickelMenu doc; [calibre Kobo driver](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/kobo/driver.py)]. Which 2026 models expose Dropbox or Drive (believed: Sage, Elipsa 2E, Libra Colour; not Clara) is [M]. KoboCloud adds cloud sync by script [V1: [KoboCloud](https://raw.githubusercontent.com/fsantini/KoboCloud/master/README.md)].
- Sync hijack: the device's `api_endpoint` can be pointed at Calibre-Web or Komga, which then emulate the Kobo store sync [V1: [Calibre-Web wiki](https://raw.githubusercontent.com/wiki/janeczku/calibre-web/Kobo-Integration.md), [Komga](https://raw.githubusercontent.com/gotson/komga/master/README.md)]. This shows that "the book appears by sync" is the smoothest channel, and that only the store operator can offer it to ordinary users.
- Kobo devices sign in by a device-flow: "web-based activation… open an activation link… and enter the code" [V1: [kobo-book-downloader](https://raw.githubusercontent.com/TnS-hun/kobo-book-downloader/master/README.md)].
- calibre's driver lists Clara BW, Clara Colour, Libra Colour, Sage, Elipsa 2E, Libra 2, Clara 2E and supports firmware up to 6.0 [V1].

**tolino**

- "New 2024 Tolino devices… use a modified version of the Kobo firmware, so they are detected as Kobo devices" (calibre 7.16, 2024-07-31) [V1]. A user describes the Shine Color as "(= Kobo)", firmware 5.0.175773; send2ereader added detection of `eReader` on 2024-07-22 and the reporter confirmed the fix on 2024-08-30 [V1: [#28](https://github.com/daniel-j/send2ereader/issues/28), re-read] → [V2] for "Kobo-based".
- The issue does not say which format was transferred. The source shows that KEPUB conversion runs only when the user agent contains `Kobo` [V1: index.js, re-read], so the confirmed transfer was not converted (analysis).
- Older Android-based tolino (Vision 4 HD): the browser download completes but the file is reported "damaged" [V1: [#57](https://github.com/daniel-j/send2ereader/issues/57)].
- tolino cloud lets users "upload / sync own files to the user's device(s)" through the bookseller's web reader [V1: [tolino-python](https://raw.githubusercontent.com/hzulla/tolino-python/master/README.md), [tolino-calibre-sync](https://raw.githubusercontent.com/darkphoenix/tolino-calibre-sync/master/README.md)]. Older tolino formats: [calibre eb600 driver](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/eb600/driver.py) [V1].
- tolino is a DACH and Benelux brand, marginal in France [M].

**PocketBook and Vivlio**

- calibre special-cases `@pbsync.com` next to `@kindle.com`: random ASCII file name and subject, because the service "can't handle non-ascii, filenames that are long…, commas" [V1: [email.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/gui2/email.py)]. Sender confirmation on first use is [M].
- PocketBook OS is Linux-based; KOReader installs by copying folders over USB [V1: [KOReader wiki](https://github.com/koreader/koreader/wiki/Installation-on-PocketBook-devices)].
- Vivlio = PocketBook hardware with Vivlio firmware and a store operated by the former TEA, sold through French booksellers (Cultura, Decitre, Furet du Nord…) [M]. Vivlio is a named contributor of the Readium "LCP Automatic Key Retrieval" note [V1: [lcp-key-retrieval](https://raw.githubusercontent.com/readium/lcp-specs/master/notes/lcp-key-retrieval.md)].
- Browser engine, download handling, on-device `.acsm` and `.lcpl` handling, cloud details: all [M]. These are the highest-priority items for a hands-on test, because Vivlio, Kobo/Fnac and Kindle are the three French e-ink ecosystems [M].

**Onyx Boox, reMarkable, Bookeen**

- Boox: Send2Boox is an account-based cloud push; login uses a 6-digit code sent by e-mail [V1: [onyx-send2boox](https://raw.githubusercontent.com/hrw/onyx-send2boox/main/README.md)]. Being Android, a Boox behaves like a phone for redemption [M]. Generic Android MTP defaults: [calibre mtp defaults](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/mtp/defaults.py) [V1].
- reMarkable: no browser [M]; the ecosystem relies on cloud uploaders, a USB web interface and e-mail bridges [V1: [awesome-reMarkable](https://raw.githubusercontent.com/reHackable/awesome-reMarkable/master/README.md)].
- Bookeen Diva HD formats [V1: [cybook driver](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/cybook/driver.py)]. Company status and LCP support [M].

**Apps**

- Thorium: [README](https://raw.githubusercontent.com/edrlab/thorium-reader/develop/README.md), [package.json](https://raw.githubusercontent.com/edrlab/thorium-reader/develop/package.json), [EDRLab repositories matching "thorium"](https://github.com/orgs/edrlab/repositories?q=thorium) [V1].
- KOReader: [README](https://raw.githubusercontent.com/koreader/koreader/master/README.md), [plugins](https://github.com/koreader/koreader/tree/master/plugins), [cloud providers](https://github.com/koreader/koreader/tree/master/plugins/cloudstorage.koplugin/providers), [OPDS wiki](https://github.com/koreader/koreader/wiki/OPDS-support), [OPDS plugin source](https://raw.githubusercontent.com/koreader/koreader/master/plugins/opds.koplugin/main.lua), [Kindle install](https://github.com/koreader/koreader/wiki/Installation-on-Kindle-devices), [Android install](https://raw.githubusercontent.com/wiki/koreader/koreader/Installation-on-Android-devices.md) [V1].

---

## 3. How resellers deliver and gift today

### 3.1 Stores and platforms

"Card lane" refers to §5.2. Rows marked [M] are recollections to re-check, not findings.

| Seller | What the device sees (rail) | Protection | Gift a specific eBook today? | Redemption | Card lane | Conf. |
|---|---|---|---|---|---|---|
| **Amazon.fr (Kindle)** | Account library pushed to devices and apps | Amazon DRM | **Yes.** "Acheter pour d'autres personnes" on the eBook page; the buyer gets prepaid redemption links or enters the recipient's e-mail | Recipient signs in to an Amazon account; **must live in the same country**; unredeemed copies can be reassigned or refunded | S1 (gift link) | [V2, via doc 12: [Amazon.fr help](https://www.amazon.fr/gp/help/customer/display.html?nodeId=GVWGP284MQ6ZRM59), [KDP help](https://kdp.amazon.com/fr_FR/help/topic/G200652260), [IDBoox](https://www.idboox.com/news-livres/amazon-on-peut-enfin-offrir-un-livre-numerique-kindle/)] |
| **Kobo by Fnac** | Fnac account linked to a Kobo account; sync to Kobo devices and apps [M, high] | Adobe DRM, watermark or none, per publisher [M] | **No title-level gift found**; users fall back on gift cards | — | R1 (needs Fnac and Kobo to create the entitlement) | [V1, negative finding, via doc 12: [Dealabs](https://www.dealabs.com/discussions/carte-cadeau-kobo-ou-fnac-2273026), [ebook-enligne.fr](https://ebook-enligne.fr/blog/offrir-ebook-guide-cadeau-numerique/)] |
| **Decitre, Furet du Nord** (Vivlio network, ex-TEA) | Bookseller account; Vivlio e-readers and app sync [M] | CARE, Adobe, LCP, watermark [M] | **Yes.** "Offrir un ebook… en 2 clics": gift e-mail on a chosen date with an optional message | E-mail link, read in the Vivlio app or on a Vivlio e-reader | S1 today; R1 if integrated | [V2, via doc 12: [Decitre](https://www.decitre.fr/offrir-un-ebook), [Decitre FAQ](https://www.decitre.fr/faq/numerique/offrir-un-ebook), [Liseuses.net](https://www.liseuses.net/offrir-des-ebooks-avec-tea/)] |
| **Cultura, E.Leclerc, Vivlio store** | Same Vivlio back end [M, medium] | as above | Unknown. If Decitre's feature is a platform feature, the others could switch it on (inference) | — | R1 | [M] |
| **7switch** (Immatériel) | Own store; account library with re-download [M] | Mostly none or watermark ("tatouage numérique"); some DRM titles [M] | Unknown | — | R1 or R3 | [M, medium] |
| **leslibraires.fr, Mollat, independents** | ePagine, Numilog or in-house [M, low] | mixed | Unknown | — | R1 / R2 | [M, low] |
| **Kobo (kobo.com)** | Account library; KEPUB sync | Kobo DRM, Adobe, LCP since Kobo OS 6.0 [FC], or none | No native gifting found (the fact-check found no announcement of a title-gift feature; doc 09's checker recalled one in English-speaking markets, unconfirmed); value gift cards only. Title codes only through partners (Walmart 2018; some Humble bundles [M]) | — | — | [V1 via sibling, low authority] |
| **Apple Books** | Apple Account library | Apple | Yes: in the Livres app, item → "Offrir" → personalise and send; "may not be available in all regions" [FC, Apple support text quoted in an Apple Community answer, 2022; France not stated explicitly] | Recipient gets a link by message or e-mail [FC]; same-country storefront [M] | S1 | [V1] |
| **Google Play Books** | Google account library | Google / Adobe export | Yes, "Buy as gift" in a list of countries; France not recalled [M] | E-mail with a code; same country [M] | S1 | [M, medium] |
| **Thalia, Hugendubel** (tolino alliance, DE/AT) | Shop account + tolino cloud [M] | Mostly watermark [M] | **Yes**, "Verschenken" / "eBooks verschenken" on eBook pages | Mechanics (e-mail or code) not confirmed. A print-it-yourself voucher at one shop is recalled [M, low] | S1 | [V2, via doc 12: [Hugendubel](https://www.hugendubel.de/de/shortcut/ebooksverschenken), [heise](https://www.heise.de/tipps-tricks/E-Books-verschenken-so-klappt-s-7476189.html), [test.de](https://www.test.de/E-Books-Nuetzliche-Tipps-und-Tricks-5391265-0/)] |
| **bol.com** (NL/BE) | Kobo partner; Kobo library [M] | — | "ebook cadeau geven" existed; status unknown | — | — | [M, low-medium] |
| **Casa del Libro** (ES), **IBS / laFeltrinelli** (IT) | Own ecosystems [M] | — | Unknown / "Regala un eBook" recalled | — | — | [M, low] |
| **B&N Nook** (US) | B&N account | B&N | "Buy as Gift" [M] | — | — | [M, medium] |
| **Nextory, Storytel, BookBeat** | Subscription apps | in-app | Subscription time only, not titles | Code for N months | — | [M, medium-high] |

Two side rails for standalone mode [M, medium]: Apple Books and Google Play Books give publishers promo codes for their own titles; an Amazon gift link sent to oneself can be forwarded or printed. Vivlio gift cards cover physical products only, not eBooks [V2 as recorded in doc 09: [Vivlio help](https://help.vivlio.com/hc/fr/articles/11334184838428-Je-souhaite-utiliser-un-code-promotionnel-ou-une-carte-cadeau)].

French actors inside the LCP and OPDS community: the "LCP Automatic Key Retrieval" note is co-authored by staff of EDRLab, De Marque, Vivlio and ePagine [V1: [note](https://raw.githubusercontent.com/readium/lcp-specs/master/notes/lcp-key-retrieval.md)]; OPDS 2.0 is edited by Hadrien Gardeur [V1: [OPDS 2.0](https://raw.githubusercontent.com/opds-community/specs/master/opds-2.0.md)]. Which storefronts ship LCP to consumers is [M]: Vivlio network yes; French library lending (PNB) migrated to LCP; Fnac/Kobo, Amazon, Apple, Google no.

Two more recollections, both [M, medium] with nothing retrieved: TEA's **CARE** is a proprietary account-bound DRM used inside the Vivlio apps and e-readers, possibly superseded by LCP; and the French market is split between watermark or no protection (independent, genre and technical publishers) and Adobe DRM plus LCP (several large trade groups), while the German trade moved to watermark in 2015–16.

### 3.2 Direct-sales tools and delivery specialists

| Tool | Token | Redemption | Conf. |
|---|---|---|---|
| **BookFunnel print codes** | Single-use download codes an author prints on anything; one code can deliver several books. No per-code charge; needs a USD 200 or 300 per year plan (as of 2026) | Code typed at a short URL or in the app; landing page → "choose your device" → device-specific steps; own app, in-browser reader, raw EPUB/PDF, guided Send to Kindle; human support | Codes and prices [V2, via doc 12: [BookFunnel help](https://authors.bookfunnel.com/help/print-codes/), [BookFunnel pricing](https://bookfunnel.com/pricing/)]; flow wording [M] |
| **Bookwire download codes** (DE) | Codes issued by publishers; the publisher sets the number of redemptions and the validity | Redeemed on a Bookwire page; download link valid 28 days | [V1, via doc 12: [Bookwire FAQ](https://bookwire.net/faq-download-codes)] |
| **StoryOrigin** | Download links; watermark for review copies | Download page with Send-to-Kindle instructions | [M] |
| **Smashwords, Leanpub, Gumroad, Payhip, Humble Bundle** | Coupons, 100 % codes, gift links | Link or code → account or download page → multi-format files | [M, medium] |

Lessons (analysis): ask "what do you read on?" before showing steps; keep one option that needs nothing installed; never assume the scanning device is the reading device; pair every QR code with a typed code.

### 3.3 Physical eBook cards, past and present

| Product | What it was | Outcome | Conf. |
|---|---|---|---|
| **Enthrill** (CA, 2013–16) | Title-specific cards in Walmart Canada and others | Ceased | [V1, via sibling: [Publishers Weekly](https://www.publishersweekly.com/pw/print/20140922/64046-enthrill-partners-with-walmart-canadian-publishing-2014.html)] |
| **Livrada** (US, 2012) | Title-specific cards at Target | Ceased | [V1, via sibling: [The Book Designer](https://www.thebookdesigner.com/e-books-for-real-e-book-gift-cards-from-livrada-and-enthrill/)] |
| **Walmart × Kobo** (US, 2018) | About 40 titles as cards | Unknown | [V1, via sibling: [Boing Boing](https://boingboing.net/2018/08/22/walmarts-selling-ebooks-now.html/amp)] |
| **Kindle cards** (2016) | Book-specific Kindle gift cards | Unknown | [V1, via sibling: [The eBook Reader](https://blog.the-ebook-reader.com/2016/01/22/kindle-cards-new-book-specific-kindle-gift-cards)] |
| **Seebook** (ES) | Card with cover, QR and typed code, sold in bookshops | Unknown | [M, medium] |
| Livrio, Epubli cards, tolino "eBook-Geschenkkarte" | Named in the brief | Nothing recalled; not verified | — |

Pattern: every title-specific card was issued by, or with, the rightsholder or the platform. All used "code typed on a website → account → download or library" [V1 via sibling + analysis].

### 3.4 Vocabulary seen in the ecosystems

| Language | "Gift this eBook" | "Redeem" | Conf. |
|---|---|---|---|
| FR | "Offrir", "Offrir un ebook" (Decitre [V2, via doc 12]); Amazon.fr "Acheter pour d'autres personnes" [V2, via doc 12] | "Utiliser / saisir le code", "Récupérer mon livre" | [M] otherwise |
| DE | "Verschenken", "eBooks verschenken" (Hugendubel [V2, via doc 12]); "Als Geschenk kaufen" | "Gutschein einlösen" | [M] otherwise |
| NL | "ebook cadeau geven" | "code verzilveren" | [M] |
| ES | "regalar (un) eBook" | "canjear código" | [M] |
| IT | "regala un eBook" | "riscatta il codice" | [M] |
| EN | "Buy for others", "Gift This Book", "Buy as gift" | "Redeem" | [M] |

The terminology notes hold the full analysis; French has no settled verb for "redeem", which supports the plain button "Récupérer mon livre".

### 3.5 What follows

1. Gifting a specific eBook exists, **as an e-mail**, inside each store's account system. No current flow produces a physical object [V2 for Amazon.fr, Decitre, Hugendubel via doc 12; M beyond].
2. The redemption target is nearly always an **account library**, not a file. "QR → claim into the account → the book appears at the next sync" never touches the e-reader's browser [M, high].
3. Kobo by Fnac, believed to be the largest French channel outside Amazon [M; no market-share figure was found], shows no gift entitlement at all [V1, negative]. An integration there must create the entitlement, not only the card.
4. Open-file stores (the 7switch type) are the easiest integration: the card's code is a claim on a download [M + analysis].
5. France is fragmented by platform, not by brand: Fnac = Kobo; Cultura, Decitre, Furet du Nord, E.Leclerc = Vivlio; independents = ePagine and others; 7switch = Immatériel [M, medium]. Integrating one platform reaches several storefronts.

---

## 4. Rails

### 4.1 Comparison

| Rail | Reaches | Pros | Cons | Role in the product |
|---|---|---|---|---|
| **Account-library sync** (store entitlement) | Every device of the store's ecosystem, DRM included | Zero typing on e-ink; DRM untouched; the store's normal support | Needs the store; recipient needs an account; often same-country [V2 for Amazon via doc 12] | **Primary** in reseller mode (R1) and in the gift-link lane (S1) |
| **Browser download** of a file | Kobo [V2], Kobo-based tolino [V1], PocketBook/Vivlio [M], phones, tablets, computers | No partner, no account, no app | Not Kindle (EPUB) [V2]; DRM-free only; typing on e-ink; old browsers | **Primary** for rights-cleared files (S2, S3, R3) |
| **Send to Kindle** (Kindle app, Amazon web page, e-mail) | Kindle devices and apps | The only way an EPUB reaches a Kindle [V2] | E-mail needs the approved-sender list and maybe a confirmation click [V1]; flawed EPUB intake [V1]; app and web routes [M] | Kindle path for open files; e-mail is the last option |
| **Send-to-PocketBook** (`@pbsync.com`) | PocketBook, maybe Vivlio [M] | Exists [V1] | ASCII short file names, no commas [V1]; sender confirmation [M] | Secondary option |
| **Cloud links** (Dropbox/Drive on Kobo, tolino cloud, PocketBook Cloud, Send2Boox) | Selected devices | Zero typing once set up | Needs the user's cloud already linked; model lists [M]; an integration would need access to the user's cloud | Help-page tip only; no integration |
| **OPDS** (`opds://`, shelf) | Thorium, KOReader, PocketBook Reader app, others [V1 for the first two] | Standard post-purchase handoff [V1, draft] | No stock Kindle or Kobo firmware [M, high]; no "gift" relation [V1] | Kept as a secondary link for app users |
| **Readium LCP** (`.lcpl` + passphrase) | LCP apps and e-readers; not Kindle; Kobo since Kobo OS 6.0 on four models, outside France first [FC] | A licence file that points to the book; provider-defined passphrase allowed; licence status gives "gift opened" [V1] | Browser apps cannot implement it [V1]; certification needed for production [V2] | Lane R2, issued by the reseller |
| **Adobe ACSM** | Kobo (via computer), PocketBook/Vivlio, tolino [M] | Legacy coverage | Third account (Adobe ID), expiring token, desktop software [M, high] | Last resort inside R2 |
| **USB from a computer** | Everything except EPUB on Kindle [V2] | Always works | Six actions, a cable, a computer | Printed nowhere; help page |

### 4.2 OPDS

| Fact | Source | Conf. |
|---|---|---|
| OPDS 1.2 (Atom) was released 2018-11-11; OPDS 2.0 (JSON) is a "living standard" stable since 2017 | [specs README](https://raw.githubusercontent.com/opds-community/specs/master/README.md), [OPDS 2.0](https://raw.githubusercontent.com/opds-community/specs/master/opds-2.0.md) | [V2] |
| Media types: `application/opds+json`, `application/opds-publication+json`; 1.2 uses `application/atom+xml;profile=opds-catalog;kind=acquisition` | OPDS 2.0, [OPDS 1.2](https://raw.githubusercontent.com/opds-community/specs/master/opds-1.2.md) | [V1] |
| Acquisition relations: generic, `open-access`, `borrow`, `buy`, `sample`, `subscribe`. **No "gift" or "redeem" relation** | same | [V1] |
| `indirectAcquisition` describes what the user finally gets through a licence, e.g. LCP licence → `application/epub+zip` | OPDS 1.2, [buying best practice](https://raw.githubusercontent.com/opds-community/drafts/master/best-practices/buying.md) | [V2] |
| Authentication for OPDS 1.0: `401` with an Authentication Document; flows HTTP Basic, OAuth 2.0 implicit and password; shared callback `opds://authorize/` | [Auth spec](https://raw.githubusercontent.com/opds-community/drafts/master/authentication-for-opds-1.0.md), [LCP key-retrieval note](https://raw.githubusercontent.com/readium/lcp-specs/master/notes/lcp-key-retrieval.md) | [V2] |
| Draft best practice: every purchase appears in an OPDS **bookshelf** (`rel="http://opds-spec.org/shelf"`); after payment a store "may also display a callback using the `opds://` URI scheme", e.g. `<a href="opds://example.com/user/shelf">Download in your reading app</a>` | buying best practice; [drafts README](https://raw.githubusercontent.com/opds-community/drafts/master/README.md) | [V1] |
| Clients: Thorium registers the `opds` and `thorium` URI schemes and the `.lcpl` file type; the community list names only KOReader under "eReaders" | [Thorium package.json](https://raw.githubusercontent.com/edrlab/thorium-reader/develop/package.json), [awesome-opds](https://raw.githubusercontent.com/opds-community/awesome-opds/master/README.md) | [V1] |

Reading: OPDS is a clean rail for a claim, but it serves app users, not stock Kindle or Kobo firmware [M, high].

Deep links: `opds://` and `thorium://` are registered by Thorium [V1]. Platform links (Apple Books universal links, Play Store links, `kindle://`) are platform-specific [M]. **No cross-vendor "add this book to my e-reader" link exists** [M, high]; this is why the gift page must ask where the book will be read.

### 4.3 Readium LCP

| Fact | Source | Conf. |
|---|---|---|
| Passphrase-based; managed by EDRLab; no per-transaction cost, a yearly certification fee | [LCP README](https://raw.githubusercontent.com/readium/lcp-specs/master/README.md) | [V1] |
| "Browser based applications cannot integrate LCP" | same | [V1] |
| Licence document: `.lcpl`, `application/vnd.readium.lcp.license.v1.0+json`; embedded at `META-INF/license.lcpl` | [LCP 1.0](https://raw.githubusercontent.com/readium/lcp-specs/master/releases/lcp/latest.md) | [V1] |
| Required links `publication` and `hint`; given a bare licence, a reading system **must** download the publication and **must** show the hint when asking for the passphrase | same | [V1] |
| The passphrase "can be anything at all: a User-defined password, a Provider-defined password, an e-mail address, a library card number" | same | [V1] |
| The provider **should** use one user key for all licences of a user; passphrases **must not** be transmitted in plaintext | same | [V1] |
| Licence Status Document: `ready` → `active` on first device registration; also `revoked`, `returned`, `cancelled`, `expired` | [LSD 1.0](https://raw.githubusercontent.com/readium/lcp-specs/master/releases/lsd/latest.md) | [V1] |
| Automatic key retrieval: an authenticated OPDS shelf may carry `lcp_hashed_passphrase`, so the user never types it | [key-retrieval note](https://raw.githubusercontent.com/readium/lcp-specs/master/notes/lcp-key-retrieval.md) | [V1] |
| The open-source server and Thorium build handle only the "basic" test profile; production needs a certificate from EDRLab | [readium-lcp-server README](https://raw.githubusercontent.com/readium/readium-lcp-server/master/README.md), [Thorium README](https://raw.githubusercontent.com/edrlab/thorium-reader/develop/README.md) | [V2] |
| EDRLab publishes a demonstration store wired to an LCP server | [pubstore README](https://raw.githubusercontent.com/edrlab/pubstore/main/README.md) | [V1] |

Consequences for a gift (analysis on verified facts):

- The licence is issued **at claim time, to the recipient**, by the reseller. The card's code is a claim on a future licence.
- The status document gives a free state machine: `ready` = claimed, `active` = opened on a device, `cancelled` = refunded before activation.
- A passphrase printed on the card is allowed by the specification ("provider-defined"). It weakens the one-key-per-user model, and it must never be inside the QR payload. It is a reseller option for recipients without an account, not the default.
- Our browser and WebAssembly code cannot open LCP books. The gift page can only hand the `.lcpl` to an LCP app or e-reader.

### 4.4 Adobe ACSM, and the DRM landscape in one paragraph

From memory [M, high]: an `.acsm` file is a short-lived fulfilment token; an Adobe-RMSDK reader authorised with an Adobe ID exchanges it for an encrypted file bound to that ID. Kindle does not support it. The only evidence read in this run is indirect: a calibre plugin whose stated purpose is to "turn ACSM files into EPUB or PDF files without the need for Adobe Digital Editions" [V1: [README](https://raw.githubusercontent.com/Leseratte10/acsm-calibre-plugin/master/README.md)]. For a gift it is the worst rail: a third account, desktop software or on-device activation, and an expiring token.

The landscape, all [M] unless marked: Adobe ADE works on Kobo (through a computer), PocketBook/Vivlio, tolino, Bookeen, Aldiko and the PocketBook app; not on Kindle, Apple Books or Thorium. Readium LCP covers EPUB 2/3, PDF and audiobooks [V1: lcp-specs README]; readers include Thorium [V1], PocketBook/Vivlio/tolino/Bookeen devices and the Aldiko, Cantook, Vivlio and PocketBook apps; not Kindle; **Kobo since Kobo OS 6.0** (Clara BW, Clara Colour, Libra Colour, Elipsa 2E; first countries Canada, UK, Ireland, Australia, New Zealand, Spain) [FC: [Kobo staff post](https://www.threads.com/@mtamblyn/post/DfXW-aInzAr/lcp-at-last-kobo-ereaders-with-kobo-os-now-support-readium-lcp-protected/), [IDBoox](https://www.idboox.com/news-livres/les-liseuses-kobo-accueillent-enfin-readium-lcp-pour-faciliter-la-lecture-et-lemprunt-debooks/)]; Vivlio and PocketBook devices read LCP [FC, LCP promoters' pages]. Kindle and Kobo DRM are closed and bound to the account. Design consequence: a card can carry (a) a claim on a file for DRM-free titles, (b) a claim on an account entitlement for closed ecosystems, or (c) for LCP, a claim on a licence.

### 4.5 Send to Kindle and other mail-in addresses

- EPUB sent to `@kindle.com` is accepted and converted [V2]; MOBI by e-mail was dropped; AZW3 is refused [V2] (§2.5).
- The service e-mailing the recipient's Kindle costs the recipient three things: finding the `@kindle.com` address, adding our sender address to the approved list, and possibly a verification click within 48 hours [V2, FC]. That is too much for a gift. It stays available, never first.
- The route through the Kindle app's share sheet on the phone, and Amazon's Send to Kindle web page, are believed to avoid the approved-sender step [M]; a 2025 forum answer says the verification e-mail is not expected for the app route [FC, forum-grade]. They are the recommended Kindle routes, pending a test.
- Send-to-PocketBook needs a random ASCII file name without commas [V1].

### 4.6 Cloud links

Kobo reads Dropbox and Google Drive folders on selected models [V1/M]; tolino cloud and PocketBook Cloud accept uploads [V1/M]; Boox has Send2Boox [V1]. All require the recipient to have linked a cloud account beforehand. The product does not integrate with any of them. The help page may mention them as "if your e-reader shows a Dropbox tab, save the file there".

### 4.7 Browser download

Works where §2.1 says so. Rules learnt from calibre, Calibre-Web and send2ereader [V1 each]: ports 80/443 only; no HTTP authentication dialog ("some e-ink devices have browsers that do not handle authentication" [V1: [calibre server manual](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/server.rst)]); URL path ending in the real file name; ASCII file names; a non-JavaScript page (calibre's `/mobile`).

### 4.8 The community pattern: send2ereader, calibre, Calibre-Web

send2ereader, read from source (all [V1]: [README](https://github.com/daniel-j/send2ereader), [index.js](https://raw.githubusercontent.com/daniel-j/send2ereader/master/index.js), [download.html](https://raw.githubusercontent.com/daniel-j/send2ereader/master/static/download.html), [upload.html](https://raw.githubusercontent.com/daniel-j/send2ereader/master/static/upload.html), [common.js](https://raw.githubusercontent.com/daniel-j/send2ereader/master/static/common.js)):

| Aspect | Fact |
|---|---|
| Pairing direction | The **e-reader shows** the key; the phone or computer types it and uploads. Nothing is typed on e-ink except the site URL |
| Key | 4 characters from `23456789ACDEFGHJKLMNPRSTUVWXYZ` (30 symbols, no 0/1/B/I/O/Q) = 810,000 values |
| Lifetime | The key dies 30 s after the e-reader stops polling; hard cap 1 h; the file is deleted with the key |
| Binding | Status and download are refused unless the requesting user agent equals, exactly, the one that created the key (re-read in this pass) |
| E-reader page | About 3.8 kB of HTML, ES5 only, polls `/status/KEY` every 5 s, then shows a plain link. **JavaScript is required** |
| Device routing | User agent contains `Kobo`, `Kindle`, `tolino` or `eReader` → receiver page; otherwise the upload form; `/receive` forces the receiver page |
| Conversion | KEPUB only when the user agent contains `Kobo`; MOBI (KindleGen) only when it contains `Kindle`; both opt-in. `Content-Disposition: attachment` is set for Kindle only (re-read) |
| Accepted | EPUB, MOBI, PDF, CBZ, CBR, HTML, TXT; 800 MB |
| Licence and status | MIT ([LICENSE](https://raw.githubusercontent.com/daniel-j/send2ereader/master/LICENSE)); last commit 2024-10-20 ([commits](https://github.com/daniel-j/send2ereader/commits/master)); the hosted site was reported down and "should be reopened" on 2026-10-29 ([#80](https://github.com/daniel-j/send2ereader/issues/80)) |

Derivatives and reports [V1]: [epub2kobo](https://github.com/ahxxm/epub2kobo) (MIT fork, same key and expiry); bookdrop.cc, leaf.casa and send2ereader.net named by users as live replacements, not visited ([#70](https://github.com/daniel-j/send2ereader/issues/70), #80; [forks](https://github.com/daniel-j/send2ereader/forks)). An open pull request makes the e-reader page display a QR code so that the phone gets the key by scanning ([#27](https://github.com/daniel-j/send2ereader/issues/27), [PR #59](https://github.com/daniel-j/send2ereader/pull/59)); KOReader also renders QR codes on e-ink ([qrclipboard](https://raw.githubusercontent.com/koreader/koreader/master/plugins/qrclipboard.koplugin/main.lua)). Other issues read by the devices note: [#32](https://github.com/daniel-j/send2ereader/issues/32), [#36](https://github.com/daniel-j/send2ereader/issues/36), [#50](https://github.com/daniel-j/send2ereader/issues/50), [#51](https://github.com/daniel-j/send2ereader/issues/51), [#58](https://github.com/daniel-j/send2ereader/issues/58).

calibre and Calibre-Web [V1]: calibre serves "any device that has a web browser", with `/mobile` as "a simplified, non-JavaScript interface" and OPDS at `/opds` ([server manual](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/server.rst)). Calibre-Web has a **"magic link" remote login** "made for devices where it is hard to enter a lot of text, like on ebook readers": the reader shows a token, the user confirms it on another device ([README](https://raw.githubusercontent.com/janeczku/calibre-web/master/README.md), [Configuration](https://raw.githubusercontent.com/wiki/janeczku/calibre-web/Configuration.md)). Kobo UNCaGED and KOReader speak calibre's wireless protocol ([UNCaGED](https://raw.githubusercontent.com/shermp/Kobo-UNCaGED/master/README.md), [KOReader calibre wiki](https://raw.githubusercontent.com/wiki/koreader/koreader/calibre.md)); enthusiast-only. calibre is GPL: patterns only [M].

Do not depend on send.djazz.se. Re-implement the pattern; the reference is small and MIT-licensed.

---

## 5. Recommended redemption architecture

### 5.1 Principles

1. **The code is a claim, never a file URL.** `/k/{CODE}` always returns an HTML gift page. A file or licence is served only from a short-lived session created by an explicit action.
2. **The phone scans, something else may read.** The gift page asks where the book will be read whenever it cannot know.
3. **Prefer the rail with no typing on e-ink:** account sync first, typed code on the e-reader second, pairing third, cable last.
4. **Every path works with no JavaScript, no cookie, no e-mail address and no app installed** — except where a store account is the destination.
5. **Nothing depends on user-agent detection.** Detection only reorders choices.

### 5.2 Lanes

Lane names follow doc 09 §1.3.

| Lane | Mode | `delivery` in `docs/api.md` | What the code or QR resolves to | Destination |
|---|---|---|---|---|
| **R1** | Reseller | `reseller_redirect` | Gift page → the reseller's claim page | Recipient's account library → device sync |
| **R2** | Reseller | `reseller_licence` | Gift page → licence file (`.lcpl`, `.acsm`) issued by the reseller | LCP or Adobe reading system |
| **R3** | Reseller | `hosted_file`, basis `licensed_distributor` | Gift page → file marked by the reseller | Any device that opens the file |
| **S1** | Standalone | none (new: store gift link, client-side) | The store's own redemption URL, pasted by the giver. Encoded in the QR in the browser; never sent to our server (doc 09, rules 16–17) | Recipient's store account → device sync |
| **S2 / S3** | Standalone | `hosted_file`, basis `own_work`, `public_domain` or `open_licence` | Gift page → file | Any device that opens the file |

### 5.3 Decision tree

```
A. Which lane?  (decided when the card is made)

Reseller-integrated checkout
 ├─ the reseller has an account library that devices sync with ........ R1
 │    (Fnac/Kobo type, Vivlio-network type, 7switch library)
 ├─ the reseller delivers LCP or Adobe licences without a synced library  R2
 └─ the reseller delivers open or watermarked files .................... R3
Standalone
 ├─ the title is sold in a store that has title-level gifting .......... S1
 │    (giver buys the gift there and pastes the store's link)
 ├─ the giver is the author or publisher .............................. S2
 └─ the work is public domain in France or openly licensed ............ S3

B. Which path?  (decided on the gift page, at redemption)

R1  phone or computer: "Ajouter à ma bibliothèque" → reseller sign-in → done
    then on the e-reader: sync.          Nothing is typed on e-ink.
S1  phone: scan → store sign-in → accept. No gift page of ours.
R2  LCP app on this device: open the .lcpl
    LCP e-reader: sign in on the phone → pair code → type it on the e-reader
R3 / S2 / S3  "Où lire ce livre ?"
    ├─ this phone or tablet ...... download → open in the reading app
    ├─ e-reader with a browser ... on the e-reader: browser → host → card code → Télécharger
    │    (Kobo, tolino 2024+, PocketBook, Vivlio, other)
    ├─ Kindle .................... phone: download → share to the Kindle app,
    │                              or Amazon's Send to Kindle page; e-mail form last
    ├─ no browser (reMarkable) ... phone or computer → the vendor's app
    └─ computer .................. download → read, or copy by USB
```

### 5.4 Step counts

An action is one deliberate act: a scan, a tap, a field typed, a sign-in. "E-ink characters" are typed on the e-reader. Counts are reasoning on §2, not measurements; no conversion study could be found.

| Situation | Lane | Path | Actions | E-ink characters |
|---|---|---|---|---|
| Reads on the phone or tablet | R3/S2/S3 | scan → "Lire sur ce téléphone" → open in the reading app | 3 | 0 |
| Same, with a future in-browser reader | R3/S2/S3 | scan → "Lire maintenant" | 2 | 0 |
| Kobo/Fnac, Vivlio network, tolino | **R1** | scan → "Ajouter à ma bibliothèque" → sign in → sync the e-reader | 3–4 (+1 form if no account) | **0** |
| Any store with title gifting, Kindle included | **S1** | scan → store sign-in → accept | 3 | 0 — [M] |
| Kobo, tolino 2024+, PocketBook, Vivlio | R3/S2/S3 | on the e-reader: open browser → type host → type code → "Télécharger" | 4 | about 18 |
| Same, typing `host/code` in the address bar | R3/S2/S3 | open browser → type `host/code` → "Télécharger" | 3 | about 19 |
| LCP e-reader after a phone sign-in | R2 | scan → sign in → "Envoyer sur ma liseuse" → open browser → host → 6 letters | 6 | about 14 |
| Kindle, open EPUB, Kindle app on the phone | R3/S2/S3 | scan → "Envoyer vers mon Kindle" → pick Kindle in the share sheet → send | 4 | 0 — [M] |
| Kindle, e-mail route | R3/S2/S3 | scan → type the `@kindle.com` address → send; once: approve our sender in the Amazon account | 4 + setup | 0 — fragile [V1] |
| Kindle, browser download (only if a Kindle-format file exists) | — | open browser → host → code → download → OK | 5 | about 18; no cover, not in the library [V1] |
| No smartphone: computer and cable | R3/S2/S3 | host → code → download → plug in → copy → eject | 6 | 0 |
| No reading device at all | R3/S2/S3 | host → code → read in the browser (future) | 3 | 0 |

Order of preference on the gift page: reseller library (if the gift is R1) → read on this device → e-reader instructions (type the card code there) → Kindle routes → pair code (R2 only) → USB.

### 5.5 Gift page: primary action per device class

This replaces the "Device classes" table of `docs/api.md` v0.1. The class comes from the user agent (§7.2) or from the explicit `?as=` choice, which always wins.

| Class | R1 | R2 | R3 / S2 / S3 (file) |
|---|---|---|---|
| `mobile` | **Ajouter à ma bibliothèque {revendeur}** | Open the licence in an LCP app; second: "Envoyer sur ma liseuse" (pair code) | **Lire sur ce téléphone** (download). Directly below, same weight: "Sur une liseuse", "Sur un Kindle", "Sur un ordinateur" |
| `desktop` | Ajouter à ma bibliothèque | Download the licence | **Télécharger**; then the device list |
| `kobo`, `pocketbook`, `tolino`, `eink_other` | Ajouter à ma bibliothèque (sign-in on e-ink; the page first suggests doing it on a phone) | Download the licence if the family is believed to open it [M]; otherwise instructions | **Télécharger** (`.epub`) |
| `kindle` | Ajouter à ma bibliothèque | Message: not readable on Kindle | **No download button for EPUB.** Text: "Ce Kindle ne peut pas ouvrir ce fichier depuis son navigateur", then the phone or computer route, then the e-mail form |

Rules:

- **`mobile` no longer leads with pairing.** `docs/api.md` v0.1 put "Send to my e-reader (pair number)" first, on the idea that the phone is only the scanner. With 48 % of digital readers on a smartphone and 33 % on a tablet [V1, via doc 12], the phone is a reading device as often as not, and for a file the e-reader path needs no pairing (§5.7).
- **The giver may declare the recipient's device** when making the card (optional field: Kobo, Kindle, Vivlio/PocketBook, tolino, phone or tablet, unknown). The declared device orders the choices on the gift page, selects the wording of printed step 2, and lets the checkout warn "non lisible directement sur Kindle" before purchase, which doc 09 §7.3 asks for (analysis).
- The page states format, protection and compatibility in one line before the action (doc 09, rule 33).
- After an R1 claim, the page says where the book is and how to sync ("Sur la liseuse : Accueil → Synchroniser"); the exact menu names per device are [M].

### 5.6 Kindle

| Case | Recommended path | Conf. |
|---|---|---|
| Title bought on Amazon | Lane S1: Amazon's own gift link. The recipient must live in the same country [V2, via doc 12] | [V2] for the gift flow; wrapping it in a card is unchecked against Amazon's terms |
| Open EPUB, recipient has the Kindle app on a phone | Download on the phone → share sheet → Kindle | [M] |
| Open EPUB, no Kindle app | Amazon's Send to Kindle web page, signed in to Amazon, from a phone or computer | [M] |
| Open EPUB, e-mail | Our server e-mails the `@kindle.com` address. The page explains the approved-sender step first and shows our sender address | [V1] for the constraint |
| Open EPUB typed into the Kindle browser | Not offered. The Kindle browser saves only Kindle formats [V2] | [V2] |
| Kindle-format file supplied by a rights-holder (future) | Browser download; not in the library, no cover [V1] | [V1] |
| DRM title from a non-Amazon store | Impossible. Must be stated before purchase and on the card | [M, high] |

Converting EPUB to a Kindle format on our side is out of scope for v1: KindleGen is discontinued [V1], calibre is GPL [M], and doc 09 rule 13 requires byte-identical delivery by default.

### 5.7 Pairing a phone with an e-reader

**Decision.** The card's code is already a pairing code: a recipient holding the card can type it on the e-reader with no phone. A separate pair code is offered only when the phone did something the e-reader should not have to repeat — in practice, a reseller sign-in before a licence download (R2).

Two directions exist:

| | A. Phone shows, e-reader types | B. E-reader shows, phone types |
|---|---|---|
| Precedent | `docs/api.md` v0.1 | send2ereader [V1]; Kobo's activation [V1]; OAuth device flow [V2: [GitHub](https://raw.githubusercontent.com/github/docs/main/content/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps.md), [Microsoft](https://raw.githubusercontent.com/MicrosoftDocs/entra-docs/main/docs/identity-platform/v2-oauth2-device-code.md)]; Calibre-Web magic link [V1] |
| Typed on e-ink | host + pair code | host only |
| E-reader page | Plain form, no polling | Must refresh: 5 s meta refresh or a "Continuer" button |
| What a guessed code yields | **Someone's download** | Nothing; the guesser must hold a gift |
| Residual attack | Brute force on `POST /e` | Tricking a victim into typing the attacker's key |
| Shape | Linear: phone, then e-reader | Back and forth |

The devices note prefers B ("pair, don't type"). The UX note prefers typing the card code and keeps A for the residual case. **We keep A for v1**, hardened, because it is linear, needs no refresh on e-ink, and is confined to lane R2 where few codes are live. B is specified as the upgrade (§7.5) and becomes the default if the hands-on test shows that typing on e-ink is worse than assumed (§9, question 1).

Hardening of A (numbers in §7.6): 6 letters instead of 6 digits, single use, 15 minutes, one live pair code per gift, a global failure budget. With 6 digits, 50 live codes and 10,000 guesses an hour, 0.5 sessions an hour would be stolen; with 6 letters, 0.008 [calc].

Supporting evidence: device-flow codes live 15 minutes and are polled every 5 s [V2/V1: GitHub, Microsoft; [GOV.UK](https://raw.githubusercontent.com/alphagov/govuk-design-system/main/src/patterns/confirm-a-phone-number/index.md) uses 15 minutes for SMS codes]; a short verification URI that redirects is an accepted pattern [V1: [Keycloak](https://raw.githubusercontent.com/keycloak/keycloak/main/docs/documentation/server_admin/topics/sso-protocols/con-oidc-auth-flows.adoc)]; the cross-device channel is unauthenticated, so codes must be short-lived and limited in use, and the user must see who is asking [V1: [IETF cross-device draft](https://raw.githubusercontent.com/oauth-wg/oauth-cross-device-security/main/draft-ietf-oauth-cross-device-security.md)]; an implementation exposes the requesting device's IP and user agent to the confirmation screen [V1: [node-oidc-provider](https://raw.githubusercontent.com/panva/node-oidc-provider/main/docs/README.md)]; Libby uses a "setup code" to add a device [V1: [odmpy](https://raw.githubusercontent.com/ping/odmpy/master/README.md); 8 digits and one minute are M].

### 5.8 Claim rules

| Question | Rule | Basis |
|---|---|---|
| States | `created` → (opening date reached) → `redeemed` → `fulfilled`; plus `cancelled` (with reason, including `reissued`) and `expired` | doc 09, rule 30 |
| Hosted file (R3, S2, S3) | The card is the key. Re-downloadable by its holder, capped at 10 download sessions (the uploader may lower the cap to 1). The gift page shows "téléchargé n fois, dernière fois le …" | `docs/api.md`; doc 09, rule 14; Bookwire lets the publisher set redemptions and validity [V1, via doc 12]; BookFunnel print codes are single use [V2, via doc 12] |
| Reseller library (R1) | One claim, bound to the account that claims. Afterwards the page says the book is already in an account, without showing which | `docs/api.md` §4.4 |
| Licence (R2) | One licence, issued to the recipient at claim time; re-download of the same licence allowed within the cap | LCP: one user key per user [V1] |
| Store link (S1) | Governed by the store. Our server knows nothing | doc 09, rule 17 |
| Expiry | None by default; never under 5 years for a paid gift. The card prints "sans date limite" or the date | doc 09, rule 31 |
| Opening date | Optional, chosen by the giver ("ne s'ouvre pas avant le 25/12"); also set to purchase + 14 days when the buyer did not waive withdrawal. Before it, the page says when the gift opens and nothing else | doc 09, rule 32 |
| Lost before claim | The giver's manage link cancels the code and issues a new card | doc 09, rule 30 |
| Lost after claim | Account: nothing to do. Hosted file: the copy is on the device; reissue while sessions remain | — |
| Identity | No account, e-mail or cookie for hosted files. Two opt-in extras: "m'envoyer un lien de secours" (the address is used once and not stored), and telling the giver that the gift was opened | doc 09, rules 40–42 |
| Storage | The code is stored as an HMAC, never logged in clear | NIST §5.1.2 [V1] |

### 5.9 Failure recovery

| Failure | What the recipient sees | Way out |
|---|---|---|
| The QR code will not scan | — | Host and code in clear on the card |
| Typo (check symbol fails) | "Il y a une faute de frappe dans le code. Cet essai ne compte pas." | Retype |
| Unknown code | Neutral message, attempts left, waiting time [V1: NIST §10] | Help link |
| Wrong device guessed | "Autres façons de lire" is always visible | `?as=` choice |
| Kindle typed into its own browser | Explanation and the phone route | §5.6 |
| Download fails or the file is reported damaged (older tolino [V1]) | "Le téléchargement n'a pas abouti ?" under the button | New session; USB route from a computer |
| Send to Kindle rejects the file [V1] | — | Kindle app share sheet; Amazon's web page |
| Already claimed by someone else | Date and count, no identity | Support contact; the giver checks the manage link |
| Pair code expired | Absolute expiry time shown; one tap to renew | New code |
| Title withdrawn from sale | — | Substitute or refund rule (doc 09 §7.4) |

One help address, printed on the card and at the same place on every page [V1: NIST §10; [WCAG 3.2.6](https://raw.githubusercontent.com/w3c/wcag/main/understanding/22/consistent-help.html)].

### 5.10 Contradictions between the inputs, and how they are resolved

| # | A says | B says | Kept, and why |
|---|---|---|---|
| 1 | Devices note: code of 10 or more symbols from a 30-symbol alphabet with digits 2–9, grouped `XXXX-XXXX-XX` (about 49 bits). `docs/api.md` v0.1: 10 Crockford symbols (45 bits) | UX note: 12 consonants, `XXXX-XXXX-XXXX` (47.5 bits) | **12 consonants.** Digits cost a layer switch on phone keyboards [M, high] and Shift on AZERTY [M, high], and the consonant set removes every digit/letter confusion and every accidental word. `docs/architecture.md` already specifies it. `crates/ebc-model/src/code.rs` and `docs/api.md` still implement the Crockford form and must follow. The 1.5-bit gap to the devices note's proposal is covered by rate limits and length agility (§7.6) |
| 2 | Devices note: "pair, don't type"; the e-reader should display a key | UX note: the card code is the pairing code; pairing only after a phone sign-in | **UX note**, see §5.7. Typing 12 letters on the e-reader is 4 actions with no phone; direction B is 6 actions on two devices |
| 3 | Ecosystems note and doc 09 §4.3 (item 6): the watermark must name the recipient, so the file is generated at claim time after the recipient gives a name or e-mail | UX note: no identity for hosted files; mark with the gift or order reference; a name only as an optional ex-libris | **Both, by lane.** S2/S3 files are delivered byte-identical with no mark (doc 09, rule 13). In reseller mode the reseller decides under its publisher contracts. A mark that needs the claimant's identity means lane **R1**: the reseller's own claim page asks for it and serves the file. Lane **R3** is only for files the reseller has already marked, with the order reference or with the recipient name given at checkout; our server never alters a file |
| 4 | Ecosystems note: the token is a claim, never a file; no store delivers a file straight from a token | UX note: for hosted files the card is the key, re-downloadable | **Compatible.** The code opens a gift page; the file comes from a 15-minute session created by an explicit action. Possession of the card is the right for hosted files; an account is the right for reseller titles |
| 5 | `docs/api.md` v0.1: on `mobile`, pairing first | UX note: reseller library → read on this device → e-reader instructions → Kindle → pair → USB | **UX note**, see §5.5 |
| 6 | UX note and `docs/architecture.md`: QR payload fully upper-case, `HTTPS://…` | New evidence: upper-case schemes are a known hazard on Android | **Lower-case `https://`, upper-case remainder**, see §6.3 |
| 7 | `docs/api.md` v0.1: for `kindle`, the Send to Kindle e-mail form is the primary action | Devices note: do not make the service e-mail the recipient's Kindle | **E-mail last.** Phone routes first (§5.6). The form stays because it is the only route that needs neither the Kindle app nor an Amazon sign-in in a browser |
| 8 | Devices and ecosystems notes: Kobo wants `.kepub.epub` | Doc 09, rule 13: delivery is byte-identical; conversion only if the uploader allows it | **Plain `.epub` by default.** It opens on Kobo with the older engine [V2] and a plain EPUB download is confirmed on a 2026 Kobo [V1]. KEPUB is an opt-in improvement, served only to user agents containing `Kobo` |
| 9 | Doc 09, rules 16–17: in lane S1 the store link is encoded in the QR in the browser and never reaches our server | UX note: every QR payload is our short URL, with a typed fallback | **Doc 09.** The price is a denser QR code (§6.3), no gift page and no typed fallback of ours. Storing the link server-side would give a small QR and a gift page, but it would put a third party's bearer secret on our server; rejected |
| 10 | Devices note summary: user-agent detection broke "three times"; its key findings say "twice" | — | **Three events**, each [V1]: tolino Shine Color (2024), Kindle 2024 (reported 2025-12 and 2026-03), Kobo Libra Colour (2026-08) |
| 11 | Doc 09, rules 46–47: code grouped `XXXXX-XXXXX`, look-alike mapping as Crockford | `docs/architecture.md`: `XXXX-XXXX-XXXX`, consonants | **Architecture.** Doc 09's two rules predate the change and need the same edit: grouping `XXXX-XXXX-XXXX`; no look-alike mapping is needed because the alphabet has no digit and no vowel |

---

## 6. Card content spec

### 6.1 What is printed where

| Panel | Content |
|---|---|
| Outside front | The cover. No code, no QR |
| Spine (10 mm) | Title; no code |
| Outside back | Back-cover text, mandatory mentions (doc 09 §11). No code |
| **Inside** | Headline, giver, message or blank space; three numbered steps; QR code; host and code in clear; one fallback line; legal line; help line |

The code is on the inside only: a printer or a postal worker cannot read it without opening the jacket, and an optional seal sticker closes it (§6.7).

### 6.2 Text (French)

Infinitive form, so the card chooses neither tu nor vous. Braces are variables.

```
Ce livre est un cadeau
De la part de {prénom}
{message, ou espace libre pour écrire à la main}

POUR LE LIRE
1  Scanner ce code avec l'appareil photo d'un téléphone.
2  Choisir où lire : liseuse, tablette ou téléphone.
3  Le livre arrive. Bonne lecture !

[QR 20 mm min.]     {domaine}
                    BCDF-GHJK-LMNL

Sans téléphone ? Sur la liseuse ou un ordinateur, ouvrir le navigateur,
aller sur {domaine} et saisir le code ci-dessus.

{Titre}, {Auteur}. Un exemplaire numérique, déjà payé : rien à régler.
{Format et compatibilité}. {Sans date limite | Valable jusqu'au …}.
Une aide ? {domaine}/aide · {contact}
```

Variants of steps 2 and 3:

| Lane or hint | Step 2 | Step 3 |
|---|---|---|
| R3 / S2 / S3, no device declared | Choisir où lire : liseuse, tablette ou téléphone. | Le livre arrive. Bonne lecture ! |
| R1 | Se connecter à son compte {revendeur}, ou en créer un. | Le livre apparaît dans la bibliothèque, sur la liseuse et l'application. |
| S1 | Se connecter à son compte {boutique} et accepter le cadeau. | Le livre apparaît dans la bibliothèque {boutique}. |
| File, giver declared "Kindle" | Choisir « Envoyer vers mon Kindle ». | Le livre apparaît sur le Kindle. |
| File, giver declared an e-reader with a browser | Step 1 becomes: "Sur la liseuse, ouvrir le navigateur et aller sur {domaine}." Step 2: "Saisir le code ci-dessous." | Le livre se télécharge. Bonne lecture ! |

Format and compatibility line, examples: "EPUB sans DRM. Lisible sur liseuses Kobo, Vivlio, PocketBook et tolino, tablettes et téléphones ; sur Kindle avec « Envoyer vers Kindle »." / "À lire avec un compte {revendeur}, en France." Each device name on that line is a promise; print only what has been tested (§9).

In lane S1 the fallback line is replaced by the store's own code in clear, when the store gives one, and by the store's country restriction.

The gift moment (UX note, design reasoning):

- The object says "book", not "voucher": headline "Ce livre est un cadeau"; never a price (doc 09, rule 35).
- A blank area is left for a handwritten note; the typed message is printed only if the giver wrote one.
- The scan shows the cover and the message first; the technical choice comes second.
- The card stays useful: a later scan shows "Vous lisez ce livre depuis le …" and a way back to the file or the library.

### 6.3 QR payload

**Format (lanes R1, R2, R3, S2, S3).**

```
https://{HOST}/K/{CODE}
```

- `https://` in lower case; `{HOST}`, `/K/` and `{CODE}` in upper case; no query string, no fragment.
- `{CODE}` is the 12 symbols without hyphens.
- The encoder emits two segments: 8 bytes in byte mode for `https://`, the rest in alphanumeric mode.

**Why the scheme is lower-case (new in this document).** The UX note recommended a fully upper-case payload, which fits one alphanumeric segment, and flagged phone behaviour as untested [M]. Two primary sources read in this pass show that upper-case schemes are a known hazard on Android: ZXing's Android client lower-cases `HTTP://` and `HTTPS://` before opening them because "some Android browsers don't seem to register to handle HTTP:// or HTTPS://" [V1: [ResultHandler.java](https://raw.githubusercontent.com/zxing/zxing/master/android/src/com/google/zxing/client/android/result/ResultHandler.java)]; Android's `Uri.normalizeScheme()` exists because URIs from outside Android should be lower-cased before an Intent is built, with the example "HTTP://www.android.com" [V1: [Uri.java](https://raw.githubusercontent.com/aosp-mirror/platform_frameworks_base/master/core/java/android/net/Uri.java)] → [V2] for the hazard. A scanner that normalises is unaffected; one that does not may fail to open the link. Upper-case host and path are safe: host names are case-insensitive, and our server matches the path without regard to case. ZXing's parser itself accepts any scheme case [V1: [URIResultParser.java](https://raw.githubusercontent.com/zxing/zxing/master/core/src/main/java/com/google/zxing/client/result/URIResultParser.java)].

**Cost of the lower-case scheme** [calc: bit counts against the ISO capacity table; the single-mode rows agree with the UX note's segno results]:

| Payload (12-symbol code) | Characters | Version at L / M / Q / H | Modules at M | Modules at Q |
|---|---|---|---|---|
| `HTTPS://XXX.FR/K/…` all upper case | 29 | 2 / 2 / 2 / 3 | 25 | 25 |
| **`https://XXX.FR/K/…` mixed (recommended)** | 29 | 2 / 2 / 3 / 3 | 25 | 29 |
| `https://xxx.fr/k/…` all lower case | 29 | 2 / 3 / 3 / 4 | 29 | 29 |
| `HTTPS://EBOOK-CADEAU.FR/K/…` all upper case | 38 | 2 / 2 / 3 / 4 | 25 | 29 |
| **`https://EBOOK-CADEAU.FR/K/…` mixed** | 38 | 2 / 3 / 3 / 4 | 29 | 29 |
| `https://ebook-cadeau.fr/k/…` all lower case | 38 | 3 / 3 / 4 / 5 | 29 | 33 |
| **`https://WWW.FNAC.COM/EBOOK-CADEAU/K/…` mixed** | 48 | 3 / 3 / 4 / 5 | 29 | 33 |
| same, all lower case | 48 | 3 / 4 / 5 / 6 | 33 | 37 |

With a 6-character host the mixed payload is still version 2 at level M. It loses only the free boost to level Q. That is a fair price for removing a failure that cannot be fixed after printing.

**Lane S1 payload.** The store's redemption URL as given, in byte mode. Sizes [calc]:

| URL length | Version at M | Modules | Symbol at 0.6 mm per module |
|---|---|---|---|
| 60 bytes | 4 | 33 | 19.8 mm |
| 80 bytes | 5 | 37 | 22.2 mm |
| 100 bytes | 6 | 41 | 24.6 mm |
| 120 bytes | 7 | 45 | 27.0 mm |
| 150 bytes | 8 | 49 | 29.4 mm |
| 200 bytes | 10 | 57 | 34.2 mm |

The length of real store links is unknown [M]. The generator computes the version and refuses a link that needs more than version 10 at level M.

**Print rules.**

| Rule | Value | Basis |
|---|---|---|
| Error correction | M minimum; raise to Q when the version does not change | Levels recover about 7, 15, 25, 30 % [V1: [python-qrcode](https://raw.githubusercontent.com/lincolnloop/python-qrcode/main/README.rst)]; free boost [V2: [Nayuki](https://raw.githubusercontent.com/nayuki/QR-Code-generator/master/Readme.markdown), [segno](https://raw.githubusercontent.com/heuer/segno/master/docs/make.rst)]. H is for logo overlays, which we do not use |
| Quiet zone | 4 modules on every side | "the minimum according to the specs" [V2: python-qrcode, segno] |
| Symbol width, our short URL | 22 mm target (0.88 mm modules at version 2, 0.76 mm at version 3); 20 mm minimum | [calc]; the "width ≈ distance / 10" rule is [M] |
| Module size floor | 0.5 mm (about 6 printer dots at 300 dpi [calc]); the generator fails below it | The 0.4 mm floor quoted elsewhere is [M] |
| Ink | K100 vector, on white | [V2, via the prepress note, for K-only fine elements] |
| Placement | At least 5 mm from folds and trim; never across a spine crease | prepress note |
| Beside the symbol | Host and code in clear | The request's context must be clear to the user [V1: IETF cross-device draft] |
| Alphanumeric mode | Digits, upper-case A–Z, space and `$%*+-./:`; lower case forces byte mode | [V1: [segno modes](https://raw.githubusercontent.com/heuer/segno/master/docs/qrcode-modes.rst)]; "Inside QR codes uppercase SHOULD be used" [V1: [BIP-173](https://raw.githubusercontent.com/bitcoin/bips/master/bip-0173.mediawiki)] |

Identifiers in URLs leak through logs, history, bookmarks and the Referer header [V1: [OWASP Session Management](https://raw.githubusercontent.com/OWASP/CheatSheetSeries/master/cheatsheets/Session_Management_Cheat_Sheet.md)]; §7.7 deals with it.

### 6.4 Short URL

- `{domaine}` = letters and one dot. No hyphen, no digit, no `www`. As short as the budget allows: 6 to 8 characters with the dot (proposal).
- Printed in lower case without `https://`; encoded in upper case in the QR code.
- Typing the host alone must show the code form. Typing `host/code` must work too.
- In reseller mode the reseller may use its own host (`fnac.com/cadeau`); the cost is a longer string to type and a larger QR code (table above).

### 6.5 Typed code

**Format.** 12 symbols from `BCDFGHJKLMNPQRSTVWXZ` (the 20 consonants without Y), printed `XXXX-XXXX-XXXX`: 11 random symbols and 1 check symbol; 20¹¹ ≈ 2.05 × 10¹⁴ values, 47.5 bits [calc].

**Why this alphabet.**

| Fact | Source | Conf. |
|---|---|---|
| `BCDFGHJKLMNPQRSTVWXZ` is the default user-code charset of an OAuth device-flow implementation ("excludes easily confused characters"); GitHub's sample code `WDJB-MJHT` fits it | [node-oidc-provider](https://raw.githubusercontent.com/panva/node-oidc-provider/main/docs/README.md), [GitHub docs](https://raw.githubusercontent.com/github/docs/main/content/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps.md) | [V1]; that RFC 8628 recommends it is [M] |
| Open Location Code uses 20 symbols "selected to reduce writing errors and prevent accidentally spelling words" | [OLC spec](https://raw.githubusercontent.com/google/open-location-code/main/Documentation/Specification/specification.md) | [V1] |
| Crockford base32 maps I/L to 1 and O to 0 on input; its check symbol needs five extra symbols `* ~ $ = U` that an e-ink keyboard does not offer | [base32-crockford README](https://raw.githubusercontent.com/jbittel/base32-crockford/master/README.rst), [source](https://raw.githubusercontent.com/jbittel/base32-crockford/master/base32_crockford.py), [crockford crate](https://raw.githubusercontent.com/archer884/crockford/master/README.md) | [V1] |
| Bech32 drops 1, b, i, o on visual-similarity data and rejects mixed case | [BIP-173](https://raw.githubusercontent.com/bitcoin/bips/master/bip-0173.mediawiki) | [V1] |
| Digits sit on a second layer of iOS and Android keyboards and need Shift on French AZERTY keyboards | — | [M, high] |
| Digits sit on a second layer of Kobo, Kindle and PocketBook keyboards | — | [M, low]; not verified on hardware |

Taps on a keyboard that hides digits behind a layer key [calc]: 10 Crockford symbols 14.2 taps (3.2 bits per tap); 12 Crockford symbols 17.0; 12 consonants 12.0 (4.0 bits per tap).

A known weakness: French consonant names sound alike (bé, dé, pé, té, vé; èf, èss). A code dictated by voice is more error-prone than digits. The check symbol catches a single misheard letter (analysis).

**Check symbol.** A Damm-style check of order 20, verified here on 684,000 substitutions and 31,323 adjacent swaps, all detected [calc]:

- Index each symbol 0–19 in alphabet order (`B`=0 … `Z`=19). Write the index as `5·h + l`, with `h` in 0–3 and `l` in 0–4.
- State `(sh, sl)` starts at `(0, 0)`. For each symbol: `sh ← α·sh ⊕ h` in GF(4) (elements as 2-bit values, modulus x²+x+1, α = 2, ⊕ = XOR); `sl ← (2·sl + l) mod 5`.
- The check symbol is the one that brings the state to `(0, 0)`: `h = α·sh`, `l = (−2·sl) mod 5`.
- A 12-symbol code is valid when the final state is `(0, 0)`.
- Test vectors [calc]: `BCDF-GHJK-LMNL`, `ZXWV-TSRQ-PNMP`, `QRST-VWXZ-BCDT`.
- Codes made of one repeated symbol can validate (`BBBB-BBBB-BBBB`, `CCCC-CCCC-CCCC`). The generator rejects any code with fewer than 5 distinct symbols and any code on a short blocklist of acronyms.

**Input.** Case-insensitive; spaces, hyphens, dots and underscores ignored. A failed check is a typo: the page says so and the attempt is not counted.

**Length agility.** The parser accepts 12 symbols now and reserves 15 (`XXXXX-XXXXX-XXXXX`, 60.5 bits) for later. New codes move to 15 symbols before 250,000 codes are live (§7.6).

### 6.6 Accessibility and print rules

| Requirement | Source | Conf. |
|---|---|---|
| Steps at 12 pt or more; black on white; a typeface that separates confusable glyphs | [NIST SP 800-63B §10](https://raw.githubusercontent.com/usnistgov/800-63-3/nist-pages/sp800-63b/sec10_usability.md) | [V1] |
| Code at 16 pt bold, monospaced or open-shaped, letter-spaced, groups of four; never under 12 pt | NIST §10; [GOV.UK text input](https://raw.githubusercontent.com/alphagov/govuk-design-system/main/src/components/text-input/index.md); doc 09 rule 46 | [V1] + (proposal) |
| Contrast 4.5:1 for text, 3:1 from 18 pt or 14 pt bold; the code at 7:1 | [WCAG 1.4.3](https://raw.githubusercontent.com/w3c/wcag/main/understanding/20/contrast-minimum.html); doc 09 rule 46 | [V1] |
| Retyping a code is a "cognitive function test"; a path without transcription must exist. The criterion is scoped to authentication; applied here by analogy: the QR code is that path | [WCAG 3.3.8](https://raw.githubusercontent.com/w3c/wcag/main/understanding/22/accessible-authentication-minimum.html) | [V1] |
| A blind recipient cannot find a printed QR code: the giver can also send the same link by e-mail or SMS | — | (analysis) |
| European Accessibility Act: Directive (EU) 2019/882 | [W3C WAI policy list](https://raw.githubusercontent.com/w3c/wai-policies-prototype/master/_policies/european-union.md) | [V1] for existence; scope and dates [M], see doc 09 §10 |

### 6.7 Anti-fraud on paper

| Threat | Applies? | Measure |
|---|---|---|
| Guessing codes | Yes | §7.6 |
| Tampering on a retail rack | No: cards are made to order | — |
| Printer or postal staff reading the code | Low | Code on the inside; optional seal sticker; opening date; cancel and reissue |
| Photo of the open card posted online | Low | Download cap; claim history on the gift page |
| A sticker over the QR code | No for a mailed card | Host printed in clear beside the symbol |
| Pair-code theft | Only in lane R2 | §5.7, §7.6 |

Scratch-off panels are not recommended: extra finishing cost [M], impossible for print-at-home, hard for people with limited dexterity, and they make a keepsake look like a lottery ticket.

---

## 7. Server requirements derived from the UX

### 7.1 Pages for e-ink browsers

| Constraint | Rule | Basis |
|---|---|---|
| Engines | Assume WebKit from about 2009–2014: sampled user agents report AppleWebKit 528 to 538. No flexbox or grid dependence, no web fonts, no CSS animation, no `position: fixed` | Strings [V1: [uap-core tests](https://raw.githubusercontent.com/ua-parser/uap-core/master/tests/test_device.yaml), [Matomo fixtures](https://raw.githubusercontent.com/matomo-org/device-detector/master/Tests/fixtures/tablet-3.yml)]; dating and current firmware [M] |
| JavaScript | **None required anywhere on the recipient side.** Any script is an enhancement written in ES5 | calibre `/mobile` [V1]; send2ereader needs ES5 + XHR and works on Kobo and Kindle [V1] |
| Weight | Under 30 kB per page, excluding one optional cover thumbnail of at most 20 kB | `docs/api.md`; send2ereader's page is 3.8 kB [V1] |
| The WebAssembly and PDF tooling | Never referenced from a recipient page | (analysis) |
| Transport | HTTPS on port 443 only; no HTTP authentication; no cookie needed | Old Kindles and Kobo browsers fail on other ports [V1 each]; "browsers that do not handle authentication" [V1] |
| TLS | Keep TLS 1.2 enabled; do not go TLS 1.3-only or ECDSA-only, and do not pick a certificate chain, before a test on real devices | TLS levels and root stores of e-reader browsers are unknown [M] |
| Forms | `GET` and `POST` forms with `<input type="submit">`; 303 after every `POST` | (analysis) |
| Code field | `type="text"`, `autocapitalize="characters"`, `autocomplete="off"`, `spellcheck="false"`, no `maxlength`, paste allowed, letter-spaced | [V1: [GOV.UK security code](https://raw.githubusercontent.com/alphagov/govuk-design-system/main/src/patterns/confirm-a-phone-number/index.md), [MDN autocapitalize](https://raw.githubusercontent.com/mdn/content/main/files/en-us/web/html/reference/global_attributes/autocapitalize/index.md)]. `autocomplete="one-time-code"` is for sign-in OTPs, not this [V1: [MDN autocomplete](https://raw.githubusercontent.com/mdn/content/main/files/en-us/web/html/reference/attributes/autocomplete/index.md)] |
| Auto-refresh | Only on a waiting page, 5 s, always with a manual button | It disorients assistive-technology users [V1: [MDN meta refresh](https://raw.githubusercontent.com/mdn/content/main/files/en-us/web/html/reference/elements/meta/http-equiv/index.md)] |
| Targets | At least 24 × 24 CSS px by WCAG; 48 px on e-ink classes | [V1: [WCAG 2.5.8](https://raw.githubusercontent.com/w3c/wcag/main/understanding/22/target-size-minimum.html)]; 48 px is a (proposal) |
| Time limits | A pair code must be renewable in one tap; show the absolute expiry time | [V1: [WCAG 2.2.1](https://raw.githubusercontent.com/w3c/wcag/main/understanding/20/timing-adjustable.html)] |
| Redundant entry | The code survives a sign-in redirect; never ask for it twice | [V1: [WCAG 3.3.7](https://raw.githubusercontent.com/w3c/wcag/main/understanding/22/redundant-entry.html)] |
| First screen | The gift, not a form: cover, title, "De la part de …", message, one primary button, the visible list of other ways | UX note |
| In-browser reader | Possible later for phones and computers with foliate-js (EPUB, MOBI, KF8; "does not care about older browsers"); never loaded on an e-ink class | [V1: [foliate-js](https://raw.githubusercontent.com/johnfactotum/foliate-js/main/README.md)] |

### 7.2 User-agent classification

Rules are applied in order; the first match wins. `?as=kindle|kobo|eink|phone|desktop` on any gift URL overrides the result and is carried by the links of the "Autres façons de lire" list.

| # | Test on the `User-Agent` | Class | Evidence | Notes |
|---|---|---|---|---|
| 1 | contains `Kindle` | `kindle` | Parsers key on `Kindle/x.y` [V2: [uap-core](https://raw.githubusercontent.com/ua-parser/uap-core/master/regexes.yaml), [Matomo devices](https://raw.githubusercontent.com/matomo-org/device-detector/master/regexes/device/mobiles.yml)]; sample `…Kindle/3.0 (screen 600x800; rotate)` [V1] | **A 2024 Kindle on firmware 5.18.6 is not matched** [V1]; its string is unknown. Strings with `Silk/` or `Kindle Fire` (Fire tablets) are tested first and go to `mobile` [M] |
| 2 | contains `Kobo` | `kobo` | Both parsers [V2]; sample `…Mobile Safari/533.1 (Kobo Touch)` [V1]; see also [Matomo browsers](https://raw.githubusercontent.com/matomo-org/device-detector/master/regexes/client/browsers.yml) | The only class that may be served `.kepub.epub` |
| 3 | contains `PocketBook/` | `pocketbook` | One parser; sample `…AppleWebKit/534.34 PocketBook/623 (screen 758x1024; FW T623.4.4.616) Mobile` [V1] | Vivlio devices are assumed to match [M] |
| 4 | contains `tolino` (any case) or `TOLINO_BROWSER` | `tolino` | One parser each [V1]; send2ereader [V1] | Older Android tolino: show the "damaged file" fallback text [V1] |
| 5 | contains `eReader` | `eink_other` | Kobo Libra Colour 2026 and tolino Shine Color 2024 [V1 each] | Kobo and tolino cannot be told apart: serve plain `.epub` |
| 6 | contains `iPhone`, `iPad`, or `Android` with `Mobile` | `mobile` | — | An Android e-ink device lands here and behaves like a phone [M] |
| 7 | anything else | `desktop` | — | **A current Kindle lands here.** The `desktop` page must therefore be as light and script-free as the others and must list "Sur un Kindle" |

Unknown strings are counted by class, not stored (doc 09, rule 39). Detection broke three times in two years (§5.10, row 10); the typed entry point `/` must work with no detection at all.

### 7.3 Serving files

| Rule | Basis |
|---|---|
| The URL path ends with the real file name: `/d/{session}/{name}.epub` | "needed for Kobo browsers" [V1: calibre] |
| File names are ASCII: letters, digits, `.`, `_`, `-`; at most 64 characters; transliterated from the title; fallback `livre.epub`. No `!`, no comma | Kindle browser [V1: send2ereader, calibre]; Send-to-PocketBook [V1]; Kobo `!` [V1] |
| `Content-Type: application/epub+zip` (or `application/pdf`); `Content-Length` always; no compression of the body; `Accept-Ranges`; `HEAD` supported | `docs/api.md` |
| `Content-Disposition: attachment; filename="{ascii}"`. Add `filename*` only for `mobile` and `desktop` | calibre removes the header for `Kindle/3` when names are non-ASCII [V1]; send2ereader sets it for Kindle only, with a safe name [V1] |
| Format served: `.epub` to every class. `.kepub.epub` only when the uploader set `allow_format_conversion` **and** the user agent contains `Kobo` | §5.10, row 8; doc 09, rule 13 |
| No EPUB download button on class `kindle` | [V2] |
| Largest file: 50 MB, the upload limit | `docs/api.md`; equal to the Send to Kindle e-mail limit as recalled [M] |
| Licence files: `.lcpl` as `application/vnd.readium.lcp.license.v1.0+json`, `.acsm` as `application/vnd.adobe.adept+xml` | LCP 1.0 [V1]; the ACSM type is [M] |

### 7.4 Tokens

| Token | Form | Size | Lifetime | Uses | Stored as | Appears in |
|---|---|---|---|---|---|---|
| **Gift code** | 12 consonants | 47.5 bits | No expiry by default | Hosted file: 10 sessions; reseller: one claim | HMAC-SHA-256 with a server secret | Card, QR, `/k/{CODE}` |
| **Download session** | Random, URL-safe | 128 bits | 15 minutes (proposal, doc 09 rule 15) | Any number of `GET`/`Range` requests within the lifetime; counts once against the cap | Hash | `/d/{session}/…`, `/o/{session}/…` |
| **Outbound session** | Random | 128 bits | 15 minutes, single use | One redirect | Hash | `/out/{session}` (§7.7) |
| **Pair code** | 6 consonants, `XXX-XXX`, no check symbol | 25.9 bits | 15 minutes | Single use; one live per gift | Hash | Phone screen, typed at `/e` |
| **Manage token** | Random | 128 bits or more | Life of the gift | — | Hash | Giver's link |

`redeemed_at` is set when the first session is created. The download counter shown to the recipient counts completed transfers.

### 7.5 Pairing flow

**Direction A (v1), lane R2 only.**

1. Phone, after the reseller sign-in: `POST /k/{code}/pair` → page with the pair code (`pair-number`), the host to type (`pair-url`), the absolute expiry time, and a "Nouveau code" button.
2. E-reader: `GET /e` → form (`pair-form`, `pair-input`, `pair-submit`).
3. E-reader: `POST /e` → the code is consumed → 303 to the session that serves the licence.
4. Creating a new pair code voids the previous one. At most 3 pair codes per gift per hour.

**Direction B (specified, not built in v1).** The e-reader opens `/r` and is shown a 4-consonant key with a 5 s meta refresh and a "Continuer" button. The phone, on the gift page, chooses "Envoyer sur ma liseuse" and types the key; the confirmation screen names the device class that asked ("Une liseuse Kobo demande ce livre"). The key dies 30 s after the e-reader stops refreshing, 15 minutes at most. A guessed key yields nothing without a gift. The simulator keeps a scenario for it so that the switch stays cheap.

### 7.6 Rate limits

A gift code has no username: every guess is a guess against all live codes. Per-code lockout is useless; the controls are entropy, a per-client limit, a global failure budget, and a bounded number of live codes.

Evidence: printed "look-up secrets" under 64 bits must be rate-limited, stored hashed and used once [V1: [NIST SP 800-63B §5.1.2](https://raw.githubusercontent.com/usnistgov/800-63-3/nist-pages/sp800-63b/sec5_authenticators.md)]; growing delays are an allowed tool, and the user should be told how many attempts remain and how long to wait [V1: NIST §5.2.2, §10]; lockout per account invites denial of service and per-IP limits are evaded with proxy lists [V1: [OWASP brute force](https://raw.githubusercontent.com/OWASP/www-community/master/pages/controls/Blocking_Brute_Force_Attacks.md)]; codes must be random, long enough and single use [V1: [OWASP Forgot Password](https://raw.githubusercontent.com/OWASP/CheatSheetSeries/master/cheatsheets/Forgot_Password_Cheat_Sheet.md)]; GitHub allows 50 device-code submissions an hour per application [V1].

Expected successful guesses, for an attacker sustaining 200,000 guesses a day (an assumption, not a measurement) [calc]:

| Scheme | Bits | 10,000 live codes | 100,000 | 1,000,000 |
|---|---|---|---|---|
| Crockford, 9 random + check (`docs/api.md` v0.1) | 45.0 | 1 per 48 years | 1 per 4.8 years | 1 per 6 months |
| **20 consonants, 11 random + check (kept)** | 47.5 | 1 per 280 years | 1 per 28 years | 1 per 2.8 years |
| 20 consonants, 14 random + check (reserved) | 60.5 | — | — | 1 per 22,000 years |
| 12 digits | 39.9 | 1 per 1.4 years | 1 per 7 weeks | 1 per 5 days |

Limits (all proposals):

| Limit | Value | Effect |
|---|---|---|
| Unknown, well-formed codes per client | 10 per 10 minutes (`EBC_RATE_UNKNOWN_PER_10MIN`) | `429` with `Retry-After`; the page states the waiting time. No CAPTCHA (doc 09, rule 45) |
| Client | IPv4 /24 or IPv6 /56 | — |
| Codes failing the check symbol | Not counted | They reveal nothing: the check is public arithmetic |
| Global unknown-code budget | Alert above 50 an hour; above 500 an hour, failures are delayed by 3 s and the per-client limit drops to 3 per 10 minutes for one hour | Valid codes are never delayed. At 500 an hour (12,000 a day) and 1,000,000 live codes: one success per 47 years [calc] |
| Live codes | Dead on cancellation, on fulfilment (R1), or when the session cap is reached | Bounds the target |
| Length switch | New codes get 15 symbols before 250,000 codes are live | At 200,000 guesses a day, 280,000 live codes mean one success per 10 years [calc] |
| Pair code failures per client | 5 per 15 minutes | `429` |
| Global pair failures | Alert above 20 an hour; above 200 an hour pairing is switched off for one hour and the page says to type the card code | At 200 an hour and 50 live pair codes: one theft per 267 days of sustained attack [calc] |
| Send to device | 3 per gift per day; allowed domains `kindle.com`, `pbsync.com`; the address is not stored | doc 09, rule 42 |

### 7.7 Privacy and headers

- Every gift page: `Cache-Control: no-store`, `Referrer-Policy: no-referrer`, `X-Robots-Tag: noindex, nofollow`, `X-Content-Type-Options: nosniff`, `Content-Security-Policy: default-src 'self'`; no cookie, no analytics, no third-party resource (`docs/api.md`; doc 09, rule 41).
- **Old engines may ignore `Referrer-Policy`** (assumption; the header is younger than the sampled engines). So no page whose URL contains the code carries an external link. Outbound navigation (to a reseller, to a store) goes `POST /k/{code}/redeem` → 303 → `GET /out/{session}` → the external site. The Referer, if sent, holds a spent session token, not the code.
- The server never fetches third-party URLs (`docs/api.md`).
- The redemption record holds the code hash, a timestamp, the device class and the country; raw IP and user agent stay in security logs for 6 months at most (doc 09, rule 39).

### 7.8 Changes to `docs/api.md` v0.1

| # | Change |
|---|---|
| 1 | `GiftCode`: 12 consonants, `XXXX-XXXX-XXXX`, check symbol of §6.5; parser accepts 12 and 15 |
| 2 | Claim URL in the QR: `https://{HOST}/K/{CODE}`; `/K/` and `/k/` are equivalent; `GET /{code}` 303-redirects to `/k/{CODE}` |
| 3 | Pair number → pair code: 6 consonants, single use, 15 minutes, one live per gift, global failure budget; offered only for `reseller_licence` |
| 4 | Device-class table replaced by §5.5; `?as=` override; `mobile` leads with reading on the phone |
| 5 | `kindle` class: no EPUB download; phone routes first; e-mail form last |
| 6 | `GET /out/{session}` interstitial for every outbound link |
| 7 | Gift page shows the download count and the last date; error pages show attempts left and the waiting time |
| 8 | Optional `reading_hint` on gift creation; optional opening date; opt-in giver notification |
| 9 | Format served is `.epub` unless conversion is allowed and the user agent contains `Kobo` |
| 10 | New standalone delivery "store gift link", handled entirely in the browser (no server route) |

---

## 8. Test matrix for fake e-readers

The simulator in `demo/ereaders/` already has ten profiles. Their capabilities are hypotheses graded like the notes; nothing was measured on hardware. This section says what each profile must and must not be able to do, and which profiles are missing.

### 8.1 Device profiles

| Profile id | Emulates | Must be able to | Must not be able to | Expected primary action (file lanes) | Grade |
|---|---|---|---|---|---|
| `kobo-clara` | Kobo with `Kobo Touch` in the user agent | Load pages without JavaScript; type host and code; download `.epub` and `.kepub.epub` on 443; open both | Scan a QR; run scripts; open LCP; reach other ports; save a non-ASCII or `!` file name | Télécharger | V2 download; V1 quirks; M engine |
| `kobo-ereader-ua` | Kobo Libra Colour, user agent with `eReader` only | Same; download plain `.epub` | Be detected as `kobo`; receive `.kepub.epub` | Télécharger (`.epub`) | V1 |
| `kindle-paperwhite` | Kindle with `Kindle/3` | Load pages; type; download `.azw3`/`.mobi`/`.txt` with ASCII names; receive Send to Kindle deliveries | Download or open `.epub`; download `.pdf`; non-ASCII names; open LCP or Adobe | No download; Kindle instructions | V2 |
| `kindle-2024` | Kindle 2024, user agent without `Kindle` | Load the `desktop` page and still find "Sur un Kindle"; follow `?as=kindle` | Open the `.epub` if it follows the desktop button (the simulator reports a clear failure) | `desktop` page; test asserts the Kindle link exists and works | V1 for the misdetection; the UA string is an assumption |
| `pocketbook` | PocketBook with `PocketBook/` | Download and open `.epub`; receive `@pbsync.com` mail with ASCII names; open `.lcpl` and `.acsm` on the device | Scan; scripts; non-ASCII mail attachment names | Télécharger | M (browser), V1 (mail) |
| `vivlio` | Vivlio (PocketBook hardware) signed in to a bookseller account | As `pocketbook`; **sync the fake store's library** | — | Télécharger; in R1: library sync | M |
| `tolino` | tolino 2024+ on Kobo firmware, `eReader` | Download and open plain `.epub` | Be detected as `tolino` by name | Télécharger (`.epub`) | V1 |
| `tolino-legacy-android` | tolino Vision 4 HD | Complete the download | Open it: the file is reported damaged; the test asserts that the fallback text is on the page | Télécharger, then fallback | V1 |
| `generic-eink` | Unknown e-ink browser | Forms only; plain `.epub` | Everything optional | Télécharger | assumption |
| `thorium-desktop` | LCP and OPDS reading app | Follow `opds://`; read OPDS 1.2 and 2.0 feeds; open `.lcpl`; show the hint; accept the passphrase; register with the status document (`ready` → `active`) | Open Adobe or Kobo DRM | Download; "Ouvrir dans une application" | V1 |

Profiles to add:

| Profile id | Emulates | Must be able to | Must not be able to | Grade |
|---|---|---|---|---|
| `phone-ios`, `phone-android` | The scanning phone | Decode the QR payload from the PDF; open the claim URL; modern browser; download and "open in" a reading app; share to a Kindle account (simulated); sign in to a fake store | — | M |
| `phone-android-strict` | An Android scanner that does not normalise the scheme | Open `https://…`; **fail on `HTTPS://…`** | — | V2 for the hazard; guards §6.3 |
| `android-eink` | Onyx Boox | As a phone; camera optional | — | M |
| `no-browser` | reMarkable | Receive a file from a phone or computer relay only | Open any URL | M |
| `kobo-fnac-sync` | Kobo linked to a fake Fnac account | Show a title after the store confirms fulfilment and the device syncs; no browser use at all | Receive anything without the entitlement | M |
| `kindle-account` | Amazon account behind a Kindle | Accept EPUB by mail from an approved sender; require the confirmation click when configured; reject AZW3; reject a share of EPUBs as "intake failure" (configurable) | Accept mail from an unapproved sender | V1 / V2 |

Every e-ink profile runs with JavaScript off. A separate run repeats the phone profiles with JavaScript off.

### 8.2 Store and service fixtures

| Fixture | Type | Must implement |
|---|---|---|
| Fake 7switch (fictional name, "demo — not affiliated") | Open-file store | "Offrir en eBook cadeau" at checkout; `hosted_file` with basis `licensed_distributor`, the file pre-marked with the order reference and the recipient name given at checkout; an R1 variant in which the store's own claim page marks the file at claim time; one title delivered as `reseller_licence` with an LCP test-profile licence; account library with re-download |
| Fake Fnac (fictional name, "demo — not affiliated") | Account-sync platform without gifting | "eBook cadeau" at checkout; `reseller_redirect`; sign in or create an account; entitlement added; fulfilment callback; a sync endpoint read by `kobo-fnac-sync` |
| Fake store with gifting (fictional name) | The kind of store that sells its own title-level gift | Issues its own redemption link for lane S1; same-country rule; the link works once |
| Fake LCP licence server | Test profile only | `.lcpl` with `publication` and `hint` links; status document with `register` |
| Fake mail-in service | Send to Kindle, Send-to-PocketBook | Approved-sender list, optional confirmation click, file-name rules, size limit |

No fixture shows a customer uploading a purchased file (doc 09, rule 4).

### 8.3 Scenarios

| # | Scenario | Profiles | Pass condition |
|---|---|---|---|
| 1 | Scan the card PDF, open the gift page | phones | The payload decodes to the claim URL; the page shows title, giver, message, primary action |
| 2 | Upper-case scheme regression | `phone-android-strict` | The generated card opens; a card forced to `HTTPS://` fails |
| 3 | Type host and code on the e-reader, download | `kobo-clara`, `kobo-ereader-ua`, `tolino`, `pocketbook`, `vivlio`, `generic-eink` | Book in the device library; at most 4 actions; typed characters = host + 12; byte-identical file |
| 4 | Type `host/code` in the address bar | same | 303 to `/k/{CODE}`; 3 actions |
| 5 | Typo | all | Check failure → "faute de frappe"; not counted |
| 6 | Unknown codes | any | 11th attempt in 10 minutes → `429` with `Retry-After` and a stated wait |
| 7 | Kindle typed entry | `kindle-paperwhite`, `kindle-2024` | No EPUB offered on `kindle`; the Kindle instructions are reachable from the `desktop` page |
| 8 | Send to Kindle by e-mail | `kindle-account` | Delivered only with an approved sender; the address is not stored; the page explained the step first |
| 9 | Reseller library | phone + `kobo-fnac-sync`, phone + `vivlio` | Sign in → fulfilled → title on the device after sync; nothing typed on e-ink; second claim refused |
| 10 | Licence with pairing | phone + `pocketbook`, `thorium-desktop` | Pair code works once, within 15 minutes; the licence opens; status becomes `active` |
| 11 | Pair-code guessing | attacker client | Limits of §7.6 hold; pairing switches off at the global budget |
| 12 | Damaged download | `tolino-legacy-android` | Fallback text shown; USB route documented |
| 13 | Download cap | any | 11th session refused; the page shows count and last date |
| 14 | Opening date, cancellation, reissue | any | Before the date: only the date; cancelled code → `410`; reissued code works |
| 15 | Store gift link | phones, fake store with gifting | The link appears in no request to our server; the store accepts it once |
| 16 | No referrer leak | all | No request to another origin carries the code in its URL or Referer |
| 17 | OPDS | `thorium-desktop` | `catalog.xml` and `catalog.json` list the one publication and it downloads |

### 8.4 Invariants checked on every recipient page

HTML under 30 kB; no script needed for any action; no external resource; every action is a form or a link; stable element ids of `docs/api.md`; no `maxlength` on the code field; URL paths of files end with an ASCII file name; no page with the code in its URL contains an external link; text contrast at least 4.5:1; targets at least 24 px, 48 px on e-ink classes.

---

## 9. Open questions

Ordered by what they block. "Test" means hands-on with a real device; "search" means a follow-up run with search budget or with the listed hosts allowed in the sandbox.

| # | Question | Blocks | How to close |
|---|---|---|---|
| 1 | On-screen keyboards of Kobo, Kindle, PocketBook, Vivlio: are digits on a second layer? Is `/` on the URL keyboard? How long does typing 18 characters take? | The letters-only code; the choice between direct typing and pairing direction B | Test on a Kobo and a Vivlio |
| 2 | Does the browser of a current Kobo, Vivlio/PocketBook and Kindle reach our HTTPS host (TLS version, root store), render the pages, and save an `.epub`? Maximum download size? | Every "works on …" claim on the card; certificate choice | Test |
| 3 | Vivlio and PocketBook: browser download, on-device `.lcpl` and `.acsm` handling, Send-to-PocketBook confirmation, native OPDS | Lane R2; the `pocketbook` and `vivlio` profiles | Test; search |
| 4 | Do iOS and Android stock cameras open our mixed-case payload, and what do they do with a fully upper-case one? | §6.3 (a formality for the mixed form) | Test on two phones |
| 5 | Current gift flows of French resellers: Fnac/Kobo, 7switch, Cultura, Furet du Nord, leslibraires.fr, ePagine shops. Can Fnac/Kobo create a title entitlement for a recipient at all? | Lane R1 with the main French channel | Search; contact the resellers |
| 6 | Amazon.fr: is the redemption link copyable, how long is it, do the terms allow it on a third-party card? Same for Decitre and the tolino shops | Lane S1 | Search; legal check (doc 09 §13) |
| 7 | Kindle: the share-sheet route through the Kindle app, the Send to Kindle web page, current limits (50 MB / 200 MB), whether those routes skip the approved-sender step; the user-agent string of 2024+ Kindles; whether the newer browser is Chromium-based and which file types it saves | §5.6; UA rule 1 | Test; search |
| 8 | Which Kobo models have Dropbox or Google Drive in 2026; what firmware 5.x/6.0 changed in the browser; exact user-agent strings | Help text; UA rules 2 and 5 | Search; test |
| 9 | Which e-readers and apps are LCP-certified (EDRLab's list); whether a Thorium mobile app exists; status of TEA's CARE; which French storefronts ship LCP | Lane R2 scope | Search (`www.edrlab.org`) |
| 10 | Does tolino's Kobo-based firmware open `.kepub.epub`? Is tolino cloud still operated as described? | KEPUB rule; tolino help text | Test; search |
| 11 | Measured conversion: scan-to-download rates, drop-off per step, BookFunnel support data | The step counts of §5.4 are reasoning only | Own analytics-free funnel counters in a pilot; search |
| 12 | RFC 8628 wording on user codes; Libby's setup code; Kobo's activation code format | Precedents only | Search |
| 13 | Physical QR sizing rules (0.4 mm floor, distance / 10) from a standard or a printer guide | §6.3 margins of safety | Search; print tests |
| 14 | BookFunnel's exact flow; Seebook, Livrio, Epubli and tolino cards; tolino print-at-home voucher | Precedents; competitor claims | Search |
| 15 | Watermark versus DRM share among French publishers; 7switch's protection mix | Size of lane R3 | Search (SNE / KPMG baromètre) |
| 16 | The attack rate assumed in §7.6 (200,000 guesses a day) | Length switch threshold | Revisit with real traffic |
| 17 | European Accessibility Act scope and French transposition for this service | §6.6 | doc 09 §10; counsel |
| 18 | Whether send.djazz.se is permanently offline; who operates bookdrop.cc, leaf.casa, send2ereader.net | Nothing: we do not depend on them | — |
| 19 | Scratch-off and seal-sticker costs; gift-card fraud figures | §6.7 | Printer quotes |

Hosts worth allowing for a verification pass: `www.edrlab.org`, `send.djazz.se`, `blog.the-ebook-reader.com`, `help.kobo.com`, `www.amazon.fr`, `www.fnac.com`, `www.7switch.com`, `help.vivlio.com`, `www.rfc-editor.org`.

---

## Sources

**Read or re-read in the consolidation pass (2026-11-12)**

- send2ereader [index.js](https://raw.githubusercontent.com/daniel-j/send2ereader/master/index.js); issues [#28](https://github.com/daniel-j/send2ereader/issues/28), [#63](https://github.com/daniel-j/send2ereader/issues/63), [#70](https://github.com/daniel-j/send2ereader/issues/70), [#76](https://github.com/daniel-j/send2ereader/issues/76)
- ZXing Android client [ResultHandler.java](https://raw.githubusercontent.com/zxing/zxing/master/android/src/com/google/zxing/client/android/result/ResultHandler.java)
- Android [Uri.java](https://raw.githubusercontent.com/aosp-mirror/platform_frameworks_base/master/core/java/android/net/Uri.java)

**From `notes/ereader-ux.devices.md` (all fetched 2026-11-12)**

- send2ereader: [README](https://github.com/daniel-j/send2ereader), [index.js](https://raw.githubusercontent.com/daniel-j/send2ereader/master/index.js), [download.html](https://raw.githubusercontent.com/daniel-j/send2ereader/master/static/download.html), [upload.html](https://raw.githubusercontent.com/daniel-j/send2ereader/master/static/upload.html), [common.js](https://raw.githubusercontent.com/daniel-j/send2ereader/master/static/common.js), [LICENSE](https://raw.githubusercontent.com/daniel-j/send2ereader/master/LICENSE), [commits](https://github.com/daniel-j/send2ereader/commits/master), [forks](https://github.com/daniel-j/send2ereader/forks), issues [#27](https://github.com/daniel-j/send2ereader/issues/27) [#28](https://github.com/daniel-j/send2ereader/issues/28) [#32](https://github.com/daniel-j/send2ereader/issues/32) [#36](https://github.com/daniel-j/send2ereader/issues/36) [#50](https://github.com/daniel-j/send2ereader/issues/50) [#51](https://github.com/daniel-j/send2ereader/issues/51) [#57](https://github.com/daniel-j/send2ereader/issues/57) [#58](https://github.com/daniel-j/send2ereader/issues/58) [#63](https://github.com/daniel-j/send2ereader/issues/63) [#70](https://github.com/daniel-j/send2ereader/issues/70) [#76](https://github.com/daniel-j/send2ereader/issues/76) [#80](https://github.com/daniel-j/send2ereader/issues/80), [PR #59](https://github.com/daniel-j/send2ereader/pull/59); fork [epub2kobo](https://github.com/ahxxm/epub2kobo)
- calibre: [FAQ](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/faq.rst), [server manual](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/server.rst), [Changelog](https://raw.githubusercontent.com/kovidgoyal/calibre/master/Changelog.txt), drivers [kindle](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/kindle/driver.py) [kobo](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/kobo/driver.py) [eb600/tolino/pocketbook](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/eb600/driver.py) [cybook](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/cybook/driver.py) [mtp defaults](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/mtp/defaults.py), [email.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/gui2/email.py), [send_email wizard](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/gui2/wizard/send_email.py), [srv/legacy.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/srv/legacy.py), [srv/content.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/srv/content.py)
- Kobo: [epub-spec](https://raw.githubusercontent.com/kobolabs/epub-spec/master/README.md), [kobolabs org](https://github.com/kobolabs), [Kobo-Reader README](https://raw.githubusercontent.com/kobolabs/Kobo-Reader/master/README), [NickelMenu doc](https://raw.githubusercontent.com/pgaskin/NickelMenu/master/res/doc), [kepubify](https://raw.githubusercontent.com/pgaskin/kepubify/master/README.md), [KoboCloud](https://raw.githubusercontent.com/fsantini/KoboCloud/master/README.md), [Kobo UNCaGED](https://raw.githubusercontent.com/shermp/Kobo-UNCaGED/master/README.md)
- Calibre-Web: [README](https://raw.githubusercontent.com/janeczku/calibre-web/master/README.md), [Kobo integration](https://raw.githubusercontent.com/wiki/janeczku/calibre-web/Kobo-Integration.md), [Configuration](https://raw.githubusercontent.com/wiki/janeczku/calibre-web/Configuration.md), [FAQ](https://raw.githubusercontent.com/wiki/janeczku/calibre-web/FAQ.md); [Calibre-Web-Automated](https://raw.githubusercontent.com/crocodilestick/Calibre-Web-Automated/main/README.md); [Komga](https://raw.githubusercontent.com/gotson/komga/master/README.md)
- KOReader: [README](https://raw.githubusercontent.com/koreader/koreader/master/README.md), [OPDS wiki](https://github.com/koreader/koreader/wiki/OPDS-support), [plugins](https://github.com/koreader/koreader/tree/master/plugins), [cloud providers](https://github.com/koreader/koreader/tree/master/plugins/cloudstorage.koplugin/providers), [qrclipboard](https://raw.githubusercontent.com/koreader/koreader/master/plugins/qrclipboard.koplugin/main.lua), [opds main.lua](https://raw.githubusercontent.com/koreader/koreader/master/plugins/opds.koplugin/main.lua), [calibre wiki](https://raw.githubusercontent.com/wiki/koreader/koreader/calibre.md), [Kindle install](https://github.com/koreader/koreader/wiki/Installation-on-Kindle-devices), [PocketBook install](https://github.com/koreader/koreader/wiki/Installation-on-PocketBook-devices), [Android install](https://raw.githubusercontent.com/wiki/koreader/koreader/Installation-on-Android-devices.md)
- [Standard Ebooks — how to use our ebooks](https://raw.githubusercontent.com/standardebooks/web/master/www/help/how-to-use-our-ebooks.php); [kindle-send](https://raw.githubusercontent.com/nikhil1raghav/kindle-send/main/README.md)
- Readium / EDRLab: [lcp-specs README](https://raw.githubusercontent.com/readium/lcp-specs/master/README.md), [LCP automatic key retrieval](https://raw.githubusercontent.com/readium/lcp-specs/master/notes/lcp-key-retrieval.md), [Thorium README](https://raw.githubusercontent.com/edrlab/thorium-reader/develop/README.md), [EDRLab repositories matching "thorium"](https://github.com/orgs/edrlab/repositories?q=thorium)
- tolino: [tolino-python](https://raw.githubusercontent.com/hzulla/tolino-python/master/README.md), [tolino-calibre-sync](https://raw.githubusercontent.com/darkphoenix/tolino-calibre-sync/master/README.md); Boox: [onyx-send2boox](https://raw.githubusercontent.com/hrw/onyx-send2boox/main/README.md); reMarkable: [awesome-reMarkable](https://raw.githubusercontent.com/reHackable/awesome-reMarkable/master/README.md)

**From `notes/ereader-ux.ecosystems.md` (fetched 2026-11-12)**

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

**From `notes/ereader-ux.ux.md` (fetched 2026-11-12)**

- Device flow and cross-device security: [GitHub OAuth docs](https://raw.githubusercontent.com/github/docs/main/content/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps.md) · [Microsoft device code](https://raw.githubusercontent.com/MicrosoftDocs/entra-docs/main/docs/identity-platform/v2-oauth2-device-code.md) · [IETF OAuth cross-device security draft](https://raw.githubusercontent.com/oauth-wg/oauth-cross-device-security/main/draft-ietf-oauth-cross-device-security.md) · [node-oidc-provider](https://raw.githubusercontent.com/panva/node-oidc-provider/main/docs/README.md) · [Keycloak](https://raw.githubusercontent.com/keycloak/keycloak/main/docs/documentation/server_admin/topics/sso-protocols/con-oidc-auth-flows.adoc)
- Codes and throttling: [NIST SP 800-63B §5](https://raw.githubusercontent.com/usnistgov/800-63-3/nist-pages/sp800-63b/sec5_authenticators.md) · [NIST SP 800-63B §10](https://raw.githubusercontent.com/usnistgov/800-63-3/nist-pages/sp800-63b/sec10_usability.md) · [OWASP Forgot Password](https://raw.githubusercontent.com/OWASP/CheatSheetSeries/master/cheatsheets/Forgot_Password_Cheat_Sheet.md) · [OWASP Blocking Brute Force](https://raw.githubusercontent.com/OWASP/www-community/master/pages/controls/Blocking_Brute_Force_Attacks.md) · [OWASP Session Management](https://raw.githubusercontent.com/OWASP/CheatSheetSeries/master/cheatsheets/Session_Management_Cheat_Sheet.md) · [base32-crockford README](https://raw.githubusercontent.com/jbittel/base32-crockford/master/README.rst) and [source](https://raw.githubusercontent.com/jbittel/base32-crockford/master/base32_crockford.py) · [crockford crate](https://raw.githubusercontent.com/archer884/crockford/master/README.md) · [BIP-173](https://raw.githubusercontent.com/bitcoin/bips/master/bip-0173.mediawiki) · [Open Location Code](https://raw.githubusercontent.com/google/open-location-code/main/Documentation/Specification/specification.md)
- QR: [python-qrcode](https://raw.githubusercontent.com/lincolnloop/python-qrcode/main/README.rst) · [segno make](https://raw.githubusercontent.com/heuer/segno/master/docs/make.rst) · [segno modes](https://raw.githubusercontent.com/heuer/segno/master/docs/qrcode-modes.rst) · [Nayuki](https://raw.githubusercontent.com/nayuki/QR-Code-generator/master/Readme.markdown) · [ZXing URIResultParser](https://raw.githubusercontent.com/zxing/zxing/master/core/src/main/java/com/google/zxing/client/result/URIResultParser.java)
- Devices and pairing precedents: [send2ereader index.js](https://raw.githubusercontent.com/daniel-j/send2ereader/master/index.js) · [kobo-book-downloader](https://raw.githubusercontent.com/TnS-hun/kobo-book-downloader/master/README.md) · [odmpy](https://raw.githubusercontent.com/ping/odmpy/master/README.md) · [calibre server manual](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/server.rst) · [calibre FAQ](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/faq.rst) · [Calibre-Web FAQ](https://raw.githubusercontent.com/wiki/janeczku/calibre-web/FAQ.md) · [Standard Ebooks help](https://raw.githubusercontent.com/standardebooks/web/master/www/help/how-to-use-our-ebooks.php) · [foliate-js](https://raw.githubusercontent.com/johnfactotum/foliate-js/main/README.md)
- User agents: [uap-core regexes](https://raw.githubusercontent.com/ua-parser/uap-core/master/regexes.yaml) · [uap-core tests](https://raw.githubusercontent.com/ua-parser/uap-core/master/tests/test_device.yaml) · [Matomo devices](https://raw.githubusercontent.com/matomo-org/device-detector/master/regexes/device/mobiles.yml) · [Matomo browsers](https://raw.githubusercontent.com/matomo-org/device-detector/master/regexes/client/browsers.yml) · [Matomo fixtures](https://raw.githubusercontent.com/matomo-org/device-detector/master/Tests/fixtures/tablet-3.yml)
- Accessibility and forms: WCAG 2.2 Understanding — [3.3.8](https://raw.githubusercontent.com/w3c/wcag/main/understanding/22/accessible-authentication-minimum.html) · [2.5.8](https://raw.githubusercontent.com/w3c/wcag/main/understanding/22/target-size-minimum.html) · [1.4.3](https://raw.githubusercontent.com/w3c/wcag/main/understanding/20/contrast-minimum.html) · [2.2.1](https://raw.githubusercontent.com/w3c/wcag/main/understanding/20/timing-adjustable.html) · [3.3.7](https://raw.githubusercontent.com/w3c/wcag/main/understanding/22/redundant-entry.html) · [3.2.6](https://raw.githubusercontent.com/w3c/wcag/main/understanding/22/consistent-help.html) · [W3C WAI EU policies](https://raw.githubusercontent.com/w3c/wai-policies-prototype/master/_policies/european-union.md) · [GOV.UK security code pattern](https://raw.githubusercontent.com/alphagov/govuk-design-system/main/src/patterns/confirm-a-phone-number/index.md) · [GOV.UK text input](https://raw.githubusercontent.com/alphagov/govuk-design-system/main/src/components/text-input/index.md) · MDN [autocapitalize](https://raw.githubusercontent.com/mdn/content/main/files/en-us/web/html/reference/global_attributes/autocapitalize/index.md), [autocomplete](https://raw.githubusercontent.com/mdn/content/main/files/en-us/web/html/reference/attributes/autocomplete/index.md), [meta refresh](https://raw.githubusercontent.com/mdn/content/main/files/en-us/web/html/reference/elements/meta/http-equiv/index.md)

**Reused from sibling documents (found by other agents' searches; not re-opened here)**

- Gifting flows (doc 12 §4.2, doc 09 §2.3): [Amazon.fr help](https://www.amazon.fr/gp/help/customer/display.html?nodeId=GVWGP284MQ6ZRM59) · [KDP help](https://kdp.amazon.com/fr_FR/help/topic/G200652260) · [IDBoox](https://www.idboox.com/news-livres/amazon-on-peut-enfin-offrir-un-livre-numerique-kindle/) · [Decitre](https://www.decitre.fr/offrir-un-ebook) · [Decitre FAQ](https://www.decitre.fr/faq/numerique/offrir-un-ebook) · [Liseuses.net](https://www.liseuses.net/offrir-des-ebooks-avec-tea/) · [Vivlio help](https://help.vivlio.com/hc/fr/articles/11334184838428-Je-souhaite-utiliser-un-code-promotionnel-ou-une-carte-cadeau) · [Hugendubel](https://www.hugendubel.de/de/shortcut/ebooksverschenken) · [heise](https://www.heise.de/tipps-tricks/E-Books-verschenken-so-klappt-s-7476189.html) · [test.de](https://www.test.de/E-Books-Nuetzliche-Tipps-und-Tricks-5391265-0/) · [Dealabs](https://www.dealabs.com/discussions/carte-cadeau-kobo-ou-fnac-2273026) · [ebook-enligne.fr](https://ebook-enligne.fr/blog/offrir-ebook-guide-cadeau-numerique/)
- Code tools: [BookFunnel print codes](https://authors.bookfunnel.com/help/print-codes/) · [BookFunnel pricing](https://bookfunnel.com/pricing/) · [Bookwire FAQ](https://bookwire.net/faq-download-codes)
- Past cards: [Publishers Weekly](https://www.publishersweekly.com/pw/print/20140922/64046-enthrill-partners-with-walmart-canadian-publishing-2014.html) · [The Book Designer](https://www.thebookdesigner.com/e-books-for-real-e-book-gift-cards-from-livrada-and-enthrill/) · [Boing Boing](https://boingboing.net/2018/08/22/walmarts-selling-ebooks-now.html/amp) · [The eBook Reader](https://blog.the-ebook-reader.com/2016/01/22/kindle-cards-new-book-specific-kindle-gift-cards)
- Device use in France: [SGDL baromètre 2026](https://www.sgdl.org/sgdl-accueil/le-guide-pratique/ressources-documentaires/barometres-et-observatoire/4867-barometre-sgdl-sofia-sne-2026-des-usages-dachat-et-de-lecture-des-livres-imprimes-numeriques-et-audio)

---

## Implications for the implementation

Numbers marked "(proposal)" are design choices, not measured or legal thresholds. Route, field and element names follow `docs/api.md`.

**Lanes and gift page**

1. `GET /k/{CODE}` always returns an HTML gift page. No request that carries only the code returns a file or a licence.
2. The gift page shows, in this order: cover thumbnail, title, author, "De la part de …", message, the one-line format / protection / compatibility statement, one primary action, and the list "Autres façons de lire". The list is never collapsed or hidden.
3. Primary action by lane and device class follows §5.5. On `mobile` with a file lane the primary action is "Lire sur ce téléphone"; "Sur une liseuse", "Sur un Kindle" and "Sur un ordinateur" follow at the same visual weight. Pairing is not offered for file lanes.
4. `?as=kindle|kobo|eink|phone|desktop` overrides the detected class on every gift URL, and every link in "Autres façons de lire" uses it.
5. Gift creation accepts an optional `reading_hint` (`kobo`, `kindle`, `vivlio`, `pocketbook`, `tolino`, `phone_tablet`, `unknown`; default `unknown`). It orders the choices on the gift page and selects the printed wording of steps 1–3 (§6.2). A reseller checkout that receives `kindle` for a title that is not readable on Kindle shows a warning before payment.
6. Standalone mode offers three deliveries: store gift link (S1, first choice for "a book I bought"), rights-holder file (S2), public-domain or open-licence file (S3). The S1 link is encoded into the QR in the browser and reaches no server route.
7. After an R1 fulfilment the gift page states that the book is in the recipient's library and tells how to sync the e-reader; it shows no account identifier.
8. The gift page shows "téléchargé n fois, dernière fois le …" for hosted files, and the attempts left and waiting time on error pages.

**Code**

9. `GiftCode` = 12 symbols from `BCDFGHJKLMNPQRSTVWXZ`, displayed `XXXX-XXXX-XXXX`; 11 symbols from a CSPRNG without modulo bias, 1 check symbol computed as in §6.5. Unit tests include the vectors `BCDF-GHJK-LMNL`, `ZXWV-TSRQ-PNMP`, `QRST-VWXZ-BCDT`, and assert detection of every single substitution, adjacent transposition, twin error and jump transposition on random codes.
10. The generator rejects codes with fewer than 5 distinct symbols and codes containing a blocklisted acronym (list in data, not code).
11. Parsing is case-insensitive and ignores space, `-`, `.`, `_`. Errors are distinct: wrong length, invalid character, failed check. All three produce HTTP `400` and the "faute de frappe" message, and none counts as an attempt.
12. The parser accepts 12 symbols and 15 symbols (`XXXXX-XXXXX-XXXXX`, 14 random + 1 check). Generation switches to 15 when the count of live codes passes 250,000 (proposal); the threshold is configuration.
13. `crates/ebc-model/src/code.rs` (currently 10 Crockford symbols), `docs/api.md` §1 and doc 09 rules 46–47 are brought in line with items 9–12.
14. Codes are stored as HMAC-SHA-256 with `EBC_CODE_SECRET`. The clear code appears only in the creation response and on the card; no log line contains it (test).

**QR code and card**

15. QR payload for our lanes: `https://` in lower case as a byte segment, then `{HOST}/K/{CODE}` in upper case as an alphanumeric segment; no query string. `claim_url_qr` returns exactly this. A test decodes the QR from the rendered PDF and compares it with the claim URL, ignoring case of host and path.
16. The server treats `/K/` and `/k/` as the same route, and accepts the code in any case. `GET /{code}` with a well-formed code 303-redirects to `/k/{CODE}`.
17. Error correction: level M minimum, raised to Q only when the version stays the same. Quiet zone: 4 modules. Symbol: K100 vector on white, at least 5 mm from folds and trim.
18. Symbol width: 22 mm for versions 2 and 3 (proposal), 20 mm minimum. The generator fails when a module would be under 0.5 mm.
19. S1 payload: the store URL in byte mode. The generator computes the version, sizes the symbol at 0.6 mm per module or more, and refuses a URL that needs more than version 10 at level M (about 213 bytes). Accepted hosts come from the allow-list of doc 09 rule 16.
20. The inside panel carries: headline, giver, message or blank area, three steps (variant by lane and `reading_hint`), QR, host in lower case, code in groups, one fallback line, legal line with format / protection / compatibility and validity, help line. Preflight fails a card that lacks the host or the code in clear.
21. Typography: steps at 12 pt or more; code at 16 pt bold in a monospaced or open-shaped face, never under 12 pt; all K100. Outside panels carry no code.
22. The compatibility line names only device families whose path has passed a hands-on test; until then it reads "liseuses, tablettes et téléphones" and adds the Kindle caveat for file lanes.
23. The printed host is letters and one dot, no hyphen, no digit, no `www`; 6 to 8 characters with the dot (proposal). `EBC_PUBLIC_URL` is validated against that shape at start-up with a warning when it is longer.

**Pages for e-ink**

24. Every recipient page works with JavaScript disabled, sets no cookie, loads nothing from another origin, and weighs under 30 kB excluding one cover thumbnail of at most 20 kB. CI lints these on every route.
25. Layout uses no flexbox or grid dependence, no web font, no animation, no `position: fixed`. Controls are links and `<input type="submit">` forms; every `POST` answers 303.
26. The code field is `type="text"` with `autocapitalize="characters"`, `autocomplete="off"`, `spellcheck="false"`, no `maxlength`, and accepts pasted text.
27. Targets are at least 24 × 24 CSS px, and 48 px when the class is `kindle`, `kobo`, `pocketbook`, `tolino` or `eink_other` (proposal).
28. The server listens for recipients on port 443 only, with TLS 1.2 and 1.3. TLS 1.3-only, ECDSA-only certificates and HTTP authentication are not used until the hands-on test of §9 question 2 has passed.
29. The WebAssembly bundle, the PDF tooling and any in-browser reader are never referenced from a page served to an e-ink class or to a client with `?as=eink|kobo|kindle`.

**User agent**

30. Classification follows the ordered table of §7.2: `Kindle` → `kindle`; `Kobo` → `kobo`; `PocketBook/` → `pocketbook`; `tolino` or `TOLINO_BROWSER` → `tolino`; `eReader` → `eink_other`; `iPhone`, `iPad`, `Android`+`Mobile` → `mobile`; else `desktop`. The table is data with a "last checked" date per row.
31. The `desktop` page meets items 24–27 and lists "Sur un Kindle", because current Kindles are classified `desktop`.
32. `/` shows the code form to every client, whatever its class.

**Files**

33. Download URLs are `/d/{session}/{name}.{ext}` with an ASCII name of at most 64 characters from `[A-Za-z0-9._-]`, transliterated from the title, fallback `livre`.
34. Responses carry the exact media type, `Content-Length`, `Accept-Ranges: bytes`, no content encoding, and `Content-Disposition: attachment; filename="{ascii}"`. `filename*` is added only for `mobile` and `desktop`.
35. Every class receives the stored `.epub`, byte-identical (SHA-256 asserted by a test). `.kepub.epub` is served only when the uploader enabled `allow_format_conversion` and the user agent contains `Kobo`; never to `eink_other`.
36. Class `kindle` gets no EPUB or PDF download button. It gets the Kindle instructions: phone routes first, then the e-mail form with the sender address to approve shown before the field.
37. `POST /k/{code}/send` accepts only addresses in `EBC_SEND_DOMAINS` (`kindle.com`, `pbsync.com`), at most 3 sendings per gift per day (proposal), uses a random ASCII attachment name without commas for `pbsync.com`, and does not store the address.
38. The page under every download button carries a "Le téléchargement n'a pas abouti ?" line leading to a new session and to the USB instructions.

**Tokens, pairing, limits**

39. Download sessions are 128-bit random tokens valid 15 minutes (proposal). Requests within the lifetime, `Range` included, count once against the cap of 10 sessions per gift; the uploader may lower the cap to 1.
40. Outbound navigation goes through `GET /out/{session}`, a single-use 128-bit token valid 15 minutes. No page whose URL contains the code has an external link; a test checks that no cross-origin request carries the code in its URL or Referer.
41. `PairCode` = 6 consonants, displayed `XXX-XXX`, single use, valid 15 minutes, one live code per gift, at most 3 created per gift per hour (proposal). The phone page shows the absolute expiry time and a renew button. Pairing is offered only for `reseller_licence`.
42. Unknown well-formed codes: 10 per client per 10 minutes, then `429` with `Retry-After` and the wait stated in text; no CAPTCHA. A client is an IPv4 /24 or an IPv6 /56 (proposal).
43. Global budget for unknown codes: alert above 50 an hour; above 500 an hour, failed lookups are delayed by 3 s and the per-client limit is 3 per 10 minutes for one hour (proposals). Valid codes are never delayed.
44. `POST /e` failures: 5 per client per 15 minutes; alert above 20 an hour globally; above 200 an hour pairing is disabled for one hour and the page tells the recipient to type the card code (proposals).
45. A code stops being live on cancellation, on R1 fulfilment, and when the session cap is reached.
46. Direction B pairing (`/r`, 4-consonant key shown by the e-reader, 5 s meta refresh plus a manual button, confirmation naming the device class) is not built in v1; the simulator keeps a disabled scenario for it.

**Claim rules**

47. No default expiry; the card prints "sans date limite" or the date. An optional opening date is supported; before it the gift page shows only the date.
48. `reseller_redirect` gifts are claimed once; a second fulfilment with another account returns `409`. `reseller_licence` gifts issue one licence to the recipient at claim time.
49. The giver is not told of the claim unless the recipient opts in on the gift page (default off).
50. The OPDS feeds at `/o/{session}/catalog.xml` and `catalog.json` stay, as a secondary link "Ouvrir dans une application de lecture".

**Simulator and tests**

51. `demo/ereaders` keeps its ten profiles and adds `phone-ios`, `phone-android`, `phone-android-strict`, `android-eink`, `no-browser`, `kobo-fnac-sync` and `kindle-account` (§8.1). Each capability carries its evidence grade.
52. The `kindle-2024` profile's user-agent string is marked as an assumption; issues #63 and #70 quote none.
53. The end-to-end suite covers the 17 scenarios of §8.3. Scenario 3 asserts at most 4 actions and host + 12 typed characters on the e-reader; scenario 9 asserts zero characters typed on e-ink.
54. The fake 7switch, fake Fnac, fake store with gifting, fake LCP server (test profile only) and fake mail-in services implement §8.2. The fake LCP server is labelled as a test profile everywhere.
55. A written hands-on protocol (Kobo Clara, a Vivlio, a current Kindle, one iPhone, one Android phone) covers §9 questions 1, 2, 3, 4 and 7. No device name is added to card text or marketing copy until its row has passed.

**Added after the fact-check** (items 1–55 keep their numbers)

56. Lane R2 (`.lcpl` licence download) may be offered to the `kobo` class only behind a per-country flag, off for France until Kobo OS 6.0 with LCP is released there and a `.lcpl` downloaded by the Kobo browser has been seen to open on a device. Vivlio/PocketBook remain the French e-ink target for R2.
57. The Kindle e-mail route explains, before the user sends, that Amazon may e-mail a verification request to be confirmed within 48 hours; the app and web routes stay listed first.
58. Send to Kindle constants are data: 50 MB per e-mail, 200 MB by web or app; the 50 MB upload cap stays aligned with the e-mail limit.

---

## Verification log

Fact-check of 2026-11-12. Method: 12 fresh web searches (8 extended) written from the claims; result summaries only, vendor pages not opened. Full table with sources: `factcheck/ereader-distribution-ux.md`.

| # | Claim | Verdict | Change made |
|---|---|---|---|
| 1 | 48 % smartphone, 34 % e-reader, 33 % tablet | CONFIRMED | [V2]; sample size added (§1) |
| 2 | Send to Kindle takes EPUB; MOBI dropped; AZW3 refused | CONFIRMED | Marked [FC] |
| 3 | Send to Kindle 50 MB / 200 MB | CONFIRMED (secondary) | [M] replaced (§2.5); implication 58 |
| 4 | Approved senders; confirmation click | CONFIRMED, stronger | 48-hour verification (§2.5, §4.5); implication 57 |
| 5 | Kindle browser Chromium from 5.16.4 | CONFIRMED | §2.1 |
| 6 | Kindle browser saves only Kindle formats | UNSUPPORTED by search | Kept on the primary files |
| 7 | Current Kindle user agent unknown | CONFIRMED (still unknown) | §2.5 note |
| 8 | Kobo has no LCP | **OUTDATED** | Kobo OS 6.0 LCP (§2.2, §2.4, §4.1, §4.4); implication 56 |
| 9 | Vivlio/PocketBook read LCP | CONFIRMED (secondary) | §2.4, §4.4 |
| 10 | Kobo cloud on selected models | CONFIRMED for Libra Colour and Clara | §2.1 |
| 11 | Kindle DRM-free EPUB/PDF downloads (omitted) | CONFIRMED | Added to §2.5 |
| 12 | No Kobo title gifting | UNSUPPORTED either way | §3.1 note |
| 13 | Apple Books gifting | PARTLY CONFIRMED | §3.1 |
| 14–16 | Amazon same-country gifts; tolino/Kobo user agents; cameras | Not re-searched | Kept |

Not re-checked: PocketBook and Vivlio browser downloads and user agents, on-device `.acsm`, tolino cloud, Boox, reMarkable, Bookeen, the OPDS and LCP specification details, TLS support of e-reader browsers, Google Play Books gifting.
