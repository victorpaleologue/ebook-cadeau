# What to call it: "eBook cadeau" and its equivalents per website language — attested usage

Research notes, 2026-11-12. Angle: strings that real products ship, per language, for (a) gifting at checkout, (b) the gift product, (c) redemption, (d) the words for eBook and e-reader. Companion to `terminology.naming.md` (naming, trademark, recommended strings — all from memory).

## Method and limits — read first

- **No web search could be run.** Six WebSearch calls, six refusals: the workflow's shared budget (200 calls) was already spent. No bookseller page, help page, Google Trends chart or terminology database was opened.
- The sandbox proxy refused (HTTP 403, policy; not retried) `culture.fr` (FranceTerme), `vitrinelinguistique.oqlf.gouv.qc.ca` (OQLF), `duden.de`, `fr.wikipedia.org`, `github.com` and `api.github.com`. Only `raw.githubusercontent.com` answered.
- What I did instead: fetched and parsed **about 190 locale and dictionary files** from public repositories (Shopify's reference theme, Odoo, Signal, Mullvad, calibre, KOReader, calibre-web, Foliate, Open Library, Thorium Reader, Readest, BookWyrm, LanguageTool, OpenTaal, LibreOffice dictionaries). These are strings actually shipped to users. They are **not bookseller sites**.
- Tags:
  - **[V2]** = the same form found in two or more independent files fetched this session. **[V1]** = one file fetched this session. Every [V1]/[V2] link is a file I downloaded (HTTP 200) and read; none comes from a search result.
  - **[V1-sib] / [V2-sib]** = sourced by another agent in a sibling note in this folder; URL copied from that note, not re-opened by me.
  - **[M]** = from memory, not verified in this session (high / medium / low = my recall confidence).
- **Every retailer or platform UI string in this note is [M] or -sib.** The brief asked for quoted strings from Amazon, Kobo, Apple Books, Fnac, Thalia, bol.com, Casa del Libro, IBS, Empik, Adlibris, Saxo: that could not be done first-hand. Section 7 lists what I recall, as hypotheses to re-check.
- Search-volume comparisons ("offrir ebook" vs "ebook cadeau" vs "livre numérique cadeau") were **not obtained**.

## 1. Key findings

1. **Where title-specific gifting exists, the label is the verb of giving**: Decitre « Offrir un ebook », button « Offrir »; Hugendubel/Thalia « Verschenken »; Amazon.fr « Acheter pour d'autres personnes » [V2-sib] (links in section 3). No retailer flow is known to use a noun such as "eBook cadeau" as the option label.
2. **The generic e-commerce default is a first-person checkbox.** Shopify's reference theme ships "I want to send this as a gift" in 14 of the target languages, e.g. « Je veux envoyer ce produit comme cadeau », „Ich möchte dies als Geschenk senden", "Ik wil dit als cadeau sturen" [V1] (table in section 3).
3. **"Redeem" is stable in ten languages and unstable in four.** Three independent products agree on *einlösen, canjear, riscattare, lösa in, indløse, løse inn, lunastaa*; two agree on *uplatnit, εξαργύρωση, resgatar*. French, Dutch and Polish each show **three different verbs in three products** (fr: *utiliser / récupérer / échanger*); Romanian varies inside one app [V2] (section 5).
4. **"Gift card" is stable in fr, es, nl, sv, da, nb, fi and split in de, it, pt, pl, cs** (e.g. *Geschenkgutschein* vs *Geschenkkarte*; *buono regalo* vs *carta regalo*; *karta prezentowa* vs *karta podarunkowa*) [V2] (section 4).
5. **French: the device word is settled, the book word is not.** « liseuse » is used by all five reading-software corpora [V2]. For the book, the same corpora use « livre numérique », « livre électronique », « e-book » and « eBook » [V2]; inside calibre's French file « livre(s) numérique(s) » occurs on 169 lines against 3 for « livre(s) électronique(s) » [V1]. The official and statutory term is « livre numérique » [M high; V2-sib for the statute title].
6. **German: both spellings ship.** calibre writes „eBook"; KOReader, calibre-web and Open Library write „E-Book" [V2]; LanguageTool's rule `E_BOOK` corrects „EBook" to „E-Book" [V1]; German booksellers write „eBook" [M, high].
7. **Dutch: hyphenated lower case is the norm** — the OpenTaal list has `e-book`, `e-boek`, `e-reader` and no `ebook`/`ereader`; LanguageTool corrects "ereader" and "e-Reader" to "e-reader" [V2]. bol.com's "ebook" spelling [M] is a trade deviation.
8. **The device noun diverges from the dictionary in Scandinavia**: OSS corpora write *läsenhet / e-boksläsare / e-läsare* (sv), *e-boglæser* (da), *e-bokleser* (nb) [V1 each]; retailers say *läsplatta*, *e-bogslæser*, *lesebrett* [M]; the Norwegian spelling dictionary does list `lesebrett` [V1].
9. **"Passphrase" has no stable translation** (Thorium Reader: *Mot de passe, Passwort, Santo y seña, Wachtwoordzin, Frase-passe, Senha, Nøgleord…*) [V1]. "Code" is uniform across all files. The card's typed fallback should be called a code.
10. **"QR code" word order is a locale marker**: Shopify's single French file says « code QR » and « carte-cadeau » (hyphen) [V1], the Canadian/OQLF forms [M]; French retailers write « QR code » and « carte cadeau » [M, high].

## 2. The verified corpus

All files fetched 2026-11-12 from default branches. Full list in Sources.

| Corpus | What it is | Languages fetched | Used for |
|---|---|---|---|
| [Shopify Dawn `locales/`](https://raw.githubusercontent.com/Shopify/dawn/main/locales/en.default.json) | Shopify's reference storefront theme; default strings of the gift-card page and the "send as a gift" recipient form | fr, de, es, it, nl, pt-PT, pt-BR, pl, sv, da, nb, fi, cs, el (no `ro-RO`, no `ca`: 404) | (a), (b), (c), QR wording |
| [Odoo 17.0 `loyalty`](https://raw.githubusercontent.com/odoo/odoo/17.0/addons/loyalty/i18n/fr.po), [`website_sale_loyalty`](https://raw.githubusercontent.com/odoo/odoo/17.0/addons/website_sale_loyalty/i18n/fr.po) | ERP / web-shop gift-card module | same + es_419, ro, ca (el, nb mostly untranslated) | (b) |
| [Signal-Android `strings.xml`](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values/strings.xml) | Messaging app; "Redeem" button on a received gift | fr, de, es, it, nl, pt, pt-BR, pl, sv, da, nb, fi, cs, ro, el, ca | (c) |
| [Mullvad VPN app `messages.po`](https://raw.githubusercontent.com/mullvad/mullvadvpn-app/main/desktop/packages/mullvad-vpn/locales/fr/messages.po) | "Redeem voucher", "Enter voucher code", error messages | fr, de, es, it, nl, pt, pl, sv, da, nb, fi | (c), code entry |
| [calibre](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/fr.po), [KOReader](https://raw.githubusercontent.com/koreader/koreader-translations/master/fr/koreader.po), [calibre-web](https://raw.githubusercontent.com/janeczku/calibre-web/master/cps/translations/fr/LC_MESSAGES/messages.po), [Foliate](https://raw.githubusercontent.com/johnfactotum/foliate/gtk4/po/fr.po), [Open Library](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/i18n/fr/messages.po) | Reading software and a book catalogue; community translations | 8–19 languages each | (d) |
| [Thorium Reader](https://raw.githubusercontent.com/edrlab/thorium-reader/develop/src/resources/locales/fr.json) | EDRLab's LCP reading app | 14 languages | passphrase wording |
| [LanguageTool de](https://raw.githubusercontent.com/languagetool-org/languagetool/master/languagetool-language-modules/de/src/main/resources/org/languagetool/rules/de/grammar.xml), [nl](https://raw.githubusercontent.com/languagetool-org/languagetool/master/languagetool-language-modules/nl/src/main/resources/org/languagetool/rules/nl/grammar.xml); [OpenTaal word list](https://raw.githubusercontent.com/OpenTaal/opentaal-wordlist/master/wordlist.txt); LibreOffice spelling dictionaries | Normative tooling | de, nl, da, nb, cs, it, ca, en-GB | spelling norms |

Limits of this corpus: translators of free software are not booksellers; community files contain errors (one Shopify Finnish label repeats the wrong string; calibre-web's Polish "Send to eReader" is a past tense). Agreement across independent files is the signal; a single file is weak evidence.

## 3. (a) Gifting a specific item at checkout

### 3.1 Bookseller and platform labels (not verified first-hand)

| Site | Label | Tag |
|---|---|---|
| Decitre / Furet du Nord (Vivlio network), FR | « Offrir un ebook … en 2 clics »; button « Offrir »; message and send date | [V2-sib] ([Decitre](https://www.decitre.fr/offrir-un-ebook), [Decitre FAQ](https://www.decitre.fr/faq/numerique/offrir-un-ebook), [Liseuses.net](https://www.liseuses.net/offrir-des-ebooks-avec-tea/)) |
| Amazon.fr (Kindle) | « Acheter pour d'autres personnes » | [V2-sib] ([Amazon.fr help](https://www.amazon.fr/gp/help/customer/display.html?nodeId=GVWGP284MQ6ZRM59), [IDBoox](https://www.idboox.com/news-livres/amazon-on-peut-enfin-offrir-un-livre-numerique-kindle/)) |
| Hugendubel / Thalia (tolino), DE | „Verschenken", „Artikel verschenken", „eBooks verschenken" | [V2-sib] ([Hugendubel](https://www.hugendubel.de/de/shortcut/ebooksverschenken), [heise](https://www.heise.de/tipps-tricks/E-Books-verschenken-so-klappt-s-7476189.html)) |
| Kobo by Fnac, FR | No "offrir" function found | [V1-sib] ([Dealabs](https://www.dealabs.com/discussions/carte-cadeau-kobo-ou-fnac-2273026)) |
| Amazon.com / .de / .es / .it | "Buy for others" / „Für andere kaufen" / "Comprar para otros" / "Acquista per altri" | [M] high / medium-high / medium / medium |
| Apple Books; Google Play Books; B&N | "Gift This Book"; "Buy as gift"; "Buy as Gift" | [M, medium] (exact casing not certain) |
| Polish eBook shops (Ebookpoint, Publio, Woblink) | „Kup na prezent" | [M, medium] |
| Czech shops (Alza, Palmknihy) | „Koupit jako dárek" | [M, medium] |
| 7switch, Cultura, bol.com, Casa del Libro, IBS, Empik, Adlibris, Saxo | Not known | — |

Amazon avoids the word "gift" in the button ("for others"); every other attested label uses the gift verb.

### 3.2 Generic checkout wording — Shopify reference theme [V1 per row]

Source key: `recipient.form.checkbox`, English "I want to send this as a gift". The language code links to the file.

| Lang | Checkbox label | "Recipient" word in the form |
|---|---|---|
| [fr](https://raw.githubusercontent.com/Shopify/dawn/main/locales/fr.json) | Je veux envoyer ce produit comme cadeau | destinataire |
| [de](https://raw.githubusercontent.com/Shopify/dawn/main/locales/de.json) | Ich möchte dies als Geschenk senden | Empfänger |
| [es](https://raw.githubusercontent.com/Shopify/dawn/main/locales/es.json) | Quiero enviar esto como un regalo. | persona destinataria |
| [it](https://raw.githubusercontent.com/Shopify/dawn/main/locales/it.json) | Voglio inviarlo come regalo | destinatario |
| [nl](https://raw.githubusercontent.com/Shopify/dawn/main/locales/nl.json) | Ik wil dit als cadeau sturen | ontvanger |
| [pt-PT](https://raw.githubusercontent.com/Shopify/dawn/main/locales/pt-PT.json) | Quero enviar isto como um presente | destinatário |
| [pt-BR](https://raw.githubusercontent.com/Shopify/dawn/main/locales/pt-BR.json) | Quero enviar como presente | destinatário |
| [pl](https://raw.githubusercontent.com/Shopify/dawn/main/locales/pl.json) | Chcę to wysłać jako prezent | odbiorca |
| [sv](https://raw.githubusercontent.com/Shopify/dawn/main/locales/sv.json) | Jag vill skicka detta som en gåva | mottagare |
| [da](https://raw.githubusercontent.com/Shopify/dawn/main/locales/da.json) | Jeg vil sende som en gave | modtager |
| [nb](https://raw.githubusercontent.com/Shopify/dawn/main/locales/nb.json) | Jeg vil sende dette som en gave | mottaker |
| [fi](https://raw.githubusercontent.com/Shopify/dawn/main/locales/fi.json) | Haluan lähettää tämän lahjaksi | vastaanottaja |
| [cs](https://raw.githubusercontent.com/Shopify/dawn/main/locales/cs.json) | Chci zboží poslat jako dárek | příjemce |
| [el](https://raw.githubusercontent.com/Shopify/dawn/main/locales/el.json) | Θέλω να το στείλω ως δώρο | παραλήπτης |

The same form asks for "Recipient email", "Recipient name (optional)", "Message (optional)" and "Send on (optional)" — the same shape as the Decitre flow described in the sibling notes (e-mail, message, chosen date) [V1-sib]. Other platforms' English source strings: PrestaShop "I would like my order to be gift wrapped" and "If you'd like, you can add a note to the gift:" [V1] ([file](https://raw.githubusercontent.com/PrestaShop/PrestaShop/develop/translations/default/ShopThemeCheckout.xlf)); Magento "Gift Options", "Do you have any gift items in your order?" [V1] ([file](https://raw.githubusercontent.com/magento/magento2/2.4-develop/app/code/Magento/GiftMessage/i18n/en_US.csv)). Translations of these two were not reachable.

Portuguese note: Shopify uses *presente* for both variants, but pt-PT retail prefers *oferta / oferecer* [M, medium]; the pt-PT gift card in the same file is « cartão de oferta » [V1].

## 4. (b) The product: gift card / voucher nouns

No corpus contains a noun for "a voucher for one specific eBook". Names used by earlier products are in the sibling notes: "eBook gift cards" (Livrada, Enthrill — press term), "Kindle Cards", "digital book cards" (Walmart/Kobo), "print codes" (BookFunnel), "download cards" (Dropcards), „Download-Codes" (Bookwire) [V1-sib/V2-sib] ([Publishers Weekly](https://publishersweekly.com/pw/print/20120827/53710-livrada-tries-its-luck-with-e-book-gift-cards.html), [Shelf Awareness](https://shelf-awareness.com/theshelf/2016-01-26/amazon_testing_title-specific_kindle_gift_cards_.html), [BookFunnel](https://authors.bookfunnel.com/help/print-codes/), [Bookwire](https://bookwire.net/faq-download-codes)).

The stored-value noun ("gift card") matters as the word to *avoid* for a single prepaid title (see `terminology.naming.md` §4) and as a search keyword.

| Lang | Shopify Dawn ("gift card") | Odoo 17 ("Gift Card") | Agreement | Retail usage recalled [M] |
|---|---|---|---|---|
| fr | carte-cadeau | Carte-cadeau, pl. Cartes-cadeaux (one string: "Carte cadeau") | [V2] | France: « carte cadeau », « e-carte cadeau », « chèque-cadeau » (high); BE/CH: also « bon cadeau » (medium); Québec: « carte-cadeau », « chèque-cadeau », « certificat-cadeau » (medium) |
| de | Geschenkgutschein; Gutschein | Geschenkkarte | split | Thalia/Hugendubel „Geschenkkarte"; Amazon.de „Gutschein" (high) |
| es | tarjeta de regalo | Tarjeta de regalo (es and es_419) | [V2] | Spain retail: « tarjeta regalo »; Amazon.es « cheque regalo » (high) |
| it | buono regalo | Carta regalo | split | Amazon.it « Buoni Regalo » (high); bookshops « Gift Card » (medium) |
| nl | cadeaubon | Cadeaubon | [V2] | bol.com « cadeaukaart » (high). OpenTaal lists `cadeaubon`, `cadeaukaart`, `boekenbon` [V1] |
| pt-PT | cartão de oferta | Cartão de presente (file `pt`) | split | Fnac.pt « Cartão Oferta »; Wook/Bertrand « cheque-prenda » (medium) |
| pt-BR | cartão-presente | Cartão de presente; Cartão-presente | [V2] | Amazon.com.br « vale-presente » (medium-high) |
| pl | karta prezentowa | Karta podarunkowa | split | Empik « karta prezentowa »; Amazon.pl « karty podarunkowe » (medium) |
| sv | presentkort | Presentkort | [V2] | same (high) |
| da | gavekort | Gavekort | [V2] | same (high) |
| nb | gavekort | "Her er ditt gavekort" | [V2] | same (high) |
| fi | lahjakortti | Lahjakortti | [V2] | same (high) |
| cs | dárková karta | Dárkový poukaz | split | « dárkový poukaz » dominant in bookshops (medium) |
| ro | — | Card cadou | [V1] | « card cadou », « voucher cadou » (medium) |
| el | δωροκάρτα | — | [V1] | « δωροκάρτα », « δωροεπιταγή » (medium) |
| ca | — | Targeta Regal; Targetes de regal | [V1] | « targeta regal », « val regal » (medium-low) |

Shopify links: per-language files as in section 3.2. Odoo links follow the pattern of the [French file](https://raw.githubusercontent.com/odoo/odoo/17.0/addons/loyalty/i18n/fr.po) with the language code replaced.

Typography: Shopify's French file writes "carte‑cadeau" with a non-breaking hyphen (U+2011) in several strings. Odoo also hyphenates. French retailers do not hyphenate [M, high].

## 5. (c) Redemption

### 5.1 "Redeem" in three shipped products

Shopify: QR alt text "QR code — scan to redeem gift card". Signal: button "Redeem" on a received gift (`GiftMessageView__redeem`). Mullvad: "Redeem voucher". The language code links to the Signal file; Mullvad files follow the pattern of the [French file](https://raw.githubusercontent.com/mullvad/mullvadvpn-app/main/desktop/packages/mullvad-vpn/locales/fr/messages.po).

| Lang | Shopify | Signal | Mullvad | Verdict |
|---|---|---|---|---|
| [fr](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-fr/strings.xml) | scannez-le pour **utiliser** votre carte-cadeau | **Récupérer** | **Échanger** un bon | unstable — 3 verbs |
| [de](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-de/strings.xml) | Scannen, um Gutschein **einzulösen** | Einlösen | Gutschein einlösen | stable [V2] |
| [es](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-es/strings.xml) | escanea para **canjear** | Canjear | Canjear cupón | stable [V2] |
| [it](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-it/strings.xml) | scansiona per **riscattare** | Riscatta | Riscatta voucher | stable [V2] |
| [nl](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-nl/strings.xml) | scannen om cadeaubon te **verzilveren** | **In ontvangst nemen** | Voucher **inwisselen** | unstable — 3 verbs |
| [pt-PT](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-pt/strings.xml) | digitalizar para **resgatar** | Resgatar | **Reclamar** voucher | mostly *resgatar* [V2] |
| [pt-BR](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-pt-rBR/strings.xml) | faça a leitura para **resgatar** | Resgatar | — | stable [V2] |
| [pl](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-pl/strings.xml) | zeskanuj, aby **wykorzystać** | **Odbierz** | **Zrealizuj** kupon | unstable — 3 verbs |
| [sv](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-sv/strings.xml) | skanna för att **lösa in** | Lös in | Lös in kupong | stable [V2] |
| [da](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-da/strings.xml) | scan for at **indløse** | Indløs | Indløs kupon | stable [V2] |
| [nb](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-nb/strings.xml) | skann for å **løse inn** | Løs inn | Løs inn kupong | stable [V2] |
| [fi](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-fi/strings.xml) | **lunasta** lahjakortti skannaamalla | Lunasta | Lunasta kuponki | stable [V2] |
| [cs](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-cs/strings.xml) | po naskenování můžete **uplatnit** | Uplatnit | — | stable [V2] |
| [ro](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-ro/strings.xml) | — | **Revendică**; past: "Ai **răscumpărat** o insignă" | — | unstable inside one app [V1] |
| [el](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-el/strings.xml) | σαρώστε για να **εξαργυρώσετε** | Εξαργύρωση | — | stable [V2] |
| [ca](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-ca/strings.xml) | — | Bescanvia | — | [V1] |

Reading:

- French has no settled word. « Utiliser » is also what I recall from Apple (« Utiliser une carte cadeau ou un code ») and Google Play (« Utiliser un code ») [M, medium-high]. « Récupérer » is the only one of the three used for a *gift* rather than a balance, which supports the sibling note's button « Récupérer mon livre ». « Échanger » also means returning goods in French retail [M, high].
- Dutch: *verzilveren* ("cash in") and *inwisselen* ("exchange") are both voucher verbs; Signal avoided both for a gift ("take receipt of").
- In the stable languages the voucher verb is what users will type into a help search. Whatever the button says, help pages and metadata should contain it.
- The same Shopify string gives the scan verb: *scannez, scannen, escanea, scansiona, zeskanuj, skanna, scan, skann, σαρώστε*; pt-BR *faça a leitura*, pt-PT *digitalizar* [V1]. Italian *scansiona* differs from the sibling note's « Inquadra » [M].

### 5.2 Entering the code — Mullvad [V1 per language]

| Lang | "Enter voucher code" | "Voucher code has already been used." |
|---|---|---|
| fr | Saisir un code de bon | Le code du bon a déjà été utilisé. |
| de | Gutscheincode eingeben | Der Gutscheincode wurde bereits verwendet. |
| es | Escriba el código del cupón | El código del cupón ya se ha usado. |
| it | Inserisci codice voucher | Il codice voucher è già stato utilizzato. |
| nl | Vouchercode invoeren | Vouchercode is al gebruikt. |
| pt | Introduza o código do voucher | O código do voucher já foi utilizado. |
| pl | Wprowadź kod kuponu | Kod z tego kuponu został już użyty. |
| sv | Ange kupongkod | Kupongkoden har redan använts. |
| da | Indtast kuponkode | Kuponkode er allerede brugt. |
| nb | Skriv inn kupongkode | Kupongkoden er allerede brukt. |
| fi | Syötä kuponkikoodi | Kuponkikoodi on jo käytetty. |

"Voucher" itself is rendered *bon* (fr), *Gutschein* (de), *cupón* (es), *voucher* (it, nl, pt), *kupon/kupong/kuponki* (pl, sv, da, nb, fi) — a discount-coupon word in half the languages, one more reason not to call the card a voucher.

### 5.3 Do not say "passphrase" — Thorium Reader [V1]

The LCP prompt label "Passphrase" is translated *Mot de passe* (fr; the sentence says « clé LCP »), *Passwort* (de), *Santo y seña* (es; the sentence says *contraseña*), *Passphrase* (it), *Wachtwoordzin* (nl), *Frase-passe* (pt-PT), *Senha* (pt-BR), *Hasło dostępu* (pl), *Lösenord* (sv), *Nøgleord* (da), *Salasana* (fi), *Heslová fráze* (cs), *Συνθηματική φράση* (el), *Contrassenya* (ca) ([fr](https://raw.githubusercontent.com/edrlab/thorium-reader/develop/src/resources/locales/fr.json), [es](https://raw.githubusercontent.com/edrlab/thorium-reader/develop/src/resources/locales/es.json), [nl](https://raw.githubusercontent.com/edrlab/thorium-reader/develop/src/resources/locales/nl.json)). If the card ever carries an LCP passphrase, print it as "code".

### 5.4 QR wording — Shopify [V1]

« Code QR » (fr), „QR-Code" (de), "Código QR" (es, pt), "Codice QR" (it), "QR-code" (nl), "Kod QR" (pl), "QR-kod" (sv), "QR-kode" (da, nb), "QR-koodi" (fi), "QR kód" (cs), "Κωδικός QR" (el). In France the everyday form is « QR code » [M, high]; « code QR » is the Québec form [M, medium-high].

## 6. (d) Words for eBook and e-reader

### 6.1 What reading software ships

Sources per cell: C = calibre, K = KOReader ("Ebook reader"), W = calibre-web, F = Foliate, O = Open Library. The language code links to the calibre file; other files follow the patterns in section 2.

| Lang | eBook | e-reader (device) |
|---|---|---|
| [fr](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/fr.po) | livre numérique (C); livre électronique (F, O, W); e-book and Livre numérique ([Readest](https://raw.githubusercontent.com/readest/readest/main/apps/readest-app/public/locales/fr/translation.json)); eBook ([BookWyrm](https://raw.githubusercontent.com/bookwyrm-social/bookwyrm/main/locale/fr_FR/LC_MESSAGES/django.po)) | **liseuse** (C, K, W, O, Readest) [V2] |
| [fr-CA](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/fr_CA.po) | livre numérique (C) | liseuse (C) |
| [de](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/de.po) | eBook (C: „eBook-Betrachter", „eBook-Händler"); E-Book (K, W, O) | eBook-Reader (C); E-Book-Reader (K); E-Reader (W, O) |
| [es](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/es.po) | libro electrónico (C, also es_MX; F; O); ebooks (W) | lector de libros electrónicos (C, K) [V2]; lectores electrónicos (O); eReader (W) |
| [it](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/it.po) | e-book (C, F); eBook / Ebook (O, W) | lettore di e-book (C); Lettore ebook (K); e-reader (O); eReader (W) |
| [nl](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/nl.po) | e-boek (C, W, F) [V2] | e-reader (C); Ebooklezer (K) |
| [pt-PT](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/pt.po) | livro digital (C); ebooks (W) | leitor de livros digitais (C, K) [V2]; dispositivo de leitura (W) |
| [pt-BR](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/pt_BR.po) | e-book (C, F, W) [V2]; livro eletrônico (C, K) | leitor de e-books (C); leitor de livros eletrônicos (K); E-Reader (W) |
| [pl](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/pl.po) | e-book, declined: e-booka, e-booki, e-booków (C, W, O) [V2]; store = „E-księgarnia" (C) | czytnik (C, W); czytnik e-booków (K, O) [V2] |
| [sv](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/sv.po) | e-bok, e-böcker (C, F, W) [V2] | läsenhet (C); E-boksläsare (K); e-läsare (W) |
| [da](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/da.po) | e-bog, e-bøger (C) | e-boglæser (C) |
| [nb](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/nb.po) | e-bok, e-bøker (C) | e-bokleser (C); Ebokleser (K) |
| [fi](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/fi.po) | e-kirja (C, W) [V2] | lukulaite (C); E-kirjan lukija (K) |
| [cs](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/cs.po) | e-kniha (C, O); elektronická kniha (F); "eknih" (W) | čtečka e-knih (C, K, O) [V2] |
| [ro](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/ro.po) | carte electronică (C); E-book (C); eBook, E-books (O) | cititor de cărți electronice (C); Cititor e-book (K); e-readere (O) |
| [el](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/el.po) | ηλεκτρονικό βιβλίο (C, K, F) [V2]; ψηφιακό βιβλίο (C); eBook (W) | αναγνώστης ψηφιακών βιβλίων (C); αναγνώστης ηλεκτρονικών βιβλίων (K) |
| [ca](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/ca.po) | llibre electrònic (C) | lector de llibres electrònics (C, K) [V2] |
| [en-GB](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/en_GB.po) | e-book ("An e-book shop.") | — |

"Send to eReader" (calibre-web) [V1]: « Envoyer à la liseuse », „An E-Reader senden", "Enviar al eReader", "Invia all'eReader", "Enviar para dispositivo de leitura" (pt), "Enviar para E-Reader" (pt-BR), "Skicka till e-läsare".

### 6.2 Spelling norms found in tooling [V1 each]

| Lang | Evidence | Source |
|---|---|---|
| de | Rule group `E_BOOK` "E-Book/E-Commerce": „EBook" → „E-Book"; „EBook Reader" → „E-Book-Reader"; „E Books" → „E-Books" | [LanguageTool de](https://raw.githubusercontent.com/languagetool-org/languagetool/master/languagetool-language-modules/de/src/main/resources/org/languagetool/rules/de/grammar.xml) |
| de | The LibreOffice dictionary nevertheless lists `eBook/S` | [de_DE_frami.dic](https://raw.githubusercontent.com/LibreOffice/dictionaries/master/de/de_DE_frami.dic) |
| nl | List contains `e-book`, `e-books`, `e-boek`, `e-boeken`, `e-bookreader`, `e-reader`; no `ebook`, no `ereader` | [OpenTaal](https://raw.githubusercontent.com/OpenTaal/opentaal-wordlist/master/wordlist.txt) |
| nl | Rule group `EREADER`: "ereader" → "e-reader" (« Na de 'e' komt een streepje, net als bij 'e-mail' »); "e-Reader" → lower case | [LanguageTool nl](https://raw.githubusercontent.com/languagetool-org/languagetool/master/languagetool-language-modules/nl/src/main/resources/org/languagetool/rules/nl/grammar.xml) |
| da | `e-bog` | [da_DK.dic](https://raw.githubusercontent.com/LibreOffice/dictionaries/master/da_DK/da_DK.dic) |
| nb | `e-bok`; `lesebrett` | [nb_NO.dic](https://raw.githubusercontent.com/LibreOffice/dictionaries/master/no/nb_NO.dic) |
| cs | `e-kniha`; `čtečka` | [cs_CZ.dic](https://raw.githubusercontent.com/LibreOffice/dictionaries/master/cs_CZ/cs_CZ.dic) |
| it | `ebook` (no `e-book` entry) | [it_IT.dic](https://raw.githubusercontent.com/LibreOffice/dictionaries/master/it_IT/it_IT.dic) |
| ca | both `e-book` and `ebook` | [ca.dic](https://raw.githubusercontent.com/LibreOffice/dictionaries/master/ca/dictionaries/ca.dic) |
| en-GB | `e-book` | [en_GB.dic](https://raw.githubusercontent.com/LibreOffice/dictionaries/master/en/en_GB.dic) |

A spelling dictionary shows that a form is accepted, not that it is preferred. The LanguageTool `grammar.xml` / `style.xml` files for French, Spanish, Catalan, Portuguese, Italian and Polish contain no rule on these words (grep, this session; other rule files not checked).

### 6.3 Official terminology (all [M]; the official sites were blocked)

| Body | Recommendation | Confidence |
|---|---|---|
| France — Commission d'enrichissement de la langue française (FranceTerme), *Journal officiel* of 4 April 2012 | « livre numérique » for the work, « liseuse » for the device; "e-book" and "e-reader" given as foreign equivalents | terms high; date medium-high |
| France — statute | Loi n° 2011-590 « relative au prix du livre numérique » — the legal term is « livre numérique » | [V2-sib] for the title ([WIPO Lex](https://www.wipo.int/wipolex/en/legislation/details/13861), [Livres Hebdo](https://www.livreshebdo.fr/article/la-loi-sur-le-prix-en-vigueur)) |
| Québec — OQLF, *Grand dictionnaire terminologique* | « livre numérique » preferred; « livre électronique » and « livrel » (built like « courriel ») as synonyms; device « liseuse »; « carte-cadeau » and « code QR » | medium |
| Germany — Duden | „E-Book", „E-Book-Reader", „E-Reader" (capital E, hyphen, as „E-Mail") | high; consistent with the LanguageTool rule above |
| Netherlands/Flanders — Taalunie word list | "e-book", "e-boek", "e-reader" | high; consistent with OpenTaal/LanguageTool above |
| Spain — Fundéu / RAE | « libro electrónico » (also « libro digital »); device « lector de libros electrónicos » | medium-high |
| Italy — Treccani, Crusca | "e-book" and "ebook" both recorded; « libro elettronico / digitale » | medium |
| Catalonia — TERMCAT | « llibre electrònic »; device « lector de llibres electrònics » | medium |
| Sweden / Denmark / Norway / Finland | "e-bok" + « läsplatta »; "e-bog"; "e-bok" + « lesebrett »; "e-kirja" (Kotus also « sähkökirja ») + « lukulaite » | medium-high |
| Poland / Czechia / Romania | PWN "e-book"; ÚJČ "e-kniha"; DOOM3 "e-book" | medium / medium / medium-low |

« Livrel »: I know of no retailer using it. Québec booksellers and libraries (Les libraires, Renaud-Bray, BAnQ, pretnumerique.ca) say « livres numériques » [M, medium-high]. It does not appear in any French file fetched this session [V1, negative].

## 7. Retailer and platform strings recalled from memory (hypotheses, not findings)

| Lang | eBook / e-reader as written by retailers | Redeem wording | Conf. |
|---|---|---|---|
| fr-FR | Amazon.fr « Ebooks Kindle », « liseuse »; Kobo and Fnac « eBooks » / « Ebooks », « liseuses »; Decitre « ebook »; independents and libraries « livres numériques » | Apple « Utiliser une carte cadeau ou un code »; Google Play « Utiliser un code » | medium-high |
| fr-BE / fr-CH | Same vocabulary as France; « bon cadeau » more frequent; Belgian independents' portal Librel sells « livres numériques » | — | medium-low |
| fr-CA | « livres numériques », « liseuse », « carte-cadeau », « courriel » | « Échanger » / « Utiliser » both seen | medium |
| en | Amazon "Kindle eBooks", "Kindle E-readers"; Kobo "eBooks", "eReaders"; style guides (AP, Chicago, Oxford) "e-book", "e-reader"; trade press "ebook" | "Redeem" everywhere; UK also "gift voucher", "book token" | high |
| de | Thalia, Hugendubel, ebook.de, Amazon.de „eBook"; tolino „eReader"; press and Börsenverein „E-Book" | „Gutschein einlösen", „Geschenkkarte einlösen" | high |
| nl | bol.com "ebooks", "e-readers"; Kobo "eBooks" | "verzilveren" (bol.com), "inwisselen" (Apple, Google, Amazon.nl) | medium |
| es | Casa del Libro, Amazon.es, El Corte Inglés "eBooks"; "eReader" | "Canjear" (Amazon.es « Canjear un cheque regalo ») | medium-high |
| it | IBS, Feltrinelli, Mondadori Store, Kobo "eBook"; "eReader" | "Riscatta" (Apple); "Utilizza codice" (Google Play) | medium |
| pt | Wook, Bertrand, Fnac.pt "eBooks"; BR: Amazon.com.br "eBooks Kindle", press "e-book", trade body « livro digital » | "Resgatar" (BR); PT also "Usar" | medium |
| pl | Empik, Woblink, Publio, Legimi « ebooki » / « e-booki »; « czytniki (ebooków) » | « Zrealizuj », « Wykorzystaj kod », « Aktywuj kod » | medium |
| sv / da / nb / fi | Adlibris, Bokus « E-böcker », « Läsplattor »; Saxo « E-bøger »; ARK, Norli « E-bøker », « Lesebrett »; Suomalainen Kirjakauppa « E-kirjat », « Lukulaitteet » | « Lös in », « Indløs », « Løs inn », « Lunasta » | medium-high |
| cs | Palmknihy, Kosmas, Alza « e-knihy », « čtečky e-knih » | « Uplatnit » | medium |
| ro / el / ca | Elefant, Libris, Cărturești « eBook » / « ebook »; Public « eBooks »; Catalan shops « ebooks » / « llibres electrònics » | « Folosește / Aplică »; « Εξαργύρωση »; « Bescanviar » | low-medium |

Pattern [M, medium-high]: in every market the large retailers write an English-derived trade form ("eBook", "ebook", « ebooki ») while dictionaries, public bodies, libraries and free-software translators prefer a native or hyphenated form. Nordic, Czech, Finnish and Dutch are the exceptions: retail uses the native word (*e-bok, e-bog, e-kirja, e-kniha*).

## 8. Consequences for `crates/ebc-i18n` and the demo stores

1. **Make the eBook noun a per-reseller variable.** Defaults when the host site gives none: fr « livre numérique » (and « ebook » as a search synonym), fr-CA « livre numérique », de „E-Book", nl "e-book", es "libro electrónico" / "eBook", it "eBook", pt-PT "ebook", pt-BR "e-book", pl "e-book", sv "e-bok", da "e-bog", nb "e-bok", fi "e-kirja", cs "e-kniha", ro "eBook", el "eBook" / « ηλεκτρονικό βιβλίο », ca « llibre electrònic ». The demo "fake Fnac" should read « Ebook »/« liseuse »; a fake Thalia „eBook"/„eReader".
2. **Device noun defaults with the best evidence**: fr « liseuse », cs « čtečka », pl « czytnik », ca « lector de llibres electrònics », nl "e-reader", es « lector de libros electrónicos » or "eReader", de „E-Reader"/„eReader", nb « lesebrett », sv « läsplatta », fi « lukulaite », da « e-bogslæser ». Always list "liseuse, tablette ou téléphone"-type alternatives: the scanning device is rarely the reading device.
3. **Redeem verb**: in de, es, it, sv, da, nb, fi, cs, el, pt-BR the voucher verb is unambiguous; use it at least in help text and page metadata. In fr, nl, pl, ro there is no consensus, so a plain verb (« Récupérer mon livre », "Mijn boek ophalen", „Odbierz książkę") loses nothing. Avoid fr « échanger ».
4. **Call the typed fallback "code"** in every language; never "passphrase/mot de passe" and never "voucher/bon".
5. **Locale markers to split fr-FR from fr-CA**: « QR code » / « code QR », « carte cadeau » / « carte-cadeau », « e-mail » / « courriel ».
6. **Checkout option**: the attested pattern is a verb or first-person checkbox, not a product noun. "eBook cadeau" can remain the feature's name in documentation and the Dutch/French marketing line.
7. **Grammar**: German compounds hyphenate after "E-Book" („E-Book-Reader"); Dutch "e-book" / "e-reader" stay lower case mid-sentence; Polish declines « e-book » (e-booka, e-booków), so the noun variable needs case forms, not one string.

## 9. Search-volume hints

None obtained. Google Trends and keyword tools were unreachable. From memory only [M, low]: French users search « ebook » far more than « livre numérique »; how-to intent sits on « offrir un ebook »; « ebook cadeau » and « ebook offert » are dominated by free lead-magnet pages. `terminology.naming.md` §1 reaches the same judgment, also unverified.

## Sources

Fetched and read this session (all `raw.githubusercontent.com`, HTTP 200, 2026-11-12). `{l}` = language code.

- Shopify Dawn — `https://raw.githubusercontent.com/Shopify/dawn/main/locales/{l}.json` for `en.default`, `fr`, `de`, `es`, `it`, `nl`, `pt-BR`, `pt-PT`, `pl`, `sv`, `da`, `nb`, `fi`, `cs`, `el` (`ro-RO` and `ca`: 404)
- Odoo 17.0 — `https://raw.githubusercontent.com/odoo/odoo/17.0/addons/loyalty/i18n/{l}.po` and `.../addons/website_sale_loyalty/i18n/{l}.po` for `fr`, `de`, `es`, `es_419`, `it`, `nl`, `pt`, `pt_BR`, `pl`, `sv`, `da`, `nb`, `fi`, `cs`, `ro`, `el`, `ca`; plus `loyalty` `el`/`nb` on branches `18.0` and `master`
- Signal-Android — https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values/strings.xml and `.../res/values-{l}/strings.xml` for `fr`, `de`, `es`, `it`, `nl`, `pt`, `pt-rBR`, `pl`, `sv`, `da`, `nb`, `fi`, `cs`, `ro`, `el`, `ca`
- Mullvad VPN app — `https://raw.githubusercontent.com/mullvad/mullvadvpn-app/main/desktop/packages/mullvad-vpn/locales/{l}/messages.po` for `fr`, `de`, `es`, `it`, `nl`, `pt`, `pl`, `sv`, `da`, `nb`, `fi`
- calibre — `https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/{l}.po` for `fr`, `fr_CA`, `de`, `es`, `es_MX`, `it`, `nl`, `pt`, `pt_BR`, `pl`, `sv`, `da`, `nb`, `fi`, `cs`, `ro`, `el`, `ca`, `en_GB`
- KOReader — `https://raw.githubusercontent.com/koreader/koreader-translations/master/{l}/koreader.po` for `fr`, `de`, `es`, `it_IT`, `nl_NL`, `pt_PT`, `pt_BR`, `pl`, `sv`, `da`, `nb_NO`, `fi`, `cs`, `ro`, `el`, `ca`
- calibre-web — `https://raw.githubusercontent.com/janeczku/calibre-web/master/cps/translations/{l}/LC_MESSAGES/messages.po` for `fr`, `de`, `es`, `it`, `nl`, `pt`, `pt_BR`, `pl`, `sv`, `no`, `fi`, `cs`, `el`
- Foliate — `https://raw.githubusercontent.com/johnfactotum/foliate/gtk4/po/{l}.po` for `fr`, `de`, `es`, `it`, `nl`, `pt_BR`, `sv`, `nb`, `cs`, `el`
- Open Library — `https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/i18n/{l}/messages.po` for `fr`, `de`, `es`, `it`, `pt`, `pl`, `cs`, `ro`
- Thorium Reader — `https://raw.githubusercontent.com/edrlab/thorium-reader/develop/src/resources/locales/{l}.json` for `en`, `fr`, `de`, `es`, `it`, `nl`, `pt-pt`, `pt-br`, `pl`, `sv`, `da`, `fi`, `cs`, `el`, `ca`
- Readest — https://raw.githubusercontent.com/readest/readest/main/apps/readest-app/public/locales/fr/translation.json
- BookWyrm — https://raw.githubusercontent.com/bookwyrm-social/bookwyrm/main/locale/fr_FR/LC_MESSAGES/django.po
- LanguageTool — https://raw.githubusercontent.com/languagetool-org/languagetool/master/languagetool-language-modules/de/src/main/resources/org/languagetool/rules/de/grammar.xml ; https://raw.githubusercontent.com/languagetool-org/languagetool/master/languagetool-language-modules/nl/src/main/resources/org/languagetool/rules/nl/grammar.xml ; same path pattern for `fr`, `es`, `ca`, `pt`, `it`, `pl`, `sv`, `da` (no relevant rule found)
- OpenTaal — https://raw.githubusercontent.com/OpenTaal/opentaal-wordlist/master/wordlist.txt
- LibreOffice dictionaries — https://raw.githubusercontent.com/LibreOffice/dictionaries/master/de/de_DE_frami.dic ; `.../da_DK/da_DK.dic` ; `.../no/nb_NO.dic` ; `.../cs_CZ/cs_CZ.dic` ; `.../it_IT/it_IT.dic` ; `.../ca/dictionaries/ca.dic` ; `.../en/en_GB.dic`
- PrestaShop — https://raw.githubusercontent.com/PrestaShop/PrestaShop/develop/translations/default/ShopThemeCheckout.xlf
- Magento — https://raw.githubusercontent.com/magento/magento2/2.4-develop/app/code/Magento/GiftMessage/i18n/en_US.csv

Reused from sibling notes, not re-opened (-sib):

- https://www.decitre.fr/offrir-un-ebook ; https://www.decitre.fr/faq/numerique/offrir-un-ebook ; https://www.liseuses.net/offrir-des-ebooks-avec-tea/
- https://www.amazon.fr/gp/help/customer/display.html?nodeId=GVWGP284MQ6ZRM59 ; https://www.idboox.com/news-livres/amazon-on-peut-enfin-offrir-un-livre-numerique-kindle/
- https://www.hugendubel.de/de/shortcut/ebooksverschenken ; https://www.heise.de/tipps-tricks/E-Books-verschenken-so-klappt-s-7476189.html
- https://www.dealabs.com/discussions/carte-cadeau-kobo-ou-fnac-2273026
- https://publishersweekly.com/pw/print/20120827/53710-livrada-tries-its-luck-with-e-book-gift-cards.html ; https://shelf-awareness.com/theshelf/2016-01-26/amazon_testing_title-specific_kindle_gift_cards_.html ; https://authors.bookfunnel.com/help/print-codes/ ; https://bookwire.net/faq-download-codes
- https://www.wipo.int/wipolex/en/legislation/details/13861 ; https://www.livreshebdo.fr/article/la-loi-sur-le-prix-en-vigueur
- Sibling files: `market-business.competitors-costs.md` §1.2, `market-business.market.md` §5.2, `ereader-ux.ecosystems.md` §1–3, `legal-covers-reseller.reseller.md` §1.1, `terminology.naming.md`

## Could not verify

1. **Every live retailer string**: Amazon (all marketplaces), Kobo, Apple Books, Google Play Books, Fnac, 7switch, Cultura, Thalia, Hugendubel, bol.com, Casa del Libro, IBS, Feltrinelli, Mondadori Store, Wook, Empik, Woblink, Adlibris, Bokus, Saxo, ARK, Suomalainen Kirjakauppa, Palmknihy, Elefant, Public, Abacus. Section 7 and the [M] rows of section 3.1 are recollection only.
2. **Which of these retailers offer title-specific gifting in 2026**, and the exact button text, beyond the sibling-sourced Decitre, Amazon.fr and Hugendubel/Thalia cases.
3. **Official terminology pages**: FranceTerme entry and *Journal officiel* date for « livre numérique » / « liseuse »; OQLF entries for « livre numérique », « livrel », « liseuse », « carte-cadeau », « code QR »; Duden entry „E-Book"; Fundéu, RAE, Treccani, TERMCAT, SAOL, Kotus, PWN, ÚJČ, DOOM3 entries. All hosts tried were blocked.
4. **Search volumes** and any Google Trends comparison, in any language.
5. **Regional French retail usage** (Belgium, Switzerland, Québec): no retailer page and no fr-BE/fr-CH locale file was found; the only fr-CA file is calibre's.
6. **Romanian, Greek and Catalan** rest on one or two files each; Shopify ships no Romanian or Catalan default locale on the branch read.
7. **Translation provenance**: I did not check how each project produces its translations (professional or community), nor the date of each string; Odoo's `pt` file looks Brazilian-influenced.
8. **Apple, Google and Amazon localised "redeem" strings** quoted in sections 5.1 and 7 are from memory; their glossaries are not in any reachable repository.
