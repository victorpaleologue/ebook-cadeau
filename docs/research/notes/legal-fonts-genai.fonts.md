# Legal: fonts on covers — identification, reuse, look-alikes, generative reproduction

Angle: typeface and font law. Date: 2026-11-12. Not legal advice; have a French IP lawyer review before commercial launch.

## 0. Status of this note — read first

**No web search could be run for this note.** Every WebSearch call was refused because the workflow's shared search budget (200 calls per turn) was already spent by other agents. WebFetch is blocked by the sandbox allowlist. Nothing below was cross-checked online in this session.

Confidence tags used:

- **[L]** — verified in this session against a primary document inside this repository (licence texts and font binaries in `/home/user/ebook-cadeau/assets/fonts/`).
- **[M]** — from memory (training knowledge, cutoff mid-2026), not verified in this session. Statute numbers, case names and dates are given so that a later pass can check them.
- **[M?]** — from memory and I am not confident of the detail (date, outcome, exact wording).
- [V1]/[V2] are **not used**: no claim here has web verification. Treat the whole note as a structured checklist for a verification pass, not as verified research.

No URL is cited for any legal source, because none appeared in a search result and the rules forbid guessing URLs. The only URLs printed are ones that appear verbatim in repository files.

## 1. Key findings

1. A **typeface** (the design of the letters) and a **font** (the software file) are protected by different rights. A third object, **custom lettering**, is neither: it is a drawing.
2. In the **USA** the design is free and the software is protected. In the **EU** the design can also be protected: registered design (25 years), unregistered design (3 years, against copying only), and national copyright where the design is original. So a look-alike that is lawful in the USA is not automatically lawful in France. [M]
3. **Identifying** a font from a cover image is lawful. The risks sit in how the classifier was trained and in how the result is presented (trade marks, "imitation of X" claims). [M]
4. **Typesetting the spine in the same commercial font is a licence problem, not a design problem.** A desktop licence covers a person making printed output on a licensed workstation. It does not cover a service that renders for third parties on a server or ships the font to browsers. That needs a server or web-to-print licence, quote-based and sold per font. [M]
5. The only architecture in which a commercial font can be used without the service buying a licence is **bring-your-own-font, processed in the browser only**, with the user's own licence, the `fsType` bits honoured and the text converted to outlines. [M]
6. **Open-licence fonts are the workable default.** The SIL OFL 1.1 allows bundling, embedding, selling with software and use in any document; the document is not bound by the licence. [L]
7. The eight font files already in the repo are all OFL 1.1, all `fsType = 0` (installable embedding), and **none declares a Reserved Font Name**, so subsetting them for WASM or PDF is allowed under their current names. Two carry trade mark notices (Spectral, Bebas Neue). [L]
8. Never extract fonts embedded in the publisher's EPUB or PDF. Never load fonts from Google's CDN at run time in the EU; self-host. [M]
9. Custom title lettering cannot be "retyped". Tracing it or generating missing letters in its style is the riskiest option in this whole topic under French law (reproduction or adaptation of a graphic work, plus *parasitisme*). Fall back to a neutral face. [M]
10. Do not describe substitutes as "our free Helvetica". EU comparative-advertising law prohibits presenting goods as imitations or replicas of trade-marked goods. [M]

## 2. Vocabulary

| Term | What it is | Typical right |
|---|---|---|
| Typeface (*caractère*, *Schrift*) | The visual design of a coordinated set of glyphs | Design right; copyright in some countries |
| Font (*fonte*, *police*) | The file (.otf, .ttf, .woff2): outline data, tables, hinting code | Copyright as software and/or data; contract (EULA) |
| Font name | "Helvetica", "Times New Roman" | Trade mark |
| Lettering | Letters drawn for one title; no reusable file | Copyright as a graphic work |

## 3. Protection regimes

### 3.1 Summary

