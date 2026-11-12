# Spine design — conventions by country and publisher, and how to synthesise a 10 mm spine

Research notes, 2026-11-12. Angle: spine conventions (text direction, elements, order, type size, trade dress, long titles, French typography), legibility of rotated text, relation between spine and front cover, spines for eBook-only titles.

> **STATUS: NOT VERIFIED ONLINE.** The web-search budget shared by all agents of this run was already exhausted when this
> note was started: all five opening queries were refused, so **zero searches were run**. WebFetch is blocked by the sandbox
> allowlist and was not used. Everything below is therefore recalled from memory or computed, and **no URL is cited**,
> because the brief forbids citing a URL that did not appear in a search result. This note is a structured hypothesis sheet
> for the fact-check pass, not a sourced report. Section 13 lists the exact queries to run.

Confidence markers used here (no claim reaches [V1] or [V2]):

- **[M-hi]** from memory, textbook-level and stable, I would be surprised to be wrong
- **[M-med]** from memory, believed correct, details (year, number, name) may be off
- **[M-lo]** from memory, vague or possibly confused; treat as a lead only
- **[calc]** arithmetic or design inference made here from stated assumptions

## 0. Take-aways for the implementation

1. Two conventions exist for a title set along the spine: **descending** (top-to-bottom, text rotated 90° clockwise) and **ascending** (bottom-to-top, rotated 90° counter-clockwise). ISO 6357 and ANSI/NISO Z39.41 prescribe descending; France, Germany, Italy, Spain and most of continental Europe mostly print ascending [M-hi].
2. Choose the direction from the **book** (language / country of publication / ISBN registration group), not from the site locale. A French novel bought on a Dutch shop still gets an ascending spine [calc].
3. Default table: `fr, de, it, es, pt` → ascending; `en, nl, da, sv, nb` → descending; everything else → descending (the ISO rule) with a user toggle [M-med].
4. In the flat outside spread (back | spine | front), ascending = rotate +90° (counter-clockwise), baseline on the front-cover side; descending = rotate −90°, baseline on the back-cover side [calc].
5. Element order on a French paperback spine: collection number at the head, author then title along the spine, publisher mark at the foot [M-med]. For the gift card: no number, no publisher logo (legal note), the service's own small mark at the foot.
6. On 10 mm, with the 6 mm text band already fixed by the prepress notes, use **one line, 9–11 pt mixed case** (or 10–13 pt capitals for short titles), minimum 7.5 pt; about 68 characters fit at 10 pt on a 178 mm-high card [calc].
7. Long titles: drop the subtitle first (standard practice, Chicago Manual of Style says so [M-med]), then shorten the author, then shrink, then two lines at 7.5 pt, and only then truncate at a word boundary with "…".
8. French capitals keep their accents and ligatures (É, À, Ç, Œ) [M-hi]. Never re-capitalise a French title by algorithm; only uppercase it, with Unicode-aware mapping.
9. Rotated text is read markedly slower than horizontal text, stacked upright letters ("marquee" / pillar) slower still, and the two rotation directions do not differ much [M-med]. So: never stack letters, and direction can follow convention without a legibility penalty.
10. No statistics were found (or recalled) on how often real spines continue the cover art versus use a solid band or a series template. The background-class decision must be calibrated on a sample of real flat covers (section 10).

## 1. Vocabulary

| English | French | German | Italian | Spanish | Dutch |
|---|---|---|---|---|---|
| spine | **dos** (popularly, and wrongly, *tranche*) | Buchrücken | dorso, costa | lomo | rug |
| spine title | titre au dos | Rückentitel | titolo sul dorso | título de lomo | rugtitel |
| along the spine / transverse | titre en long / en travers | Längstitel / Quertitel | — | — | — |
| head / tail (foot) | tête / queue (pied) | Kopf / Fuß | testa / piede | cabeza / pie | kop / staart |
| front / back cover | première / quatrième de couverture; premier / second plat | Vorderdeckel / Rückseite | prima / quarta di copertina | cubierta / contracubierta | voorplat / achterplat |
| edges of the book block | tranches (tête, queue, gouttière) | Schnitt | taglio | cortes | snede |
| hinge, joint | mors | Falz, Gelenk | — | cajo | — |
| library spine label | cote, étiquette de cote | Signaturschild | — | tejuelo | — |

