# From paper to e-reader — device capability matrix and redemption UX (as of 2026-11-12)

Scope: what each reading device/app family can actually do to turn a printed QR code + short code
into a book on the screen, and what the community "no cable" tools teach about UX.

## 0. Method and reliability warning (read first)

- **WebSearch was unavailable for this note**: the shared per-turn search budget (200 calls) was already
  exhausted when this agent started; all five initial searches were refused. No search was run.
- Instead, primary sources hosted on GitHub (`github.com`, `raw.githubusercontent.com`) were fetched and
  read directly: source code and docs of send2ereader, calibre, KOReader, Kobo's own `epub-spec`,
  NickelMenu, Calibre-Web, Thorium, Readium LCP specs, Standard Ebooks, and others (~50 documents).
  Every link below is a URL that was actually fetched in this session. Other hosts tried
  (`send.djazz.se`, `www.edrlab.org`, `blog.the-ebook-reader.com`) are blocked by the sandbox.
- Consequence: vendor-side facts (Amazon/Kobo/PocketBook/Vivlio help pages, LCP certification lists,
  model line-ups, limits) could not be re-verified and are tagged **[M]** (memory, model knowledge up to
  mid-2026). Treat every [M] as a hypothesis to re-check before it becomes a product promise.
- Markers: **[V2]** two independent fetched sources · **[V1]** one fetched source · **[M]** memory only.

## 1. Key findings

1. **No mainstream e-ink reader can scan a QR code** (no camera on Kindle, Kobo, PocketBook, Vivlio,
   tolino, reMarkable) [M]; exceptions are Android e-ink devices with a camera (Onyx Boox Palma family,
   Tab Ultra family) [M]. The QR code is therefore a *phone* entry point; the e-reader needs a
   *typed* path (short URL + short code) or an account/cloud path.