| Jurisdiction | Typeface design | Font software | Main sources [all M] |
|---|---|---|---|
| USA | Not copyrightable. Design patent possible (15 years from grant for applications filed since 13 May 2015) | Copyrightable as a computer program | 37 CFR § 202.1(e); *Eltra Corp. v. Ringer*, 579 F.2d 294 (4th Cir. 1978); *Adobe Systems v. Southern Software*, N.D. Cal. 1998; 35 U.S.C. § 171 |
| EU | Registered EU design: 5-year periods up to 25 years. Unregistered: 3 years from first disclosure in the EU, against copying only. Typographic typefaces are named as "products" | Not harmonised specifically; Software Directive 2009/24/EC if the font qualifies as a program | Reg. (EC) 6/2002 arts. 3, 11, 12, 19; Reg. (EU) 2024/2822; Dir. (EU) 2024/2823 |
| France | Copyright if original (life + 70 years, moral rights perpetual) and *dessins et modèles* (up to 25 years), cumulable | Copyright; software rules if treated as *logiciel* | CPI L112-2 8°, L511-1, L513-1, L112-2 13°, L122-6 |
| Germany | Design right (up to 25 years). Copyright rarely granted to text faces | Held to be a computer program by one regional court | DesignG § 1; SchriftzG 1981; BGH *Candida-Schrift* 1958; LG Köln 12.1.2000, 28 O 133/97; UrhG § 69a |
| UK | Copyright as artistic work, but use in the ordinary course of printing is not infringement; 25-year cut-off after first marketing | Copyright as a literary work (program) | CDPA 1988 ss. 54, 55, 178 |
| International | Vienna Agreement 1973: never in force | — | Signed 12 June 1973; ratified only by France and Germany, five ratifications needed |

### 3.2 USA [M]

- 37 CFR § 202.1 lists material not subject to copyright; item (e) is "typeface as typeface". Item (a) excludes "mere variations of typographic ornamentation, lettering or coloring".
- The House Report on the 1976 Act (H.R. Rep. 94-1476) states that Congress did not regard typeface designs as copyrightable pictorial or graphic works. *Eltra v. Ringer* upheld the Register's refusal to register a typeface.
- The Copyright Office's 1988 policy decision held digitised typefaces unregistrable; its 1992 rule accepted registration of computer programs that generate typefaces. The registration covers the program, not the letter shapes (Compendium III §§ 723, 906.4).
- *Adobe v. Southern Software* (1998): the choice of outline control points by a font editor is protectable expression. The defendant had loaded Adobe's Utopia into a font tool, shifted the points slightly and renamed it; summary judgment for Adobe. The lesson: redrawing a look-alike from printed specimens is lawful in the USA; deriving one from the font file is not.
- Design patents on "type fonts" are granted routinely (MPEP § 1504.01(a) III instructs examiners not to refuse them for lack of an article of manufacture).
- Enforcement in practice is contractual. Known disputes concern use outside the licence scope: *Font Bureau v. NBC Universal* (2009), *Berthold v. Target* (2017). The closest to this project is ***Laatz v. Zazzle*** (N.D. Cal., filed 2022, no. 5:22-cv-04844): a designer alleged that a print-on-demand platform bought a single-seat licence and installed the font on its servers for all users of its online design tool. Claims included breach of contract, fraud and software copyright. Outcome not verified. [M?]

### 3.3 EU design law [M]

- Reg. 6/2002 art. 3(b) defines "product" as any industrial or handicraft item including "packaging, get-up, graphic symbols and typographic typefaces", excluding computer programs. Locarno class 18-03 is "type and type faces".
- Requirements: novelty and individual character (arts. 5–6). Registered protection lasts up to 25 years (art. 12). Unregistered protection lasts 3 years from first making available within the Union (art. 11) and protects only against copying (art. 19(2)).
- The right covers making, offering, marketing and **using** a product in which the design is incorporated (art. 19). Private non-commercial acts are exempt (art. 20).
- The 2024 reform (Reg. 2024/2822, in force 8 December 2024, applicable in two phases: 1 May 2025 and 1 July 2026; Directive 2024/2823 to be transposed by 9 December 2027) renames the right "EU design", keeps the 25-year and 3-year terms, and rewrites the product definition to cover items "regardless of whether [they are] embodied in a physical object or materialise in a non-physical form", listing "graphic works or symbols, logos, surface patterns, typographic typefaces, and graphical user interfaces". Computer programs stay excluded.
- EU copyright: *Cofemel* (C-683/17, 2019) and *Brompton* (C-833/18, 2020) hold that a design is a "work" if it is the author's own intellectual creation, with no extra artistic threshold. *Mio / konektra* (C-580/23 and C-795/23, judgment 4 December 2025 [M?]) confirmed that no stricter test applies to applied art. This erodes the old German rule that text typefaces are below the copyright threshold.