All [M-hi] except the Italian/Spanish/Dutch secondary terms [M-med]. In French UI copy "dos" is correct but many users say "tranche"; see the terminology notes.

## 2. Standards

| Standard | Title | What it says (as recalled) | Conf. |
|---|---|---|---|
| **ISO 6357:1985** | Documentation — Spine titles on books and other publications | Defines transverse, descending and ascending spine titles. A transverse (horizontal) title is preferred when the spine is wide enough; otherwise the title must be **descending**, so that it reads normally when the book lies flat, front cover up. Still a current standard (periodically confirmed), single edition. | title and descending rule [M-hi]; "transverse preferred" and status [M-med] |
| ISO 6357, additional provisions | | An **edge title** (title printed on the cover along the spine edge) is recommended when the spine is too narrow for a spine title; a zone at the foot of the spine (about 30 mm) is left free for the library shelf mark. | [M-lo] |
| **ANSI/NISO Z39.41-1997** | Printed Information on Spines | US counterpart, descending title; covers books and containers of other media; lists author, title, publisher as the spine elements; also describes the edge title. Earlier editions 1979 ("Book Spine Formats") and 1990; later reaffirmed, then given "stabilized" status. | title and descending rule [M-hi]; history and status [M-med] |
| BS 6738:1986 | Recommendations for spine titles on books and other publications | British adoption of ISO 6357. | [M-lo] |
| UNE 50-120-92 | Títulos en los lomos de los libros y otras publicaciones | Spanish adoption of ISO 6357, so the official Spanish norm is descending although practice is not. | [M-lo] |
| DIN 1429 | Titelblätter und Einbandbeschriftung von Büchern | German standard on title pages and binding lettering; I do not recall its direction rule. | [M-lo] |
| France (AFNOR) | — | I recall no French standard that fixes the direction; practice is by tradition. | [M-lo] |

Three ways to letter a spine in left-to-right scripts, as named in the English Wikipedia article on bookbinding: **transverse** (horizontal lines across the spine), **along the spine** (rotated), and **pillar** (upright letters stacked one per line) [M-hi]. Pillar lettering is rejected by typographic manuals for Latin script [M-med].

## 3. Direction by country

Rationales usually given [M-hi]: *descending* reads upright when the book lies on a table with its front cover up, and a pile of books stays readable; *ascending* is read by tilting the head to the left, and a shelf run sorted left-to-right is then read in the normal order (the first volume is "on top").

| Area | Prevailing practice | Notes | Conf. |
|---|---|---|---|
| UK, Ireland, USA, Canada (English), Australia, NZ | Descending | Universal today. British spines were mixed before about 1950; early Penguins (1935 onwards) ran bottom-to-top and switched in the late 1940s / early 1950s. | practice [M-hi]; Penguin history [M-lo] |
| Netherlands, Flanders | Descending | Named explicitly by the Wikipedia article alongside the English-speaking countries. | [M-med] |
| Denmark, Norway, Sweden | Descending | Same source. Finland: not known. | [M-med] |
| France, Wallonia, French-speaking Switzerland | **Ascending** | Near-universal in trade publishing, including every poche collection. Exceptions are isolated (co-editions printed with an English edition, some art / technical / comics publishers). | rule [M-hi]; exceptions [M-lo] |
| Quebec / French Canada | Ascending | Listed with continental Europe by Wikipedia; a market next to the US one, so expect a mix. | [M-med] |
| Germany, Austria, German-speaking Switzerland | Ascending in the majority, **not uniform** | German Wikipedia ("Buchrücken") describes bottom-to-top as usual in the German-speaking area. A visible minority (science publishers, international co-editions) prints descending. No share figure known. | majority [M-med]; share unknown |
| Italy | Ascending | | [M-med] |
| Spain | Ascending traditionally, mixed in practice | Style authors (Martínez de Sousa) note the lack of uniformity; the UNE norm says descending. | [M-lo] |
| Portugal, Brazil, Spanish-speaking Latin America | Ascending | Latin America is named by Wikipedia. | [M-med] / Portugal, Brazil [M-lo] |
| Central and Eastern Europe | Unknown / mixed | Not researched. | — |
| Japan, China, Korea | Native vertical script, upright characters | Out of scope. | [M-hi] |

