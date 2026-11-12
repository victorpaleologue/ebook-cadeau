# What to call it: "eBook cadeau" and its equivalents per website language

_Status: draft — research date 2026-11-12_

> **Read this first.** No web search could be run for this topic. The two researchers sent 12 queries and all were refused (the workflow's shared budget of 200 searches was already spent). Six more were attempted for this synthesis and were refused too. **No bookseller page, trademark register, official terminology database or keyword tool was opened by anyone.** The 25-search and two-source rules were not met.

**What the evidence is.**

- About 190 locale and dictionary files downloaded and read by the usage researcher from `raw.githubusercontent.com` (Shopify's reference theme, Odoo, Signal, Mullvad, calibre, KOReader, calibre-web, Foliate, Open Library, Thorium Reader, LanguageTool, OpenTaal, LibreOffice dictionaries). These are strings shipped to real users, but **not by booksellers**.
- Facts about three live retailer gift flows (Decitre, Amazon.fr, Hugendubel/Thalia), sourced by other researchers in sibling notes from their own search results.
- Memory, for everything else: every other retailer string, all official terminology, all law, and every recommended string.

**Confidence markers.**

| Marker | Meaning |
|---|---|
| [V2] | Same form found in two or more independent files fetched in the research session |
| [V1] | One file fetched in the research session |
| [V2-sib] / [V1-sib] | Sourced by another researcher in a sibling note (two or more sources / one source). The URL is copied from that note; it was not re-opened here |
| [M] | From memory, not verified. "high / medium / low" is the researcher's recall grade |

No marker was upgraded in this synthesis. Every link below is either a file the usage researcher fetched, or a URL recorded in a sibling note. None comes from a search made for this document, and none is invented.

**Inputs.** `notes/terminology.usage.md` (attested strings), `notes/terminology.naming.md` (naming, trademark, recommended strings). Cross-read for consistency: `notes/legal-gifting-commerce.commerce.md` (VAT vouchers, card mentions), `notes/ereader-ux.ux.md` (text printed on the card), `notes/ereader-ux.ecosystems.md`, `notes/market-business.competitors-costs.md`, `docs/architecture.md` (the `Msg` enum and `Locale::ALL`).

---

## 1. Executive summary and recommended strings

### 1.1 Findings and decisions

1. **Keep "eBook Cadeau" as the project name and as the option's name in French and Dutch. Do not pay for a word mark.** The phrase is plain French and plain Dutch for the service, so a word mark would probably be refused in France and EU-wide [M] (§3.2). That weakness helps the reseller mode: any shop can use a generic label without a licence.
2. **The option label is the verb of giving, not a noun.** The only three sourced retailer flows use it: Decitre « Offrir un ebook », button « Offrir »; Hugendubel/Thalia „Verschenken"; Amazon.fr « Acheter pour d'autres personnes » [V2-sib]. Recommended: « Offrir cet ebook », „eBook verschenken", "Cadeau doen", "Regalar este eBook", "Regala questo eBook", "Give as a gift". No title-level gift option was found at Kobo by Fnac [V1-sib]; 7switch is unknown.
3. **The card says "book" and "gift".** Headline « Ce livre est un cadeau ». No price, no balance, no "valid until".
4. **Never use a stored-value word** for the object or the option: carte/chèque/bon cadeau, Gutschein, Geschenkkarte, cadeaubon, cadeaukaart, tarjeta/cheque regalo, buono/carta regalo, gift card, voucher, book token (§4). They promise a balance and bring validity rules into the reader's mind. A lint must enforce this.
5. **The redemption button uses a plain first-person verb**: « Récupérer mon livre », "Get my book". French, Dutch and Polish have no settled word for "redeem" (three verbs in three products each) [V2]. German, Spanish, Italian, Portuguese and the Nordic languages do (einlösen, canjear, riscattare, resgatar…) [V2]: put that verb in help pages and page metadata, where people search for it, not on the button.
6. **The typed fallback is a "code" in every language.** "Passphrase" has 14 different translations in one app [V1]; "voucher code" turns into a discount-coupon word in half the languages [V1].
7. **The device noun is settled in French (« liseuse », five corpora [V2]); the book noun is not settled anywhere.** Retailers write a trade form (ebook, eBook); dictionaries and public bodies write another (livre numérique, E-Book, libro electrónico). The noun is therefore a per-reseller setting with two styles, `trade` and `official` (§1.4).
8. **Default to register-neutral wording**: infinitive instructions on the card, first-person buttons. Where a pronoun is unavoidable: FR vous, NL je, ES tú, IT tu, PT formal third person; German has no safe default and takes du/Sie from the reseller [M].
9. **Whether the code is a "voucher" for VAT is a tax question, separate from the wording.** The contract and the invoice can use the legal term if the VAT adviser wants it; the card and the screens never do (§4.2).
10. **Every recommended string is [M] and needs a native reviewer.** The weakest: all Portuguese, the Italian button, the German du/Sie choice, and the help sentences drafted for this document.

### 1.2 Recommended strings — six primary languages

Short strings. Long strings (help sentence, the three printed steps) are in the TOML block of §5. All [M]; the second table shows which choices a sourced string supports.

| Key | fr | en | de | es | it | nl |
|---|---|---|---|---|---|---|
| `product_name` | eBook Cadeau | eBook Gift | eBook-Geschenk | eBook Regalo | eBook Regalo | eBook Cadeau |
| `checkout_option` | Offrir cet ebook | Give as a gift | eBook verschenken | Regalar este eBook | Regala questo eBook | Cadeau doen |
| `card_headline` | Ce livre est un cadeau | This book is a gift | Dieses Buch ist ein Geschenk | Este libro es un regalo | Questo libro è un regalo | Dit boek is een cadeau |
| `card_from` | De la part de {name} | From {name} | Von {name} | De parte de {name} | Da parte di {name} | Van {name} |
| `redeem_cta` | Récupérer mon livre | Get my book | Mein Buch abholen | Conseguir mi libro | Ricevi il libro | Mijn boek ophalen |
| `ebook_word` | livre numérique | eBook | eBook | eBook | eBook | e-book |
| `ereader_word` | liseuse | e-reader | eReader | eReader | eReader | e-reader |
| `code_label` | Code | Code | Code | Código | Codice | Code |

Support for these choices:

| Choice | Support | Tag |
|---|---|---|
| fr « Offrir cet ebook » | Decitre page title « Offrir un ebook », button « Offrir » ([Decitre](https://www.decitre.fr/offrir-un-ebook), [Decitre FAQ](https://www.decitre.fr/faq/numerique/offrir-un-ebook)) | [V2-sib] for the pattern; the exact string is [M] |
| de „eBook verschenken" | Hugendubel/Thalia „Verschenken", „eBooks verschenken" ([Hugendubel](https://www.hugendubel.de/de/shortcut/ebooksverschenken), [heise](https://www.heise.de/tipps-tricks/E-Books-verschenken-so-klappt-s-7476189.html)) | [V2-sib] for the pattern |
| fr « Récupérer mon livre » | Signal's French button on a received gift is « Récupérer » ([file](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-fr/strings.xml)) | [V1] for the verb |
| fr « liseuse » | calibre, KOReader, calibre-web, Open Library, Readest (§2.2) | [V2] |
| nl "e-book", "e-reader" | OpenTaal word list and LanguageTool rule (§2.7) | [V2] |
| "Code" as the label | Uniform across every file read; "passphrase" is not (§2.9) | [V1] |
| All other cells | Researcher's native-usage judgment | [M] |

### 1.3 Alternates, sub-options and register

From the naming note; all [M].

| Lang | Option-label alternates | Card-headline alternates (giver's choice in standalone mode) | Button alternates | Name for the printed object | Pronoun if unavoidable |
|---|---|---|---|---|---|
| fr | « C'est pour offrir » ; « Acheter pour offrir » | « {Prénom} vous offre ce livre » / « {Prénom} t'offre ce livre » ; « Un livre pour vous / pour toi » | « Ajouter à ma bibliothèque » ; « Lire maintenant » | « jaquette cadeau », « carte à offrir » | vous |
| en | "This is a gift"; "Buy as a gift" | "A book for you" | "Add to my library"; "Read now" | "gift jacket", "book card" | — |
| de | „Als Geschenk kaufen" ; „Das ist ein Geschenk" | „Ein Buch für dich" / „Ein Buch für Sie" ; „{Name} schenkt dir / Ihnen dieses Buch" | „Zur Bibliothek hinzufügen" ; „Jetzt lesen" | „Geschenkumschlag", „Klappkarte" | none: du or Sie from the reseller |
| es | « Es para regalar » ; « Comprar para regalar » | « Un libro para ti » ; « {Nombre} te regala este libro » | « Añadir a mi biblioteca » ; « Leer ahora » | « tarjeta-libro », « sobrecubierta de regalo » | tú (Latin America: prefer the infinitive) |
| it | « È un regalo » ; « Acquista come regalo » | « Un libro per te » ; « {Nome} ti regala questo libro » | « Aggiungi ai miei libri » ; « Leggi ora » | « biglietto-libro », « sovraccoperta regalo » | tu |
| nl | "Dit e-book cadeau doen"; "Geef als cadeau"; "Het is een cadeautje" | "Een boek voor jou"; nl-BE formal "Een boek voor u" | "Toevoegen aan mijn bibliotheek"; "Nu lezen" | "cadeau-omslag", "kaart" | je |

Sub-options under the checkout option (separate price line for the printed card, see §4.3):

| Lang | Print at home | Printed and posted |
|---|---|---|
| fr | Carte à imprimer chez moi (gratuit) | Carte imprimée et envoyée par la poste |
| en | Card to print at home (free) | Printed card, sent by post |
| de | Karte zum Selbstausdrucken (kostenlos) | Gedruckte Karte per Post |
| es | Tarjeta para imprimir en casa (gratis) | Tarjeta impresa y enviada por correo |
| it | Biglietto da stampare a casa (gratis) | Biglietto stampato e spedito |
| nl | Kaart zelf printen (gratis) | Gedrukte kaart per post |

Shop-counter questions that every buyer knows and that work as a checkbox label [M, high]: « C'est pour offrir ? » (fr), « ¿Es para regalo? » (es), « É para oferta? » (pt-PT).

### 1.4 The book noun and the device noun: two styles per language

`trade` = what large retailers write [M unless tagged]. `official` = what dictionaries, statutes and public bodies write (evidence in §2). The default column is what ships when the host site sets nothing.

| Lang | eBook, `trade` | eBook, `official` | e-reader, `trade` | e-reader, `official` | Default |
|---|---|---|---|---|---|
| fr | ebook (Decitre [V2-sib]); eBook / Ebook (Kobo, Fnac [M]) | livre numérique (statute title [V2-sib]; calibre [V1]) | liseuse | liseuse [V2] | `official` for running text, `trade` inside the option label |
| en | eBook | e-book (en-GB dictionary [V1]) | eReader (Kobo) / e-reader | e-reader | `trade` for the book, hyphenated "e-reader" for the device |
| de | eBook (Hugendubel „eBooks" [V2-sib]) | E-Book (LanguageTool rule [V1]; three corpora [V2]) | eReader | E-Reader (two corpora), E-Book-Reader (one) [V1 per file] | `trade` |
| es | eBook | libro electrónico (three corpora, [V1] per file) | eReader | lector de libros electrónicos [V2] | `trade` |
| it | eBook | e-book (calibre, Foliate) / ebook (dictionary) [V1] | eReader | lettore di e-book [V1] | `trade` |
| nl | e-book (bol.com writes "ebook" [M]) | e-book (OpenTaal [V1]); e-boek [V2] | e-reader | e-reader [V2] | `official` (same word) |
| pt | eBook | livro digital (pt-PT), e-book (pt-BR) [V1/V2] | eReader | leitor de livros digitais [V2] | `trade` |

### 1.5 Other languages (secondary)

Portuguese ships now, because `Locale::ALL` in the code already contains `pt` and Fnac also trades in Portugal [M]. Its strings are the least reliable in this document.

| Key | pt (pt-PT) | Note |
|---|---|---|
| `product_name` | eBook para Oferecer | — |
| `checkout_option` | Oferecer este eBook | alt. « É para oferta » |
| `card_headline` | Este livro é um presente | see §1.6, item 9 |
| `card_from` | De {name} | — |
| `redeem_cta` | Receber o meu livro | voucher verb for help pages: « resgatar » [V2] |
| `ebook_word` / `ereader_word` | eBook / eReader | — |
| `code_label` | Código | — |

Regional deltas for the shipped languages:

| Variant | Change | Tag |
|---|---|---|
| fr-CA | « livre numérique » only; « code QR », « carte-cadeau », « courriel »; tu is more accepted | Shopify's French file uses « code QR » and « carte-cadeau » [V1]; the rest [M, medium] |
| fr-BE, fr-CH, fr-LU | Same strings as fr-FR; « bon cadeau » is the common stored-value word there (still banned) | [M, medium-low] |
| de-AT, de-CH | Same strings. The German strings in §5 contain no « ß », so they work unchanged in Switzerland | [M] |
| nl-BE | Same strings; "u" in formal settings | [M, medium-high] |
| es-419 | "celular" for "móvil"; prefer infinitive forms | [M, medium] |
| pt-BR | "presente" (already used), "celular" for "telemóvel", "câmera" for "câmara", "e-book" for the noun [V2]; verb "resgatar" [V2] | mixed |

Languages not shipped. The table gives a glossary to hand to a translator. The draft labels tagged [M, low] were added in this synthesis from memory and are a starting point only.

| Lang | Draft option label | Redeem verb in shipped software | eBook | e-reader | Stored-value nouns to avoid |
|---|---|---|---|---|---|
| pl | „Kup na prezent" (Ebookpoint, Publio, Woblink) [M, medium] | unstable: wykorzystać / Odbierz / Zrealizuj [V2] | e-book, declined (e-booka, e-booków) [V2] | czytnik; czytnik e-booków [V2] | karta prezentowa, karta podarunkowa, kupon |
| cs | „Koupit jako dárek" (Alza, Palmknihy) [M, medium] | uplatnit [V2] | e-kniha [V1 dictionary + V1 corpus] | čtečka e-knih [V2] | dárková karta, dárkový poukaz |
| sv | "Ge bort som present" [M, low] | lösa in [V2] | e-bok [V2] | läsplatta (retail [M]); läsenhet, e-boksläsare, e-läsare [V1 each] | presentkort, kupong |
| da | "Giv som gave" [M, low] | indløse [V2] | e-bog [V1 + dictionary V1] | e-bogslæser (retail [M]); e-boglæser [V1] | gavekort, kupon |
| nb | "Gi som gave" [M, low] | løse inn [V2] | e-bok [V1 + dictionary V1] | lesebrett (dictionary [V1]; retail [M]); e-bokleser [V1] | gavekort, kupong |
| fi | "Osta lahjaksi" [M, low] | lunastaa [V2] | e-kirja [V2] | lukulaite [V1] | lahjakortti, kuponki |
| ro | "Oferă cadou" [M, low] | unstable inside one app: Revendică / răscumpărat [V1] | eBook, E-book, carte electronică [V1 each] | cititor de cărți electronice; e-readere [V1 each] | card cadou, voucher cadou |
| el | "Κάντε δώρο" [M, low] | εξαργύρωση [V2] | ηλεκτρονικό βιβλίο [V2]; eBook [V1] | αναγνώστης ηλεκτρονικών βιβλίων [V1] | δωροκάρτα, δωροεπιταγή |
| ca | "Regala aquest ebook" [M, low] | bescanviar [V1] | llibre electrònic [V1] | lector de llibres electrònics [V2] | targeta regal, val regal |

### 1.6 Contradictions between the notes, and how they are resolved

1. **German noun.** The usage note proposes „E-Book" as the default (LanguageTool rule [V1], three of four reading-software corpora [V2], Duden [M]). The naming note writes „eBook" in every German string. **We keep „eBook"** as the default: the resellers this product targets write it (Hugendubel „eBooks verschenken" [V2-sib]), the LibreOffice dictionary accepts it [V1], the LanguageTool rule that was read targets „EBook", „EBook Reader" and „E Books" [V1] (whether it also flags „eBook" was not checked), and the project name uses the same casing. „E-Book" ships as the `official` style for publishers, libraries and the press.
2. **Spanish and Portuguese nouns.** The usage note proposes « libro electrónico » or "eBook" (es) and "ebook" (pt-PT); the naming note writes "eBook". **We keep "eBook"**: the product lives in retail, and one casing across es, it, pt, de, en reduces the number of variants. The verified native forms ship as the `official` style.
3. **French print wording.** The naming note's primary print line is « Scannez le code pour recevoir votre livre » (vous), but its own convention, and the UX note's card text, use the infinitive so that the card does not choose tu or vous on behalf of the giver. **We keep the infinitive** (« Scanner ce code… »), in every language that allows it.
4. **German button.** The naming note has „Mein eBook abholen"; its buttons in the five other languages say "my book". **We keep „Mein Buch abholen"**: it matches the others and removes the eBook/E-Book spelling question from a hard-coded string.
5. **Italian scan verb.** The naming note says « Inquadra » [M, high]; Shopify's Italian file says « scansiona » [V1]. **We keep « Inquadrare » on the card**, because step 1 names the phone camera and that is the verb Italian pairs with a camera [M]. « Scansiona » is used for alternative text and help pages, where no camera is mentioned. Flagged for native review (§6).
6. **Portuguese scan verb.** Shopify's two Portuguese files disagree: « digitalizar » (pt-PT) and « faça a leitura » (pt-BR) [V1]. The naming note avoids both with « Aponte a câmara ao código » [M]. **We keep the camera wording** (« Apontar a câmara… »), in the infinitive, because the scan verb is visibly not settled.
7. **"Redeem".** The usage note says to use the established voucher verb at least in help text; the naming note says to keep it off buttons because it belongs to vouchers. These are two surfaces, and **both rules are kept**: plain verb on the button, voucher verb in help headings, page titles and search keywords.
8. **"Voucher".** The commerce note says a code for one named eBook is a voucher for VAT and suggests the card line « Bon pour un exemplaire numérique de… »; the naming note bans « bon » and "voucher". Resolved in §4.2: **the legal term stays in the contract and the invoice; the card line becomes « Un exemplaire numérique, déjà payé »**.
9. **Portuguese word for "gift".** The naming note has « Este livro é uma prenda » for pt-PT and « presente » for pt-BR; Shopify's pt-PT file itself says « presente » [V1]. The code has one `pt` locale for both countries. **We keep « presente »**: it is the verified form and the only one read the same way in both.
10. **Recall grades on validity rules.** The Dutch two-year rule is "medium-high" in one note and "low" in the other; the Irish five-year rule "high" and "medium". Both stay [M]; neither is relied on for a decision.

---

## 2. Attested usage per language

### 2.1 The corpus

Files fetched 2026-11-12 from default branches by the usage researcher.

| Corpus | What it is | Languages fetched | Used for |
|---|---|---|---|
| [Shopify Dawn `locales/`](https://raw.githubusercontent.com/Shopify/dawn/main/locales/en.default.json) | Shopify's reference storefront theme: gift-card page and "send as a gift" recipient form | fr, de, es, it, nl, pt-PT, pt-BR, pl, sv, da, nb, fi, cs, el (`ro-RO` and `ca`: 404) | gifting checkbox, gift-card noun, redeem, QR wording |
| [Odoo 17.0 `loyalty`](https://raw.githubusercontent.com/odoo/odoo/17.0/addons/loyalty/i18n/fr.po), [`website_sale_loyalty`](https://raw.githubusercontent.com/odoo/odoo/17.0/addons/website_sale_loyalty/i18n/fr.po) | Web-shop gift-card module | same, plus es_419, ro, ca (el, nb mostly untranslated) | gift-card noun |
| [Signal-Android `strings.xml`](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values/strings.xml) | "Redeem" button on a received gift | fr, de, es, it, nl, pt, pt-BR, pl, sv, da, nb, fi, cs, ro, el, ca | redeem |
| [Mullvad VPN app `messages.po`](https://raw.githubusercontent.com/mullvad/mullvadvpn-app/main/desktop/packages/mullvad-vpn/locales/fr/messages.po) | "Redeem voucher", "Enter voucher code", error messages | fr, de, es, it, nl, pt, pl, sv, da, nb, fi | redeem, code entry |
| [calibre](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/fr.po), [KOReader](https://raw.githubusercontent.com/koreader/koreader-translations/master/fr/koreader.po), [calibre-web](https://raw.githubusercontent.com/janeczku/calibre-web/master/cps/translations/fr/LC_MESSAGES/messages.po), [Foliate](https://raw.githubusercontent.com/johnfactotum/foliate/gtk4/po/fr.po), [Open Library](https://raw.githubusercontent.com/internetarchive/openlibrary/master/openlibrary/i18n/fr/messages.po) | Reading software and a book catalogue; community translations | 8 to 19 languages each | eBook and e-reader words |
| [Thorium Reader](https://raw.githubusercontent.com/edrlab/thorium-reader/develop/src/resources/locales/fr.json) | EDRLab's LCP reading app | 14 languages | passphrase wording |
| [LanguageTool de](https://raw.githubusercontent.com/languagetool-org/languagetool/master/languagetool-language-modules/de/src/main/resources/org/languagetool/rules/de/grammar.xml), [nl](https://raw.githubusercontent.com/languagetool-org/languagetool/master/languagetool-language-modules/nl/src/main/resources/org/languagetool/rules/nl/grammar.xml); [OpenTaal word list](https://raw.githubusercontent.com/OpenTaal/opentaal-wordlist/master/wordlist.txt); LibreOffice spelling dictionaries | Normative tooling | de, nl, da, nb, cs, it, ca, en-GB | spelling norms |
| [PrestaShop](https://raw.githubusercontent.com/PrestaShop/PrestaShop/develop/translations/default/ShopThemeCheckout.xlf), [Magento](https://raw.githubusercontent.com/magento/magento2/2.4-develop/app/code/Magento/GiftMessage/i18n/en_US.csv) | Checkout source strings (English only; translations not reachable) | en | gifting checkbox |

Limits: translators of free software are not booksellers. Community files contain errors (one Shopify Finnish label repeats the wrong string; calibre-web's Polish "Send to eReader" is a past tense). Agreement across independent files is the signal; one file alone is weak. How each project produces its translations was not checked.

In the tables below, "Shopify", "Signal" and "calibre" link to the file for that language. Mullvad, Odoo, KOReader, calibre-web, Foliate, Open Library and Thorium files follow the path of the French file linked above, with the language code replaced.

### 2.2 French

Files: [Shopify fr](https://raw.githubusercontent.com/Shopify/dawn/main/locales/fr.json), [Signal fr](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-fr/strings.xml), [calibre fr](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/fr.po), [calibre fr-CA](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/fr_CA.po), [Readest fr](https://raw.githubusercontent.com/readest/readest/main/apps/readest-app/public/locales/fr/translation.json), [BookWyrm fr](https://raw.githubusercontent.com/bookwyrm-social/bookwyrm/main/locale/fr_FR/LC_MESSAGES/django.po).

| Function | Quoted string | Where | Tag |
|---|---|---|---|
| Gift one title (retailer) | « Offrir un ebook … en 2 clics »; button « Offrir »; message and send date; delivered by e-mail | Decitre / Furet du Nord ([Decitre](https://www.decitre.fr/offrir-un-ebook), [Decitre FAQ](https://www.decitre.fr/faq/numerique/offrir-un-ebook), [Liseuses.net](https://www.liseuses.net/offrir-des-ebooks-avec-tea/)) | [V2-sib] |
| Gift one title (retailer) | « Acheter pour d'autres personnes » | Amazon.fr ([Amazon.fr help](https://www.amazon.fr/gp/help/customer/display.html?nodeId=GVWGP284MQ6ZRM59), [IDBoox](https://www.idboox.com/news-livres/amazon-on-peut-enfin-offrir-un-livre-numerique-kindle/); also [KDP help](https://kdp.amazon.com/fr_FR/help/topic/G200652260)) | [V2-sib] |
| Gift one title (retailer) | No « offrir » function found; users fall back on gift cards | Kobo by Fnac ([Dealabs](https://www.dealabs.com/discussions/carte-cadeau-kobo-ou-fnac-2273026); a low-authority guide says the same: [ebook-enligne.fr](https://ebook-enligne.fr/blog/offrir-ebook-guide-cadeau-numerique/)) | [V1-sib], negative finding |
| Gift one title (retailer) | 7switch, Cultura: not known. One sibling note has an unconfirmed recollection of an « offrir » option at 7switch | — | [M, low] |
| Generic gift checkbox | « Je veux envoyer ce produit comme cadeau »; recipient = « destinataire » | Shopify | [V1] |
| Gift-card noun | « carte-cadeau » (with a non-breaking hyphen, U+2011) | Shopify | [V2] with Odoo |
| Gift-card noun | « Carte-cadeau », plural « Cartes-cadeaux »; one string « Carte cadeau » | Odoo | [V2] with Shopify |
| Gift-card noun (retail) | France: « carte cadeau », « e-carte cadeau », « chèque-cadeau », no hyphen. BE/CH: also « bon cadeau ». Québec: « carte-cadeau », « chèque-cadeau », « certificat-cadeau » | recalled | [M] high / medium / medium |
| Redeem | « scannez-le pour **utiliser** votre carte-cadeau » | Shopify | [V1] |
| Redeem | « **Récupérer** » (button on a received gift) | Signal | [V1] |
| Redeem | « **Échanger** un bon » | Mullvad | [V1] |
| Redeem (platforms) | Apple « Utiliser une carte cadeau ou un code »; Google Play « Utiliser un code » | recalled | [M, medium-high] |
| Enter the code | « Saisir un code de bon »; « Le code du bon a déjà été utilisé. » | Mullvad | [V1] |
| QR | « Code QR » | Shopify | [V1] |
| QR (France) | « QR code » in everyday French retail; « code QR » is the Québec form | recalled | [M, high] / [M, medium-high] |
| Passphrase | label « Mot de passe »; the sentence says « clé LCP » | [Thorium fr](https://raw.githubusercontent.com/edrlab/thorium-reader/develop/src/resources/locales/fr.json) | [V1] |
| eBook | « livre numérique » — 169 lines against 3 for « livre(s) électronique(s) » | calibre | [V1] |
| eBook | « livre électronique » | Foliate, Open Library, calibre-web | [V2] |
| eBook | « e-book » and « Livre numérique »; « eBook » | Readest; BookWyrm | [V1] each |
| eBook (fr-CA) | « livre numérique » | calibre fr-CA | [V1] |
| e-reader | « liseuse » | calibre, KOReader, calibre-web, Open Library, Readest; calibre fr-CA | [V2] |
| Send to e-reader | « Envoyer à la liseuse » | calibre-web | [V1] |
| Official term | « livre numérique » (work), « liseuse » (device); FranceTerme, *Journal officiel* of 4 April 2012 | recalled; site blocked | [M] terms high, date medium-high |
| Statutory term | Loi n° 2011-590 « relative au prix du livre numérique » | [WIPO Lex](https://www.wipo.int/wipolex/en/legislation/details/13861), [Livres Hebdo](https://www.livreshebdo.fr/article/la-loi-sur-le-prix-en-vigueur) | [V2-sib] for the title |
| Québec (OQLF) | « livre numérique » preferred; « livre électronique », « livrel » as synonyms; « liseuse »; « carte-cadeau »; « code QR » | recalled; site blocked | [M, medium] |
| « livrel » | Appears in no French file fetched. Québec booksellers and libraries say « livres numériques » | corpus; recalled | [V1, negative]; [M, medium-high] |
| Retail nouns | Amazon.fr « Ebooks Kindle », « liseuse »; Kobo and Fnac « eBooks » / « Ebooks », « liseuses »; Decitre « ebook »; independents and libraries « livres numériques » | recalled | [M, medium-high] |
| Regional retail | BE/CH: same vocabulary, « bon cadeau » more frequent, Librel sells « livres numériques ». Québec: « livres numériques », « liseuse », « courriel »; « Échanger » and « Utiliser » both seen | recalled | [M, medium-low] / [M, medium] |

Reading: French has no settled verb for "redeem". « Récupérer » is the only one of the three used for a gift rather than a balance. « Échanger » also means returning goods in French retail [M, high]: avoid it.

### 2.3 English

Files: [Shopify en](https://raw.githubusercontent.com/Shopify/dawn/main/locales/en.default.json), [Signal en](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values/strings.xml), [calibre en-GB](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/en_GB.po), [LibreOffice en_GB.dic](https://raw.githubusercontent.com/LibreOffice/dictionaries/master/en/en_GB.dic).

| Function | Quoted string | Where | Tag |
|---|---|---|---|
| Gift one title (platforms) | Amazon "Buy for others"; Apple Books "Gift This Book"; Google Play Books "Buy as gift"; Barnes & Noble "Buy as Gift" | recalled (casing not certain) | [M] high / medium |
| Generic gift checkbox | "I want to send this as a gift" (key `recipient.form.checkbox`); fields "Recipient email", "Recipient name (optional)", "Message (optional)", "Send on (optional)" | Shopify | [V1] |
| Generic gift options | "I would like my order to be gift wrapped"; "If you'd like, you can add a note to the gift:" | [PrestaShop](https://raw.githubusercontent.com/PrestaShop/PrestaShop/develop/translations/default/ShopThemeCheckout.xlf) | [V1] |
| Generic gift options | "Gift Options"; "Do you have any gift items in your order?" | [Magento](https://raw.githubusercontent.com/magento/magento2/2.4-develop/app/code/Magento/GiftMessage/i18n/en_US.csv) | [V1] |
| Redeem | "QR code — scan to redeem gift card"; "Redeem"; "Redeem voucher" | Shopify; Signal; Mullvad | [V1] per file; the same verb in all three |
| Enter the code | "Enter voucher code"; "Voucher code has already been used." | Mullvad | [V1] |
| Passphrase | "Passphrase" (LCP prompt) | Thorium | [V1] |
| eBook | "e-book" (« An e-book shop. »); dictionary entry `e-book` | calibre en-GB; LibreOffice | [V1] each |
| e-reader | "Ebook reader"; "Send to eReader" | KOReader; calibre-web | [V1] each |
| Retail nouns | Amazon "Kindle eBooks", "Kindle E-readers"; Kobo "eBooks", "eReaders"; style guides (AP, Chicago, Oxford) "e-book", "e-reader"; trade press "ebook" | recalled | [M, high] |
| Stored-value words | "Redeem" everywhere; UK also "gift voucher", "book token" | recalled | [M, high] |
| Earlier products | "eBook gift cards" (Livrada, Enthrill — press term), "Kindle Cards", "digital book cards" (Walmart/Kobo), "print codes" (BookFunnel), "download cards" (Dropcards) | §3.5 | [V1-sib] / [V2-sib] |

Amazon avoids the word "gift" in the button ("for others"). Every other label uses the gift verb.

### 2.4 German

Files: [Shopify de](https://raw.githubusercontent.com/Shopify/dawn/main/locales/de.json), [Signal de](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-de/strings.xml), [calibre de](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/de.po), [LanguageTool de](https://raw.githubusercontent.com/languagetool-org/languagetool/master/languagetool-language-modules/de/src/main/resources/org/languagetool/rules/de/grammar.xml), [LibreOffice de_DE_frami.dic](https://raw.githubusercontent.com/LibreOffice/dictionaries/master/de/de_DE_frami.dic).

| Function | Quoted string | Where | Tag |
|---|---|---|---|
| Gift one title (retailer) | „Verschenken", „Artikel verschenken", „eBooks verschenken" | Hugendubel / Thalia, tolino alliance ([Hugendubel](https://www.hugendubel.de/de/shortcut/ebooksverschenken), [heise](https://www.heise.de/tipps-tricks/E-Books-verschenken-so-klappt-s-7476189.html), [test.de](https://www.test.de/E-Books-Nuetzliche-Tipps-und-Tricks-5391265-0/)) | [V2-sib] |
| Gift one title (platform) | Amazon.de „Für andere kaufen" | recalled | [M, medium-high] |
| Generic gift checkbox | „Ich möchte dies als Geschenk senden"; recipient = „Empfänger" | Shopify | [V1] |
| Gift-card noun | „Geschenkgutschein"; „Gutschein" | Shopify | split with Odoo |
| Gift-card noun | „Geschenkkarte" | Odoo | split with Shopify |
| Gift-card noun (retail) | Thalia/Hugendubel „Geschenkkarte"; Amazon.de „Gutschein" | recalled | [M, high] |
| Redeem | „Scannen, um Gutschein **einzulösen**"; „Einlösen"; „Gutschein einlösen" | Shopify; Signal; Mullvad | [V2], stable |
| Enter the code | „Gutscheincode eingeben"; „Der Gutscheincode wurde bereits verwendet." | Mullvad | [V1] |
| QR | „QR-Code" | Shopify | [V1] |
| Passphrase | „Passwort" | Thorium | [V1] |
| eBook | „eBook" („eBook-Betrachter", „eBook-Händler") | calibre | [V1] |
| eBook | „E-Book" | KOReader, calibre-web, Open Library | [V2] |
| e-reader | „eBook-Reader"; „E-Book-Reader"; „E-Reader" | calibre; KOReader; calibre-web, Open Library | [V1] per file |
| Send to e-reader | „An E-Reader senden" | calibre-web | [V1] |
| Spelling rule | Rule group `E_BOOK`: „EBook" → „E-Book"; „EBook Reader" → „E-Book-Reader"; „E Books" → „E-Books" | LanguageTool | [V1] |
| Spelling dictionary | lists `eBook/S` | LibreOffice | [V1] |
| Official | Duden: „E-Book", „E-Book-Reader", „E-Reader" | recalled; site blocked | [M, high] |
| Retail nouns | Thalia, Hugendubel, ebook.de, Amazon.de „eBook"; tolino „eReader"; press and Börsenverein „E-Book"; „Gutschein einlösen", „Geschenkkarte einlösen" | recalled | [M, high] |

Both spellings ship. The split follows the speaker: shops write „eBook", the press and the dictionary write „E-Book". German compounds hyphenate after the noun („E-Book-Reader").

### 2.5 Spanish

Files: [Shopify es](https://raw.githubusercontent.com/Shopify/dawn/main/locales/es.json), [Signal es](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-es/strings.xml), [calibre es](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/es.po), [Thorium es](https://raw.githubusercontent.com/edrlab/thorium-reader/develop/src/resources/locales/es.json).

| Function | Quoted string | Where | Tag |
|---|---|---|---|
| Gift one title | Amazon.es « Comprar para otros »; Casa del Libro: not known | recalled | [M, medium] |
| Generic gift checkbox | « Quiero enviar esto como un regalo. »; recipient = « persona destinataria » | Shopify | [V1] |
| Gift-card noun | « tarjeta de regalo » (Shopify; Odoo es and es_419) | Shopify, Odoo | [V2] |
| Gift-card noun (retail) | Spain: « tarjeta regalo »; Amazon.es « cheque regalo » | recalled | [M, high] |
| Redeem | « escanea para **canjear** »; « Canjear »; « Canjear cupón » | Shopify; Signal; Mullvad | [V2], stable |
| Redeem (retail) | Amazon.es « Canjear un cheque regalo » | recalled | [M, medium-high] |
| Enter the code | « Escriba el código del cupón »; « El código del cupón ya se ha usado. » | Mullvad | [V1] |
| QR | « Código QR » | Shopify | [V1] |
| Passphrase | label « Santo y seña »; the sentence says « contraseña » | Thorium | [V1] |
| eBook | « libro electrónico » | calibre (also es_MX), Foliate, Open Library | [V1] per file; three corpora agree |
| eBook | « ebooks » | calibre-web | [V1] |
| e-reader | « lector de libros electrónicos » | calibre, KOReader | [V2] |
| e-reader | « lectores electrónicos »; « eReader » | Open Library; calibre-web | [V1] each |
| Send to e-reader | « Enviar al eReader » | calibre-web | [V1] |
| Official | Fundéu / RAE: « libro electrónico » (also « libro digital »); « lector de libros electrónicos » | recalled | [M, medium-high] |
| Retail nouns | Casa del Libro, Amazon.es, El Corte Inglés "eBooks"; "eReader" | recalled | [M, medium-high] |

### 2.6 Italian

Files: [Shopify it](https://raw.githubusercontent.com/Shopify/dawn/main/locales/it.json), [Signal it](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-it/strings.xml), [calibre it](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/it.po), [LibreOffice it_IT.dic](https://raw.githubusercontent.com/LibreOffice/dictionaries/master/it_IT/it_IT.dic).

| Function | Quoted string | Where | Tag |
|---|---|---|---|
| Gift one title | Amazon.it « Acquista per altri »; IBS « Regala un eBook » (e-mail and code) | recalled | [M] medium / low |
| Generic gift checkbox | « Voglio inviarlo come regalo »; recipient = « destinatario » | Shopify | [V1] |
| Gift-card noun | « buono regalo » | Shopify | split with Odoo |
| Gift-card noun | « Carta regalo » | Odoo | split with Shopify |
| Gift-card noun (retail) | Amazon.it « Buoni Regalo »; bookshops « Gift Card » | recalled | [M] high / medium |
| Redeem | « scansiona per **riscattare** »; « Riscatta »; « Riscatta voucher » | Shopify; Signal; Mullvad | [V2], stable |
| Redeem (platforms) | Apple « Riscatta »; Google Play « Utilizza codice » | recalled | [M, medium] |
| Enter the code | « Inserisci codice voucher »; « Il codice voucher è già stato utilizzato. » | Mullvad | [V1] |
| QR | « Codice QR » | Shopify | [V1] |
| Passphrase | « Passphrase » | Thorium | [V1] |
| eBook | « e-book » | calibre, Foliate | [V1] per file |
| eBook | « eBook » / « Ebook » | Open Library, calibre-web | [V1] per file |
| e-reader | « lettore di e-book »; « Lettore ebook »; « e-reader »; « eReader » | calibre; KOReader; Open Library; calibre-web | [V1] each |
| Send to e-reader | « Invia all'eReader » | calibre-web | [V1] |
| Spelling dictionary | lists `ebook`; no `e-book` entry | LibreOffice | [V1] |
| Official | Treccani, Crusca: "e-book" and "ebook" both recorded; « libro elettronico / digitale » | recalled | [M, medium] |
| Retail nouns | IBS, Feltrinelli, Mondadori Store, Kobo "eBook"; "eReader" | recalled | [M, medium] |

### 2.7 Dutch

Files: [Shopify nl](https://raw.githubusercontent.com/Shopify/dawn/main/locales/nl.json), [Signal nl](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-nl/strings.xml), [calibre nl](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/nl.po), [Thorium nl](https://raw.githubusercontent.com/edrlab/thorium-reader/develop/src/resources/locales/nl.json), [OpenTaal](https://raw.githubusercontent.com/OpenTaal/opentaal-wordlist/master/wordlist.txt), [LanguageTool nl](https://raw.githubusercontent.com/languagetool-org/languagetool/master/languagetool-language-modules/nl/src/main/resources/org/languagetool/rules/nl/grammar.xml).

| Function | Quoted string | Where | Tag |
|---|---|---|---|
| Gift one title | "ebook cadeau geven / cadeau doen"; bol.com's own label: not known | recalled | [M] |
| Generic gift checkbox | "Ik wil dit als cadeau sturen"; recipient = "ontvanger" | Shopify | [V1] |
| Gift-card noun | "cadeaubon" | Shopify, Odoo | [V2] |
| Gift-card noun | word list has `cadeaubon`, `cadeaukaart`, `boekenbon` | OpenTaal | [V1] |
| Gift-card noun (retail) | bol.com "cadeaukaart" | recalled | [M, high] |
| Redeem | "scannen om cadeaubon te **verzilveren**" | Shopify | [V1] |
| Redeem | "**In ontvangst nemen**" (button on a received gift) | Signal | [V1] |
| Redeem | "Voucher **inwisselen**" | Mullvad | [V1] |
| Redeem (retail) | "verzilveren" (bol.com); "inwisselen" (Apple, Google, Amazon.nl) | recalled | [M, medium] |
| Enter the code | "Vouchercode invoeren"; "Vouchercode is al gebruikt." | Mullvad | [V1] |
| QR | "QR-code" | Shopify | [V1] |
| Passphrase | "Wachtwoordzin" | Thorium | [V1] |
| eBook | "e-boek" | calibre, calibre-web, Foliate | [V2] |
| e-reader | "e-reader"; "Ebooklezer" | calibre; KOReader | [V1] each |
| Spelling list | contains `e-book`, `e-books`, `e-boek`, `e-boeken`, `e-bookreader`, `e-reader`; no `ebook`, no `ereader` | OpenTaal | [V1] |
| Spelling rule | Rule group `EREADER`: "ereader" → "e-reader" (« Na de 'e' komt een streepje, net als bij 'e-mail' »); "e-Reader" → lower case | LanguageTool | [V1] |
| Official | Taalunie word list: "e-book", "e-boek", "e-reader" | recalled | [M, high] |
| Retail nouns | bol.com "ebooks", "e-readers"; Kobo "eBooks" | recalled | [M, medium] |

Dutch has three verbs for "redeem" in three products. Signal avoided both voucher verbs for a gift. Hyphenated lower case is the norm for the nouns [V2]; free-software translators write "e-boek", retail and the press write "e-book" [M].

### 2.8 Portuguese

Files: [Shopify pt-PT](https://raw.githubusercontent.com/Shopify/dawn/main/locales/pt-PT.json), [Shopify pt-BR](https://raw.githubusercontent.com/Shopify/dawn/main/locales/pt-BR.json), [Signal pt](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-pt/strings.xml), [Signal pt-BR](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-pt-rBR/strings.xml), [calibre pt](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/pt.po), [calibre pt-BR](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/pt_BR.po).

| Function | pt-PT | pt-BR | Tag |
|---|---|---|---|
| Generic gift checkbox (Shopify) | « Quero enviar isto como um presente » | « Quero enviar como presente » | [V1] each; recipient = « destinatário » |
| Gift-card noun | « cartão de oferta » (Shopify); « Cartão de presente » (Odoo `pt`, which looks Brazilian-influenced) | « cartão-presente » (Shopify); « Cartão de presente », « Cartão-presente » (Odoo) | split / [V2] |
| Gift-card noun (retail) | Fnac.pt « Cartão Oferta »; Wook/Bertrand « cheque-prenda » | Amazon.com.br « vale-presente » | [M] medium / medium-high |
| Redeem | « digitalizar para **resgatar** » (Shopify); « Resgatar » (Signal); « **Reclamar** voucher » (Mullvad) | « faça a leitura para **resgatar** »; « Resgatar » | mostly stable [V2] / stable [V2] |
| Enter the code (Mullvad `pt`) | « Introduza o código do voucher »; « O código do voucher já foi utilizado. » | — | [V1] |
| QR | « Código QR » | « Código QR » | [V1] |
| Passphrase (Thorium) | « Frase-passe » | « Senha » | [V1] |
| eBook | « livro digital » (calibre); « ebooks » (calibre-web) | « e-book » (calibre, Foliate, calibre-web); « livro eletrônico » (calibre, KOReader) | [V1] each / [V2] |
| e-reader | « leitor de livros digitais » (calibre, KOReader); « dispositivo de leitura » (calibre-web) | « leitor de e-books »; « leitor de livros eletrônicos »; « E-Reader » | [V2] / [V1] each |
| Send to e-reader (calibre-web) | « Enviar para dispositivo de leitura » | « Enviar para E-Reader » | [V1] |
| Retail nouns | Wook, Bertrand, Fnac.pt "eBooks"; also « Usar » for redeem | Amazon.com.br "eBooks Kindle"; press "e-book"; trade body « livro digital » | [M, medium] |

Portuguese retail in Portugal prefers « oferta / oferecer » for gifting [M, medium], while Shopify uses « presente » in both files [V1].

### 2.9 Cross-language tables (secondary languages included)

**Gifting checkbox, Shopify key `recipient.form.checkbox`** [V1 per row]:

| Lang | Checkbox label | "Recipient" |
|---|---|---|
| [pl](https://raw.githubusercontent.com/Shopify/dawn/main/locales/pl.json) | Chcę to wysłać jako prezent | odbiorca |
| [sv](https://raw.githubusercontent.com/Shopify/dawn/main/locales/sv.json) | Jag vill skicka detta som en gåva | mottagare |
| [da](https://raw.githubusercontent.com/Shopify/dawn/main/locales/da.json) | Jeg vil sende som en gave | modtager |
| [nb](https://raw.githubusercontent.com/Shopify/dawn/main/locales/nb.json) | Jeg vil sende dette som en gave | mottaker |
| [fi](https://raw.githubusercontent.com/Shopify/dawn/main/locales/fi.json) | Haluan lähettää tämän lahjaksi | vastaanottaja |
| [cs](https://raw.githubusercontent.com/Shopify/dawn/main/locales/cs.json) | Chci zboží poslat jako dárek | příjemce |
| [el](https://raw.githubusercontent.com/Shopify/dawn/main/locales/el.json) | Θέλω να το στείλω ως δώρο | παραλήπτης |

The form asks for recipient e-mail, name, message and send date — the same shape as the Decitre flow [V1-sib].

**"Gift card"** (the word to avoid, and a search keyword):

| Lang | Shopify | Odoo 17 | Agreement | Retail, recalled [M] |
|---|---|---|---|---|
| pl | karta prezentowa | Karta podarunkowa | split | Empik « karta prezentowa »; Amazon.pl « karty podarunkowe » (medium) |
| sv | presentkort | Presentkort | [V2] | same (high) |
| da | gavekort | Gavekort | [V2] | same (high) |
| nb | gavekort | « Her er ditt gavekort » | [V2] | same (high) |
| fi | lahjakortti | Lahjakortti | [V2] | same (high) |
| cs | dárková karta | Dárkový poukaz | split | « dárkový poukaz » dominant in bookshops (medium) |
| ro | — | Card cadou | [V1] | « card cadou », « voucher cadou » (medium) |
| el | δωροκάρτα | — | [V1] | « δωροκάρτα », « δωροεπιταγή » (medium) |
| ca | — | Targeta Regal; Targetes de regal | [V1] | « targeta regal », « val regal » (medium-low) |

"Gift card" is stable in fr, es, nl, sv, da, nb, fi and split in de, it, pt, pl, cs [V2].

**"Redeem"** in three shipped products:

| Lang | Shopify | Signal | Mullvad | Verdict |
|---|---|---|---|---|
| [pl](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-pl/strings.xml) | zeskanuj, aby **wykorzystać** | **Odbierz** | **Zrealizuj** kupon | unstable, 3 verbs |
| [sv](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-sv/strings.xml) | skanna för att **lösa in** | Lös in | Lös in kupong | stable [V2] |
| [da](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-da/strings.xml) | scan for at **indløse** | Indløs | Indløs kupon | stable [V2] |
| [nb](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-nb/strings.xml) | skann for å **løse inn** | Løs inn | Løs inn kupong | stable [V2] |
| [fi](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-fi/strings.xml) | **lunasta** lahjakortti skannaamalla | Lunasta | Lunasta kuponki | stable [V2] |
| [cs](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-cs/strings.xml) | po naskenování můžete **uplatnit** | Uplatnit | — | stable [V2] |
| [ro](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-ro/strings.xml) | — | **Revendică**; past « Ai **răscumpărat** o insignă » | — | unstable inside one app [V1] |
| [el](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-el/strings.xml) | σαρώστε για να **εξαργυρώσετε** | Εξαργύρωση | — | stable [V2] |
| [ca](https://raw.githubusercontent.com/signalapp/Signal-Android/main/app/src/main/res/values-ca/strings.xml) | — | Bescanvia | — | [V1] |

Summary across all 16 languages: three products agree in de, es, it, sv, da, nb, fi; two agree in cs, el, pt-BR; fr, nl and pl have three verbs each; ro varies inside one app [V2].

**Entering the code, Mullvad** [V1 per language]:

| Lang | "Enter voucher code" | "Voucher code has already been used." |
|---|---|---|
| pl | Wprowadź kod kuponu | Kod z tego kuponu został już użyty. |
| sv | Ange kupongkod | Kupongkoden har redan använts. |
| da | Indtast kuponkode | Kuponkode er allerede brugt. |
| nb | Skriv inn kupongkode | Kupongkoden er allerede brukt. |
| fi | Syötä kuponkikoodi | Kuponkikoodi on jo käytetty. |

"Voucher" itself is rendered *bon* (fr), *Gutschein* (de), *cupón* (es), *voucher* (it, nl, pt), *kupon / kupong / kuponki* (pl, sv, da, nb, fi): a discount-coupon word in half the languages.

**Scan verb** (same Shopify string) [V1]: *scannez, scannen, escanea, scansiona, zeskanuj, skanna, scan, skann, σαρώστε*; pt-BR *faça a leitura*, pt-PT *digitalizar*.

**"QR code"** (Shopify) [V1]: « Code QR » (fr), „QR-Code" (de), "Código QR" (es, pt), "Codice QR" (it), "QR-code" (nl), "Kod QR" (pl), "QR-kod" (sv), "QR-kode" (da, nb), "QR-koodi" (fi), "QR kód" (cs), "Κωδικός QR" (el).

**"Passphrase"** (Thorium, LCP prompt) [V1]: *Mot de passe* (fr), *Passwort* (de), *Santo y seña* (es), *Passphrase* (it), *Wachtwoordzin* (nl), *Frase-passe* (pt-PT), *Senha* (pt-BR), *Hasło dostępu* (pl), *Lösenord* (sv), *Nøgleord* (da), *Salasana* (fi), *Heslová fráze* (cs), *Συνθηματική φράση* (el), *Contrassenya* (ca). If a card ever carries an LCP passphrase, print it as "code".

**eBook and e-reader in reading software.** C = calibre, K = KOReader, W = calibre-web, F = Foliate, O = Open Library. The language code links to the calibre file.

| Lang | eBook | e-reader (device) |
|---|---|---|
| [pl](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/pl.po) | e-book, declined: e-booka, e-booki, e-booków (C, W, O) [V2]; store = „E-księgarnia" (C) | czytnik (C, W); czytnik e-booków (K, O) [V2] |
| [sv](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/sv.po) | e-bok, e-böcker (C, F, W) [V2] | läsenhet (C); E-boksläsare (K); e-läsare (W) |
| [da](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/da.po) | e-bog, e-bøger (C) | e-boglæser (C) |
| [nb](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/nb.po) | e-bok, e-bøker (C) | e-bokleser (C); Ebokleser (K) |
| [fi](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/fi.po) | e-kirja (C, W) [V2] | lukulaite (C); E-kirjan lukija (K) |
| [cs](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/cs.po) | e-kniha (C, O); elektronická kniha (F); "eknih" (W) | čtečka e-knih (C, K, O) [V2] |
| [ro](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/ro.po) | carte electronică (C); E-book (C); eBook, E-books (O) | cititor de cărți electronice (C); Cititor e-book (K); e-readere (O) |
| [el](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/el.po) | ηλεκτρονικό βιβλίο (C, K, F) [V2]; ψηφιακό βιβλίο (C); eBook (W) | αναγνώστης ψηφιακών βιβλίων (C); αναγνώστης ηλεκτρονικών βιβλίων (K) |
| [ca](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/ca.po) | llibre electrònic (C) | lector de llibres electrònics (C, K) [V2] |

Other calibre files read for the primary languages: [es](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/es.po), [it](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/it.po), [nl](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/nl.po), [de](https://raw.githubusercontent.com/kovidgoyal/calibre-translations/master/calibre/de.po). "Send to eReader" in Swedish (calibre-web): « Skicka till e-läsare » [V1].

**Spelling dictionaries** [V1 each]: Danish `e-bog` ([da_DK.dic](https://raw.githubusercontent.com/LibreOffice/dictionaries/master/da_DK/da_DK.dic)); Norwegian `e-bok` and `lesebrett` ([nb_NO.dic](https://raw.githubusercontent.com/LibreOffice/dictionaries/master/no/nb_NO.dic)); Czech `e-kniha` and `čtečka` ([cs_CZ.dic](https://raw.githubusercontent.com/LibreOffice/dictionaries/master/cs_CZ/cs_CZ.dic)); Catalan both `e-book` and `ebook` ([ca.dic](https://raw.githubusercontent.com/LibreOffice/dictionaries/master/ca/dictionaries/ca.dic)). A spelling dictionary shows that a form is accepted, not that it is preferred. The LanguageTool rule files for French, Spanish, Catalan, Portuguese, Italian and Polish contain no rule on these words.

**Official terminology and retail usage for the secondary languages** (all [M]; official sites were blocked):

| Area | Official / dictionary | Retail, recalled |
|---|---|---|
| Sweden, Denmark, Norway, Finland | "e-bok" + « läsplatta »; "e-bog"; "e-bok" + « lesebrett »; "e-kirja" (Kotus also « sähkökirja ») + « lukulaite » (medium-high) | Adlibris, Bokus « E-böcker », « Läsplattor »; Saxo « E-bøger »; ARK, Norli « E-bøker », « Lesebrett »; Suomalainen Kirjakauppa « E-kirjat », « Lukulaitteet »; redeem « Lös in », « Indløs », « Løs inn », « Lunasta » (medium-high) |
| Poland | PWN "e-book" (medium) | Empik, Woblink, Publio, Legimi « ebooki » / « e-booki »; « czytniki (ebooków) »; « Zrealizuj », « Wykorzystaj kod », « Aktywuj kod » (medium) |
| Czechia | ÚJČ "e-kniha" (medium) | Palmknihy, Kosmas, Alza « e-knihy », « čtečky e-knih »; « Uplatnit » (medium) |
| Romania, Greece, Catalonia | DOOM3 "e-book" (medium-low); TERMCAT « llibre electrònic », « lector de llibres electrònics » (medium) | Elefant, Libris, Cărturești « eBook » / « ebook »; Public « eBooks »; Catalan shops « ebooks » / « llibres electrònics »; « Folosește / Aplică », « Εξαργύρωση », « Bescanviar » (low-medium) |

**Pattern** [M, medium-high]: in most markets the large retailers write an English-derived trade form ("eBook", "ebook", « ebooki »), while dictionaries, public bodies, libraries and free-software translators prefer a native or hyphenated form. Nordic, Czech, Finnish and Dutch retail use the native word (*e-bok, e-bog, e-kirja, e-kniha, e-book*). In Scandinavia the device noun differs between software (*läsenhet, e-boglæser, e-bokleser* [V1 each]) and retail (*läsplatta, e-bogslæser, lesebrett* [M]).

---

## 3. Naming, trademark and descriptiveness

All of §3.2 to §3.4 is [M]. The legal rules are well established; their application to this name is analysis. **No register (INPI, EUIPO/TMview, DPMA, BOIP, OEPM, UIBM, UKIPO) and no WHOIS was searched.**

### 3.1 Candidate names

| Candidate | Clarity | Word-mark chance | Stored-value connotation | Verdict |
|---|---|---|---|---|
| **eBook Cadeau** (fr, nl) | High for "an eBook as a present". Says nothing about the printed object. Can be read as "free eBook" | Low: descriptive in French and Dutch | None | Keep as project name and option name |
| **Offrir un ebook / Offrir cet ebook** (fr) | Highest: it is the action. Decitre already uses it [V2-sib] | None (plain language) | None | Use for the option label, page titles, H1 |
| **Livre numérique à offrir** (fr) | High, formal; fits Québec and public-sector style | None | None | Alternate; fr-CA variant |
| **Livre cadeau (numérique)** | Confusing: « livre cadeau » is the retail category of printed gift books [M, high]. Same trap: „Geschenkbuch", "cadeauboek", « libro regalo », "gift book" | Low | None | Reject |
| **Carte cadeau ebook / eBook-Gutschein / cadeaubon** | Clear but wrong: implies a balance | Low | Strong | Reject |
| **Chèque Lire / Chèque Culture / book token / Boekenbon** | Existing scheme names (§4.1) | — | Strong | Reject |
| **Jaquette cadeau** (fr) | Medium: names the object; needs one line of explanation | Better, still partly descriptive | None | Alternate name for the printed object |

### 3.2 Why "eBook Cadeau" is a weak word mark [M]

- **France.** Code de la propriété intellectuelle, art. L. 711-2 (as rewritten by ordonnance n° 2019-1169): a sign is refused if it lacks distinctive character, if it consists exclusively of elements that designate a characteristic of the goods or services (kind, intended purpose), or of terms customary in trade. "eBook cadeau" for a service of giving eBooks falls under the second ground on its face.
- **EU.** Regulation (EU) 2017/1001, art. 7(1)(b) and (c), and art. 7(2): a ground that exists in only part of the Union is enough. A term descriptive for French-speaking (FR, BE, LU) and Dutch-speaking (NL, BE) consumers is refused for the whole EU.
- **Case law on combinations.** CJEU C-363/99 *Postkantoor* and C-265/00 *Biomild* (2004): a combination of descriptive elements is itself descriptive unless the whole creates an impression clearly different from the sum. C-191/01 P *Doublemint* (2003): one descriptive meaning among several is enough. "eBook" + "cadeau" is an ordinary French and Dutch construction.
- **Acquired distinctiveness** (art. 7(3) EUTMR; last paragraph of L. 711-2) needs heavy, long use. French reference: the "vente-privee.com" litigation [M, medium on the procedure].
- **Risk of infringing someone else** is low for the same reason. A prior figurative or composite mark, company name or domain name could still exist. Not checked. The researcher recalls no service or mark named "eBook Cadeau", „Geschenk-eBook" or « Regala un eBook » [M, low — an absence of recollection, not a search result].

### 3.3 What can be protected, and what it costs

- A **figurative mark** (logo containing the words) protects the graphic, not the words. Art. L. 713-6 CPI and art. 14(1)(b) EUTMR let others keep using descriptive terms in honest practice [M].
- A **distinctive house brand** above the generic label, if protection is wanted. Directions only, none cleared: bookbinding and bibliophile vocabulary (*jaquette*, *dos*, *ex-dono* — the inscription recording that a book was given). Each needs an INPI, TMview and domain check first.
- Fees: INPI EUR 190 for one class, EUR 40 per extra class [V2-sib] ([LegalPlace](https://www.legalplace.fr/guides/prix-depot-marque-inpi/), [Legalstart](https://www.legalstart.fr/fiches-pratiques/depot-marque/cout-depot-marque/)); EUIPO about EUR 850 for one class [V1-sib, sources conflict] ([Elly Assurance](https://www.elly-assurance.fr/actualites/inpi/)). `12-market-and-business-inputs.md` carries the same figures and the same advice: choose a distinctive name before paying the INPI fee.

**Recommended structure.** Two levels: (1) generic, localised option labels and card vocabulary, free for every reseller to use; (2) optionally, a logo or house brand for the standalone site and the B2B offer. Do not budget for a word mark on "eBook Cadeau".

### 3.4 Other people's marks, and language law [M]

- **Reading-system names** (Kindle, Kobo, tolino, Vivlio, PocketBook): naming them in text to state compatibility is allowed (art. 14(1)(c) EUTMR). No logos, and never inside the product name.
- **"QR Code"** is a registered trademark of DENSO WAVE INCORPORATED; the usual practice is an attribution line when the term is printed. Printing « Scanner ce code » avoids the term altogether.
- **Loi Toubon** (n° 94-665): French is required in consumer-facing presentation; brand names are tolerated. Pairing "eBook" (in the name) with « livre numérique » (in the sentence) covers it. Québec's Charter rules are stricter: use « livre numérique » and « code QR » there [M, medium-high].

### 3.5 What earlier products called the object

| Product | Name for the object | Status | Source |
|---|---|---|---|
| Livrada (US), Enthrill (CA) | "eBook gift cards" (press term) | Closed 2016–2017 | [V2-sib] ([Publishers Weekly](https://publishersweekly.com/pw/print/20120827/53710-livrada-tries-its-luck-with-e-book-gift-cards.html), [The Digital Reader](https://the-digital-reader.com/ebook-startup-livrada-shut/)) |
| Amazon, 2016 | "Kindle Cards" | Test, not rolled out | [V2-sib] ([Shelf Awareness](https://shelf-awareness.com/theshelf/2016-01-26/amazon_testing_title-specific_kindle_gift_cards_.html)) |
| Walmart eBooks by Rakuten Kobo, 2018 | "digital book cards" | Ended by 2024 | [V1-sib] ([MobileRead](https://www.mobileread.com/forums/showthread.php?t=363478)) |
| BookFunnel | "print codes" | Active | [V2-sib] ([BookFunnel](https://authors.bookfunnel.com/help/print-codes/)) |
| Dropcards, Distro Print | "download cards", "drop cards" | Active | [V1-sib] ([Dropcards](https://www.dropcards.com/pricing/)) |
| Bookwire (DE) | „Download-Codes" | Active | [V1-sib] ([Bookwire](https://bookwire.net/faq-download-codes)) |
| Decitre / Furet du Nord | « Offrir un ebook », button « Offrir » — e-mail only | Active | [V2-sib] (§2.2) |
| Hugendubel / Thalia | „Verschenken", „eBooks verschenken" — e-mail | Active | [V2-sib] (§2.4) |
| Amazon.fr | « Acheter pour d'autres personnes » | Active | [V2-sib] (§2.2) |

Two observations [analysis]. Every earlier physical product was named "card" plus "code"; none used the book-jacket idea, so the object deserves its own word per language (§1.3, "name for the printed object"). The living retailer flows are all named with the verb of giving, which is the vocabulary buyers already meet.

### 3.6 Search intent

**No search-volume data was obtained**, in any language. From memory only [M, low]:

- Intent sits on the verb: « offrir un ebook », „eBook verschenken", "ebook cadeau doen", « regalar un ebook », « regalare un ebook », "gift an ebook".
- The noun phrases « ebook cadeau », « ebook offert », « eBook de regalo », « eBook in regalo », „eBook geschenkt" are the stock wording of free lead-magnet eBooks, which pollutes results [M, high for the phenomenon].
- French users search « ebook » far more than « livre numérique ».

### 3.7 Domains and URL vocabulary [M; no availability check]

- A descriptive domain is hard to defend: AFNIC's Syreli procedure (.fr) and the UDRP (.com) both require a prior right. Register the obvious variants early (hyphenated and not; .fr, .eu, .com) instead of relying on disputes.
- Exact-match descriptive domains have brought little ranking benefit since Google's 2012 "EMD" update. Rank with localised how-to pages on the verb queries.
- One domain with language paths, not one descriptive domain per country.
- Path vocabulary: `/cadeau` (fr, nl), `/geschenk` (de), `/regalo` (es, it), `/gift` (en), `/oferta` (pt).
- The typed-code host printed on the card must be short, with no hyphen, accent, digit or look-alike character, because it may be typed on an e-reader keyboard.
- In reseller mode the QR code and the typed URL sit on the **reseller's own domain** (for example `{reseller}/cadeau`). The recipient knows that name, and it lowers QR-phishing suspicion.

---

## 4. Legal vocabulary traps: voucher versus a gift of one title

The product is one identified, prepaid title from one seller, with no balance. The words below describe something else. Statutes are cited from memory; none was re-read.

### 4.1 Words that imply stored value

| Term(s) | Where | What the public and the law understand | Why it is a trap here |
|---|---|---|---|
| carte cadeau, e-carte cadeau, chèque cadeau, bon d'achat | FR | A prepaid balance spendable on anything at one chain or network; issued by the seller or by specialised issuers (Kadéos/Edenred, Cadhoc and Chèque Lire/Up, Tir Groupé/Pluxee, Illicado) [M, medium-high] | Network instruments rely on the "limited network" exclusion of PSD2 art. 3(k), transposed in the Code monétaire et financier, with a notification duty above EUR 1 million of transactions a year [M; article numbers low]. Raises expectations on validity, balance and refund |
| chèque-lire, chèque culture | FR | Cultural vouchers bought by works councils, with a social-contribution exemption [M, medium-high] | « Chèque Lire » is believed to be a brand of Groupe Up [M, medium]. Wrong product class, someone else's name |
| bon cadeau, coffret cadeau | FR, BE, CH | A voucher for a service or an experience | Implies expiry and exchange rules |
| Gutschein, Geschenkgutschein, Geschenkkarte, Büchergutschein | DE, AT, CH | A value voucher. Limitation period of three years from the end of the year of issue (§§ 195, 199 BGB); shorter terms in standard conditions are often void [M] | The reader expects a value and three years. Multi-seller use raises payment-services questions (§ 2 ZAG) [M, medium-high]. VAT law has its own „Einzweck-/Mehrzweck-Gutschein" (§ 3 Abs. 13–15 UStG) |
| cadeaubon, cadeaukaart, boekenbon | NL, BE | A value voucher. Dutch law: minimum validity of two years for gift vouchers, since 2022 [M]. Belgium: one year, from memory [M, low]. "Boekenbon" is the national book-voucher scheme [M, medium] | Validity rule; scheme confusion |
| tarjeta regalo, cheque regalo, bono | ES | A value card (El Corte Inglés, Amazon « cheques regalo »); « Bono Cultural Joven » is a state scheme [M, medium] | Balance connotation; confusion with the state scheme |
| buono regalo, carta regalo, gift card, buono | IT | A value card (Amazon.it « Buoni Regalo »); « buono » is also the unit of the state culture cards (18app / Carta della Cultura Giovani, Carta del Docente) [M, medium] | Same |
| book token, gift card, gift voucher, eGift | UK, IE | "National Book Tokens" is the Booksellers Association's scheme [M, medium-high]. Ireland: five-year minimum validity (Consumer Protection (Gift Vouchers) Act 2019) [M] | Scheme confusion; statutory validity |
| cartão oferta, cheque-prenda, vale-presente | PT, BR | Value cards (Fnac.pt, Wook/Bertrand, Amazon.com.br) [M, medium] | Balance connotation |

Two further families to avoid, for different reasons:

- **Print gift-book categories**: « livre cadeau », „Geschenkbuch", "cadeauboek", « libro regalo », "gift book" [M, high].
- **Wording that reads as "free"**: « eBook offert », „Geschenk-eBook", « eBook de regalo », « eBook in regalo / in omaggio » [M].

### 4.2 "Voucher" in tax law is a different question from "voucher" on the card

The sibling commerce note and the naming note pull in opposite directions, and both are right about their own layer.

| Layer | What the commerce note says | What the naming note says | Decision |
|---|---|---|---|
| VAT classification | A code for one named eBook is a voucher under Directive (EU) 2016/1065 (VAT Directive arts. 30a, 30b, 73a). Single-purpose (VAT due at sale) if redemption is restricted to one Member State; arguably multi-purpose if redeemable EU-wide. A third reading treats the sale as an immediate supply to the buyer with deferred download [M, medium] | Calling it a voucher "invites the multi-purpose reading"; whether a single-title prepaid code is a voucher at all "needs a lawyer" | Classification follows substance, not labels — both notes say so. The VAT adviser decides. If the answer is "single-purpose voucher", the terms of sale and the invoice say so in the statutory words (« bon à usage unique », CGI art. 256 ter [M, medium]; „Einzweck-Gutschein") |
| Card and screens | Suggests the legal line « Bon pour un exemplaire numérique de "Titre"… » | Bans « bon », "voucher", „Gutschein" on the object and the option | **The card line becomes « {Titre}, {Auteur}. Un exemplaire numérique, déjà payé : rien à régler. »** (the UX note's wording), followed by « Usage unique. Sans date limite. » The information required is the same; the stored-value word is gone |
| Consumer validity statutes (DE 3 years, NL 2 years, IE 5 years) | A fully paid, named eBook is closer to a deferred delivery than to stored value; recommends no expiry, or at least five years [M] | The stored-value words bring these rules to the reader's mind | No expiry by default, so no validity wording at all. If a tenant sets an expiry, the card says « À récupérer avant le {date} », not « Valable jusqu'au » |
| E-money / payment services | A title-specific code stores no monetary value and is accepted only by its issuer: not e-money [M, high] | Words such as « carte cadeau » would blur that | Keep the substance and the wording aligned (§4.3) |

### 4.3 Design rules that keep wording and substance aligned [analysis]

1. One code = one title at one seller. No face value, no balance, no change, no cash alternative. A printed price also ages badly: the US cards failed partly because the printed price diverged from the live eBook price [V1-sib] ([GeekWire](https://www.geekwire.com/2016/amazons-other-physical-retail-test-testing-the-kindle-e-book-kiosk-at-the-local-drug-store/)).
2. No disclaimer such as « ceci n'est pas une carte cadeau ». It only plants the idea.
3. Fixed book price laws cover eBooks in France (loi n° 2011-590) and Germany (BuchPrG) [M, high]. The eBook line stays at the publisher's price; the printed card is a separate line at the standard VAT rate [M]. The sub-option label therefore names a service (« Carte imprimée et envoyée par la poste »), never a bundle.
4. In standalone mode the service sells the card, not the eBook. The naming must not suggest a code "with value" redeemable at several shops.

---

## 5. Machine-readable appendix

### 5.1 Recommended strings (canonical block)

Primary choices only. `pt` is included because `Locale::ALL` contains it; treat it as provisional. Placeholder: `{name}` = the giver's first name. ` ` is the no-break space French typography requires before `:` and `!`. Apostrophes are typographic (U+2019). Every string is [M].

```toml
# eBook Cadeau — recommended UI strings, primary choices only.
# Source: docs/research/06-terminology.md (research date 2026-11-12).
# Not verified at any bookseller site; needs native-speaker review.

[fr]  # fr-FR; also fr-BE, fr-CH, fr-LU
product_name = "eBook Cadeau"
checkout_option = "Offrir cet ebook"
checkout_option_help = "Une carte en forme de livre, à imprimer chez soi ou envoyée par la poste. La personne qui la reçoit scanne le code et lit le livre sur liseuse, tablette ou téléphone."
card_headline = "Ce livre est un cadeau"
card_from = "De la part de {name}"
card_steps_1 = "Scanner ce code avec l’appareil photo d’un téléphone."
card_steps_2 = "Choisir où lire : liseuse, tablette ou téléphone."
card_steps_3 = "Le livre arrive. Bonne lecture !"
redeem_cta = "Récupérer mon livre"
ebook_word = "livre numérique"
ereader_word = "liseuse"
code_label = "Code"

[en]  # en-GB, en-IE
product_name = "eBook Gift"
checkout_option = "Give as a gift"
checkout_option_help = "A book-shaped card to print at home or have sent by post. The person who receives it scans the code and reads the book on an e-reader, tablet or phone."
card_headline = "This book is a gift"
card_from = "From {name}"
card_steps_1 = "Scan this code with a phone camera."
card_steps_2 = "Choose where to read: e-reader, tablet or phone."
card_steps_3 = "The book arrives. Happy reading!"
redeem_cta = "Get my book"
ebook_word = "eBook"
ereader_word = "e-reader"
code_label = "Code"

[de]  # de-DE; also de-AT, de-CH (no sharp s in any string)
product_name = "eBook-Geschenk"
checkout_option = "eBook verschenken"
checkout_option_help = "Eine Karte in Buchform, zum Selbstausdrucken oder per Post verschickt. Wer sie bekommt, scannt den Code und liest das Buch auf eReader, Tablet oder Smartphone."
card_headline = "Dieses Buch ist ein Geschenk"
card_from = "Von {name}"
card_steps_1 = "Diesen Code mit der Kamera eines Smartphones scannen."
card_steps_2 = "Auswählen, wo gelesen wird: eReader, Tablet oder Smartphone."
card_steps_3 = "Das Buch ist da. Viel Freude beim Lesen!"
redeem_cta = "Mein Buch abholen"
ebook_word = "eBook"
ereader_word = "eReader"
code_label = "Code"

[es]  # es-ES
product_name = "eBook Regalo"
checkout_option = "Regalar este eBook"
checkout_option_help = "Una tarjeta con forma de libro, para imprimir en casa o recibir por correo. Quien la recibe escanea el código y lee el libro en eReader, tableta o móvil."
card_headline = "Este libro es un regalo"
card_from = "De parte de {name}"
card_steps_1 = "Escanear este código con la cámara de un móvil."
card_steps_2 = "Elegir dónde leer: eReader, tableta o móvil."
card_steps_3 = "El libro llega. ¡Feliz lectura!"
redeem_cta = "Conseguir mi libro"
ebook_word = "eBook"
ereader_word = "eReader"
code_label = "Código"

[it]  # it-IT
product_name = "eBook Regalo"
checkout_option = "Regala questo eBook"
checkout_option_help = "Un biglietto a forma di libro, da stampare a casa o da ricevere per posta. Chi lo riceve inquadra il codice e legge il libro su eReader, tablet o smartphone."
card_headline = "Questo libro è un regalo"
card_from = "Da parte di {name}"
card_steps_1 = "Inquadrare questo codice con la fotocamera di un telefono."
card_steps_2 = "Scegliere dove leggere: eReader, tablet o smartphone."
card_steps_3 = "Il libro arriva. Buona lettura!"
redeem_cta = "Ricevi il libro"
ebook_word = "eBook"
ereader_word = "eReader"
code_label = "Codice"

[nl]  # nl-NL; also nl-BE
product_name = "eBook Cadeau"
checkout_option = "Cadeau doen"
checkout_option_help = "Een kaart in boekvorm, om zelf te printen of per post te laten bezorgen. De ontvanger scant de code en leest het boek op e-reader, tablet of telefoon."
card_headline = "Dit boek is een cadeau"
card_from = "Van {name}"
card_steps_1 = "Scan deze code met de camera van een telefoon."
card_steps_2 = "Kies waar je wilt lezen: e-reader, tablet of telefoon."
card_steps_3 = "Het boek staat klaar. Veel leesplezier!"
redeem_cta = "Mijn boek ophalen"
ebook_word = "e-book"
ereader_word = "e-reader"
code_label = "Code"

[pt]  # pt-PT; provisional, lowest confidence
product_name = "eBook para Oferecer"
checkout_option = "Oferecer este eBook"
checkout_option_help = "Um cartão em forma de livro, para imprimir em casa ou receber pelo correio. Quem o recebe aponta a câmara para o código e lê o livro no eReader, tablet ou telemóvel."
card_headline = "Este livro é um presente"
card_from = "De {name}"
card_steps_1 = "Apontar a câmara de um telemóvel para este código."
card_steps_2 = "Escolher onde ler: eReader, tablet ou telemóvel."
card_steps_3 = "O livro chega. Boa leitura!"
redeem_cta = "Receber o meu livro"
ebook_word = "eBook"
ereader_word = "eReader"
code_label = "Código"
```

Where each string comes from:

| Keys | Origin |
|---|---|
| `product_name`, `checkout_option`, `card_headline`, `card_from`, `redeem_cta` | Naming note §6 primaries, with the changes listed in §1.6 (items 4 and 9) |
| `card_steps_1..3` (fr) | The UX note's printed card text |
| `card_steps_1..3` (other languages), `checkout_option_help` (all) | Drafted for this document from the French text; lowest confidence in the block |
| `ebook_word`, `ereader_word` | §1.4 |
| `code_label` | Usage note: "code" is uniform across all files read |

### 5.2 Supplementary strings (not part of the requested key set)

`docs/architecture.md` lists further messages (`CardTo`, `CardNoPhone`, `FoldHere`, `CutHere`, `PrintedWith`), and the notes give sub-options, a "paid" line and a reseller variant of steps 2 and 3. Drafts follow under an `extra` namespace so that this block can be concatenated with §5.1. Placeholders: `{name}`, `{host}` (typed-URL host), `{reseller}`, `{product}`, `{title}`, `{author}`, `{date}`. All [M]. The sub-options and `card_paid_line` are the naming note's strings (Portuguese sub-options drafted here). In French, `card_legal_line` and `card_steps_*_reseller` are the UX note's text and `card_single_use` / `card_no_expiry` come from the commerce note's legal line. Everything else, including every translation of those French lines, is drafted here and has the lowest confidence in this document.

```toml
[extra.fr]
card_from_elided = "De la part d’{name}"  # when {name} starts with a vowel or h
card_to = "Pour {name}"
card_no_phone = "Sans téléphone ? Sur la liseuse ou un ordinateur, ouvrir le navigateur, aller sur {host} et saisir le code de cette carte."
card_paid_line = "Ce livre numérique est déjà payé : il ne reste qu’à le lire, sur liseuse, tablette ou téléphone."
card_legal_line = "{title}, {author}. Un exemplaire numérique, déjà payé\u00A0: rien à régler."
card_single_use = "Usage unique."
card_no_expiry = "Sans date limite."
card_collect_before = "À récupérer avant le {date}."
card_steps_2_reseller = "Se connecter à son compte {reseller}, ou en créer un."
card_steps_3_reseller = "Le livre apparaît dans la bibliothèque, sur la liseuse et l’application."
delivery_home = "Carte à imprimer chez moi (gratuit)"
delivery_post = "Carte imprimée et envoyée par la poste"
fold_here = "Plier ici"
cut_here = "Couper ici"
printed_with = "Carte créée avec {product}"
ebook_word_trade = "ebook"
ebook_word_official = "livre numérique"
ereader_word_trade = "liseuse"
ereader_word_official = "liseuse"
qr_word = "QR code"
pronoun_default = "vous"

[extra.en]
card_to = "For {name}"
card_no_phone = "No phone? On an e-reader or a computer, open the browser, go to {host} and enter the code on this card."
card_paid_line = "This eBook is already paid for. Read it on your e-reader, tablet or phone."
card_legal_line = "{title}, {author}. One digital copy, already paid for: nothing to pay."
card_single_use = "Single use."
card_no_expiry = "No time limit."
card_collect_before = "To be collected before {date}."
card_steps_2_reseller = "Sign in to a {reseller} account, or create one."
card_steps_3_reseller = "The book appears in the library, on the e-reader and in the app."
delivery_home = "Card to print at home (free)"
delivery_post = "Printed card, sent by post"
fold_here = "Fold here"
cut_here = "Cut here"
printed_with = "Card made with {product}"
ebook_word_trade = "eBook"
ebook_word_official = "e-book"
ereader_word_trade = "e-reader"
ereader_word_official = "e-reader"
qr_word = "QR code"
pronoun_default = "you"

[extra.de]
card_to = "Für {name}"
card_no_phone = "Kein Smartphone? Auf dem eReader oder am Computer den Browser öffnen, {host} aufrufen und den Code dieser Karte eingeben."
card_paid_line = "Dieses eBook ist bereits bezahlt – einfach lesen: auf eReader, Tablet oder Smartphone."
card_legal_line = "{title}, {author}. Ein digitales Exemplar, bereits bezahlt: nichts mehr zu zahlen."
card_single_use = "Einmalig verwendbar."
card_no_expiry = "Zeitlich unbegrenzt."
card_collect_before = "Abholen bis {date}."
card_steps_2_reseller = "Im {reseller}-Konto anmelden oder ein Konto anlegen."
card_steps_3_reseller = "Das Buch erscheint in der Bibliothek, auf dem eReader und in der App."
delivery_home = "Karte zum Selbstausdrucken (kostenlos)"
delivery_post = "Gedruckte Karte per Post"
fold_here = "Hier falten"
cut_here = "Hier schneiden"
printed_with = "Karte erstellt mit {product}"
ebook_word_trade = "eBook"
ebook_word_official = "E-Book"
ereader_word_trade = "eReader"
ereader_word_official = "E-Reader"
qr_word = "QR-Code"
pronoun_default = "none"  # du or Sie comes from the reseller

[extra.es]
card_to = "Para {name}"
card_no_phone = "¿Sin móvil? En el eReader o en un ordenador, abrir el navegador, ir a {host} e introducir el código de esta tarjeta."
card_paid_line = "Este eBook ya está pagado. Léelo en tu eReader, tableta o móvil."
card_legal_line = "{title}, {author}. Un ejemplar digital, ya pagado: no hay nada que pagar."
card_single_use = "Un solo uso."
card_no_expiry = "Sin fecha límite."
card_collect_before = "Recoger antes del {date}."
card_steps_2_reseller = "Iniciar sesión en la cuenta de {reseller}, o crear una."
card_steps_3_reseller = "El libro aparece en la biblioteca, en el eReader y en la aplicación."
delivery_home = "Tarjeta para imprimir en casa (gratis)"
delivery_post = "Tarjeta impresa y enviada por correo"
fold_here = "Doblar aquí"
cut_here = "Cortar aquí"
printed_with = "Tarjeta creada con {product}"
ebook_word_trade = "eBook"
ebook_word_official = "libro electrónico"
ereader_word_trade = "eReader"
ereader_word_official = "lector de libros electrónicos"
qr_word = "código QR"
pronoun_default = "tú"

[extra.it]
card_to = "Per {name}"
card_no_phone = "Senza telefono? Dall’eReader o da un computer, aprire il browser, andare su {host} e inserire il codice di questo biglietto."
card_paid_line = "Questo eBook è già pagato. Leggilo su eReader, tablet o smartphone."
card_legal_line = "{title}, {author}. Una copia digitale, già pagata: niente da pagare."
card_single_use = "Utilizzabile una sola volta."
card_no_expiry = "Senza scadenza."
card_collect_before = "Da ritirare entro il {date}."
card_steps_2_reseller = "Accedere all’account {reseller}, o crearne uno."
card_steps_3_reseller = "Il libro compare nella libreria, sull’eReader e nell’app."
delivery_home = "Biglietto da stampare a casa (gratis)"
delivery_post = "Biglietto stampato e spedito"
fold_here = "Piegare qui"
cut_here = "Tagliare qui"
printed_with = "Biglietto creato con {product}"
ebook_word_trade = "eBook"
ebook_word_official = "e-book"
ereader_word_trade = "eReader"
ereader_word_official = "lettore di e-book"
qr_word = "codice QR"
pronoun_default = "tu"

[extra.nl]
card_to = "Voor {name}"
card_no_phone = "Geen telefoon? Open op de e-reader of een computer de browser, ga naar {host} en vul de code van deze kaart in."
card_paid_line = "Dit e-book is al betaald. Lees het op je e-reader, tablet of telefoon."
card_legal_line = "{title}, {author}. Eén digitaal exemplaar, al betaald: er valt niets meer te betalen."
card_single_use = "Eenmalig te gebruiken."
card_no_expiry = "Geen einddatum."
card_collect_before = "Ophalen vóór {date}."
card_steps_2_reseller = "Log in op een {reseller}-account, of maak er een aan."
card_steps_3_reseller = "Het boek verschijnt in de bibliotheek, op de e-reader en in de app."
delivery_home = "Kaart zelf printen (gratis)"
delivery_post = "Gedrukte kaart per post"
fold_here = "Hier vouwen"
cut_here = "Hier knippen"
printed_with = "Kaart gemaakt met {product}"
ebook_word_trade = "e-book"
ebook_word_official = "e-book"
ereader_word_trade = "e-reader"
ereader_word_official = "e-reader"
qr_word = "QR-code"
pronoun_default = "je"

[extra.pt]
card_to = "Para {name}"
card_no_phone = "Sem telemóvel? No eReader ou num computador, abrir o navegador, ir a {host} e introduzir o código deste cartão."
card_paid_line = "Este eBook já está pago. Leia-o no seu eReader, tablet ou telemóvel."
card_legal_line = "{title}, {author}. Um exemplar digital, já pago: nada a pagar."
card_single_use = "Utilização única."
card_no_expiry = "Sem data limite."
card_collect_before = "Receber até {date}."
card_steps_2_reseller = "Iniciar sessão na conta {reseller}, ou criar uma."
card_steps_3_reseller = "O livro aparece na biblioteca, no eReader e na aplicação."
delivery_home = "Cartão para imprimir em casa (grátis)"
delivery_post = "Cartão impresso e enviado pelo correio"
fold_here = "Dobrar aqui"
cut_here = "Cortar aqui"
printed_with = "Cartão criado com {product}"
ebook_word_trade = "eBook"
ebook_word_official = "livro digital"
ereader_word_trade = "eReader"
ereader_word_official = "leitor de livros digitais"
qr_word = "código QR"
pronoun_default = "formal"  # third person (você / o seu)
```

### 5.3 Terms the lint must reject

`stored_value`, `scheme_names`, `print_category` and `reads_as_free` are errors in any default card or checkout string. `help_only` verbs are allowed in help pages, page titles and metadata, and rejected on the card and on buttons. Matching rules are in the last section. Lists come from §4.1 and §2; all [M] except where §2 tags the word.

```toml
[banned.fr]
stored_value = ["carte cadeau", "e-carte cadeau", "chèque cadeau", "bon cadeau", "bon d'achat", "bon pour", "certificat cadeau", "coffret cadeau"]
scheme_names = ["chèque lire", "chèque culture", "Kadéos", "Cadhoc", "Tir Groupé", "Illicado"]
print_category = ["livre cadeau"]
reads_as_free = ["ebook offert"]
help_only = ["échanger", "valider la carte"]

[banned.en]
stored_value = ["gift card", "gift voucher", "voucher", "eGift"]
scheme_names = ["book token"]
print_category = ["gift book"]
reads_as_free = []
help_only = ["redeem"]

[banned.de]
stored_value = ["Gutschein", "Geschenkkarte"]
scheme_names = []
print_category = ["Geschenkbuch"]
reads_as_free = ["Geschenk-eBook", "eBook geschenkt"]
help_only = ["einlösen", "einzulösen"]

[banned.es]
stored_value = ["tarjeta regalo", "tarjeta de regalo", "cheque regalo", "bono regalo", "cupón"]
scheme_names = ["Bono Cultural"]
print_category = ["libro regalo"]
reads_as_free = ["eBook de regalo"]
help_only = ["canjear"]

[banned.it]
stored_value = ["buono regalo", "carta regalo", "gift card", "voucher"]
scheme_names = ["18app", "Carta della Cultura", "Carta del Docente"]
print_category = ["libro regalo"]
reads_as_free = ["eBook in regalo", "eBook in omaggio"]
help_only = ["riscattare", "riscatta"]

[banned.nl]
stored_value = ["cadeaubon", "cadeaukaart", "voucher"]
scheme_names = ["boekenbon"]
print_category = ["cadeauboek"]
reads_as_free = []
help_only = ["verzilveren", "inwisselen"]

[banned.pt]
stored_value = ["cartão oferta", "cartão de oferta", "cartão presente", "cartão de presente", "cheque prenda", "cheque oferta", "vale presente", "voucher"]
scheme_names = []
print_category = []
reads_as_free = []
help_only = ["resgatar", "reclamar"]
```

---

## 6. Open questions for native-speaker review

One reviewer per language, ideally someone who has worked on a bookseller's site. For each language: read §5.1 and §5.2 aloud as if printed on a card, then answer the questions below.

**All languages**

1. Does the headline read as "this object is a present", with no hint of "free" or "voucher"?
2. Is the infinitive (or imperative, in Dutch and English) natural for printed instructions, or does it read as a machine translation?
3. Is "code" alone enough for the typed fallback, or does the language want "gift code" — and would that word bring back the stored-value meaning?
4. Which spelling does the largest bookseller in your country use for eBook and e-reader today?

**French**

5. « Offrir cet ebook » or « Offrir cet eBook » or « Offrir ce livre numérique » on a Fnac-type page? Lower-case "ebook" follows Decitre [V2-sib]; Fnac and Kobo are recalled as « eBook » / « Ebook » [M].
6. Is « Récupérer mon livre » natural, or is « Lire mon livre » / « Ajouter à ma bibliothèque » better once the recipient is signed in?
7. « De la part de Hugo » or « De la part d'Hugo »? The elision rule in the last section treats every initial h as mute.
8. fr-CA: confirm « livre numérique », « code QR », « courriel », and whether tu should be the default.
9. Belgium and Switzerland: does anything in the fr-FR set sound French-from-France (« téléphone » versus « GSM » / « natel » in speech)?

**German**

10. „eBook" or „E-Book" as the default for a service that is not a bookseller?
11. Is „Mein Buch abholen" understood on a button, or does a German reader look for „Code einlösen"? (The voucher verb is kept for help pages; should the code-entry form use it too?)
12. du or Sie for a standalone site in 2026? Is the infinitive on the card acceptable in both worlds?
13. „eBook-Geschenk" as the option's name: does it read as "a free eBook"?

**Spanish**

14. « Conseguir mi libro » or « Recibir mi libro » or « Obtener mi libro »?
15. « tableta » or « tablet »; « móvil » is Spain-only: is one string with « teléfono » better for Spain and Latin America together?
16. Is « eBook Regalo » natural as a name, or does Spanish want « eBook de regalo » (which also means "free eBook")?

**Italian**

17. « Inquadrare » or « Scansionare » in step 1 (§1.6, item 5)?
18. The button « Ricevi il libro » is an imperative in the tu form, unlike the first-person buttons elsewhere. Is « Ricevi il mio libro », « Ottieni il libro » or « Vai al libro » better?
19. « biglietto » for the object: does it suggest a ticket or a greeting card? Is « cartolina » or « card » better?
20. « libreria » or « biblioteca » for the reader's library in an eBook app?

**Dutch**

21. "Cadeau doen" alone as a button, or "Dit e-book cadeau doen"?
22. Is "eBook Cadeau" acceptable as a name next to the norm spelling "e-book" in running text?
23. "Mijn boek ophalen": natural, or too close to collecting a parcel?
24. Does step 2 need "je" (« Kies waar je wilt lezen »), and is that acceptable in Flanders on a printed card?

**Portuguese**

25. Everything. In particular: « Oferecer este eBook » (pt-PT) against « Dar de presente » (pt-BR); « presente » against « prenda » on a card printed for Portugal; « Apontar a câmara… » against « Ler o código… »; « telemóvel » / « celular ».
26. Should pt-PT and pt-BR be two locales in the code instead of one?

**Legal and trade checks (not for translators)**

27. INPI and TMview searches for "ebook cadeau" and for each `product_name`, classes 9, 16, 35, 41; WHOIS for the domain variants.
28. A VAT adviser's written position: single-purpose voucher, multi-purpose voucher, or direct supply (§4.2), and the statutory wording wanted in the terms of sale.
29. Owners and status of the scheme names (Chèque Lire, Chèque Culture, Boekenbon, National Book Tokens).
30. Live wording at each target reseller before integration: Fnac/Kobo, 7switch, Cultura, Decitre, Thalia, Hugendubel, bol.com, Casa del Libro, IBS, laFeltrinelli — option label, noun spelling, tu/vous or du/Sie.

---

## 7. What could not be verified, and sources

### 7.1 Not verified

1. **Every live retailer string** except the three sibling-sourced flows: Amazon (all marketplaces other than the .fr help page), Kobo, Apple Books, Google Play Books, Fnac, 7switch, Cultura, bol.com, Casa del Libro, IBS, Feltrinelli, Mondadori Store, Wook, Empik, Woblink, Adlibris, Bokus, Saxo, ARK, Suomalainen Kirjakauppa, Palmknihy, Elefant, Public.
2. **Which retailers offer title-specific gifting in 2026**, and their exact button text.
3. **Official terminology pages**: FranceTerme and the *Journal officiel* date; OQLF; Duden; Fundéu, RAE, Treccani, TERMCAT, SAOL, Kotus, PWN, ÚJČ, DOOM3. All hosts tried were blocked.
4. **Search volumes** and any Google Trends comparison.
5. **Trademark registers, company names and domain availability** for every name in this document.
6. **Statutes**: article numbers in the Code monétaire et financier; the Dutch two-year rule and its scope; the Belgian minimum validity; any French validity rule for gift cards; CGI art. 256 ter; the state of the PSD2 successor texts.
7. **Regional French retail usage** (Belgium, Switzerland, Québec); Romanian, Greek and Catalan rest on one or two files each.
8. **Whether any current European service prints a title-specific eBook gift object** under any of these names.
9. **Every recommended string**, until a native speaker has reviewed it.

### 7.2 Sources

Fetched and read by the usage researcher (all `raw.githubusercontent.com`, HTTP 200, 2026-11-12). `{l}` = language code.

- Shopify Dawn — `https://raw.githubusercontent.com/Shopify/dawn/main/locales/{l}.json` for `en.default`, `fr`, `de`, `es`, `it`, `nl`, `pt-BR`, `pt-PT`, `pl`, `sv`, `da`, `nb`, `fi`, `cs`, `el`
- Odoo 17.0 — `https://raw.githubusercontent.com/odoo/odoo/17.0/addons/loyalty/i18n/{l}.po` and `.../addons/website_sale_loyalty/i18n/{l}.po` for `fr`, `de`, `es`, `es_419`, `it`, `nl`, `pt`, `pt_BR`, `pl`, `sv`, `da`, `nb`, `fi`, `cs`, `ro`, `el`, `ca`; plus `loyalty` `el` / `nb` on branches `18.0` and `master`
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
- LanguageTool — https://raw.githubusercontent.com/languagetool-org/languagetool/master/languagetool-language-modules/de/src/main/resources/org/languagetool/rules/de/grammar.xml ; https://raw.githubusercontent.com/languagetool-org/languagetool/master/languagetool-language-modules/nl/src/main/resources/org/languagetool/rules/nl/grammar.xml
- OpenTaal — https://raw.githubusercontent.com/OpenTaal/opentaal-wordlist/master/wordlist.txt
- LibreOffice dictionaries — https://raw.githubusercontent.com/LibreOffice/dictionaries/master/de/de_DE_frami.dic ; https://raw.githubusercontent.com/LibreOffice/dictionaries/master/da_DK/da_DK.dic ; https://raw.githubusercontent.com/LibreOffice/dictionaries/master/no/nb_NO.dic ; https://raw.githubusercontent.com/LibreOffice/dictionaries/master/cs_CZ/cs_CZ.dic ; https://raw.githubusercontent.com/LibreOffice/dictionaries/master/it_IT/it_IT.dic ; https://raw.githubusercontent.com/LibreOffice/dictionaries/master/ca/dictionaries/ca.dic ; https://raw.githubusercontent.com/LibreOffice/dictionaries/master/en/en_GB.dic
- PrestaShop — https://raw.githubusercontent.com/PrestaShop/PrestaShop/develop/translations/default/ShopThemeCheckout.xlf
- Magento — https://raw.githubusercontent.com/magento/magento2/2.4-develop/app/code/Magento/GiftMessage/i18n/en_US.csv

Recorded by other researchers in sibling notes from their own search results; not re-opened:

- https://www.decitre.fr/offrir-un-ebook ; https://www.decitre.fr/faq/numerique/offrir-un-ebook ; https://www.liseuses.net/offrir-des-ebooks-avec-tea/
- https://www.amazon.fr/gp/help/customer/display.html?nodeId=GVWGP284MQ6ZRM59 ; https://kdp.amazon.com/fr_FR/help/topic/G200652260 ; https://www.idboox.com/news-livres/amazon-on-peut-enfin-offrir-un-livre-numerique-kindle/
- https://www.hugendubel.de/de/shortcut/ebooksverschenken ; https://www.heise.de/tipps-tricks/E-Books-verschenken-so-klappt-s-7476189.html ; https://www.test.de/E-Books-Nuetzliche-Tipps-und-Tricks-5391265-0/
- https://www.dealabs.com/discussions/carte-cadeau-kobo-ou-fnac-2273026 ; https://ebook-enligne.fr/blog/offrir-ebook-guide-cadeau-numerique/
- https://publishersweekly.com/pw/print/20120827/53710-livrada-tries-its-luck-with-e-book-gift-cards.html ; https://the-digital-reader.com/ebook-startup-livrada-shut/ ; https://shelf-awareness.com/theshelf/2016-01-26/amazon_testing_title-specific_kindle_gift_cards_.html ; https://www.geekwire.com/2016/amazons-other-physical-retail-test-testing-the-kindle-e-book-kiosk-at-the-local-drug-store/ ; https://www.mobileread.com/forums/showthread.php?t=363478
- https://authors.bookfunnel.com/help/print-codes/ ; https://www.dropcards.com/pricing/ ; https://bookwire.net/faq-download-codes
- https://www.legalplace.fr/guides/prix-depot-marque-inpi/ ; https://www.legalstart.fr/fiches-pratiques/depot-marque/cout-depot-marque/ ; https://www.elly-assurance.fr/actualites/inpi/
- https://www.wipo.int/wipolex/en/legislation/details/13861 ; https://www.livreshebdo.fr/article/la-loi-sur-le-prix-en-vigueur

Legal instruments cited from memory, without link: Code de la propriété intellectuelle arts. L. 711-2 and L. 713-6; Regulation (EU) 2017/1001 arts. 7 and 14; CJEU C-363/99, C-265/00, C-191/01 P; Directive (EU) 2015/2366 art. 3(k) and art. 37(2); Code monétaire et financier arts. L. 521-3 and L. 525-5; ZAG § 2; BGB §§ 195, 199; UStG § 3 Abs. 13–15; Directive (EU) 2016/1065; CGI art. 256 ter; loi n° 2011-590; Buchpreisbindungsgesetz; loi n° 94-665; Irish Consumer Protection (Gift Vouchers) Act 2019; Dutch minimum-validity rule for gift vouchers (2022).

---

## Implications for the implementation

1. **Locales.** Ship the seven locales of `Locale::ALL` (fr, en, de, es, it, nl, pt) with the strings of §5.1. `fr` is the default. Mark `pt` as provisional in the UI language list until reviewed. Do not ship pl, sv, da, nb, fi, cs, ro, el or ca from the glossaries of §1.5: they are translator briefs, not strings.
2. **Completeness tests.** For every locale, every key of §5.1 is present and non-empty. For every key, all locales use the same set of `{placeholders}`. No rendered string may contain `{` or `}`.
3. **Length budget.** In §5.1 the longest values are: `checkout_option_help` 166 characters (fr), `card_steps_*` 60 (de), `card_headline` 28 (de), `redeem_cta` 19 (fr, pt), `checkout_option` 19 (it, pt), `product_name` 19 (pt). The card layout must fit a 60-character step on one line at 12 pt or wrap it to two lines without overflow; add a test that renders each locale and fails on overflow. Tenant overrides are limited to twice these lengths.
4. **No noun interpolation.** `ebook_word` and `ereader_word` are used only as stand-alone labels and list items. Any sentence that contains the noun is stored whole, because the article and the compound change with the noun (« cet ebook » / « ce livre numérique »; „E-Book-Reader"; Polish declension). Placeholders are allowed only for proper names, hosts, codes, dates and the reseller's name.
5. **Noun style per tenant.** Add a tenant setting `noun_style` = `trade` | `official` (table in §1.4, values in §5.2). Default: `trade`, except French running text, which defaults to `official` (« livre numérique »). A tenant may also override whole strings. Demo stores: the fake Fnac uses « Ebook » and « liseuse »; the fake 7switch uses « ebook » (the real site's wording was not checked); a German demo uses „eBook" and „eReader".
6. **Option wording in the demo checkouts.** Section heading = `product_name` (« eBook Cadeau »); checkbox or button = `checkout_option` (« Offrir cet ebook »); text under it = `checkout_option_help`; then the two sub-options `delivery_home` and `delivery_post`, with the printed card as its own price line, never pre-ticked. If a tenant has no postal option, it must override `checkout_option_help`, which mentions the post.
7. **Register.** Add a tenant setting `register` = `neutral` (default) | `informal` | `formal`. The default strings contain no second-person pronoun on the card in fr, de, en, es, it and pt; Dutch step 2 uses "je". German has no pronoun default: a German tenant that wants pronoun forms must set `register`. In standalone mode, offer the giver a headline choice between the neutral headline and the two personal ones of §1.3 (« {Prénom} vous offre ce livre » / « {Prénom} t'offre ce livre »).
8. **French elision.** `card_from` in French has two forms. Use `card_from_elided` (« De la part d'{name} ») when the trimmed name starts with a, e, i, o, u, y, h or an accented form of these (case-insensitive); otherwise `card_from`. Test with « Élise », « Hugo », « Yves », « Marie ».
9. **Banned-term lint.** Add a test that fails when a default string of the card, the checkout or a button matches an entry of §5.3 for its locale. Matching: compare after lower-casing and Unicode NFKC normalisation; treat space, no-break space, hyphen and U+2011 as the same separator; a one-word entry matches as a substring (this catches German and Dutch compounds and plurals); a multi-word entry matches when each of its words is a prefix of consecutive words in the text (this catches « cartes cadeaux »). `help_only` entries are exempt in strings whose key starts with `help_` or `meta_`. Run the same lint on tenant overrides as a warning shown in the admin screen, not as an error.
10. **Redemption vocabulary.** The button uses `redeem_cta`. In de, es, it, pt, and en, the redemption page's `<title>`, meta description and help headings also contain the established verb (einlösen, canjear, riscattare, resgatar, redeem) so that site search and web search find the page. French and Dutch use the plain verb everywhere; never « échanger » in French.
11. **The typed fallback is a "code".** Label it with `code_label`. The words passphrase, password, mot de passe, Passwort, voucher, bon, Gutschein and coupon never label it, including when the code doubles as an LCP passphrase.
12. **No "QR" on the card.** Printed steps say « ce code ». On screens, when the term is needed (alternative text, help), use `qr_word` from §5.2, and « code QR » for fr-CA.
13. **Nothing that looks like stored value on the card**: no price, no balance, no "valid until". With no expiry (the default) print `card_no_expiry` (fr « Sans date limite. »). If `expires_unix` is set, print `card_collect_before` (fr « À récupérer avant le {date}. »), never « Valable jusqu'au ». The legal line is `card_legal_line` followed by `card_single_use` (§5.2) — not « Bon pour… ».
14. **Typography.** French strings carry a no-break space (U+00A0) before `:`, `!` and `?`; all locales use the typographic apostrophe U+2019; German strings contain no « ß » (assert this in a test, so that de-CH needs no variant); Spanish strings open with « ¿ » and « ¡ ». Add a test that every string of §5.1 and §5.2 renders with no missing glyph in each bundled font. When a style upper-cases text, use locale-aware upper-casing and keep accents on French capitals.
15. **Regional deltas without new locales.** Keep the full language tag of the request or tenant (for example `fr-CA`, `pt-BR`, `es-MX`) next to the folded `Locale`, and apply a small override map: fr-CA (« code QR », « courriel », noun always « livre numérique »); pt-BR (« celular », « câmera », « e-book »); es-419 (« celular »). No delta is needed for fr-BE, fr-CH, de-AT, de-CH, nl-BE.
16. **White label in reseller mode.** `product_name` and `printed_with` must be optional on the card and on the redemption page: a tenant can hide both. The QR target and the typed host are on the tenant's domain; the default paths are `/cadeau` (fr, nl), `/geschenk` (de), `/regalo` (es, it), `/gift` (en), `/oferta` (pt) for the landing page, while the code path stays language-neutral and short.
17. **Third-party names.** Device and shop names (Kindle, Kobo, tolino, Vivlio, PocketBook) may appear in compatibility text only; never in `product_name`, never as logos. Add them to a second lint list applied to `product_name` and `checkout_option`.
18. **Standalone-site pages.** One how-to page per language whose title and H1 use the verb query: « Offrir un ebook », „eBook verschenken", "E-book cadeau doen", « Regalar un eBook », « Regalare un eBook », "Gift an eBook", « Oferecer um eBook ». Do not use the noun phrases of the `reads_as_free` list in titles.
19. **Review gate.** Record, per locale, whether a native reviewer has signed off (§6). Until then show the locale as "beta" in the standalone UI, and keep the strings in data files (not in code) so that a reviewer can change them without a rebuild of the logic.