2. **The e-reader browser can download a book file on Kindle, Kobo and Kobo-based tolino** — this is the
   whole basis of send2ereader, whose source confirms per-device handling for `Kindle`, `Kobo`,
   `tolino`/`eReader` user agents [V1: [index.js](https://raw.githubusercontent.com/daniel-j/send2ereader/master/index.js)],
   and Standard Ebooks documents the Kindle path ("Web Browser" → tap the azw3 → "OK")
   [V1: [SE help](https://raw.githubusercontent.com/standardebooks/web/master/www/help/how-to-use-our-ebooks.php)] → [V2].
3. **Format must be negotiated per device**: Kindle does not open EPUB natively (only via Amazon's
   Send-to-Kindle conversion) [V2]; Kobo renders plain `.epub` with its older ADE engine and
   `.kepub.epub` with its WebKit engine [V2]; everything else takes EPUB.
4. **User-agent sniffing is brittle and broke twice in 2024–2026**: Kobo-based tolino (2024) and newer
   Kobo firmware report `eReader` instead of `Kobo`; a 2024 Kindle on firmware 5.18.6 is no longer
   recognised as `Kindle` [V1 each, GitHub issues]. Always give an explicit "I am on my e-reader" URL.
5. **Direct file delivery only works for DRM-free / watermarked files.** DRM'd titles (Adobe, LCP, Kobo,
   Kindle) must go through an account or licence step; the most immediate UX in reseller mode is
   "redeem on phone → book appears in the reseller library → device syncs" [M reasoning].
6. The reference community service **send.djazz.se looked abandoned/unavailable by Oct 2026**
   (last commit 2024-10-20; "502 Bad Gateway" and "should be reopened" issues) [V1] — do not depend on it;
   re-implement the pattern (MIT-licensed, ~600 lines).

## 2. Capability matrix — dedicated devices

| Family (2026 line-up) | Built-in browser | Download from browser | Native formats | DRM accepted | Wireless side-load | OPDS | Camera/QR | USB |
|---|---|---|---|---|---|---|---|---|
| **Amazon Kindle** (Kindle, Paperwhite, Colorsoft, Scribe) | Yes ("Web Browser" in ⋮ menu) [V1]; engine reportedly replaced by a Chromium-based one from fw 5.16.4 (2023) [M] | Yes, Kindle formats only (azw3/mobi/azw/prc/txt) [V2 for azw3/mobi; list M]; PDF download reported failing [V1] | KFX, AZW3, AZW, MOBI, PDF, TXT [V1 calibre driver]; **no EPUB** [V2] | Amazon only [M] | Send to Kindle: e-mail, web, apps (EPUB accepted and converted) [V2] | No (KOReader on jailbroken units only) [V1] | None [M] | MTP on 2024+ models [V1]; mass storage before |
| **Rakuten Kobo** (Clara BW/Colour, Libra Colour, Sage, Elipsa 2E) | Yes (Nickel browser, QtWebKit fork) [V1]; beta feature [M] | Yes: EPUB/KEPUB/PDF… added to library [V2] | EPUB 2/3, KEPUB, PDF, TXT, CBZ/CBR [V1] (+MOBI, RTF, HTML [M]) | Kobo DRM [V1]; Adobe ADE via PC [M]; **no LCP** [M] | Dropbox (fw 4.18+), Google Drive (fw 4.36+) on selected models [V1 for firmware; models M]; Instapaper (4.43+), OverDrive [V1] | No natively [M]; KOReader [V1] | None [M] | Mass storage [V2] |
| **tolino** shine 5 / shine color / vision color (2024+) | Yes; UA contains `eReader` [V1] | Yes (confirmed by a user, 2024-08) [V1] | EPUB, PDF, TXT [V1 for older driver] | Adobe; LCP/tolino DRM [M, low confidence] | tolino cloud: upload via the bookseller's web reader [V1] | No [M] | None [M] | Mass storage (detected as Kobo) [V1] |
| **PocketBook** (Verse, Era, InkPad) | Yes, WebKit-based [M] | Yes [M] | EPUB, PDF, FB2, MOBI (no DRM), DJVU, RTF, CHM, DOC(X), TXT [V1 older drivers] | Adobe ADE on device, LCP [M] | Send-to-PocketBook e-mail (`@pbsync.com`) [V1]; PocketBook Cloud, Dropbox [M] | Unclear on e-ink firmware [M]; KOReader without jailbreak [V1] | None [M] | Mass storage [M] |
| **Vivlio** (France; PocketBook hardware) | As PocketBook [M] | As PocketBook [M] | As PocketBook [M] | Adobe, LCP [M]; Vivlio staff co-authored an LCP spec note [V1] | Vivlio/TEA account sync; PocketBook channels [M] | [M unknown] | None [M] | Mass storage [M] |
| **Onyx Boox** / Android e-ink | Chromium-class Android browser [M] | Any file [M] | NeoReader: EPUB, PDF, MOBI/AZW3 (no DRM) [M]; any Android app | Whatever the installed app supports (Kindle, Kobo, PocketBook Reader, Vivlio…) [M] | Send2Boox cloud push (account + e-mailed 6-digit code) [V1]; local Wi-Fi transfer [M] | Via apps (KOReader, Librera, Moon+…) [V1] | **Yes on Palma / Tab Ultra families** [M] | MTP [V1 generic Android] |
| **reMarkable** (2, Paper Pro, Paper Pro Move) | **None** [M] | n/a | PDF, EPUB (no DRM) [M] | None [M] | reMarkable cloud apps, browser extension, USB web UI; many third-party uploaders [V1] | No (KOReader possible) [V1] | None [M] | USB web interface [V1] |
| **Bookeen** (Diva, Notea — legacy) | Basic [M] | [M unknown] | EPUB, PDF, HTML, FB2, TXT [V1] | Adobe; LCP on Diva/Notea [M] | Store sync only [M] | No [M] | None [M] | Mass storage [M] |

Sources for the [V1]/[V2] cells are detailed per family below.

## 3. Per-family evidence

### 3.1 Kindle

- **No native EPUB**: "Kindle devices still do not natively support epub files… Send to Kindle… converts the
  epub file to an Amazon format… It's still not possible to use a USB cable to transfer an epub"
  [V2: [Standard Ebooks](https://raw.githubusercontent.com/standardebooks/web/master/www/help/how-to-use-our-ebooks.php),
  [calibre FAQ](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/faq.rst)]. A third tool states
  "Amazon converts that epub into azw3" [V1: [kindle-send](https://raw.githubusercontent.com/nikhil1raghav/kindle-send/main/README.md)].
- **Send to Kindle e-mail**: the sender address must be on the account's allowed list, AZW3/KF8 are *not*
  accepted by e-mail, and "Amazon recently started sending confirmation emails that you have to click on…
  before the book is actually delivered" [V1: calibre FAQ; allowed-list also in
  [calibre wizard](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/gui2/wizard/send_email.py)].
  Standard Ebooks confirms AZW3 is refused by Send to Kindle [V2]. MOBI by e-mail was announced as
  dropped in 2022; calibre warns "Amazon's EPUB intake is very flawed, they will reject a number of EPUB
  files that work everywhere else" [V1: calibre FAQ] — Calibre-Web-Automated ships an "EPUB Fixer" for
  exactly this [V1: [CWA README](https://raw.githubusercontent.com/crocodilestick/Calibre-Web-Automated/main/README.md)].
  Limits (50 MB by e-mail, 200 MB by web/app) and the list of channels (web uploader, desktop apps,
  phone share sheet) are [M].
- **Browser download**: tap an `.azw3` link → "OK"; the file is readable but "won't add the ebook to your
  Kindle library" in the same way and gets no cover [V1: Standard Ebooks]. send2ereader converts EPUB→MOBI
  with KindleGen for Kindle, sets `Content-Disposition: attachment`, and strips every filename character
  outside `[.\w\-"'()]` "a limitation of the Kindle browser" [V1: index.js, upload.html]. calibre removes
  `Content-Disposition` for `Kindle/3` user agents because non-ASCII filenames break the download
  [V1: [legacy.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/srv/legacy.py)].
  KindleGen itself is discontinued (README links an archive copy) [V1].
- **2024+ hardware**: uses MTP instead of USB mass storage; Colorsoft and newer hide covers of side-loaded
  books unless marked as personal documents [V1: [calibre changelog](https://raw.githubusercontent.com/kovidgoyal/calibre/master/Changelog.txt), FAQ].
- **UA regression**: Kindle 11th gen (2024 release), firmware 5.18.6, opens send.djazz.se as a desktop page;
  workaround is the explicit `/receive` URL [V1: [#63](https://github.com/daniel-j/send2ereader/issues/63),
  [#70](https://github.com/daniel-j/send2ereader/issues/70)].
- Old Kindles (1/2/DX) cannot reach non-standard ports such as 8080 [V1: calibre FAQ].
- Code redemption on device: none; Amazon codes are redeemed on the account website [M].

### 3.2 Kobo

- Kobo's own guidelines: EPUB 2.0.1–3.3 supported; side-loaded `.epub` uses the "Adobe Digital Editions"
  engine, renaming to `.kepub.epub` triggers "the Kobo WebKit"; store files carry Kobo DRM (per-book,
  per-user, per-device keys); JavaScript inside books is not supported on eInk
  [V1: [kobolabs/epub-spec](https://raw.githubusercontent.com/kobolabs/epub-spec/master/README.md)];
  Standard Ebooks and [kepubify](https://raw.githubusercontent.com/pgaskin/kepubify/master/README.md) confirm the two renderers [V2].
- Browser: Kobo publishes `qtwebkit`/`qtbase` forks [V1: [kobolabs org](https://github.com/kobolabs)];
  NickelMenu documents `nickel_browser` (open URL, optional CSS injection) [V1:
  [NickelMenu doc](https://raw.githubusercontent.com/pgaskin/NickelMenu/master/res/doc)]. The send2ereader
  receiver page runs there with ES5 JavaScript + `XMLHttpRequest` [V1]. "Kobo readers [have] problems using
  the internal web browser on non-standard ports" [V1: [Calibre-Web FAQ](https://raw.githubusercontent.com/wiki/janeczku/calibre-web/FAQ.md)].
- Download quirks: calibre appends `.epub` to KEPUB filenames and replaces `!` when the UA contains
  `Kobo Touch`, and keeps a dedicated URL shape ending in the filename "needed for Kobo browsers"
  [V1: [content.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/srv/content.py), legacy.py].
- UA change: on a Libra Colour (reported 2026-08-29) the UA no longer contains `Kobo` but a generic
  `eReader`, so send2ereader stopped kepubifying [V1: [#76](https://github.com/daniel-j/send2ereader/issues/76)].
- Cloud: Dropbox from firmware 4.18 ("the Forma only at this point"), Google Drive from 4.36, Pocket removed
  after 4.38/4.42, Instapaper from 4.43, OverDrive from 4.10 [V1: NickelMenu doc;
  [calibre Kobo driver](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/kobo/driver.py)].
  Which current models expose Dropbox/Drive (believed: Sage, Elipsa 2E, Libra Colour; *not* Clara BW/Colour)
  and "no OneDrive" are [M]. KoboCloud adds Dropbox/Drive/Nextcloud sync to any Kobo by script
  [V1: [KoboCloud](https://raw.githubusercontent.com/fsantini/KoboCloud/master/README.md)].
- Sync hijack: the device's `api_endpoint` can be pointed at Calibre-Web/Komga, which then emulate the
  Kobo store sync (PDF not synced) [V1: [Calibre-Web wiki](https://raw.githubusercontent.com/wiki/janeczku/calibre-web/Kobo-Integration.md),
  [Komga](https://raw.githubusercontent.com/gotson/komga/master/README.md)] — proof that "book appears by
  sync" is the smoothest channel, but only the store operator (Kobo/Fnac) can offer it to normal users.
- calibre's driver (Nov 2026) lists Clara BW, Clara Colour, Libra Colour, Sage, Elipsa 2E, Libra 2,
  Clara 2E and supports firmware up to 6.0 [V1].

### 3.3 tolino

- "New 2024 Tolino devices… use a modified version of the Kobo firmware, so they are detected as Kobo
  devices" (calibre 7.16, 2024-07-31) [V1]; a user describes the Shine Color as Kobo-based, firmware
  5.0.175773, and send2ereader added `eReader` UA detection for it on 2024-07-22, confirmed working
  2024-08-30 [V1: [#28](https://github.com/daniel-j/send2ereader/issues/28)] → [V2].
- Older Android-based tolino (Vision 4 HD): browser download completes but the file is reported
  "damaged" [V1: [#57](https://github.com/daniel-j/send2ereader/issues/57)].
- tolino cloud lets users "upload / sync own files to the user's device(s)" through the bookseller's web
  reader (Thalia, Hugendubel, buecher.de…) [V1: [tolino-python](https://raw.githubusercontent.com/hzulla/tolino-python/master/README.md),
  [tolino-calibre-sync](https://raw.githubusercontent.com/darkphoenix/tolino-calibre-sync/master/README.md)].
- tolino is a DACH/Benelux brand, marginal in France [M].

### 3.4 PocketBook and Vivlio

- calibre special-cases `@pbsync.com` (Send-to-PocketBook) next to `@kindle.com`: random ASCII filename and
  subject because the service "can't handle non-ascii, filenames that are long…, commas"
  [V1: [email.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/gui2/email.py)].
  Sender confirmation on first use is [M].
- PocketBook OS is Linux-based; KOReader installs by copying folders over USB, no jailbreak
  [V1: [KOReader wiki](https://github.com/koreader/koreader/wiki/Installation-on-PocketBook-devices)].
- Wide format list in calibre's (older) PocketBook drivers [V1: [eb600 driver](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/eb600/driver.py)].
- Vivlio = PocketBook hardware + Vivlio firmware/store operated by TEA, sold through French booksellers
  (Cultura, Decitre, Furet du Nord…) [M]. Vivlio is named as contributor of the Readium "LCP Automatic Key
  Retrieval" note [V1: [lcp-key-retrieval](https://raw.githubusercontent.com/readium/lcp-specs/master/notes/lcp-key-retrieval.md)].
- Browser engine, download MIME handling, on-device `.acsm` fulfilment, PocketBook Cloud and Dropbox
  details: all [M] — the highest-priority items to test on a real Vivlio, since Vivlio + Kobo/Fnac + Kindle
  are the three French e-ink ecosystems.

### 3.5 Onyx Boox, reMarkable, Bookeen

- Boox: Send2Boox is an account-based cloud push; login uses a 6-digit code sent by e-mail
  [V1: [onyx-send2boox](https://raw.githubusercontent.com/hrw/onyx-send2boox/main/README.md)]. Being Android,
  a Boox behaves like a phone for redemption (modern browser, any reading app, some models with camera) [M].
- reMarkable: no browser; the ecosystem relies on cloud uploaders, a USB web interface and e-mail-to-device
  bridges [V1: [awesome-reMarkable](https://raw.githubusercontent.com/reHackable/awesome-reMarkable/master/README.md)].
  Redemption must be relayed by phone/PC.
- Bookeen Diva HD: EPUB, HTML, FB2, TXT, PDF [V1: [cybook driver](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/cybook/driver.py)].
  Company status and LCP support [M].

## 4. Reading apps (phone / tablet / desktop)

| App | Gets a DRM-free EPUB by… | DRM | OPDS | Notes |
|---|---|---|---|---|
| **Apple Books** | Safari download → "Open in Books" [V2: calibre FAQ, Standard Ebooks] | Apple only [M] | No [M] | Lowest-friction path on iPhone/iPad |
| **Google Play Books** | "Upload files" on web or open-with on Android [M] | Google/Adobe export [M] | No [M] | Upload syncs across devices [M] |
| **Kindle app** | Share sheet → Send to Kindle (conversion) [M]; calibre still targets MOBI/AZW3 for Kindle apps [V1] | Amazon [M] | No | Same account as the e-ink Kindle → lands on the device too [M] |
| **Kobo app** | iOS: Share → "open in" Kobo Books; Android: local file [V1: Kobo epub-spec] | Kobo [V1] | No [M] | Side-loaded files are local to the app [M] |
| **Thorium Reader** (EDRLab) | Import file or OPDS feed; opens `.epub` and `.lcpl` [V1: [README](https://raw.githubusercontent.com/edrlab/thorium-reader/develop/README.md)] | **LCP** (production builds) [V1]; no Adobe [M] | Yes [V1] | Windows/macOS/Linux only; no mobile repository visible in EDRLab's GitHub org, only `thorium-web` [V1: [org search](https://github.com/orgs/edrlab/repositories?q=thorium)] |
| **Vivlio app** | Vivlio/TEA account sync, file import [M] | LCP, Adobe [M] | [M unknown] | French reseller ecosystem |
| **Aldiko Next / Cantook** (De Marque) | File import, catalogues [M] | Adobe + LCP [M]; De Marque co-authored LCP notes [V1] | Yes [M] | |
| **PocketBook Reader app** | File import, cloud [M] | Adobe + LCP [M] | Yes [M] | |
| **KOReader** | OPDS, calibre wireless, Dropbox/FTP/WebDAV, SSH [V1: [README](https://raw.githubusercontent.com/koreader/koreader/master/README.md), [plugins](https://github.com/koreader/koreader/tree/master/plugins), [providers](https://github.com/koreader/koreader/tree/master/plugins/cloudstorage.koplugin/providers)] | None [M] | Yes: HTTP Basic only, no Digest; download lists; per-catalogue sync [V1: [wiki](https://github.com/koreader/koreader/wiki/OPDS-support)] | Runs on Kindle (jailbreak required [V1: [wiki](https://github.com/koreader/koreader/wiki/Installation-on-Kindle-devices)]), Kobo, PocketBook, reMarkable, Android; AGPL-3 — do not copy code into this BSD-3 project |

## 5. DRM in one paragraph (all [M] unless marked)

Adobe ADE (`.acsm` → Adobe ID): Kobo (via PC), PocketBook/Vivlio, tolino, Bookeen, Aldiko, PocketBook app;
not Kindle, not Apple Books, not Thorium. Readium LCP: passphrase-based, "Browser based applications cannot
integrate LCP", formats EPUB 2/3, PDF, audiobooks [V1: [lcp-specs](https://raw.githubusercontent.com/readium/lcp-specs/master/README.md)];
readers include Thorium [V1], PocketBook/Vivlio/tolino/Bookeen devices and Aldiko/Cantook/Vivlio/PocketBook
apps [M]; not Kindle, not Kobo. Kindle and Kobo DRM are closed, account-bound. CARE (TEA's DRM, once used by
Vivlio's ecosystem and the German Onleihe) appears to have been superseded by LCP [M, low confidence].
Design consequence: a card can carry (a) a file-delivery code for DRM-free/watermarked titles, (b) an
account-credit code for closed ecosystems, or (c) for LCP, a licence link plus the **passphrase printed on
the card** — LCP explicitly works by typing a passphrase with a hint [V1: lcp-key-retrieval note].

## 6. Community "no cable" solutions

### 6.1 send2ereader (send.djazz.se) — read from source

| Aspect | Fact (all [V1], [index.js](https://raw.githubusercontent.com/daniel-j/send2ereader/master/index.js), [download.html](https://raw.githubusercontent.com/daniel-j/send2ereader/master/static/download.html), [upload.html](https://raw.githubusercontent.com/daniel-j/send2ereader/master/static/upload.html), [common.js](https://raw.githubusercontent.com/daniel-j/send2ereader/master/static/common.js)) |
|---|---|
| Pairing direction | The **e-reader shows** the key; the phone/PC types it and uploads. Nothing is typed on e-ink except the site URL. |
| Key | 4 characters from `23456789ACDEFGHJKLMNPRSTUVWXYZ` (30 symbols: no 0/1/B/I/O/Q) = 810 000 values; input upper-cased, `maxlength=4`. |
| Lifetime | Key dies 30 s after the e-reader stops polling; hard cap 1 h; file deleted with the key. |
| Binding | Download refused unless the requesting User-Agent equals the UA that generated the key. |
| E-reader page | ~3.8 kB HTML, ES5 only (`var`, `XMLHttpRequest`, no fetch/Promise), polls `/status/KEY` every 5 s, then shows a plain `<a href="/<filename>?key=KEY">`. JavaScript **is** required. |
| Device routing | UA contains `Kobo`, `Kindle`, `tolino` or `eReader` → receiver page; everything else → upload form; `/receive` forces the receiver page. |
| Conversion | Kobo: EPUB → `.kepub.epub` (kepubify). Kindle: EPUB → MOBI (KindleGen). Optional PDF margin crop, filename transliteration. |
| Accepted | EPUB, MOBI, PDF, CBZ, CBR, HTML, TXT; 800 MB max; also "send URL". |
| Licence / status | MIT ([LICENSE](https://raw.githubusercontent.com/daniel-j/send2ereader/master/LICENSE) fetched); last commit 2024-10-20 ([commits](https://github.com/daniel-j/send2ereader/commits/master)); hosted site reported 502 and "should be reopened" on 2026-10-29 ([#80](https://github.com/daniel-j/send2ereader/issues/80)). |

Derivatives named in the tracker: `epub2kobo` (MIT fork, ~3 MB image, same 4-character key and 30 s/1 h
expiry) [V1: [repo](https://github.com/ahxxm/epub2kobo)]; BookDrop (bookdrop.cc), leaf.casa and
send2ereader.net are mentioned by users as live replacements — not visited [V1: #70, #80]. An open pull
request makes the **e-reader page display a QR code** so the phone gets the key by scanning instead of typing
[V1: [#27](https://github.com/daniel-j/send2ereader/issues/27), [PR #59](https://github.com/daniel-j/send2ereader/pull/59)];
KOReader likewise renders QR codes on e-ink to hand text to a phone
[V1: [qrclipboard](https://raw.githubusercontent.com/koreader/koreader/master/plugins/qrclipboard.koplugin/main.lua)].

### 6.2 calibre content server and Calibre-Web

- calibre: "connect wirelessly to any device that has a web browser"; add `/mobile` for "a simplified,
  non-JavaScript interface"; "some e-ink devices have browsers that do not handle authentication"; OPDS at
  `/opds`; Basic vs Digest auth switch [V1: [server.rst](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/server.rst), FAQ].
  calibre is GPL — pattern only, no code reuse [M].
- Calibre-Web: OPDS feed, one-click "send to e-reader" by e-mail, Kobo sync, and a **"magic link" remote
  login** "made for devices where it is hard to enter a lot of text, like on ebook readers": the reader
  shows a link/token, the user confirms it on an already logged-in computer and the reader is logged in a
  second later [V1: [README](https://raw.githubusercontent.com/janeczku/calibre-web/master/README.md),
  [Configuration](https://raw.githubusercontent.com/wiki/janeczku/calibre-web/Configuration.md)].
- Kobo UNCaGED: calibre's wireless-device protocol inside Nickel (needs NickelMenu)
  [V1: [README](https://raw.githubusercontent.com/shermp/Kobo-UNCaGED/master/README.md)]; KOReader speaks the
  same protocol [V1: [wiki](https://raw.githubusercontent.com/wiki/koreader/koreader/calibre.md)]. Enthusiast-only.

## 7. UX lessons for the redemption flow

1. **Two entry points on the card, one code.** QR for the phone (the only camera in the house); a short
   URL + the same short code in clear text for an e-reader or computer.
2. **Prefer "pair, don't type".** The e-reader should type at most a short hostname; let it *display* a
   pairing key (or QR) that the phone consumes after scanning the card. This is the send2ereader and
   Calibre-Web magic-link pattern, and matches the OAuth device-authorisation idea [M].
3. **Code alphabet**: upper-case, ≈30 symbols without 0/1/O/I/B/Q, case-insensitive input [V1 pattern].
   Four characters are only acceptable for a 30-second pairing key; a gift code is long-lived and needs
   ≥10 symbols (≈49 bits) plus rate limiting [reasoning]. Group as `XXXX-XXXX-XX`.
4. **Tiny, static-first receiver page**: a few kB, ES5 only, no build-time polyfills, no web fonts, no
   animation, works without JavaScript via a plain form POST + meta refresh (calibre `/mobile` proves the
   no-JS option; send2ereader proves ES5 + XHR works) [V1 both]. The WASM/PDF tooling must never load here.
5. **Never trust the User-Agent alone**: expose `/r` (receiver) explicitly and a manual "Kindle / Kobo /
   other" choice; log unknown UAs [V1: three regressions in two years].
6. **Standard ports only (443/80), no HTTP-auth dialogs, credentials in the path or a form** [V1].
7. **File naming matters**: URL path must end with the real filename; ASCII-only names for Kindle;
   `.kepub.epub` for Kobo; avoid `!`; send `Content-Disposition` conservatively [V1].
8. **Per-device format**: Kobo/tolino `.kepub.epub` (or `.epub`), Kindle AZW3/MOBI by browser *or* EPUB via
   Send to Kindle, others `.epub` [V2]. With KindleGen discontinued and calibre GPL, the pragmatic Kindle
   path is "download EPUB on phone → Send to Kindle" [reasoning].
9. **Do not make the service e-mail the recipient's Kindle**: it needs their `@kindle.com` address, an
   allow-list change and possibly a confirmation click [V1] — three steps too many for a gift.
10. **Reseller mode beats everything**: redeem on the phone into the recipient's reseller account; the book
    then reaches Kobo/Fnac, Vivlio or Kindle hardware by ordinary sync, with DRM untouched [reasoning].

### Suggested fake e-reader profiles for `demo/` and e2e tests

| Profile | Constraints to emulate | Basis |
|---|---|---|
| `kobo-nickel` | Old WebKit, ES5, UA with `Kobo Touch` **or** only `eReader`; ports 80/443; wants `.kepub.epub` | V1 facts above; engine age M |
| `kindle` | Downloads only azw3/mobi/txt; ASCII filenames; UA may lack `Kindle`; no library cover | V1/V2; type list M |
| `tolino-android-legacy` | Download succeeds, file rejected as damaged → must show fallback instructions | V1 (#57) |
| `nojs` | JavaScript off, forms only | V1 (calibre `/mobile`) |
| `android-eink` | Modern Chromium, camera optional | M |
| `no-browser` (reMarkable) | Phone relay only | M |

## Sources (all fetched 2026-11-12)

- send2ereader: [README](https://github.com/daniel-j/send2ereader), [index.js](https://raw.githubusercontent.com/daniel-j/send2ereader/master/index.js), [download.html](https://raw.githubusercontent.com/daniel-j/send2ereader/master/static/download.html), [upload.html](https://raw.githubusercontent.com/daniel-j/send2ereader/master/static/upload.html), [common.js](https://raw.githubusercontent.com/daniel-j/send2ereader/master/static/common.js), [LICENSE](https://raw.githubusercontent.com/daniel-j/send2ereader/master/LICENSE), [commits](https://github.com/daniel-j/send2ereader/commits/master), [forks](https://github.com/daniel-j/send2ereader/forks), issues [#27](https://github.com/daniel-j/send2ereader/issues/27) [#28](https://github.com/daniel-j/send2ereader/issues/28) [#32](https://github.com/daniel-j/send2ereader/issues/32) [#36](https://github.com/daniel-j/send2ereader/issues/36) [#50](https://github.com/daniel-j/send2ereader/issues/50) [#51](https://github.com/daniel-j/send2ereader/issues/51) [#57](https://github.com/daniel-j/send2ereader/issues/57) [#58](https://github.com/daniel-j/send2ereader/issues/58) [#63](https://github.com/daniel-j/send2ereader/issues/63) [#70](https://github.com/daniel-j/send2ereader/issues/70) [#76](https://github.com/daniel-j/send2ereader/issues/76) [#80](https://github.com/daniel-j/send2ereader/issues/80), [PR #59](https://github.com/daniel-j/send2ereader/pull/59); fork [epub2kobo](https://github.com/ahxxm/epub2kobo)
- calibre: [FAQ](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/faq.rst), [server manual](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/server.rst), [Changelog](https://raw.githubusercontent.com/kovidgoyal/calibre/master/Changelog.txt), drivers [kindle](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/kindle/driver.py) [kobo](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/kobo/driver.py) [eb600/tolino/pocketbook](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/eb600/driver.py) [cybook](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/cybook/driver.py) [mtp defaults](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/devices/mtp/defaults.py), [email.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/gui2/email.py), [send_email wizard](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/gui2/wizard/send_email.py), [srv/legacy.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/srv/legacy.py), [srv/content.py](https://raw.githubusercontent.com/kovidgoyal/calibre/master/src/calibre/srv/content.py)
- Kobo: [epub-spec](https://raw.githubusercontent.com/kobolabs/epub-spec/master/README.md), [kobolabs org](https://github.com/kobolabs), [Kobo-Reader README](https://raw.githubusercontent.com/kobolabs/Kobo-Reader/master/README), [NickelMenu doc](https://raw.githubusercontent.com/pgaskin/NickelMenu/master/res/doc), [kepubify](https://raw.githubusercontent.com/pgaskin/kepubify/master/README.md), [KoboCloud](https://raw.githubusercontent.com/fsantini/KoboCloud/master/README.md), [Kobo UNCaGED](https://raw.githubusercontent.com/shermp/Kobo-UNCaGED/master/README.md)
- Calibre-Web: [README](https://raw.githubusercontent.com/janeczku/calibre-web/master/README.md), [Kobo integration](https://raw.githubusercontent.com/wiki/janeczku/calibre-web/Kobo-Integration.md), [Configuration](https://raw.githubusercontent.com/wiki/janeczku/calibre-web/Configuration.md), [FAQ](https://raw.githubusercontent.com/wiki/janeczku/calibre-web/FAQ.md); [Calibre-Web-Automated](https://raw.githubusercontent.com/crocodilestick/Calibre-Web-Automated/main/README.md); [Komga](https://raw.githubusercontent.com/gotson/komga/master/README.md)
- KOReader: [README](https://raw.githubusercontent.com/koreader/koreader/master/README.md), [OPDS wiki](https://github.com/koreader/koreader/wiki/OPDS-support), [plugins](https://github.com/koreader/koreader/tree/master/plugins), [cloud providers](https://github.com/koreader/koreader/tree/master/plugins/cloudstorage.koplugin/providers), [qrclipboard](https://raw.githubusercontent.com/koreader/koreader/master/plugins/qrclipboard.koplugin/main.lua), [opds main.lua](https://raw.githubusercontent.com/koreader/koreader/master/plugins/opds.koplugin/main.lua), [calibre wiki](https://raw.githubusercontent.com/wiki/koreader/koreader/calibre.md), [Kindle install](https://github.com/koreader/koreader/wiki/Installation-on-Kindle-devices), [PocketBook install](https://github.com/koreader/koreader/wiki/Installation-on-PocketBook-devices), [Android install](https://raw.githubusercontent.com/wiki/koreader/koreader/Installation-on-Android-devices.md)
- [Standard Ebooks — how to use our ebooks](https://raw.githubusercontent.com/standardebooks/web/master/www/help/how-to-use-our-ebooks.php); [kindle-send](https://raw.githubusercontent.com/nikhil1raghav/kindle-send/main/README.md)
- Readium/EDRLab: [lcp-specs README](https://raw.githubusercontent.com/readium/lcp-specs/master/README.md), [LCP automatic key retrieval](https://raw.githubusercontent.com/readium/lcp-specs/master/notes/lcp-key-retrieval.md), [Thorium README](https://raw.githubusercontent.com/edrlab/thorium-reader/develop/README.md), [EDRLab repositories matching "thorium"](https://github.com/orgs/edrlab/repositories?q=thorium)
- tolino: [tolino-python](https://raw.githubusercontent.com/hzulla/tolino-python/master/README.md), [tolino-calibre-sync](https://raw.githubusercontent.com/darkphoenix/tolino-calibre-sync/master/README.md); Boox: [onyx-send2boox](https://raw.githubusercontent.com/hrw/onyx-send2boox/main/README.md); reMarkable: [awesome-reMarkable](https://raw.githubusercontent.com/reHackable/awesome-reMarkable/master/README.md)

## Could not verify (no search available; all need a follow-up pass or a hands-on device test)

- Browser engine versions, JavaScript level (ES5 vs newer), TLS versions / root-CA stores and maximum
  download sizes of the Kindle, Kobo, PocketBook/Vivlio and tolino browsers; whether the post-2023 Kindle
  browser is Chromium-based; the exact list of file types the Kindle browser will download.
- The exact current user-agent strings (only the substrings `Kobo Touch`, `Kindle/3`, `eReader` are sourced).
- Send to Kindle limits (50 MB e-mail / 200 MB web), full accepted-type list, and whether the web/app
  routes bypass the approved-sender step; Amazon's 2025–2026 changes (USB download removal, DRM-free
  EPUB/PDF downloads) — recalled, not checked.
- Which Kobo models ship Dropbox / Google Drive in 2026; absence of OneDrive; any Kobo model released after
  mid-2025; what Kobo firmware 5.x/6.0 changed in the browser.
- PocketBook and Vivlio: browser behaviour, on-device Adobe `.acsm` and LCP `.lcpl` handling,
  Send-to-PocketBook sender confirmation, PocketBook Cloud/Dropbox on Vivlio firmware, native OPDS.
- LCP-certified device/app list (EDRLab site blocked): tolino, Bookeen, Vivlio, PocketBook, Aldiko/Cantook
  claims are from memory; status of CARE; existence of a Thorium mobile app.
- tolino 2024+ DRM set and whether tolino cloud is still Telekom-operated (the sourced READMEs are old).
- Onyx Boox models with cameras, NeoReader OPDS, current Android versions; reMarkable formats/integrations;
  Bookeen's corporate status.
- Whether Kobo/Fnac, Vivlio or Amazon.fr let a *third-party* code be redeemed on the device itself
  (believed: no, account website only).
- Whether send.djazz.se is permanently offline, and who operates bookdrop.cc / leaf.casa / send2ereader.net.