### Deciding the direction in software

Priority order [calc]: (1) a real spine supplied with a flat cover wins; (2) explicit user or reseller setting; (3) country of publication from ONIX (`CountryOfPublication`), or the publisher's country; (4) language of the text; (5) ISBN registration group as a proxy; (6) ISO default = descending.

| ISBN prefix (after 978- unless stated) | Group | Default |
|---|---|---|
| 0, 1; 979-8 | English language; USA | descending |
| 2; 979-10 | French language; France | ascending |
| 3 | German language | ascending |
| 84 | Spain | ascending |
| 88; 979-12 | Italy | ascending |
| 90, 94 | Netherlands and Flanders | descending |
| 87 / 82 / 91 | Denmark / Norway / Sweden | descending |
| 972, 989 / 85, 65 | Portugal / Brazil | ascending |

Group identifiers [M-hi]; the mapping to a direction inherits the confidence of the table above. Group 2 also covers French-language publishers in Belgium, Switzerland and Canada, while Flemish publishers sit in 90/94, so the proxy follows the language split rather than the border.

Geometry, to avoid a 180° bug [calc]: viewed from outside, the flat spread is back cover (left), spine, front cover (right). *Ascending*: text rotated 90° counter-clockwise, reads from tail to head, tops of the letters point to the back cover, baseline on the front-cover side. *Descending*: rotated 90° clockwise, reads from head to tail, tops of the letters point to the front cover. Test: lay the folded card front-up with the spine facing you; a descending title must read upright, an ascending one upside-down.

## 4. Publishers and collections

Direction follows the country rule in every case I can recall; the column is still filled so the fact-check can confirm each one.

