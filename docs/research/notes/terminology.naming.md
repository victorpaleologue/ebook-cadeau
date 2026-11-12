# What to call it: "eBook cadeau" and its equivalents per website language — naming and brand

Research notes, 2026-11-12. Angle: product name, per-language labels, trademark/descriptiveness risk, stored-value vocabulary traps.

## Method and limits — read first

- **No web search could be run for this note.** All six WebSearch calls were refused: the shared search budget of the workflow turn (200 calls) was already used up by other agents. WebFetch is blocked in this sandbox. Nothing below was verified online in this session.
- Consequences: (a) every statement about trademarks, registers, live retailer wording and domain availability is **unverified**; (b) no INPI, EUIPO/TMview, WIPO or WHOIS lookup was done; (c) no keyword-volume data was obtained.
- Confidence tags used here:
  - **[M]** = from memory, not verified in this session (with high / medium / low where it helps).
  - **[V1-sib] / [V2-sib]** = taken from a sibling note in `docs/research/notes/` where it was sourced from search results by another agent (one source / two or more). The URLs cited are copied from those notes; they did not appear in my own search results.
  - No claim in this note reaches [V1] or [V2] on its own.
- Legal instruments are cited by name and article number without links, because I will not write a URL I did not see.
- The strings in section 6 are a native-usage judgment from memory. They need a native-speaker review per language and a check against each reseller's live wording before they are hard-coded in `crates/ebc-i18n`.

## 1. Bottom line

1. **"eBook Cadeau" is a good option label and a weak trademark.** It is immediately understood in French and, unchanged, in Dutch ("een e-book cadeau doen"). It is descriptive in both languages, so a word mark is likely to be refused in France and EU-wide [M, high for the legal rule; no register search done].
2. **Descriptiveness is an advantage in the reseller mode.** A reseller will not put a third party's brand in its checkout. A generic label ("Offrir cet ebook", "eBook verschenken") can be used by every reseller without licence or conflict.
3. **Use two levels of naming**: a generic, localised option label and card vocabulary (this note), and — only if protection is wanted — a distinctive house brand or a figurative mark for the standalone site and the B2B offer.
4. **Search intent sits on the verb, not on the noun phrase.** People ask how to *give* an eBook: "offrir un ebook", "eBook verschenken", "ebook cadeau doen", "regalar un ebook", "regalare un ebook", "gift an ebook" [M; no volumes]. The noun phrases "ebook cadeau", "ebook offert", "eBook de regalo", "eBook in regalo", "eBook geschenkt" are also the stock wording of free lead-magnet eBooks, which pollutes search results [M, high].
5. **Never use the stored-value words** for the object or the option: "carte cadeau", "chèque cadeau", "bon cadeau", "Gutschein", "Geschenkkarte", "cadeaubon", "cadeaukaart", "tarjeta regalo", "cheque regalo", "buono regalo", "carta regalo", "gift card", "voucher", "token". They signal a monetary balance, consumer validity rules and, for multi-seller instruments, payment-services analysis. The product is one prepaid title.
6. **Register**: default to register-neutral forms (infinitive instructions, first-person buttons, "Ce livre est un cadeau"). Where a pronoun is needed: FR vous, DE neutral or du by reseller, NL je, ES tú, IT tu. In standalone mode let the giver pick tu/vous (du/Sie) for the headline, because the card speaks for the giver, not for the shop.

## 2. Evaluation of "eBook Cadeau" and candidate names