### 3.4 France [M]

- CPI art. L112-2 lists as works of the mind, at 8°, "les œuvres graphiques et typographiques". A typeface is protected if original (*empreinte de la personnalité de l'auteur*). Term: life + 70 years; moral rights (paternity, integrity) are perpetual and inalienable.
- *Dessins et modèles*: CPI L511-1 names "les symboles graphiques et les caractères typographiques" as products; L513-1 gives 5 years renewable up to 25. Cumulation with copyright (*unité de l'art*) remains, subject to originality.
- France has no equivalent of UK s. 54. In principle, printing a title in an original, protected typeface without a licence reproduces the typographic work, in addition to any software infringement.
- Font files: if treated as *logiciel* (L112-2 13°), the software regime applies: no private-copy exception, only a backup copy (L122-6-1). Whether a French court treats a font file as software, as a database, or only as the carrier of the typographic work is not settled, to my knowledge.
- **Case law: I cannot cite a verified French judgment ruling directly on the originality of a typeface.** The well-known incident is the 2010 Hadopi logo, set in "Bienvenüe", a face drawn by Jean-François Porchez for France Télécom's exclusive use; the logo was withdrawn and redone. It was settled without a judgment that I know of.
- Sibling notes in this folder record that a purely typographic cover may lack originality in France (Zulma/Elior). That concerns layout, not the typeface.

### 3.5 Germany [M]

- *Schriftzeichengesetz* of 6 July 1981 ratified the Vienna Agreement and placed new and distinctive typefaces under the design statute with a term of up to 25 years. The 2004 design reform folded typefaces into the general regime; "typografische Schriftzeichen" are now products under DesignG § 1 no. 2, with a transitional rule in § 61.
- Copyright: BGH, 30 May 1958, I ZR 21/57 (*Candida-Schrift*) refused copyright to a text typeface. The higher threshold for applied art was abandoned in BGH *Geburtstagszug* (13 November 2013, I ZR 143/12), but I know of no later leading case granting copyright to a text face.
- LG Köln, 12 January 2000, 28 O 133/97: digital fonts are computer programs under § 69a UrhG. Criticised in the literature, never overruled as far as I know.
- Unfair competition: § 4 no. 3 UWG (avoidable deception as to origin, exploitation of reputation).

### 3.6 UK [M]

- CDPA s. 54(1): it is not an infringement of copyright in the design of a typeface to use the typeface in the ordinary course of typing, composing text, typesetting or printing, to possess an article for that purpose, or to do anything with material so produced, even where the article used is an infringing copy.
- s. 54(2) keeps liability for making or dealing in articles specifically designed to produce material in the typeface (that is, fonts). s. 55: 25 years after such articles are first marketed with the owner's consent, anyone may make them.
- s. 54 concerns the design copyright only. Copying font software remains infringement of the program.

### 3.7 Trade marks on names [M]

- Names are registered marks: HELVETICA, TIMES NEW ROMAN, ARIAL, FRUTIGER, UNIVERS, GOTHAM (now all in the Monotype group), FUTURA (Bauer). Names of historical models (Garamond, Bodoni, Baskerville, Caslon, Didot) are generic; the prefixed versions are marks.
- This is why clones carry other names: Swiss 721 and Nimbus Sans for Helvetica, Dutch 801 for Times, Book Antiqua for Palatino.
- Referential use of a name is allowed in the EU if it follows honest practices (EUTMR 2017/1001 art. 14(1)(c)). But Directive 2006/114/EC art. 4(g) prohibits comparative advertising that presents goods as imitations or replicas of trade-marked goods (*L'Oréal v Bellure*, C-487/07, 2009). "Metric-compatible with Arial" is customary wording; "a free Helvetica" is not safe.

### 3.8 Unfair competition [M]

- France: *concurrence déloyale* and *parasitisme* under Code civil art. 1240. The Cour de cassation's commercial chamber tightened the test on 26 June 2024 (three judgments, including n° 23-13.535 [M?]): the claimant must identify an individualised economic value and show an intention to ride on it.
- For this project the exposure is less the typeface than the **publisher's series look**: a spine imitating the typography, colours and layout of a recognisable collection can be attacked as *parasitisme* even with licensed fonts.

## 4. Question 1 — Is identifying the font lawful?

Yes. A font's identity is a fact; no right reserves the act of recognising it. [M] Conditions:

| Step | Issue | Assessment [M] |
|---|---|---|
| Analysing the cover image in the browser | Temporary copy of the cover | Covered by the same permission that lets us display the cover; no separate font issue |
| Training a classifier on rendered glyphs of **open** fonts | None | Permitted by OFL and Apache licences |
| Training on renders of **commercial** fonts | Needs a lawful copy; many EULAs restrict automated or ML use | The DSM Directive art. 4 text-and-data-mining exception (France: CPI L122-5-3) applies only if rights are not reserved. Art. 7(1) makes contract terms unenforceable against arts. 3, 5 and 6, **not** art. 4, so a EULA ban is an effective reservation. Avoid |
| Using a third-party identification service | Their terms of use; sending the cover image out | Check terms; prefer in-browser inference |
| Showing "set in Helvetica" to the user | Trade mark | Referential use; add no logo, claim no affiliation |

Recommendation: make the classifier's label space the **open fonts we can ship**, not the universe of commercial fonts. The model then answers "which of our faces is closest", which is the only answer the product can act on, and it is trained only on freely licensed material.

## 5. Question 2 — Typesetting the spine in the same commercial font

### 5.1 Licence types [M; terms as I knew them in mid-2026, to be re-read before any purchase]

| Licence | Covers | Fits this service? |
|---|---|---|
| Desktop | Install on N workstations; static output (print, PDF, images) made by licensed users | No for server or browser rendering. Yes for a user working on their own machine |
| Web font | `@font-face` display on a site, by page views | No: display only, not document generation |
| App / ePub | Embedding in one application or publication | No |
| Server / web-to-print | Font on a server, or in a tool, used by remote users or automated processes to create documents or customised products | **Yes — this is the category.** Quote-based, usually annual and per font; I have no verified price |
| Adobe Fonts (Creative Cloud) | Desktop use by the subscriber; web use through Adobe's hosting | No. The terms exclude installation on servers and use in tools where end users customise products |

### 5.2 Who must hold the licence [M]

| Architecture | Where the font runs | Licence holder |
|---|---|---|
| A. Server renders the PDF | Our server | Us: server licence |
| B. Browser (WASM) renders with a font file we deliver | User's browser; we distribute the file | Us: web-to-print or application licence. A web font licence alone is not enough |
| C. Browser renders with a font file the user supplies (file picker, or Chromium's Local Font Access API); file never uploaded | User's device | The user: their desktop licence |
| D. Publisher or reseller supplies finished spine artwork | Not our concern | Publisher |
| E. Open-licence font bundled by us | Anywhere | Nobody; comply with notice conditions |

The cover designer's or publisher's licence never extends to us. Fonts embedded in the gifted EPUB (often obfuscated) or in a publisher PDF are licensed for that file only; extracting them to set a new title is unlicensed copying. [M]

### 5.3 Embedding permissions: OS/2 `fsType` [M]

| Value | Meaning | Rule for the PDF generator |
|---|---|---|
| 0x0000 | Installable embedding | Embed and subset |
| 0x0002 | Restricted licence: no embedding without permission | Refuse to embed; outline only if the user confirms their licence allows it, otherwise substitute |
| 0x0004 | Preview & print | Embed in a print PDF |
| 0x0008 | Editable embedding | Embed |
| 0x0100 | No subsetting | Embed the whole font, or outline |
| 0x0200 | Bitmap embedding only | Rasterise at print resolution, or substitute |

- `fsType` states the foundry's intent; the licence text prevails. A US court held that the bits are not an access control under the DMCA (*Agfa Monotype v. Adobe*, N.D. Ill. 2005). Honouring them is still the industry norm and the cheapest evidence of good faith.
- The Rust crate `ttf-parser` exposes these bits (`permissions()`, `is_subsetting_allowed()`). [M?]

### 5.4 Converting text to outlines [M]

- Outlining removes the font program from the PDF: nothing is redistributed, embedding bits become moot, and printers that ask for vectorised text are satisfied (see `print-requirements.prepress.md`: Printoclock asks for outlines, Pixartprinting for embedded fonts).
- It does not cure a missing licence. The outlines were produced by running the font, and in France the shapes themselves may be a protected work.
- Costs: no selectable text, slightly heavier small type. Irrelevant for a spine title.

## 6. Question 3 — Open-licence substitutes

### 6.1 SIL OFL 1.1 — verified against the repo copy [L]

Source read: `/home/user/ebook-cadeau/assets/fonts/OFL-Spectral.txt` (the header points to https://openfontlicense.org).

| Point | Text of the licence |
|---|---|
| Grant | "to use, study, copy, merge, embed, modify, redistribute, and sell modified and unmodified copies of the Font Software" |
| Cond. 1 | The font may not "be sold by itself" |
| Cond. 2 | May be "bundled, redistributed and/or sold with any software" if each copy carries the copyright notice and the licence, as text files, headers, or "machine-readable metadata fields within text or binary files" |
| Cond. 3 | No Modified Version may use a Reserved Font Name without written permission |
| Cond. 4 | No use of the authors' names to promote a Modified Version |
| Cond. 5 | The font stays under the OFL; this "does not apply to any document created using the Font Software" |
| Definition | "Modified Version" includes "changing formats or … porting the Font Software to a new environment" |

Consequences for the project:

- A PDF with an embedded OFL subset is a "document": no licence text needed in the PDF, no restriction on selling the printed card. [L for the clause; M for the SIL FAQ reading that document embedding is not redistribution]
- Shipping font files to the browser, or compiling them into the WASM binary, is bundling with software: keep the notice and licence. The fonts' own `name` table carries them (IDs 0, 13, 14 are populated in all eight files [L]); also keep the `OFL-*.txt` files and add a credits page.
- A subset or converted file that we ship **as a font** is a Modified Version. It may keep its name only if no Reserved Font Name is declared.

### 6.2 Audit of the fonts already in the repo [L]

Method: OS/2 and `name` tables read with a script; licence headers read directly.

| File(s) | Licence | fsType | Reserved Font Name declared | Trade mark notice (name ID 7) |
|---|---|---|---|---|
| AtkinsonHyperlegible-{Regular,Italic,Bold}.ttf | OFL 1.1, © 2020 Braille Institute of America | 0x0000 | None | None |
| BebasNeue-Regular.ttf | OFL 1.1, © Dharma Type | 0x0000 | None | "Bebas Neue is a trademark of Dharma Type." |
| CourierPrime-Bold.ttf | OFL 1.1, © 2015 The Courier Prime Project Authors | 0x0000 | None | None |
| Spectral-{Regular,Italic,SemiBold}.ttf | OFL 1.1, © 2017 The Spectral Project Authors | 0x0000 | None | "Spectral is a registered trademark of Production Systems SAS." |

All eight may be embedded, subset and bundled. If the project ever alters the designs (beyond subsetting), rename Spectral and Bebas Neue derivatives because of the trade marks, even though the OFL does not require it here.

### 6.3 Other open licences [M]

| Licence | Examples | Conditions that matter |
|---|---|---|
| Apache 2.0 | Arimo, Tinos, Cousine; older Roboto and Open Sans releases | Keep licence and notices, state changes; no trade mark grant; no naming rule |
| GUST Font License | TeX Gyre | LPPL-based; rename if modified |
| AGPL v3 + font exception | URW base 35 (Artifex distribution) | The exception frees documents; without such an exception a GPL font can taint the embedding document — check each file |
| Bitstream Vera licence | DejaVu | Not sold alone; rename if modified |
| Ubuntu Font Licence 1.0 | Ubuntu | Naming rules |

Google Fonts states that its fonts may be used in commercial products, in print and digital. The licence of each family governs (mostly OFL, some Apache). **Self-host**: LG München I, 20 January 2022, 3 O 17493/20, held that loading fonts from Google's servers discloses the visitor's IP address without a legal basis under the GDPR. [M]

### 6.4 Metric-compatible and look-alike families [M]

| Commercial face | Open substitute | Licence | Relationship |
|---|---|---|---|
| Helvetica / Arial | Liberation Sans, Arimo; Nimbus Sans; TeX Gyre Heros | OFL / Apache; AGPL+exception; GUST | Metric-compatible clones |
| Times / Times New Roman | Liberation Serif, Tinos; Nimbus Roman; TeX Gyre Termes | same | Metric-compatible clones |
| Courier | Liberation Mono, Cousine; TeX Gyre Cursor; Courier Prime | same; OFL | Clones; Courier Prime is a redesign |
| Palatino | TeX Gyre Pagella | GUST | Clone (URW lineage) |
| Calibri / Cambria | Carlito / Caladea | OFL | Metric-compatible, different design |
| Garamond | EB Garamond, Cormorant Garamond | OFL | Revivals of 16th-century models |
| Baskerville, Caslon, Bodoni, Didot | Libre Baskerville, Libre Caslon, Bodoni Moda, GFS Didot | OFL | Revivals of public-domain models |
| Trajan | Cinzel | OFL | Same Roman inscriptional model |
| Futura | Jost | OFL | Same geometric genre |
| Gotham, Proxima Nova | Montserrat | OFL | Same genre, not a copy |
| Helvetica-like UI sans | Inter | OFL | Own design in the neo-grotesque genre |

### 6.5 Are clones lawful?

| | USA | EU / France |
|---|---|---|
| Redrawing a look-alike from printed samples | Lawful unless a design patent is in force | Depends on the age and originality of the model: registered design ≤ 25 years, unregistered ≤ 3 years, copyright life + 70 where original; plus unfair competition |
| Copying or converting the font file | Infringes the software | Infringes the software, and the design where protected |
| Using the original name | Trade mark infringement | Same |

Illustrations [M]: Gill Sans (Eric Gill, d. 1940) is out of copyright everywhere. Futura's designer Paul Renner died in 1956, so any copyright in the design lapses at the end of 2026. Helvetica's Max Miedinger died in 1980: a copyright, where a court would recognise one, runs to 2050. Gotham (2000) and Proxima Nova (2005) have living authors.

For this project we would be **users**, not makers, of the substitutes. Liberation, TeX Gyre and the URW set have been distributed for 15 to 25 years by Red Hat, GUST and Artifex with no challenge known to me. The residual EU risk is lowest for revivals of public-domain models and for original designs in a shared genre, and highest for close copies of recent commercial faces. Choose the catalogue accordingly.

## 7. Question 4 — Custom lettering on covers [M]

- A title drawn by hand or built for one cover is a graphic work (CPI L112-2 7° and 8°; UK CDPA s. 4, "irrespective of artistic quality"). The author is the letterer; the publisher usually holds the economic rights; moral rights stay with the author. In the USA, plain lettering is not registrable, but pictorial lettering can be.
- There is no font to license and nothing to "identify". The classifier should report low confidence.
- Moving the title from the cover image onto the spine is a reproduction of part of the cover, and a crop: covered only by the cover licence, and sensitive for the right of integrity (see the two `legal-covers-reseller` notes).
- Tracing the letters, or synthesising the missing ones in the same style, creates a derivative of the lettering. Style as such is free, but letter-for-letter imitation is not, and *parasitisme* needs no protected right.
- Rule: if the lettering is custom, use a neutral face from our own catalogue, or the publisher's spine artwork.

## 8. Generative reproduction of letterforms — short note [M]

- An image model that extends a cover's lettering to new glyphs does, by design, what section 7 warns against. In the USA the output is free if the source is a typeface; in the EU the output is judged by its similarity to a protected design, whatever the tool.
- Unregistered EU design right bites only on copying; a model conditioned on the cover image copies by construction.
- Training on renders of commercial fonts meets the EULA and art. 4 reservation problem of section 4.
- Lawful uses of image generation here: spine **backgrounds** (colour, texture sampled from the cover's edge, subject to the cover licence) and style transfer among our own open fonts. Keep letterforms typeset, not generated.
- AI Act (Reg. 2024/1689) art. 50 marking duties for synthetic images apply from August 2026, with an exception for assistive editing; whether a generated spine background falls under it, and whether later amendments delayed it, I could not check. [M?]

## 9. Risk matrix

| Option | Copyright / design | Licence breach | Trade mark / unfair competition | Verdict |
|---|---|---|---|---|
| Bundled OFL font, self-hosted, subset in PDF | Low | Low | Low | **Default** |
| Publisher-supplied spine or full jacket file | Low | Low | Low | Best in reseller mode |
| User's own font, browser only, fsType honoured, outlined | Low for us | On the user | Low | Optional feature, with a user warranty |
| Commercial font on our server or shipped to browsers, with server licence | Low | Low | Low | Lawful, cost per font; not viable for the long tail |
| Same, without the right licence | Medium (France) | **High** | Low | Exclude |
| Font extracted from EPUB or PDF | Medium | **High** | Low | Exclude |
| Close clone of a recent commercial face | Medium in the EU | Low | Medium | Avoid |
| Tracing or generating custom lettering | **High** in France | — | **High** | Exclude |

## 10. Implementation rules derived from the above

1. Spine title source, in order: publisher artwork; nearest bundled open font above a similarity threshold; house default by genre (serif, sans, condensed display).
2. Classifier labels are our own catalogue. The UI shows the open font's own name only.
3. Font manifest in the repo: file, licence, copyright line, source, hash (a `SHA256SUMS` file already exists), Reserved Font Name, fsType. Check it in CI.
4. PDF generator: read `fsType`, apply the table in 5.3, refuse any embedding the bits do not allow, and log the decision.
5. Bring-your-own-font stays in the browser; the file is never uploaded; text is outlined; the terms of use carry a licence warranty.
6. No run-time calls to third-party font CDNs.
7. Credits page listing fonts, authors and the OFL text.

## Sources

No web source was consulted (search budget exhausted, see section 0). Sources actually read in this session:

- `/home/user/ebook-cadeau/assets/fonts/OFL-Spectral.txt`, `OFL-BebasNeue.txt`, `OFL-CourierPrime.txt`, `OFL-AtkinsonHyperlegible.txt` — full SIL OFL 1.1 text and copyright headers. URLs printed in those files: https://openfontlicense.org, https://github.com/productiontype/Spectral, https://github.com/quoteunquoteapps/CourierPrime.
- The eight `.ttf` files in `/home/user/ebook-cadeau/assets/fonts/` — OS/2 `fsType` and `name` table entries 0, 1, 7, 13, 14.
- Sibling notes in `/home/user/ebook-cadeau/docs/research/notes/`: `legal-covers-reseller.reseller.md`, `legal-covers-reseller.independent.md`, `print-requirements.prepress.md` (cross-references only).

Legal references cited from memory, to be looked up on Legifrance, EUR-Lex, eCFR, legislation.gov.uk and gesetze-im-internet.de: 37 CFR § 202.1; *Eltra v. Ringer*; *Adobe v. Southern Software*; *Agfa Monotype v. Adobe*; *Laatz v. Zazzle*; Reg. (EC) 6/2002; Reg. (EU) 2024/2822; Dir. (EU) 2024/2823; Dir. (EU) 2019/790 arts. 4 and 7; Dir. 2006/114/EC art. 4(g); Reg. (EU) 2017/1001 art. 14; CJEU C-683/17, C-833/18, C-580/23, C-487/07; CPI L112-2, L122-5-3, L122-6-1, L511-1, L513-1; Code civil art. 1240; DesignG §§ 1, 61; UrhG § 69a; UWG § 4; BGH I ZR 21/57 and I ZR 143/12; LG Köln 28 O 133/97; LG München I 3 O 17493/20; CDPA 1988 ss. 4, 54, 55, 178; Vienna Agreement 1973.

## Could not verify

Everything tagged [M] or [M?]. The items that matter most, in priority order:

1. **Current licence terms and prices** (as of 2026-11) for Monotype/MyFonts server and web-to-print licences, and the exact Adobe Fonts wording on servers and product-customisation tools. No price could be obtained.
2. **French case law** on the originality of typefaces under L112-2 8°: none found or recalled with a checkable citation.
3. *Laatz v. Zazzle*: procedural history and outcome.
4. *Mio / konektra*: exact date and holding; whether any German court has since granted copyright to a text typeface.
5. Exact text and application dates of the amended art. 3 of Reg. 6/2002; French transposition status of Dir. 2024/2823.
6. The three Cour de cassation judgments of 26 June 2024 on *parasitisme*: numbers and wording.
7. Whether commercial font EULAs (Monotype in particular) expressly prohibit machine-learning use, and the licence terms of public font-recognition datasets.
8. The SIL OFL FAQ passages on document embedding, WOFF conversion and subsetting (the licence text itself is verified; the FAQ reading is from memory).
9. Current licences of Roboto, Open Sans, Arimo/Tinos/Cousine, Caladea and the URW base 35 set (several were relicensed over time); check each file before bundling.
10. AI Act art. 50 application date after any 2025–2026 amendments, and its exception for assistive editing.
11. Terms of use of font-identification services and whether any offers an API usable from a commercial product.
12. Death dates used in 6.5 and the `ttf-parser` method names in 5.3.