| Publisher / collection | Country, start | Spine direction | Spine and trade dress (as recalled) | Conf. (dress) |
|---|---|---|---|---|
| Gallimard **Folio** | FR, 1972 (design by Massin) | Ascending | White ground on cover and spine; collection number at the head; author and title along the spine; "folio" logo at the foot. Spine is a pure series template, independent of the cover picture. | [M-med] |
| Gallimard **Blanche** | FR, 1911 | Ascending | Cream ground, one black and two red rules framing the front, title in red, author in black, *nrf* monogram. Spine = same cream ground, typographic. | [M-med] |
| **Le Livre de Poche** | FR, 1953 | Ascending | Number and logo on the spine; template has changed many times; spine colour partly coded by series. | [M-lo] |
| **Pocket** | FR, 1962 (Presses Pocket) | Ascending | Number; spine colour coded by genre in several periods. | [M-lo] |
| **10/18** | FR, 1962 (name = 10 × 18 cm format) | Ascending | Number, "10/18" mark at the foot; strong coloured spines. | [M-lo] |
| **Actes Sud** / Babel | FR, 1978 / 1989 | Ascending | Narrow 10 × 19 cm format, cream laid paper, cover image in a frame; spine typographic on the same ground, "ACTES SUD" at the foot. | [M-med] |
| **Points** (Seuil) | FR, 1970 | Ascending | Letter-prefixed number (P 1234); colour per sub-series (Essais, Policier…). | [M-lo] |
| Minuit; P.O.L; Série noire; Le Masque | FR | Ascending | White with blue frame and star; white typographic; black and yellow; yellow. All typographic "continuous ground" spines. | [M-med] |
| Bibliothèque de la Pléiade | FR, 1931 | Transverse (horizontal), gilt | Leather colour codes the century (20th tobacco, 19th emerald, 18th blue, 17th Venetian red, 16th brown, Middle Ages violet, Antiquity green). | [M-med] |
| **Suhrkamp** — edition suhrkamp | DE, 1963 (Willy Fleckhaus) | Ascending [M-med] | The 48-colour spectrum: each volume one flat colour, a shelf forms a rainbow. Bibliothek Suhrkamp and suhrkamp taschenbuch are also Fleckhaus templates. | [M-med] |
| **dtv** | DE, 1961 (Celestino Piatti) | Ascending [M-med] | White ground, black grotesque type, one illustration; white typographic spine. Template changed in the 1990s. | [M-med] |
| **Fischer** Taschenbuch | DE, 1952 | Ascending [M-lo] | No distinctive system recalled. | — |
| Reclam Universal-Bibliothek | DE | Ascending [M-lo] | Yellow = German-language texts, red = foreign-language originals, orange = bilingual, blue = study guides, green = source texts. | [M-med] |
| **Penguin** | UK, 1935 (Edward Young; Tschichold 1947–49) | Descending (ascending in the first years [M-lo]) | Horizontal bands; orange fiction, green crime, dark blue biography, cerise travel, red drama, purple essays, yellow miscellaneous, grey world affairs; Pelican pale blue. Penguin Classics: black spine; Modern Classics: eau-de-nil. | codes [M-hi]; current spines [M-med] |
| Fitzcarraldo Editions | UK, 2014 | Descending | Blue (fiction) or white (essays), type only; spine = same flat colour. | [M-hi] |
| **Einaudi** | IT (Munari, Huber) | Ascending [M-med] | White ground, Garamond, ostrich device; white typographic spines across most series. | [M-med] |
| **Adelphi** — Biblioteca Adelphi | IT, 1965 | Ascending [M-med] | Matt pastel ground, framed layout derived from an Aubrey Beardsley design; spine = same pastel ground. | [M-med] |
| **Anagrama** | ES, 1969 (design Julio Vivas) | **Not recalled** | "Panorama de narrativas" yellow; "Narrativas hispánicas" grey; "Compactos" multicoloured. | [M-med] |
| Austral (Espasa-Calpe) | ES/AR, 1937 | Not recalled | Colour by genre (blue novels, green essays, orange biography, violet theatre and poetry, grey classics…). | [M-lo] |

Pattern that matters for the algorithm [calc from the table]: in every pocket collection listed, the spine is a **series template** and does not continue the cover image; in every "typographic" literary collection the spine is simply the **same flat ground** as the cover.

Legal reminder (see `legal-covers-reseller.reseller.md`, §5 and risk rows 4 and 6): reproducing a collection's trade dress or logo on a generated spine is the risky option. The table is for understanding what looks plausible, not for cloning.

## 5. Elements and their order

| Element | Usual place | Gift-card decision |
|---|---|---|
| Collection number | Head of the spine, horizontal digits (French poches); foot in some collections | Omit (no such number for a gift card) |
| Author | First along the reading direction in most French, German and Italian paperbacks; English-language spines vary (author–title or title–author) | Keep; surname alone is acceptable |
| Title | After the author, usually heavier or larger | Keep; main title only |
| Subtitle | Normally left off the spine | Omit |
| Series / volume | Short form ("t. 2", "Vol. II") after the title when the book is part of a cycle | Keep in short form: it decides shelf order |
| Publisher name or device (colophon) | Foot of the spine, horizontal if it fits, always at a fixed height in a series | Text name only, and only with publisher opt-in; otherwise the service's own mark |
| Library zone | Lowest ~30 mm often left quiet [M-lo] | Not needed |