| Candidate | Clarity | Searchability | Word-mark registrability | Stored-value connotation | Verdict |
|---|---|---|---|---|---|
| **eBook Cadeau** (FR, NL) | High for "an eBook as a present"; says nothing about the printed object. Also readable as "free eBook" | Generic query; competes with lead-magnet pages and, in NL, retailer help pages | Low: descriptive in FR and NL | None | Keep as option label and working name |
| **Offrir un ebook / Offrir cet ebook** (FR) | Highest: it is the action. Decitre already uses "Offrir un ebook" and an "Offrir" button [V2-sib] ([Decitre](https://www.decitre.fr/offrir-un-ebook), [Decitre FAQ](https://www.decitre.fr/faq/numerique/offrir-un-ebook)) | Matches the how-to query | None (plain language) | None | Use for checkout label, page titles, H1 |
| **Livre numérique à offrir** (FR) | High, formal; fits Québec and public-sector style | Lower volume than "ebook" [M] | None | None | Alternate; variant for fr-CA |
| **Livre Cadeau Numérique / livre cadeau** | Confusing: "livre cadeau" is the retail category of printed gift books (*beaux livres*) [M, high]. Same trap: DE "Geschenkbuch", NL "cadeauboek", ES/IT "libro regalo", EN "gift book" | Collides with that category | Low | None | Reject |
| **Carte cadeau ebook / eBook-Gutschein / cadeaubon** | Clear but wrong: implies a balance | Good volume, wrong intent (people looking for value cards) | Low | **Strong** | Reject |
| **Chèque Lire / Chèque Culture / book token / Boekenbon** | — | — | Existing marks or scheme names (section 4) | **Strong** | Reject |
| **Jaquette cadeau** (FR) — names the object | Medium: "jaquette" is the dust jacket; needs one line of explanation | Near-zero existing demand [M] | Better than "eBook cadeau", still partly descriptive | None | Alternate name for the printed object |

Assessment of the name for the project and repository: keep "eBook Cadeau". It is accurate, bilingual FR/NL, and free to use because nobody can monopolise it. The cost is that it cannot be defended either (section 3).

## 3. Trademark and descriptiveness

All of this section is [M]; the rules are well established, the application to this name is my analysis, and **no register was searched**.

- **France**: Code de la propriété intellectuelle, art. L. 711-2 (as rewritten by ordonnance n° 2019-1169): a sign is refused if it lacks distinctive character (2°), if it consists exclusively of elements designating a characteristic of the goods or services — kind, intended purpose — (3°), or of terms customary in trade (4°). "eBook cadeau" for a service of giving eBooks falls under 3° on its face [M, high for the rule].
- **EU**: Regulation (EU) 2017/1001, art. 7(1)(b) and (c), and **art. 7(2)**: a ground that exists in only part of the Union is enough. A term descriptive for French-speaking (FR, BE, LU) and Dutch-speaking (NL, BE) consumers is refused for the whole EU [M, high].
- Case law on combinations of descriptive words: CJEU C-363/99 *Postkantoor* and C-265/00 *Biomild* (2004) — a combination of descriptive elements is itself descriptive unless the whole creates an impression clearly different from the sum; C-191/01 P *Doublemint* (2003) — one descriptive meaning among several is enough [M, high]. "eBook" + "cadeau" is an ordinary French and Dutch construction, not an unusual juxtaposition.
- **Acquired distinctiveness** (art. 7(3) EUTMR; L. 711-2 last paragraph CPI) is possible but needs heavy, long use. The French reference is the "vente-privee.com" litigation, where the mark survived only on proof of use-acquired distinctiveness after years of proceedings [M, medium on the procedural detail].
- **What can be registered**: a figurative mark (logo with the words). It protects the graphic, not the words; art. L. 713-6 CPI and art. 14(1)(b) EUTMR let others keep using descriptive terms in honest practice [M, high]. Sibling cost data: INPI EUR 190 for one class, EUR 40 per extra class [V2-sib] ([LegalPlace](https://www.legalplace.fr/guides/prix-depot-marque-inpi/), [Legalstart](https://www.legalstart.fr/fiches-pratiques/depot-marque/cout-depot-marque/)); EUIPO about EUR 850 for one class [V1-sib, conflicting] ([Elly Assurance](https://www.elly-assurance.fr/actualites/inpi/)).
- **Risk of infringing someone else**: low for the same reason — a prior word mark on "ebook cadeau" for these services would itself be weak. A prior figurative or composite mark, or a prior company or domain name, could still exist. **Not checked.**
- **Third-party marks that will appear near the name**:
  - Reading-system names (Kindle, Kobo, tolino, Vivlio, PocketBook): referential use in text to state compatibility is allowed (art. 14(1)(c) EUTMR); do not use logos, do not put them in the product name [M, high].
  - "QR Code" is a registered trademark of DENSO WAVE INCORPORATED; the usual practice is an attribution line when the term is printed. Writing "Scannez le code" / "Code scannen" avoids printing the term at all [M, high].
- **If a protectable brand is wanted**, it has to be a coined or non-descriptive word used above the generic label. Directions only, none cleared: bookbinding and bibliophile vocabulary (*jaquette*, *dos*, *ex-dono* — the inscription recording that a book was given). Each needs an INPI + TMview + domain check before any use.
- **Language law**: "eBook" is an English word. In France the Loi Toubon (n° 94-665) requires French in consumer-facing presentation but tolerates brand names; pairing "eBook" with "livre numérique" in the explanatory sentence covers it. Québec's Charter rules are stricter; use "livre numérique" and "code QR" there [M, medium-high].

## 4. Legal vocabulary traps: words that imply stored value

The product is one identified, prepaid title from one seller, with no balance. The words below describe something else. Statutes are [M]; none was re-read in this session.

| Term(s) | Where | What the public and the law understand | Why it is a trap here |
|---|---|---|---|
| carte cadeau, e-carte cadeau, chèque cadeau, bon d'achat | FR | Prepaid balance spendable on anything at one chain or a network; issued by the seller or by specialised issuers (Kadéos/Edenred, Cadhoc and Chèque Lire/Up, Tir Groupé/Pluxee, Illicado) [M, medium-high] | Network instruments rely on the "limited network" exclusion of PSD2 art. 3(k), transposed in the Code monétaire et financier (art. L. 521-3 for payment services, L. 525-5 for e-money), with a notification duty above EUR 1 M of transactions a year [M, medium on article numbers]. Also triggers expectations on validity, balance and refund |
| chèque-lire, chèque culture | FR | Dedicated cultural vouchers bought by works councils; URSSAF exempts them from social contributions without the usual gift ceiling [M, medium-high] | "Chèque Lire" is, to my knowledge, a brand of Groupe Up [M, medium]. Wrong product class and someone else's name |
| bon cadeau, coffret cadeau | FR | Voucher for a service or experience (hotel, restaurant, Smartbox-type box) | Implies expiry and exchange rules |
| Gutschein, Geschenkgutschein, Geschenkkarte, Büchergutschein | DE, AT, CH | Value voucher or card. General limitation period of three years from the end of the year of issue (§§ 195, 199 BGB); shorter terms in T&Cs are often void [M, high] | A reader told "Gutschein" expects three years and a value. E-money/ZAG analysis for multi-seller use (§ 2 Abs. 1 Nr. 10 ZAG) [M, medium-high]. VAT law has its own "Einzweck-/Mehrzweck-Gutschein" (§ 3 Abs. 13–15 UStG) |
| cadeaubon, cadeaukaart, boekenbon | NL, BE | Value voucher. Dutch law sets a minimum validity of two years for gift vouchers (in force 1 July 2022) [M, medium-high]. Belgium: a one-year minimum, from memory [M, low]. "Boekenbon" is the national book-voucher scheme and a protected name [M, medium] | Validity rule and scheme confusion |
| tarjeta regalo, cheque regalo, bono | ES | Value card (El Corte Inglés, Amazon "cheques regalo"); "Bono Cultural Joven" is a state scheme [M, medium] | Balance connotation; confusion with the state scheme |
| buono regalo, carta regalo, gift card, buono | IT | Value card (Amazon.it "Buoni Regalo"); "buono" is also the unit of the state culture cards (18app / Carta della Cultura Giovani, Carta del Docente) [M, medium] | Same |
| book token, gift card, gift voucher, eGift | UK, IE | "National Book Tokens" is the Booksellers Association's scheme (since 1932) [M, medium-high]. Ireland's Consumer Protection (Gift Vouchers) Act 2019 imposes a five-year minimum validity [M, high] | Scheme confusion; statutory validity |
| voucher / bon / Gutschein in the VAT sense | EU | Directive (EU) 2016/1065 (arts. 30a–30b, 73a of the VAT Directive): single-purpose vs multi-purpose vouchers, different VAT timing [M, high] | A one-title prepaid code is taxed at sale either way; calling it a "voucher" invites the multi-purpose reading |

Design rules that follow, to keep the wording and the substance aligned [analysis]:

- One code = one title at one seller. No face value printed, no balance, no change, no cash alternative. This also follows the lesson of the failed US cards, where a printed price diverged from the live eBook price [V1-sib] ([GeekWire](https://www.geekwire.com/2016/amazons-other-physical-retail-test-testing-the-kindle-e-book-kiosk-at-the-local-drug-store/)).
- Do not print a disclaimer such as "ceci n'est pas une carte cadeau". Classification follows substance, not labels, and the sentence only plants the idea.
- Fixed book price laws apply to eBooks in France (loi n° 2011-590) and Germany (BuchPrG, eBooks expressly covered since 2016) [M, high]. The eBook line must stay at the publisher's price; the card is a separate line with its own VAT rate (eBook 5.5 % FR / 7 % DE; printed card at the standard rate) [M, high]. The option label must therefore name a service ("Carte imprimée et envoyée"), never a bundle price.
- In standalone mode, if eBook Cadeau itself sold codes redeemable at third-party shops, it would be handling payment for others. That question belongs to the legal notes; the naming must not make it worse by calling the code a card with value.

## 5. Existing products and the names they chose

Sourced facts come from sibling notes; nothing new was found here.

| Product | Name used for the object | Status | Source |
|---|---|---|---|
| Livrada (US), Enthrill (CA) | "eBook gift cards" (press term) | Closed 2016–2017 | [V2-sib] ([Publishers Weekly](https://publishersweekly.com/pw/print/20120827/53710-livrada-tries-its-luck-with-e-book-gift-cards.html), [The Digital Reader](https://the-digital-reader.com/ebook-startup-livrada-shut/)) |
| Amazon, 2016 | "Kindle Cards" | Test, not rolled out | [V2-sib] ([Shelf Awareness](https://shelf-awareness.com/theshelf/2016-01-26/amazon_testing_title-specific_kindle_gift_cards_.html)) |
| Walmart eBooks by Rakuten Kobo, 2018 | "digital book cards" | Ended by 2024 | [V1-sib] ([MobileRead](https://www.mobileread.com/forums/showthread.php?t=363478)) |
| BookFunnel | "print codes" | Active | [V2-sib] ([BookFunnel](https://authors.bookfunnel.com/help/print-codes/)) |
| Dropcards, Distro Print | "download cards", "drop cards" | Active | [V1-sib] ([Dropcards](https://www.dropcards.com/pricing/)) |
| Bookwire (DE) | "Download-Codes" | Active | [V1-sib] ([Bookwire](https://bookwire.net/faq-download-codes)) |
| Decitre / Furet du Nord (Vivlio) | "Offrir un ebook", button "Offrir" — e-mail only | Active | [V2-sib] ([Decitre](https://www.decitre.fr/offrir-un-ebook), [Liseuses.net](https://www.liseuses.net/offrir-des-ebooks-avec-tea/)) |
| Hugendubel / Thalia (tolino) | "Verschenken", "eBooks verschenken" — e-mail | Active | [V2-sib] ([Hugendubel](https://www.hugendubel.de/de/shortcut/ebooksverschenken), [heise](https://www.heise.de/tipps-tricks/E-Books-verschenken-so-klappt-s-7476189.html)) |
| Amazon.fr | "Acheter pour d'autres personnes" | Active | [V2-sib] ([Amazon.fr help](https://www.amazon.fr/gp/help/customer/display.html?nodeId=GVWGP284MQ6ZRM59), [IDBoox](https://www.idboox.com/news-livres/amazon-on-peut-enfin-offrir-un-livre-numerique-kindle/)) |
| Kobo by Fnac | No "offrir" function found | — | [V1-sib] ([Dealabs](https://www.dealabs.com/discussions/carte-cadeau-kobo-ou-fnac-2273026)) |

Observations [analysis]:

- Every earlier physical product was named "card" + "code". None used the book-jacket idea or a word for it. The jacket is the differentiator, so the object deserves its own word in each language (section 6, "object" row), distinct from "gift card".
- The living retailer flows are all named with the **verb of giving**, which is the vocabulary buyers already meet.
- I know of no service or registered mark named "eBook Cadeau", "Geschenk-eBook" or "Regala un eBook" [M, low]. This is an absence of recollection, not a search result.

## 6. Per-language recommendations

Conventions for all languages:

- `{ebook}` should be a per-reseller variable: the card must spell the noun the way the host site does. Dictionary forms differ from trade forms: FR "livre numérique" (official term, with "liseuse" for the device, published in the Journal officiel in 2012) vs trade "ebook"/"eBook"; DE Duden "E-Book" vs trade "eBook"; NL "e-book"; ES "libro electrónico" (recommended by Fundéu) vs trade "eBook"; IT "ebook"/"e-book" [M, medium-high].
- Card instructions are written in the infinitive or as a noun phrase where the language allows it. This avoids choosing tu/vous or du/Sie on behalf of the giver.
- Buttons on the redemption page are in the first person ("Récupérer mon livre"), which is also register-neutral.
- The card says "livre / Buch / boek / libro / book", not "eBook", in the headline: the gift is a book. The explanatory sentence then says it is digital.
- All strings are [M].

### French (fr-FR; also fr-BE, fr-CH, fr-LU)

| Field | Primary | Alternates | Notes |
|---|---|---|---|
| Product name | **eBook Cadeau** | "Livre numérique à offrir"; object: "jaquette cadeau", "carte à offrir" | Never "carte cadeau" |
| Checkout option label | **Offrir cet ebook** | "C'est pour offrir"; "Acheter pour offrir" | "C'est pour offrir ?" is the question asked at the till in French shops [M, high] |
| Sub-options | "Carte à imprimer chez moi (gratuit)" / "Carte imprimée et envoyée par la poste" | — | Separate price line |
| Card headline | **Ce livre est un cadeau** + "De la part de {prénom}" | "{Prénom} vous offre ce livre" / "{Prénom} t'offre ce livre" (giver's choice); "Un livre pour vous / pour toi" | Avoid a bare "eBook offert": "offert" also means free |
| Redemption CTA (print) | **Scannez le code pour recevoir votre livre** | Neutral: "Pour lire ce livre : scanner ce code"; tu: "Scanne le code pour recevoir ton livre"; typed fallback: "ou saisir le code {XXXX-XXXX} sur {url}" | "Flashez" is colloquial and dated. Avoid "échanger", "valider la carte" |
| Web button | **Récupérer mon livre** | "Ajouter à ma bibliothèque"; "Lire maintenant" | — |
| Explanatory sentence | **Ce livre numérique est déjà payé : il ne reste qu'à le lire, sur liseuse, tablette ou téléphone.** | "Un livre numérique déjà réglé, à lire sur liseuse, tablette ou téléphone." | Non-breaking space before the colon |

Register: vous by default in French retail. fr-CA: "livre numérique", "code QR", tu is more accepted [M, medium].

### German (de-DE; also de-AT, de-CH)

| Field | Primary | Alternates | Notes |
|---|---|---|---|
| Product name | **eBook-Geschenk** | "eBook zum Verschenken"; object: "Geschenkumschlag", "Klappkarte" | "Geschenk-eBook" reads as a free eBook; "Geschenkbuch" is a print category; never "Geschenkkarte" or "Gutschein" |
| Checkout option label | **eBook verschenken** | "Als Geschenk kaufen"; "Das ist ein Geschenk" | tolino shops use "Verschenken" [V2-sib] |
| Sub-options | "Karte zum Selbstausdrucken (kostenlos)" / "Gedruckte Karte per Post" | — | — |
| Card headline | **Dieses Buch ist ein Geschenk** + "Von {Name}" | "Ein Buch für dich" / "Ein Buch für Sie"; "{Name} schenkt dir / Ihnen dieses Buch" | — |
| Redemption CTA (print) | **Code scannen und Buch abholen** | du: "Scanne den Code und hol dir dein Buch"; Sie: "Scannen Sie den Code und holen Sie Ihr Buch ab"; typed: "oder Code {…} eingeben auf {url}" | "einlösen" is the standard verb but belongs to "Gutschein" |
| Web button | **Mein eBook abholen** | "Zur Bibliothek hinzufügen"; "Jetzt lesen" | — |
| Explanatory sentence | **Dieses eBook ist bereits bezahlt – einfach lesen: auf eReader, Tablet oder Smartphone.** | "Bereits bezahlt. Lesbar auf eReader, Tablet und Smartphone." | — |

Register: no safe default. German e-commerce has moved towards du, the book trade's older audience still meets Sie [M, medium]. Use the infinitive form on the card and take du/Sie from the reseller's configuration. The chosen strings contain no "ß", so they work unchanged in Switzerland.

### Dutch (nl-NL; nl-BE)

| Field | Primary | Alternates | Notes |
|---|---|---|---|
| Product name | **eBook Cadeau** (unchanged) | "E-book cadeau doen" (descriptor); object: "cadeau-omslag", "kaart" | Never "cadeaubon", "cadeaukaart", "boekenbon"; "cadeauboek" is a print category |
| Checkout option label | **Cadeau doen** | "Dit e-book cadeau doen"; "Geef als cadeau"; "Het is een cadeautje" | "cadeau doen" is the idiom [M, high] |
| Sub-options | "Kaart zelf printen (gratis)" / "Gedrukte kaart per post" | — | — |
| Card headline | **Dit boek is een cadeau** + "Van {naam}" | "Een boek voor jou"; nl-BE formal: "Een boek voor u" | — |
| Redemption CTA (print) | **Scan de code en haal je boek op** | Neutral: "Code scannen en boek ophalen"; typed: "of vul de code {…} in op {url}" | "verzilveren" is the voucher verb; "inwisselen" is milder |
| Web button | **Mijn boek ophalen** | "Toevoegen aan mijn bibliotheek"; "Nu lezen" | — |
| Explanatory sentence | **Dit e-book is al betaald. Lees het op je e-reader, tablet of telefoon.** | "Dit e-book is al betaald en te lezen op e-reader, tablet of telefoon." | — |

Register: je/jij is the norm in Dutch retail; Flanders accepts je, with u in formal settings [M, medium-high].

### Spanish (es-ES; note for Latin America)

| Field | Primary | Alternates | Notes |
|---|---|---|---|
| Product name | **eBook Regalo** | "Regala un eBook"; "Libro electrónico de regalo"; object: "tarjeta-libro", "sobrecubierta de regalo" | "eBook de regalo" is lead-magnet wording; never "tarjeta regalo", "cheque regalo", "bono" |
| Checkout option label | **Regalar este eBook** | "Es para regalar"; "Comprar para regalar" | "¿Es para regalo?" is the shop question [M, high] |
| Sub-options | "Tarjeta para imprimir en casa (gratis)" / "Tarjeta impresa y enviada por correo" | — | — |
| Card headline | **Este libro es un regalo** + "De parte de {nombre}" | "Un libro para ti"; "{Nombre} te regala este libro" | — |
| Redemption CTA (print) | **Escanea el código y recoge tu libro** | Neutral: "Escanear el código para recibir el libro"; typed: "o introduce el código {…} en {url}" | "canjear" is the voucher verb |
| Web button | **Conseguir mi libro** | "Añadir a mi biblioteca"; "Leer ahora" | — |
| Explanatory sentence | **Este eBook ya está pagado. Léelo en tu eReader, tableta o móvil.** | Latin America: "celular" for "móvil" | — |

Register: tú in Spain; usted survives in parts of Latin America, where the infinitive form is safer [M, medium].

### Italian (it-IT)

| Field | Primary | Alternates | Notes |
|---|---|---|---|
| Product name | **eBook Regalo** | "Regala un eBook"; object: "biglietto-libro", "sovraccoperta regalo" | Same string as Spanish. "eBook in regalo / in omaggio" means free; never "buono regalo", "carta regalo", "gift card" |
| Checkout option label | **Regala questo eBook** | "È un regalo"; "Acquista come regalo" | — |
| Sub-options | "Biglietto da stampare a casa (gratis)" / "Biglietto stampato e spedito" | — | — |
| Card headline | **Questo libro è un regalo** + "Da parte di {nome}" | "Un libro per te"; "{Nome} ti regala questo libro" | — |
| Redemption CTA (print) | **Inquadra il codice e ricevi il tuo libro** | Neutral: "Inquadrare il codice per ricevere il libro"; typed: "oppure inserisci il codice {…} su {url}" | "Inquadra" is the usual verb for pointing a camera at a code [M, high]; "riscatta" is the voucher verb |
| Web button | **Ricevi il libro** | "Aggiungi ai miei libri"; "Leggi ora" | — |
| Explanatory sentence | **Questo eBook è già pagato. Leggilo su eReader, tablet o smartphone.** | — | — |

Register: tu is standard online; Lei is rare in e-commerce [M, medium-high].

### English (en-GB, en-IE)

| Field | Primary | Alternates | Notes |
|---|---|---|---|
| Product name | **eBook Gift** | "Gift an eBook"; object: "gift jacket", "book card" | Never "book token", "gift card", "voucher"; "gift book" is a print category |
| Checkout option label | **Give as a gift** | "This is a gift"; "Buy as a gift" | — |
| Sub-options | "Card to print at home (free)" / "Printed card, sent by post" | — | — |
| Card headline | **This book is a gift** + "From {name}" | "A book for you" | — |
| Redemption CTA (print) | **Scan to get your book** | Typed: "or enter code {…} at {url}"; "Redeem" | — |
| Web button | **Get my book** | "Add to my library"; "Read now" | — |
| Explanatory sentence | **This eBook is already paid for. Read it on your e-reader, tablet or phone.** | — | — |

### Portuguese (pt-PT) — secondary, because Fnac also trades in Portugal [M, high]

| Field | Primary | Notes |
|---|---|---|
| Product name | **eBook para Oferecer** | Avoid "cartão oferta", "cheque-oferta", "vale" |
| Checkout option label | **Oferecer este eBook** (alt. "É para oferta") | "É para oferta?" is the shop question [M, high] |
| Card headline | **Este livro é uma prenda** + "De {nome}" | pt-BR: "presente" |
| Redemption CTA | **Aponte a câmara ao código para receber o livro** | pt-BR: "câmera"; voucher verb is "resgatar" |
| Web button | **Receber o meu livro** | — |
| Explanatory sentence | **Este eBook já está pago. Leia-o no seu eReader, tablet ou telemóvel.** | Formal third person is the retail default in Portugal [M, medium] |

Confidence for Portuguese is lower than for the other languages.

## 7. Domain names and URL vocabulary

All [M]; **no availability check was possible**.

- A descriptive domain cannot be defended well. AFNIC's Syreli procedure (.fr) and the UDRP (.com) both require a prior right; a descriptive, unregistered name gives a weak one. Register the obvious variants early instead of relying on dispute procedures: hyphenated and unhyphenated, .fr, .eu, .com. Eligibility is not an issue for a French owner (.fr is open to EU/EFTA residents; .eu to EU residents and citizens).
- Exact-match descriptive domains have brought little ranking benefit since Google's 2012 "EMD" update. Rank with localised how-to pages on the verb queries (section 1, point 4).
- One domain with language paths is easier than one descriptive domain per country, where the natural names ("ebook-verschenken", "regalar-ebook") are probably taken [M, low].
- Suggested path vocabulary: `/cadeau` (FR, NL), `/geschenk` (DE), `/regalo` (ES, IT), `/gift` (EN), `/oferta` (PT). Two words cover four languages.
- The typed-code URL printed on the card must be short, without hyphen, accent or look-alike characters (l/1/I, O/0), because it may be typed on an e-reader keyboard.
- In reseller mode the QR code and the typed URL should be on the **reseller's own domain** (for example `{reseller}/cadeau`). The recipient knows that name, and it reduces QR-phishing suspicion. This is one more reason the generic label matters more than a brand.

## 8. Recommendations for the repository

1. Keep "eBook Cadeau" as project name and as the FR/NL option label. Do not budget for a word mark; decide later on a figurative mark or a distinctive house brand.
2. In `ebc-i18n`, model terminology as keys with per-tenant overrides: `ebook_noun`, `register` (neutral / informal / formal), `gift_verb_label`, `redeem_cta`, `object_noun`. Ship the neutral forms as defaults.
3. Add a lint or test that fails if a shipped string contains a stored-value term from section 4 for its locale.
4. Offer the tu/vous (du/Sie) headline toggle to the giver in standalone mode.
5. Before release: native review of section 6; register searches (INPI, TMview) for "ebook cadeau" and any house brand; WHOIS checks; a look at the live wording of each target reseller.

## Sources

No source was retrieved in this session. The URLs below are copied from sibling notes (`market-business.competitors-costs.md`, `market-business.market.md`, `ereader-ux.ecosystems.md`), where other agents recorded them from their search results.

- https://www.decitre.fr/offrir-un-ebook
- https://www.decitre.fr/faq/numerique/offrir-un-ebook
- https://www.liseuses.net/offrir-des-ebooks-avec-tea/
- https://www.hugendubel.de/de/shortcut/ebooksverschenken
- https://www.heise.de/tipps-tricks/E-Books-verschenken-so-klappt-s-7476189.html
- https://www.amazon.fr/gp/help/customer/display.html?nodeId=GVWGP284MQ6ZRM59
- https://www.idboox.com/news-livres/amazon-on-peut-enfin-offrir-un-livre-numerique-kindle/
- https://www.dealabs.com/discussions/carte-cadeau-kobo-ou-fnac-2273026
- https://publishersweekly.com/pw/print/20120827/53710-livrada-tries-its-luck-with-e-book-gift-cards.html
- https://the-digital-reader.com/ebook-startup-livrada-shut/
- https://shelf-awareness.com/theshelf/2016-01-26/amazon_testing_title-specific_kindle_gift_cards_.html
- https://www.geekwire.com/2016/amazons-other-physical-retail-test-testing-the-kindle-e-book-kiosk-at-the-local-drug-store/
- https://www.mobileread.com/forums/showthread.php?t=363478
- https://authors.bookfunnel.com/help/print-codes/
- https://www.dropcards.com/pricing/
- https://bookwire.net/faq-download-codes
- https://www.legalplace.fr/guides/prix-depot-marque-inpi/
- https://www.legalstart.fr/fiches-pratiques/depot-marque/cout-depot-marque/
- https://www.elly-assurance.fr/actualites/inpi/

Legal instruments cited from memory, without link: Code de la propriété intellectuelle art. L. 711-2 and L. 713-6; Regulation (EU) 2017/1001 arts. 7 and 14; CJEU C-363/99, C-265/00, C-191/01 P; Directive (EU) 2015/2366 art. 3(k) and art. 37(2); Code monétaire et financier art. L. 521-3 and L. 525-5; ZAG § 2; BGB §§ 195, 199; UStG § 3 Abs. 13–15; Directive (EU) 2016/1065; loi n° 2011-590; Buchpreisbindungsgesetz; loi n° 94-665; Irish Consumer Protection (Gift Vouchers) Act 2019; Dutch minimum-validity rule for gift vouchers (2022).

## Could not verify

Everything, in the strict sense. The items that matter most:

1. **Trademark registers**: whether "eBook Cadeau", "ebook cadeau", "Livre Cadeau Numérique", "Geschenk-eBook", "eBook-Geschenk", "Regala un eBook", "eBook Regalo" or "eBook Gift" are registered or applied for at INPI, EUIPO, DPMA, BOIP, OEPM, UIBM or UKIPO, in classes 9, 16, 35, 41.
2. **Existing services or companies** trading under those names; any current French, German, Dutch, Spanish or Italian service that prints a title-specific eBook gift object.
3. **Domain availability** for any variant (.fr, .eu, .com, .de, .nl, .es, .it).
4. **Live retailer wording** in 2026: Fnac/Kobo, 7switch, Cultura, Amazon.de/.es/.it/.nl, Thalia, bol.com, Casa del Libro, IBS, laFeltrinelli, Waterstones; whether each uses tu/vous, du/Sie.
5. **Search demand** for the verb and noun phrases in each language (no keyword tool, no search).
6. **Owners and status of scheme names**: Chèque Lire and Chèque Culture (Groupe Up), Boekenbon, National Book Tokens.
7. **Statutory details**: exact article numbers in the Code monétaire et financier; Dutch two-year rule and its scope for single-product vouchers; Belgian minimum validity; whether France has adopted any statutory validity rule for gift cards; Spanish and Italian rules; current state of the PSD2 successor texts.
8. **Whether a single-title prepaid code is a "voucher" at all** under the VAT Directive and under national consumer rules on gift vouchers — needs a lawyer, not a search.
9. **Native-speaker validation** of every string in section 6, especially Portuguese, the Italian button label and the German du/Sie default.