Order and positions are [M-med] for French poches and [M-lo] as a statement about "most" spines elsewhere. The Chicago Manual of Style describes the spine as carrying author (full or surname), title and publisher (often reduced to a device), states that the subtitle is usually omitted, and that when not horizontal the lettering reads from the top down [M-med].

Alignment conventions [M-med, design practice]: text is centred across the spine width; along the length, series templates anchor each element at a fixed distance from head or foot so that a shelf row lines up (numbers level at the top, logos level at the bottom); a one-off trade book centres author and title in the space left. With ascending text, "author first" means the author sits nearer the foot.

Proposed zones for a 10 × 178 mm spine [calc]: head margin 8 mm; optional head mark 10 mm; text zone about 120 mm; foot mark 15–20 mm (max 6 mm wide); foot margin 8 mm. Author and title on one baseline, separated by a 1 em space plus a change of weight or colour, not by punctuation.

## 6. Type sizes on a 10 mm spine

Given the 6 mm text band (2 mm clear each side) set in `print-requirements.folding.md` and `print-requirements.prepress.md`. Assumptions: cap height 0.70 em, x-height 0.48 em, ascender + descender 0.95 em, average advance 0.50 em mixed case and 0.65 em capitals. All [calc].

| Setting | Size | Across the spine | Characters in 120 mm | Use |
|---|---|---|---|---|
| Capitals, short title | 12–13 pt | cap height 3.0–3.2 mm | about 40–43 | titles up to ~25 characters plus surname |
| Mixed case, default | 10 pt | 3.4 mm body, x-height 1.7 mm | about 68 | most titles |
| Mixed case, tight | 8 pt | x-height 1.35 mm | about 85 | long titles |
| Two lines | 7.5 pt, leading 8.25 pt | 5.8 mm in total | 2 × 90 | last resort before truncation |
| Horizontal number | 7 pt tabular | 4 digits = 5.4 mm | — | head or foot mark |

Real mass-market paperbacks with spines of 9–12 mm set titles at roughly 9–12 pt and authors at 8–10 pt [M-med, observation, not measured]. Fluent reading needs an x-height of about 0.2° of visual angle (Legge and Bigelow) [M-med]; a 10 pt title (x-height 1.7 mm) reaches that at about 48 cm and is below it at arm's length, which argues for always using the largest size that fits and capitals for short titles [calc].

A transverse (horizontal) title needs lines of at least 15–20 mm for ordinary words, so it is not an option on 10 mm; only a number, initials or a device up to 6 mm wide can sit horizontally [calc].

## 7. Long titles

No written rule was recalled beyond "the subtitle is usually omitted" (Chicago). Observed practice [M-med]: smaller type, tighter tracking, then two lines once the spine is wider than about 15 mm, author reduced to the surname, series name dropped.

Proposed ladder for the generator, stopping at the first step that fits [calc]:

1. Author + main title at the default size.
2. Drop the subtitle (ONIX gives it separately as `Subtitle`; for raw strings split at " : ", " – ", " — ", ". " or a trailing parenthesis).
3. Author's given names to initials, then surname only; more than two authors → first surname + "et al." (or "Collectif" when the metadata says so).
4. Reduce the size step by step down to 8 pt; allow up to −2 % tracking, no horizontal squeezing below 92 %.
5. Two lines at 7.5 pt, broken at a phrase boundary (after a preposition group, never after an article or before closing punctuation).
6. Truncate at a word boundary with "…", keeping at least the first four words; never truncate the author's surname. Show a warning in the UI and let the user type a short title.

## 8. French typography on the spine (and neighbours)

| Rule | Detail | Conf. |
|---|---|---|
| Accented capitals | Capitals keep accents, cedilla and diaeresis, including the preposition À (Imprimerie nationale *Lexique*; Académie française). "ÉLÉGANCE", "À LA RECHERCHE", "LEÇON". | [M-hi] |
| Ligatures | œ and æ are compulsory, also in capitals: "CŒUR", "ŒUVRES". | [M-hi] |
| Title capitalisation, traditional | Capital on the first word; if the title starts with a definite article, also on the first noun and any adjective before it: *Le Rouge et le Noir*, *La Divine Comédie*, *Les Très Riches Heures*; but *À la recherche du temps perdu*, *Un amour de Swann*. | [M-hi] |
| Title capitalisation, simplified | Capital on the first word and proper nouns only; used by library catalogues and many publishers. Both systems coexist. | [M-med] |
| Consequence | Do not recase mixed-case titles by algorithm; copy the publisher's form. Metadata that arrives in full capitals needs a human check or sentence case as the fallback. | [calc] |
| Punctuation | Typographic apostrophe ’; narrow no-break space before ; ! ?, no-break space before : and inside « ». | [M-hi] |
| Author names | Often capitals or small capitals for the surname; particles keep their case ("de", "du"). | [M-med] |
| Uppercasing in code | Use Unicode case mapping; Rust `str::to_uppercase` is locale-independent: fine for French, maps ß to SS, has no Turkish dotted-i tailoring. | [M-hi] |
| Spanish | Accents kept on capitals; sentence-case titles; ¿ ¡. | [M-hi] |
| German | Nouns capitalised; capital ẞ allowed since 2017, SS still correct. | [M-hi] |
| Dutch | IJ is one letter pair: "IJSSELMEER", initial "IJ". | [M-hi] |
| Italian | Sentence-case titles; È, not E'. | [M-hi] |

## 9. Legibility of rotated text

| Study (recalled) | Finding | Conf. |
|---|---|---|
| Byrne, 2002, "Reading vertical text: rotated vs. marquee" (Human Factors and Ergonomics Society proceedings) | Rotated text is read faster than marquee (stacked upright letters); no reliable difference between rotating up or down. | [M-med] |
| Yu, Park, Gerold, Legge, 2010, "Comparing reading speed for horizontal and vertical English text" (Journal of Vision) | Horizontal fastest; text rotated 90° either way clearly slower (order of a quarter to a third); marquee slowest (order of half); the two rotations similar; smaller visual span proposed as the cause. | direction of effects [M-med]; magnitudes [M-lo] |
| Tinker, 1950s, angular alignment of print | Reading rate falls as lines are rotated away from horizontal. | [M-lo] |
| Koriat and Norman, 1985, "Reading rotated words" | Word identification time grows with rotation angle; whole-word familiarity helps short words. | [M-lo] |

Consequences [calc]: (a) never stack letters; (b) neither direction is intrinsically more legible, so follow convention; (c) short, familiar word shapes survive rotation best, so mixed case is preferable for long titles and capitals only for short ones; (d) keep the title on one line so the eye makes one sweep.

## 10. Spine versus front cover: what real books do

I know of **no published statistics**. Qualitative archetypes [M-med, design practice]:

| Archetype | Description | Where it dominates |
|---|---|---|
| A. Continuous ground | Same flat colour or paper tone on front, spine and back; spine purely typographic | French literary "grand format" (Blanche, Minuit, P.O.L, Actes Sud), Fitzcarraldo, edition suhrkamp, Adelphi |
| B. Series template | Fixed spine layout and colour for the whole collection, whatever the cover picture | All pocket collections (Folio, Livre de Poche, Pocket, 10/18, Points, dtv, Penguin Classics) |
| C. Solid band | Picture stops at the fold; spine in one colour taken from the cover palette (often the darkest or the title colour reversed) | Anglo-American trade paperbacks, thrillers, non-fiction |
| D. Wraparound art | Illustration or photo runs across the spine onto the back | Hardcover jackets, fantasy / SF, children's and art books, comics |
| E. Vignette | Template or band plus a small cropped detail of the cover at the head or foot | Some pocket templates |

Rough expectation for a French catalogue: A and B together are the large majority, C common in translated genre fiction, D a minority [M-lo, impression only].

Decision rule for the generator (thresholds to calibrate) [calc]:

1. Real spine available (flat cover upload, publisher feed) → use it.
2. Analyse the spine-side strip of the front cover (3–5 % of the width), in Oklab:
   - uniform (colour spread below about ΔE 3 over at least 90 % of the height, no edges) → **extend the edge colour** as a flat ground (reproduces archetype A, and is what the real spine most probably looks like);
   - smooth vertical gradient → extend as a gradient;
   - a frame or border detected on the cover → use the frame's outer ground colour;
   - busy strip (photo, illustration, lettering) → **solid band** from the cover palette: prefer the dominant dark colour, or the ground behind the cover title; text in the cover title's colour if contrast ≥ 4.5:1 (target 7:1), else black or white.
3. **Series template** only when the publisher supplies it or opts in; otherwise a neutral house template. Do not imitate a collection.
4. Never mirror, stretch or outpaint the artwork onto the spine by default: besides looking fake, it is rated high-risk in the legal notes (alteration and adaptation). Archetype D is therefore reachable only through rule 1.
5. Let the ground run at least 2 mm past each crease (folding note), and reuse it on the back panel.

`font-id-tech.generation.md` §5 describes the same ladder on the image-processing side; the two should share one classifier.

## 11. Spines for eBook-only titles

- There is no reference object, so plausibility comes from following the conventions of the book's language: direction (section 3), author-then-title order, one line, a foot mark.
- eBook covers are front-only. KDP recommends a 1.6:1 portrait ratio (2560 × 1600 px) [M-hi]; the poche trim 110 × 178 mm is 1.618:1, so such a cover fits with about 1 % of crop or margin [calc]. Print-derived EPUB covers have the print ratio (roughly 1.45–1.65).
- A 10 mm spine corresponds to a 150–250-page paperback (folding note §3.3), a believable size for a novel.
- Use rule 2 of section 10 for the ground; set author and title in the matched open font (font notes) or a neutral serif.
- Foot: publisher name as plain text only with consent; otherwise the service's mark. A small head mark saying this is a digital gift (pictogram, not a fake collection number) keeps the object honest: it should look like a book on a shelf without passing for a pirated edition.
- Self-published authors can be offered a "spine editor" (short title, colours, direction toggle), since they own all the rights.

## 12. Recommended defaults

| Parameter | Default | Basis |
|---|---|---|
| Direction | from the book's language / country; fr → ascending; unknown → descending; user toggle | §3 |
| Order | author, then title; series volume in short form | §5 |
| Layout | one line, centred across the spine; anchored zones along it | §5 |
| Size | largest that fits from 13 pt down to 8 pt; two lines at 7.5 pt as last resort | §6 |
| Case | as supplied; capitals only for short titles; accents and ligatures preserved | §8 |
| Subtitle | omitted | §7 |
| Ground | edge-colour extension, else solid palette band; no artwork extension | §10 |
| Marks | no collection number, no publisher logo; service mark at the foot | §5, legal notes |
| Never | stacked upright letters; hard colour edge exactly on the crease | §9, folding note |

## 13. Queries for the fact-check pass (none could be run)

- `ISO 6357 spine titles descending transverse "edge title"`; `ANSI/NISO Z39.41 printed information on spines stabilized`
- `sens de lecture du titre au dos des livres France de bas en haut`; `"dos" livre titre "de haut en bas" éditeurs français exception`
- `Buchrücken Beschriftung Leserichtung von unten nach oben DIN 1429`; `Rückentitel Richtung Verlage Statistik`
- `lomo libro título sentido de lectura UNE 50120 Martínez de Sousa`; `dorso libro titolo verso di lettura`
- `rugtitel leesrichting boven naar beneden`; `Penguin spine lettering direction changed bottom to top 1940s`
- `Byrne 2002 reading vertical text rotated marquee`; `Yu Park Gerold Legge 2010 vertical text reading speed`
- `Chicago Manual of Style spine subtitle omitted reads from the top down`
- `book cover design spine wraparound vs solid colour statistics`; `charte graphique dos collection Folio Massin`
- Photographs of shelves per publisher (Folio, Livre de Poche, Pocket, 10/18, Points, Suhrkamp, dtv, Fischer, Einaudi, Adelphi, Anagrama) to confirm direction, order and foot marks.

## Sources

No web source was consulted in this session (search budget exhausted before the first query; WebFetch blocked). **No URL is given, on purpose.** References recalled from memory, to be located and checked:

- ISO 6357:1985, *Documentation — Spine titles on books and other publications*.
- ANSI/NISO Z39.41-1997, *Printed Information on Spines*.
- Wikipedia (English), "Bookbinding", section on spine titling; Wikipedia (German), "Buchrücken"; Wikipedia (French), "Dos (livre)".
- *The Chicago Manual of Style*, chapter 1, sections on covers and spines.
- *Lexique des règles typographiques en usage à l'Imprimerie nationale* (accented capitals, title capitalisation).
- M. D. Byrne (2002), "Reading vertical text: rotated vs. marquee", Proceedings of the Human Factors and Ergonomics Society.
- D. Yu, H. Park, D. Gerold, G. E. Legge (2010), "Comparing reading speed for horizontal and vertical English text", *Journal of Vision*.
- G. E. Legge, C. A. Bigelow (2011), "Does print size matter for reading?", *Journal of Vision*.
- P. Baines, *Penguin by Design* (2005); R. Calasso, *L'impronta dell'editore* (Adelphi design); J. Hochuli and R. Kinross, *Designing Books*.

Sibling notes in this repository used for consistency (their own sources are listed there):

- `docs/research/notes/print-requirements.folding.md` (spine geometry, 6 mm text band, tolerance, page count for 10 mm)
- `docs/research/notes/print-requirements.prepress.md` (safety margins)
- `docs/research/notes/font-id-tech.generation.md` (background ladder, sizes, contrast)
- `docs/research/notes/cover-sources.sources.md` (no public source of spines by ISBN)
- `docs/research/notes/legal-covers-reseller.reseller.md` (logo, trade dress and artwork-extension risks)

## Could not verify

Nothing in this note was verified online. The items most worth checking, by impact:

1. **Exact wording of ISO 6357 and Z39.41**: "transverse preferred", the edge title, the reserved library zone and its size, the current status of both standards.
2. **Direction per publisher**: Suhrkamp, dtv, Fischer, Reclam, Einaudi, Adelphi are assumed ascending from the country rule; **Anagrama and Spanish practice in general are unknown**. French exceptions (comics, technical, co-editions) not identified.
3. **Share of ascending versus descending in Germany and Spain**: no figure recalled.
4. Direction in Finland, Portugal, Brazil, Quebec, Central and Eastern Europe.
5. National standards: BS 6738, UNE 50-120, DIN 1429, any AFNOR text.
6. Penguin's early ascending spines and the date of the switch; Tschichold's and Hochuli's stated positions on direction.
7. Element order and positions on current Folio, Livre de Poche, Pocket, 10/18, Points and Babel spines (number at head, logo at foot, author before title) and their genre colour codes.
8. Magnitudes in the reading-speed studies (Yu et al., Byrne) and the 0.2° critical print size.
9. Typical point sizes on real 9–12 mm spines: observation only, nothing measured.
10. **Any statistics** on spine/cover relation (continuous ground, template, solid band, wraparound). None known; a sample of about 200 flat covers has to be classified by the project itself.
11. Chicago Manual of Style section numbers and exact wording on the subtitle and the direction.
12. Publisher trade-dress details and dates in section 4 (designer names, years, colour codes).
