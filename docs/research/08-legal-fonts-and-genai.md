# Legal: fonts on covers — identification, reuse, look-alikes, and generative AI reproduction

_Status: draft — research date 2026-11-12_

This document is research to brief a lawyer. It is not legal advice. A French IP lawyer should review it before commercial launch.

## 0. Verification status — read first

- **No statement about font law or AI law in this document is web-verified.** Both researcher notes were written without any web search: the workflow's shared budget of 200 searches was spent before they ran. The lead analyst sent five verification searches on 2026-11-12; all were refused for the same reason. WebFetch is blocked by the sandbox. No workaround was used.
- For that reason no URL is cited for statutes, cases, licence terms or prices. None appeared in a search result, and guessing URLs is not allowed. Formal citations (article, case name, court, date) are given so that each item can be looked up. Section 10 lists what to verify first.
- **Verified in this session, against files in this repository:** the OFL 1.1 licence texts and the eight font binaries in `/home/user/ebook-cadeau/assets/fonts/` (checksums, `fsType`, `name` table). The lead analyst re-ran these checks on 2026-11-12.
- **No price was obtained** for any commercial font licence (server, web-to-print, application). Section 4 says so again where it matters.

Confidence markers:

| Marker | Meaning |
|---|---|
| [V2] / [V1] | Verified on the web in two or more / one source. Used here **only** for facts carried over from sibling notes of this folder, with the link those notes recorded. Each such claim says "sibling note". |
| [L] | Verified in this session against a file in this repository |
| [M] | From memory (knowledge up to mid-2026), not verified in this session |
| [M?] | From memory, and the detail (date, outcome, wording, number) may be wrong |

The genai note used a fifth marker, [M+] ("memory, high confidence"). It is folded into [M] here. No marker was upgraded.

Source notes: `/home/user/ebook-cadeau/docs/research/notes/legal-fonts-genai.fonts.md` and `/home/user/ebook-cadeau/docs/research/notes/legal-fonts-genai.genai.md`.

## 1. Executive summary

### 1.1 Findings

1. **Three legal objects.** A *typeface* is the design of the letters. A *font* is the software file. *Custom lettering* is a drawing made for one title. Each is protected by different rights. The *name* of a font is a fourth object, protected as a trade mark. [M]
2. **Lawful in the USA does not mean lawful in France.** In the USA the letter shapes are free and only the font software is protected. In the EU the shapes can be protected by a registered design (up to 25 years), by the unregistered EU design (3 years, against copying only) and, in France, by copyright if the design is original (author's life + 70 years). The law of the country where protection is claimed applies. [M]
3. **Identifying the font on a cover is lawful.** The risks are in the training data of the classifier and in how the result is presented (trade marks, "imitation of X" wording). [M]
4. **Setting the spine in the same commercial font is a licence problem.** A desktop licence or an Adobe Fonts subscription does not cover a service that renders documents for third parties. The service would need a server or web-to-print licence, sold per family and on quote. No price could be obtained. With hundreds of different title fonts in a catalogue, this route does not scale. [M]
5. **Bring-your-own-font, in the browser only, is the one route to a commercial font that needs no licence held by us.** The user's own licence covers it; the file is never uploaded; the text is converted to outlines. [M]
6. **Open-licence fonts are the workable default.** The OFL 1.1 text in the repository allows bundling, embedding, subsetting and selling with software, and states that documents made with the font are not bound by the licence. [L] The eight bundled font files are all OFL 1.1, all `fsType` 0x0000, and none declares a Reserved Font Name. [L]
7. **Custom lettering cannot be "retyped".** Tracing it, cropping it out or re-drawing it with AI reproduces or adapts a graphic work. Fall back to a house face, or use artwork supplied by the publisher. [M]
8. **The spine title is the same string as the cover title.** An AI "style transfer" therefore re-draws the same letters in the same forms. Legally this is close to reproducing the lettering, not to borrowing a style. [M, reasoning]
9. **Extending the artwork onto the spine is the riskiest operation in the product.** Outpainting, stretching and mirroring all alter the work. In France the right of integrity belongs to the artist and a publisher cannot waive it. Only a flat colour sampled from the cover is clean without consent. [M; French case law on cropping and mirroring: V2, sibling note, see §6]
10. **Train only on open-licence fonts.** The OFL grants "use, study, copy, merge, embed, modify" with no field-of-use limit. [L] A EULA clause against machine-learning use is a valid reservation under the EU text-and-data-mining exception for commercial users. [M] The published font-generation checkpoints have undocumented training data. [M]
11. **Running the model in the browser helps in one case only.** For print-at-home, the user makes the copy. It gives no "mere tool" shelter for a feature built to imitate covers, it does not remove AI Act duties, and it stops helping when the PDF is sent to us or to a printer. It also makes us the distributor of the model weights. [M]
12. **No court ruling on AI-generated fonts or lettering is known**, anywhere, as of mid-2026. [M?] The analysis of AI output is reasoning from general rules.
13. **The gain is small and the legal difference is large.** On a 10 mm spine the title's cap height is about 5–7 mm. At that size a good open look-alike is hard to tell from the original. The step from rung 2 to rung 5 of the ladder below buys little and costs a lot. [M, reasoning]

### 1.2 Risk ladder

Risk is for the service, in France and the EU, with a commercial print run. "Typeset" means a title set in a font. "Lettered" means a hand-drawn title or a title logo. Rungs 0–6 come from the genai note (§6); rungs 3b, 3c and X come from the fonts note (§9).

| Rung | Option | What is taken from third parties | Risk (typeset / lettered) | Status in the product |
|---|---|---|---|---|
| 0 | Spine or flat cover file (*couverture à plat*) supplied by the publisher or reseller | The publisher's own artwork, with consent | Lowest | Preferred in reseller mode |
| 1 | Open-licence house font on a flat colour | Nothing but a colour | Very low | **Default fallback** |
| 2 | Nearest open-licence look-alike on a flat colour | General appearance only | Low / low–medium | **Default when the title is typeset and the match is good** |
| 3a | Commercial font, rendered to outlines by its licensee (publisher or reseller sends the title as SVG or PDF) | Font software, used by its licensee | Low if the licence fits; medium otherwise | Accepted as input. We never receive the font file |
| 3b | User's own font file, processed in the browser only, outlined | Font software, under the user's licence | Low for us; licence risk on the user | Optional, standalone mode only, with a warranty |
| 3c | Commercial font licensed by us (server or web-to-print licence) | Font software, under our licence | Low | Lawful, but priced per family. Not viable for the long tail |
| 4 | Crop of the lettering pixels from the cover | Part of the cover as printed | Medium / medium–high | Only with a cover licence that covers the card and a per-title publisher opt-in. Not in the default pipeline |
| 5 | AI re-rendering of the lettering ("style transfer") | The letterforms, re-drawn | Medium–high / **high** | Off. "Own rights" mode only |
| 6 | AI outpainting of the artwork onto the spine; also stretch or mirror of edge pixels | The illustration, extended or altered | **High** / **high** | Off. "Own rights" mode only, with the artist's consent |
| X | Commercial font without the right licence; font extracted from the EPUB or a publisher PDF; clone derived from a font file; tracing third-party lettering; model trained on proprietary fonts; third-party checkpoint with undocumented training data | — | **High** | Excluded. No opt-in |

"Own rights" mode means: the person who uploads the cover declares that they are the author of the cover and of its lettering, or hold written consent from those authors, and warrants it.

### 1.3 Recommended default behaviour of the spine generator

1. **If the publisher or reseller supplied a spine or a flat cover file, use it.** Scale it proportionally. Do not crop, stretch or recolour it.
2. **Otherwise build the spine from our own material:**
   - Background: one flat colour sampled from the cover. No cover pixels on the spine.
   - Title and author: typeset in an open-licence font, drawn as outlines.
   - If the classifier says the cover title is *typeset* and the best match in our open-font index is good, use that open font (rung 2).
   - If the title is *lettered*, is a *logo*, or the match is poor, use a house face chosen by class (rung 1). Do not pick the nearest imitation of a hand-drawn title.
3. **Never by default:** cover pixels on the spine, AI lettering, outpainting, stretch or mirror, commercial fonts, commercial font names in the interface.
4. **Opt-in paths, each with its own gate:** bring-your-own-font (browser only, warranty); "own rights" mode for AI lettering and outpainting (warranty, user confirmation, AI label); per-title publisher opt-in in reseller mode.

### 1.4 Where the two notes disagree, and what is kept

| Point | Fonts note | Genai note | Kept, and why |
|---|---|---|---|
| Spine backgrounds taken or generated from the cover | Lawful use of image generation: "colour, texture sampled from the cover's edge, subject to the cover licence" (§8) | Only a flat colour is clean without consent; stretch, mirror and outpainting are the same alteration (§2.4) | **Genai note.** A cover licence comes from the publisher, and the publisher cannot waive the artist's right of integrity [M]. French courts have treated cropping, mirroring, colourisation and cut-outs as violations [V2, sibling note]. A procedural texture of our own that takes only a colour from the cover remains allowed |
| Tracing or generating third-party lettering | "Exclude" (§9) | Allowed in an "own rights" mode or on per-title publisher opt-in (§6) | **Both.** Excluded for third-party covers by default. Allowed only when the uploader holds the rights. For *lettered* titles a publisher opt-in is not enough, because the letterer's moral right is not the publisher's to waive; the publisher should supply the artwork instead (rung 0) |
| Subsetting and Reserved Font Names | A subset embedded in a PDF is a "document"; a subset shipped *as a font* is a Modified Version and may keep its name only if no Reserved Font Name is declared (§6.1) | "Keep originals unmodified because format conversion or subsetting makes a Modified Version" (§6, rung 1) | **Fonts note's reading, genai note's practice.** Ship the original files unmodified (they are today [L]). Subsetting inside the PDF is free. If we ever ship subset or converted font files, CI must check that the family declares no Reserved Font Name |
| Showing the identified commercial font's name | Referential use is allowed; but "the UI shows the open font's own name only" (§4, §10) | "Never name a commercial font in the UI" (§6) | **Never show it.** Naming is lawful in itself, but it invites the "imitation of X" reading that EU comparative-advertising law prohibits, and we gain nothing from it |
| `fsType` 0x0002 (restricted) and 0x0200 (bitmap only) on a user-supplied font | 0x0002: outline only if the user confirms their licence allows it. 0x0200: rasterise at print resolution, or substitute (§5.3) | — | **Stricter: refuse both.** The existing code already documents "draw nothing with it in a PDF" (`crates/ebc-text/src/font.rs`) [L]. Such fonts are rare. An override can be added after counsel's opinion |
| German *Schriftzeichengesetz* | Folded into general design law by the 2004 reform; transitional rule in DesignG § 61 | "Repealed in 2004" | Compatible: the special typeface rules were repealed and typefaces moved into general design law. Whether the article ratifying the Vienna Agreement formally remains is not verified [M?] |
| AI Act art. 50 start date | "From August 2026" [M?] | "2 August 2026"; Digital Omnibus outcome unknown [M?] | 2 August 2026 as the nominal date [M]. Plan as if it applies now: a system first placed on the market after that date gets no transitional relief under any version we know of [M?] |

## 2. Typeface versus font; protection by jurisdiction

### 2.1 Vocabulary

| Term | What it is | Typical right |
|---|---|---|
| Typeface (*caractère*, *Schrift*) | The visual design of a coordinated set of glyphs | Design right; copyright in some countries |
| Font (*fonte*, *police*) | The file (.otf, .ttf, .woff2): outline data, tables, hinting code | Copyright as software or data; contract (EULA) |
| Font name | "Helvetica", "Times New Roman" | Trade mark |
| Lettering | Letters drawn for one title; no reusable file | Copyright as a graphic work |

### 2.2 Summary table

All cells are [M] unless marked.

| Jurisdiction | Typeface design (the shapes) | Font file | Hand lettering / title logo | Is printing a title in the face a restricted act? | Main sources |
|---|---|---|---|---|---|
| **USA** | Not copyrightable. Design patent possible: 15 years from grant for applications filed since 13 May 2015 | Copyrightable as a computer program | Usually refused as "mere lettering", unless there is separable pictorial authorship | No, unless a design patent is in force | 37 CFR § 202.1(a), (e); *Eltra Corp. v. Ringer*, 579 F.2d 294 (4th Cir. 1978); *Adobe Systems v. Southern Software* (N.D. Cal. 1998); 35 U.S.C. § 171; Compendium (Third) §§ 723, 906.4 |
| **EU** | "Typographic typefaces" are named as products. Registered design: 5-year periods up to 25 years. Unregistered: 3 years from first disclosure in the EU, against copying only. Copyright if the design is the author's own intellectual creation | Excluded from design law. Software Directive 2009/24/EC to the extent the font is a program | Graphic work if original | In principle yes: the design right covers "using" a product that incorporates the design, and the text has no typesetting carve-out. Private non-commercial acts are exempt. Untested [M?] | Reg. (EC) 6/2002 arts. 3, 10, 11, 12, 19, 20; Reg. (EU) 2024/2822; Dir. (EU) 2024/2823; CJEU C-683/17 *Cofemel*, C-833/18 *Brompton*, C-580/23 and C-795/23 *Mio/konektra* |
| **France** | Copyright if original (life + 70 years; moral rights perpetual and inalienable). *Dessins et modèles*: 5 years, renewable up to 25. Both can apply together | Copyright. Software regime if treated as *logiciel*; not settled | Graphic work; moral rights under L121-1 | In principle yes, where the face is original: France has no equivalent of UK s. 54. No judgment verified | CPI L112-2 7°, 8° and 13°, L121-1, L122-4, L122-6-1, L511-1, L513-1, L513-4 |
| **Germany** | Design right up to 25 years. Copyright rarely granted to text faces | Held to be a computer program by one regional court [M?] | Work if original | Under design law, in principle yes | DesignG §§ 1, 61; SchriftzG 1981; BGH, 30 May 1958, I ZR 21/57 *Candida-Schrift*; BGH, 13 Nov 2013, I ZR 143/12 *Geburtstagszug*; LG Köln, 12 Jan 2000, 28 O 133/97; UrhG § 69a |
| **UK** | Copyright as an artistic work, but use "in the ordinary course of typing, composing text, typesetting or printing" is not infringement, even with an infringing font. 25-year cut-off after first marketing | Copyright as a program | Artistic work, "irrespective of artistic quality" | No (s. 54) | CDPA 1988 ss. 4, 54, 55, 178 |
| **International** | Vienna Agreement for the Protection of Type Faces, 1973: never in force (ratified by France and Germany only; five ratifications needed) | — | — | Applicable law is that of the country where protection is claimed (Rome II Reg. 864/2007 art. 8(1)) | — |

### 2.3 USA [M]

- 37 CFR § 202.1 lists material not subject to copyright. Item (e) is "typeface as typeface". Item (a) excludes "mere variations of typographic ornamentation, lettering or coloring".
- The House Report on the 1976 Act (H.R. Rep. 94-1476) says Congress did not regard typeface designs as copyrightable. *Eltra v. Ringer* upheld the refusal to register a typeface.
- The Copyright Office's 1988 policy decision held digitised typefaces unregistrable. Its 1992 rule accepted registration of computer programs that generate typefaces. The registration covers the program, not the letter shapes.
- *Adobe v. Southern Software* (1998): the choice of outline control points by a font editor is protectable expression. The defendant had loaded Adobe's Utopia into a font tool, shifted the points and renamed it. **Lesson: in the USA, redrawing a look-alike from printed specimens is lawful; deriving one from the font file is not.**
- Design patents on "type fonts" are granted routinely (MPEP § 1504.01(a)).
- Enforcement is contractual in practice. Known disputes are about use outside the licence scope: *Font Bureau v. NBC Universal* (2009), *Berthold v. Target* (2017), and *Laatz v. Zazzle* (see §4.5).

### 2.4 EU design law [M]

- Reg. 6/2002 art. 3(b) defines "product" as any industrial or handicraft item, including "packaging, get-up, graphic symbols and typographic typefaces", excluding computer programs. Locarno class 18-03 is "type and type faces".
- Requirements: novelty and individual character (arts. 5–6). Scope: any design that does not produce a different overall impression on the informed user, taking the designer's degree of freedom into account (art. 10).
- The right covers making, offering, putting on the market and **using** a product in which the design is incorporated (art. 19). Private non-commercial acts are exempt (art. 20(1)(a)).
- The unregistered right protects only against copying. Use is not copying if it results from independent creation by a designer who can reasonably be thought not to know the protected design (art. 19(2)).
- For a registered typeface design, the EUIPO representation consisted of the full alphabet in upper and lower case, the numerals, and five lines of text at 16 pt (Implementing Reg. 2245/2002 art. 4(4), as it stood before the reform) [M?]. So the registered "design" is the whole character set. Whether a title of 10–30 glyphs reproduces it is an open question; no case is known.
- **2024 reform.** Reg. (EU) 2024/2822 entered into force on 8 December 2024 and applies in two phases: 1 May 2025 and 1 July 2026. Directive (EU) 2024/2823 must be transposed by 9 December 2027. The reform:
  - renames the right "EU design" and keeps the 25-year and 3-year terms;
  - rewrites the product definition to cover items "regardless of whether [they are] embodied in a physical object or materialise in a non-physical form", listing "graphic works or symbols, logos, surface patterns, typographic typefaces, and graphical user interfaces"; computer programs stay excluded;
  - adds as an infringing act the creating, downloading, copying or sharing of "any medium or software which records the design" for the purpose of making a product (new art. 19(2)(d));
  - adds permitted uses for referential use and for comment, critique or parody.
- The old German typeface law and the Vienna Agreement both aimed at the making of fonts, not at typesetting [M?]. EU design law has no such limit in its text. This is question 1 for counsel.

### 2.5 EU copyright [M]

- *Cofemel* (C-683/17, 2019) and *Brompton* (C-833/18, 2020): a design is a "work" if it is the author's own intellectual creation. No extra artistic threshold applies.
- *Mio / konektra* (C-580/23 and C-795/23, judgment of 4 December 2025 [M? for the date]): no stricter test for applied art; the copyright test is recognisable reproduction of creative elements, not "overall impression". This erodes the old German rule that text typefaces fall below the copyright threshold.
- Partial reproduction is enough if the part taken carries the originality (*Infopaq*, C-5/08; *Pelham*, C-476/17).

### 2.6 France [M]

- CPI art. L112-2 lists as works of the mind, at 8°, "les œuvres graphiques et typographiques". A typeface is protected if original (*empreinte de la personnalité de l'auteur*). Term: life + 70 years. Moral rights (paternity, integrity) are perpetual and inalienable (L121-1).
- *Dessins et modèles*: L511-1 names "les symboles graphiques et les caractères typographiques" as products. L513-1 gives 5 years, renewable up to 25. L513-4 prohibits, without consent, making, offering, marketing, importing and "l'utilisation" of a product incorporating the design.
- Font files: if treated as *logiciel* (L112-2 13°), the software regime applies. There is then no private-copy exception, only a backup copy (L122-6-1). Whether a French court treats a font file as software, as a database, or as the carrier of the typographic work is not settled, to our knowledge.
- **Case law: no verified French judgment on the originality of a typeface could be cited by either researcher.** The known incident is the 2010 Hadopi logo, set in "Bienvenüe", a face drawn by Jean-François Porchez for France Télécom's exclusive use. The logo was withdrawn and redone. It ended without a judgment that we know of. It shows that font misuse in France has a reputational cost independent of damages.
- A purely typographic cover may lack originality in France: CA Versailles, 22 March 2022, *Zulma v Elior* denied copyright in a cover but still found *parasitisme* ([Fournol, originality](https://www.fournol-avocat.fr/actualite/2022/8/17/la-difficile-reconnaissance-de-loriginalit-des-couvertures-de-livre), [Fournol, parasitisme](https://www.fournol-avocat.fr/actualite/2022/8/18/parasitisme-reconnu-pour-la-couverture-dun-livre)) [V1, sibling note `legal-covers-reseller.independent.md`]. That decision concerns the layout, not the typeface.

### 2.7 Germany [M]

- The *Schriftzeichengesetz* of 6 July 1981 ratified the Vienna Agreement and placed new and distinctive typefaces under design law, for up to 25 years. The 2004 design reform moved typefaces into the general regime: "typografische Schriftzeichen" are products under DesignG § 1 no. 2, with a transitional rule in § 61.
- Copyright: BGH *Candida-Schrift* (1958) refused copyright to a text typeface. The higher threshold for applied art was abandoned in BGH *Geburtstagszug* (2013). No later leading case granting copyright to a text face is known to us.
- LG Köln, 12 January 2000: digital fonts are computer programs under § 69a UrhG. Criticised, never overruled as far as we know. [M?]
- Unfair competition: § 4 no. 3 UWG (avoidable deception as to origin, exploitation of reputation).

### 2.8 UK [M]

- CDPA s. 54(1): it is not an infringement of copyright in the design of a typeface to use it in the ordinary course of typing, composing text, typesetting or printing, to possess an article for that purpose, or to do anything with material so produced, even where the article used is an infringing copy.
- s. 54(2) keeps liability for making or dealing in articles "specifically designed or adapted for producing material in a particular typeface" (fonts). s. 55: 25 years after such articles are first marketed with consent, anyone may make them.
- s. 54 concerns the design copyright only. Copying font software still infringes the program.

### 2.9 Trade marks on names [M]

- Names are registered marks: HELVETICA, TIMES NEW ROMAN, ARIAL, FRUTIGER, UNIVERS, GOTHAM (all now in the Monotype group), FUTURA (Bauer). Names of historical models (Garamond, Bodoni, Baskerville, Caslon, Didot) are generic; the prefixed versions are marks.
- This is why clones carry other names: Swiss 721 and Nimbus Sans for Helvetica, Dutch 801 for Times, Book Antiqua for Palatino.
- Referential use of a name is allowed in the EU if it follows honest practices (EUTMR 2017/1001 art. 14(1)(c)). But Directive 2006/114/EC art. 4(g) prohibits comparative advertising that presents goods as imitations or replicas of goods bearing a protected mark (*L'Oréal v Bellure*, C-487/07, 2009). The French transposition is in the Code de la consommation, arts. L122-1 and following [M?]. "Metric-compatible with Arial" is customary wording. "A free Helvetica" is not safe.

### 2.10 Unfair competition [M]

- France: *concurrence déloyale* and *parasitisme* under Code civil art. 1240. The commercial chamber of the Cour de cassation tightened the test on 26 June 2024 (three judgments, including n° 23-13.535 [M?]): the claimant must identify an individualised economic value and show an intention to ride on it.
- For this project the exposure is less the typeface than the **publisher's series look**. A spine that imitates the typography, colours and layout of a recognisable collection can be attacked as *parasitisme* even with properly licensed fonts.

## 3. Identifying the font: is it lawful?

Yes. The identity of a font is a fact. No right reserves the act of recognising it. [M] The conditions are in the table.

| Step | Issue | Assessment [M] |
|---|---|---|
| Analysing the cover image in the browser | Temporary copy of the cover | Covered by whatever permits us to display the cover. No separate font issue |
| Fetching a cover only to derive facts (colours, font class, hash), then deleting it | Reproduction for analysis | Text-and-data mining under DSM Directive art. 4 and CPI L122-5-3 III, if access is lawful and no opt-out is expressed. In France the opt-out may be expressed "par tout moyen", including site terms (CPI R122-28). See sibling note `cover-sources.scraping-law.md` |
| Training a classifier on rendered glyphs of **open** fonts | None | Permitted by the OFL and Apache licences |
| Training on renders of **commercial** fonts | Needs a lawful copy; many EULAs restrict automated or ML use | The art. 4 exception applies only if rights are not reserved. Art. 7(1) of the Directive makes contract terms unenforceable against arts. 3, 5 and 6, **not** art. 4. A EULA ban is therefore an effective reservation. Avoid |
| Training on scraped cover images | Copyright in the covers; opt-outs | French publishers (SNE) have promoted TDM opt-outs since 2023–24. Poor source. Use synthetic renders of open fonts |
| Using a third-party identification service | Its terms of use; the cover image leaves the device | Terms not checked. Prefer in-browser inference |
| Showing "set in Helvetica" to the user | Trade mark | Referential use is lawful, but we choose not to show commercial names (§1.4) |
| Reading font names from fonts embedded in the EPUB | The fonts are often obfuscated; they are licensed for that file only | Do not open them. The cover title is a bitmap anyway; embedded fonts are for body text |

**Recommendation: make the classifier's label space the open fonts we can ship**, not the universe of commercial fonts. The model then answers "which of our faces is closest", which is the only answer the product can act on. It is trained only on freely licensed material, and no commercial font data or name enters the system.

The classifier should also output a class: *typeset*, *lettered* or *logo*. For lettering there is no font to identify; the correct output is low confidence (§6).

## 4. Reusing the same font: licences, embedding bits, outlines

### 4.1 Licence types

All [M]: terms as known in mid-2026, to be re-read before any purchase. **No price was obtained** (as of 2026-11).

| Licence | Covers | Fits this service? |
|---|---|---|
| Desktop | Installation on N workstations; static output (print, PDF, images) made by licensed users | No for server or browser rendering. Yes for a user working on their own machine |
| Web font | `@font-face` display on a site, by page views | No: display only, not document generation |
| App / ePub | Embedding in one application or one publication | No |
| Server / web-to-print | Font on a server, or in a tool, used by remote users or automated processes to create documents or customised products | **Yes: this is the category.** Quote-based, usually annual and per family |
| Adobe Fonts (Creative Cloud) | Desktop use by the subscriber; web use through Adobe's hosting | No. The terms exclude installation on servers and use in tools where end users customise products |

A catalogue of trade books uses hundreds of distinct title fonts. A per-family licence is therefore not viable for the long tail, whatever the unit price. [M, reasoning]

### 4.2 Who must hold the licence [M]

| Architecture | Where the font runs | Licence holder |
|---|---|---|
| A. Server renders the PDF | Our server | Us: server licence |
| B. Browser (WASM) renders with a font file we deliver | The user's browser; we distribute the file | Us: web-to-print or application licence. A web font licence alone is not enough |
| C. Browser renders with a font file the user supplies (file picker, or Chromium's Local Font Access API); the file is never uploaded | The user's device | The user: their desktop licence |
| D. Publisher or reseller supplies finished spine artwork, or the title as outlines | Not our concern | The publisher or reseller |
| E. Open-licence font bundled by us | Anywhere | Nobody; comply with the notice conditions |

- The licence of the cover designer or of the publisher never extends to us.
- Fonts embedded in the gifted EPUB (often obfuscated) or in a publisher PDF are licensed for that file only. Extracting them to set a new title is unlicensed copying.
- Architecture C is rung 3b of the ladder. Architecture D is rungs 0 and 3a. For 3a, the licensee renders the title to outlines and sends SVG or PDF; we never receive the font file; we ask for written confirmation of the licence scope.
- The Local Font Access API (`queryLocalFonts()`) exists in Chromium browsers only and needs a user permission [M]. The file picker is the fallback.

### 4.3 Embedding permissions: OS/2 `fsType`

Meaning of the bits: [M], from the OpenType specification. Product rule: decision of this document. The decoding itself is already implemented in `crates/ebc-text/src/font.rs` [L].

| `fsType` | Meaning | Bundled or indexed font | User-supplied font (browser) |
|---|---|---|---|
| 0x0000 | Installable embedding | **Required value** | Allowed; outlined |
| 0x0004 | Preview & Print embedding | Reject | Allowed; outlined |
| 0x0008 | Editable embedding | Reject | Allowed; outlined |
| 0x0002 | Restricted licence: not to be "modified, embedded or exchanged in any manner" without the owner's permission | Reject | **Refuse**; substitute an open font |
| + 0x0100 | No subsetting | Reject | Outline only; never embed a subset |
| + 0x0200 | Bitmap embedding only | Reject | **Refuse**; substitute |
| No OS/2 table | Unknown | Reject | Refuse |

- From OS/2 version 3 the permission bits are mutually exclusive. If a font sets several anyway, take the most restrictive. For versions 0–2 the specification says the least restrictive applies. The code does both. [M for the specification; L for the code]
- `fsType` states the foundry's intent; the licence text prevails. A US court held that the bits are not an access control under the DMCA (*Agfa Monotype v. Adobe*, N.D. Ill. 2005) [M]. Honouring them is the industry norm and the cheapest evidence of good faith.

### 4.4 Converting text to outlines

- Outlining removes the font program from the PDF. Nothing is redistributed and the embedding bits become moot. [M]
- **It does not cure a missing licence.** The outlines were produced by running the font, and in France the shapes themselves may be a protected work. [M]
- Printers differ: Printoclock asks for outlined fonts ([bien réaliser son fichier](https://www.printoclock.com/bien-realiser-son-fichier)) [V1, sibling note `print-requirements.prepress.md`]; Pixartprinting asks for embedded fonts, not outlined; Bookvault and Lulu ask for embedded fonts [V1 each, sibling note]. An outlined spine title plus embedded, subset body fonts satisfies all of them.
- Cost of outlining: no selectable text and slightly heavier small type. Irrelevant for a spine title.

### 4.5 The closest precedent: *Laatz v. Zazzle* [M?]

A type designer alleged that a print-on-demand platform bought a single-seat licence and installed the font on its servers for all users of its online design tool (N.D. Cal., filed 2022, no. 5:22-cv-04844). Claims included breach of contract, fraud and software copyright. **Outcome not verified.** The facts are the scenario that architectures A and B without a server licence would reproduce.

## 5. Open-licence substitutes and look-alikes

### 5.1 SIL OFL 1.1 — verified against the repository copy [L]

Source read: `/home/user/ebook-cadeau/assets/fonts/OFL-Spectral.txt`. Its header gives [https://openfontlicense.org](https://openfontlicense.org) as the home of the licence and its FAQ. The four `OFL-*.txt` files have the same licence body, apart from whitespace.

| Point | Text of the licence |
|---|---|
| Grant | "to use, study, copy, merge, embed, modify, redistribute, and sell modified and unmodified copies of the Font Software" |
| Condition 1 | Neither the font nor its components "may be sold by itself" |
| Condition 2 | May be "bundled, redistributed and/or sold with any software, provided that each copy contains the above copyright notice and this license", as "stand-alone text files, human-readable headers or in the appropriate machine-readable metadata fields within text or binary files as long as those fields can be easily viewed by the user" |
| Condition 3 | No Modified Version may use the Reserved Font Name without written permission. "This restriction only applies to the primary font name as presented to the users" |
| Condition 4 | The authors' names may not be used to promote a Modified Version |
| Condition 5 | The font must stay under the OFL. This "does not apply to any document created using the Font Software" |
| Definition | "Modified Version" is any derivative made by adding, deleting or substituting components, "by changing formats or by porting the Font Software to a new environment" |
| Termination | "This license becomes null and void if any of the above conditions are not met" |

Consequences:

- A PDF with an embedded OFL subset is a "document". No licence text is needed in the PDF and the printed card may be sold. [L for the clause; M for the reading of the OFL FAQ that document embedding is not redistribution]
- Shipping font files to the browser, or compiling them into the WASM binary, is bundling with software. Keep the copyright notice and the licence with each copy. The `name` tables of all eight files carry the copyright notice (ID 0) and a one-line licence statement with a URL (IDs 13 and 14), **not the full licence text** [L]. The `OFL-*.txt` files must therefore ship with every build. Add a credits page as well: the termination clause makes notice compliance worth the small effort.
- A subset or converted file that we ship **as a font** is a Modified Version. It may keep its name only if no Reserved Font Name is declared.
- The grant has no field-of-use limit. Training a model on OFL fonts is within "use" and "study". Model weights are not obviously a "Modified Version". A generated *installable font* that closely reproduces one OFL font arguably is. Whether the OFL FAQ addresses AI training was not checked. [L for the text; M for the reading]

### 5.2 Audit of the fonts in the repository [L]

Method: OS/2 and `name` tables read with a script; licence headers read directly; `SHA256SUMS` checked (all eight files match). Re-run on 2026-11-12.

| File(s) | Licence and notice | `fsType` | Reserved Font Name | Trade mark notice (name ID 7) |
|---|---|---|---|---|
| `AtkinsonHyperlegible-{Regular,Italic,Bold}.ttf` (v1.006) | OFL 1.1, © 2020 Braille Institute of America, Inc. | 0x0000 | None | None |
| `BebasNeue-Regular.ttf` (v2.000) | OFL 1.1. Notice in the binary: "Copyright 2019 The Bebas Neue Project Authors". Notice in `OFL-BebasNeue.txt`: "Copyright © 2010 by Dharma Type." | 0x0000 | None | "Bebas Neue is a trademark of Dharma Type." |
| `CourierPrime-Bold.ttf` (v3.018) | OFL 1.1, © 2015 The Courier Prime Project Authors | 0x0000 | None | None |
| `Spectral-{Regular,Italic,SemiBold}.ttf` (v2.005) | OFL 1.1, © 2017 The Spectral Project Authors | 0x0000 | None | "Spectral is a registered trademark of Production Systems SAS." |

- All eight may be embedded, subset and bundled.
- The README states they were downloaded unmodified from [https://github.com/google/fonts](https://github.com/google/fonts) [L].
- **One housekeeping item:** the Bebas Neue copyright line differs between the binary and the licence file. Reproduce both on the credits page and check the upstream file when network access allows.
- If the project ever alters the designs beyond subsetting, rename derivatives of Spectral and Bebas Neue because of the trade marks, even though the OFL does not require it here.

### 5.3 Other open licences [M]

| Licence | Examples | Conditions that matter | Use in the bundle |
|---|---|---|---|
| Apache 2.0 | Arimo, Tinos, Cousine; older Roboto and Open Sans releases | Keep licence and notices; state changes; no trade mark grant; no naming rule | Yes |
| Ubuntu Font Licence 1.0 | Ubuntu | Naming rules for modified versions | Yes, unmodified |
| GUST Font License | TeX Gyre | LPPL-based; rename if modified | After per-file review, unmodified |
| Bitstream Vera licence | DejaVu | Not sold alone; rename if modified | After per-file review, unmodified |
| AGPL v3 + font exception | URW base 35 (Artifex distribution) | The exception frees documents. Without it a GPL font can bind the embedding document | No, in the first version |

- Several of these families were relicensed over time. Check the licence of each *file* before bundling it.
- "Public domain" in the bundle rule means a font *file* dedicated to the public domain (for example CC0). An old typeface design does not make a modern digitisation free: the file is software with its own copyright.
- Google Fonts states that its fonts may be used in commercial products, in print and digital; the licence of each family governs. **Self-host**: LG München I, 20 January 2022, 3 O 17493/20, held that loading fonts from Google's servers discloses the visitor's IP address without a legal basis under the GDPR. [M]

### 5.4 Metric-compatible and look-alike families [M]

| Commercial face | Open substitute | Licence | Relationship |
|---|---|---|---|
| Helvetica / Arial | Liberation Sans, Arimo; Nimbus Sans; TeX Gyre Heros | OFL / Apache; AGPL + exception; GUST | Metric-compatible clones |
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

### 5.5 Are clones and look-alikes lawful?

| Act | USA | EU / France |
|---|---|---|
| Redrawing a look-alike from printed samples | Lawful unless a design patent is in force | Depends on the age and originality of the model: registered design up to 25 years; unregistered 3 years; copyright life + 70 where original; plus unfair competition |
| Copying or converting the font file | Infringes the software | Infringes the software, and the design where protected |
| Using the original name | Trade mark infringement | Same |

All [M]. Points that matter for us:

- **We would be users of the substitutes, not their makers.** Liberation, TeX Gyre and the URW set have been distributed for 15 to 25 years by Red Hat, GUST and Artifex with no challenge known to us. Liberation, Arimo, Tinos and Cousine were drawn by Ascender, which Monotype later acquired [M?].
- **Choosing the nearest existing open font is not "copying" a design.** The unregistered EU design bites on copying only. A *registered* design, however, is infringed by use of a product that incorporates it, with or without copying. If an open font were itself a close copy of a registered design less than 25 years old, using it would be exposed. Catalogue curation is therefore the control point.
- **Residual EU risk by type of substitute:** lowest for revivals of public-domain models and for original designs in a shared genre; highest for close copies of recent commercial faces.
- Illustrations of terms: Gill Sans (Eric Gill, d. 1940) is out of copyright everywhere. Paul Renner died in 1956, so any copyright in the design of Futura lapses at the end of 2026. Max Miedinger died in 1980: a copyright in Helvetica, where a court would recognise one, runs to 2050. Gotham (2000) and Proxima Nova (2005) have living authors. [M? for the dates]

**Catalogue rule.** Admit a family to the index if it is (a) a revival of a model whose designer died more than 70 years ago, or (b) an original design, or (c) a long-distributed metric-compatible clone of a model more than 25 years old. Do not admit a close copy of a commercial face first published less than 25 years ago.

## 6. Hand lettering and artwork

### 6.1 Lettering [M]

- A title drawn by hand or built for one cover is a graphic work (CPI L112-2 7° and 8°; UK CDPA s. 4). The author is the letterer. The publisher usually holds the economic rights. The moral rights stay with the letterer.
- In the USA, plain lettering is not registrable; pictorial lettering can be.
- There is no font to license and nothing to identify. The classifier should report low confidence.
- CPI L122-4 prohibits any reproduction, "adaptation, transformation, arrangement" without consent.
- Franchise title logos are often registered figurative marks. Plain retyped text is referential use; an imitated logo is not.

### 6.2 Operations on the cover and their exposure

| Operation | What it is in law | Exposure | Sources |
|---|---|---|---|
| Retyping the title in an open font on our own panel | Reuse of the title as text; our own typography | Low. Mind the series look (§2.10) | [M] |
| Flat colour sampled from the cover as spine background | A colour is not a work | Low. The only clean background without consent | [M, reasoning] |
| Procedural texture of our own, in a colour taken from the cover | Our own material | Low | [M, reasoning] |
| Proportional scaling, CMYK conversion of supplied artwork | Technical reproduction | Low | Sibling note `legal-covers-reseller.reseller.md` §4 |
| Moving the title pixels from the cover to the spine (crop, cut-out) | Partial reproduction; recomposition | Medium to medium–high. Covered only by a cover licence that includes the card; sensitive for integrity | [M] |
| Tracing the lettering into vectors | Reproduction of the graphic work | High. Same as AI re-rendering, without the AI | [M] |
| AI re-rendering of the lettering | Adaptation; integrity; trade mark for logos | High | [M] |
| Stretching or mirroring edge pixels onto the spine | Alteration of the work | High | [M, reasoning]; case law on cropping and mirroring: [V2, sibling note, below] |
| AI outpainting of the artwork onto the spine | Adaptation; alteration; attributes to the artist content they did not make | Highest | [M] |

Verified French background, carried from the sibling notes:

- Permission to use a work does not dispense from respecting moral rights ([APIE / economie.gouv](https://www.economie.gouv.fr/node/40472)) [V1, sibling note].
- French case law treats cropping, mirrored reproduction, colourisation and cut-outs (*détourage*) as infringements of the right of integrity ([Livres Hebdo](https://www.livreshebdo.fr/article/pop-et-droit-moral), [practical note](https://www.alsace.catholique.fr/wp-content/uploads/sites/14/2016/04/notions-cle-droit-image.pdf)) [V2, sibling note `legal-covers-reseller.reseller.md`].
- In the case reported by Livres Hebdo, a reproduction that altered the work, omitted the signature and changed the format cost €15,000 in damages ([Livres Hebdo](https://www.livreshebdo.fr/article/pop-et-droit-moral)) [V1, sibling note].
- Cropping an image to fit a cover format without express consent was treated as an integrity issue (TGI Paris, 5 October 2007, [Légifrance](https://www.legifrance.gouv.fr/juri/id/JURITEXT000018862984)) [V1, sibling note `legal-covers-reseller.independent.md`].

Other jurisdictions [M]: Germany, §§ 14 and 23 UrhG (distortion; adaptations need consent for exploitation). UK, false attribution, CDPA s. 84.

### 6.3 Rules

- If the title is lettered or is a logo: use a house face, or the publisher's spine artwork. Do not use the nearest imitation.
- No cover pixels on the spine in the default pipeline. This excludes blurred or extended edge strips as well: a blur is still derived from the artwork, and where the line falls is a question for counsel (question 9).
- Neutral spine layout: title and author as text. No publisher logo, monogram or collection device unless the publisher supplies the spine.
- If rung 4 (cropped lettering) is ever enabled for a title: keep the title with its own background strip instead of cutting it out, do not recolour or redraw it, print the cover credit, and require the publisher's per-title opt-in.

## 7. Generative AI: output, training, liability

### 7.1 Output: re-rendering a typeset title

- **USA:** the shapes are free, and the output is raster or outlines, not font software. The font-software cases do not apply. Residual: design patents (rare, short) and font-name trade marks. [M]
- **EU design right:** a few-shot model that reads the cover and regenerates the glyphs derives them from the design. That satisfies "copying" for the unregistered right. The private non-commercial exemption covers a consumer printing at home, not a service or a reseller. [M]
- **French copyright:** partial reproduction is enough if the part carries the originality. [M]
- **Open point:** whether 10–30 glyphs are a use of "the design" of a whole typeface. No case known. Text faces leave the designer little freedom, which narrows their protection. Display faces leave more freedom and get broader protection. [M, reasoning]
- **Why this is worse than cropping (rung 4):** the glyphs printed on the cover come from the publisher's licensed use of the font, and a licensed reproduction of the cover carries that chain. An AI re-drawing has no licence chain at all. [M, reasoning]
- **Where the exposure concentrates:** recent display faces on new releases (inside the 3-year unregistered window); faces from foundries that register designs; distinctive faces by living French or EU designers. Classic text faces are practically safe. [M, reasoning]

### 7.2 Output: re-rendering lettering, and outpainting

- AI re-rendering of a hand-drawn title re-draws the same word in the same forms, rotated and resized. That is an adaptation, not an independent creation. [M]
- Moral rights: integrity (a degraded or "almost right" redraw is the textbook complaint) and attribution. They belong to the letterer. [M]
- Exceptions do not fit. Pastiche (CPI L122-5 4°; CJEU reference *Pelham II*, C-590/23, Advocate General's opinion of June 2025, judgment status unknown [M?]) requires some artistic dialogue; a functional spine label has none. Private copying covers the home user at most. [M]
- Trade marks: in *Getty Images v Stability AI* ([2025] EWHC 2863 (Ch)) the model provider was held liable for trade-mark infringement where outputs reproduced a watermark [M]. A tool that redraws a registered title logo is in a comparable position.
- Outpainting adds invented content next to the work and presents it as part of it. That is a stronger alteration than a crop. [M, reasoning]
- Generic inpainting models bring their own problems: training-data litigation (§7.8) and model licences (§7.5).

### 7.3 Authorship and labelling of AI output

- **USA:** Copyright Office registration guidance of 16 March 2023 (88 Fed. Reg. 16190); report "Copyright and Artificial Intelligence", Part 2 (29 January 2025): prompts alone do not make the user an author. *Thaler v. Perlmutter* (D.C. Cir., 18 March 2025) confirmed the human-authorship requirement [M]; certiorari reportedly denied in 2026 [M?]. No Copyright Office statement specific to AI font generation is known [M?]. By inference, an AI-generated glyph set is doubly unprotected in the USA.
- **EU:** no harmonised rule, but "author's own intellectual creation" presupposes a human. The Prague Municipal Court (2023) refused copyright in an AI image. We cannot own a generated spine. That is harmless for us. [M]
- **AI Act (Reg. (EU) 2024/1689), art. 50(2):** providers of AI systems that generate synthetic image content must mark outputs in a machine-readable format, detectable as artificially generated. Nominal application date: 2 August 2026. [M]
  - Exception for systems that "perform an assistive function for standard editing" or "do not substantially alter the input data". Whether spine lettering or outpainting fits is untested. [M]
  - Fines for art. 50 breaches: up to €15 million or 3 % of turnover (art. 99(4)), with the lower cap for SMEs. [M]
  - The open-source exclusion does not cover art. 50 systems (art. 2(12)). [M]
  - The 2025 "Digital Omnibus" proposed adjustments to these dates, including a grace period for systems already on the market. Outcome not verified. [M?] A system first placed on the market after 2 August 2026 should be designed as if the duty applies.
- A nearest-font classifier and a colour sampler generate no synthetic content. Art. 50(2) does not concern them. [M, reasoning]

### 7.4 Training: contracts and the EU exceptions

- **Commercial EULAs.** Font licences are contracts over the font software. Monotype's current EULAs are reported to prohibit using the fonts to train AI/ML; several independent foundries have added similar clauses since 2023; Adobe's general terms contain a no-AI-training clause. **Exact wording not checked.** [M?]
- A EULA binds licensees only. Someone who never licensed the font and learns from rendered images is not bound by it, but still needs a copyright basis in the EU. [M]
- Contract can restrict beyond copyright where no mandatory exception applies (*Ryanair v PR Aviation*, C-30/14). [M]
- **DSM Directive (EU) 2019/790, art. 3:** research organisations and cultural heritage institutions, for scientific research; no opt-out; cannot be overridden by contract. Not us. [M]
- **Art. 4:** anyone with lawful access, unless the rightholder has expressly reserved the use "in an appropriate manner". A EULA clause against ML training is such a reservation. Training a commercial model on EULA-restricted fonts is therefore outside the exception and a breach of contract. [M]
- **France:** CPI L122-5 10° and L122-5-3 (ordonnance of 24 November 2021). The opt-out need not be reasoned and may be expressed by any means, including site terms (R122-28, décret n° 2022-928 of 23 June 2022). [M; both notes and the sibling note `cover-sources.scraping-law.md` agree on the article number, none verified it]
- **Case law** [M unless marked]: LG Hamburg, 27 September 2024, 310 O 227/23, *Kneschke v LAION* (dataset building covered by the research exception). Appeal decided by OLG Hamburg in December 2025, reportedly upheld [M?]. LG München I, 11 November 2025, 42 O 14139/24, *GEMA v OpenAI*: memorised works inside model weights are reproductions, not covered by TDM, and the provider, not the user, answers for outputs. First CJEU reference on generative AI: *Like Company v Google*, C-250/25, pending at last information [M?].
- **Open fonts.** OFL: see §5.1 [L]. Apache 2.0 and Ubuntu Font Licence fonts are likewise permissive [M]. Google Fonts holds on the order of 1,800 families, mostly OFL [M].

### 7.5 Datasets and weights in the literature

| Work | Training data, as recalled | Reuse issue | Conf. |
|---|---|---|---|
| FontDiffuser (AAAI 2024) | About 400 Chinese fonts × about 800 characters, collected by the authors | Font licences undocumented; repository licence not checked | [M] |
| DG-Font (CVPR 2021) | About 400 Chinese fonts | Same | [M] |
| LF-Font, MX-Font (NAVER Clova, 2021) | Chinese and Korean font collections; code under MIT | The code licence does not clear the data | [M] |
| Diff-Font (2022–24) | Chinese and Korean fonts | Same | [M] |
| MC-GAN "Capitals64" (CVPR 2018) | About 10,000 Latin fonts gathered from the web, capitals only | Scraped; includes commercial faces | [M] |
| SVG-Fonts / "Glyphazzn" (Lopes et al., 2019), used by DeepVecFont | About 14 million glyphs from fonts found online | Scraped | [M] |
| AdobeVFR (DeepFont, 2015) | 2,383 Adobe font classes | Adobe-owned | [M] |
| MyFonts tag dataset (Chen et al., 2019) | About 18,800 MyFonts fonts | Monotype catalogue | [M] |
| Stable Diffusion 1.5 / SDXL | LAION subsets; OpenRAIL-M / OpenRAIL++-M | Use restrictions; data under litigation | [M] |
| FLUX.1 schnell / dev | Undisclosed; Apache-2.0 / non-commercial | dev unusable commercially | [M] |
| LaMa, MI-GAN (small inpainting models that can run in a browser) | Places2 and similar; Apache-2.0 / MIT | Dataset research terms | [M?] |

**Conclusion: no off-the-shelf font-generation checkpoint has a clean, documented data lineage for commercial use in the EU.** A model trained from scratch on Google Fonts, Latin only, with a published manifest, is the defensible route.

### 7.6 AI Act: general-purpose model duties versus a small specialised model [M]

- Art. 53 (technical documentation, copyright policy honouring opt-outs, public summary of training content) binds providers of general-purpose AI models, since 2 August 2025.
- A general-purpose model shows "significant generality" (art. 3(63)). The Commission guidelines of July 2025 use an indicative threshold around 10^23 FLOP of training compute, plus text, text-to-image or text-to-video generation. A font-style model trained on a few thousand fonts is orders of magnitude below. It is not a general-purpose model.
- If we embedded someone else's general-purpose model (a Stable Diffusion or FLUX inpainting checkpoint), the upstream provider would carry art. 53. Light fine-tuning does not make us a provider of such a model (the guidelines use about one third of the original training compute as the mark).
- We remain the **provider of an AI system** (art. 3(3)) wherever inference runs. Art. 50(2) is ours. A consumer who uses the system privately is not a "deployer" (art. 3(4)).

### 7.7 Who is liable, and what browser-side execution changes

| Scenario | User | Service (us) | Reseller | Printer |
|---|---|---|---|---|
| **S1** Standalone, print at home, generation fully in the browser | Makes the copy. The private-copy exception and the private non-commercial design exemption are available in principle; whether a gift is "usage privé du copiste" is unsettled. Practically untouchable | Tool supplier. France: *complicité par fourniture de moyens* (Code pénal art. 121-7), *faute* (Code civil art. 1240), *parasitisme*. Exposure depends on how the tool is designed and marketed | — | — |
| **S2** Standalone, professional print order | Warrants rights | **Primary**: we have the copy made and distributed (CPI L335-2, L335-3). Good faith is irrelevant in civil *contrefaçon*. No hosting safe harbour: we play an active role (*L'Oréal v eBay*, C-324/09) | — | Co-liable; will ask for a warranty |
| **S3** Reseller checkout option | Buyer: none in practice | Primary or joint for generated elements. The draft reseller clause X.6(ii) has us **warranting** fonts, backgrounds and generated elements (sibling note `legal-covers-reseller.reseller.md`) | Primary: commercial offer; breach of its publisher licence if the cover is altered | Co-liable |

All [M] unless marked. Verified background from the sibling note `legal-covers-reseller.independent.md`:

- The private-copy exception (CPI L122-5 2°) covers copies "réalisées à partir d'une source licite et strictement réservées à l'usage privé du copiste" ([Baumann](https://www.baumann-avocats.com/CPI/article-l122-5-du-code-de-la-propriete-intellectuelle.php), [Doctrine](https://www.doctrine.fr/l/texts/codes/LEGITEXT000006069414/sections/LEGISCTA000006179045)) [V2, sibling note].
- In French doctrine the copier is the party that operates the equipment (Rannou-Graphie line, [thesis summary](https://www.memoireonline.com/12/06/297/m_l-exception-de-copie-privee-face-aux-dispositifs-techniques-de-protection-des-oeuvres10.html)) [V1, sibling note]. The online recorder Wizzgo could not shelter behind its users' private copying ([Village de la justice](https://www.village-justice.com/articles/affaire-wizzgo-prend-tournant,4999.html)) [V1, sibling note].
- *VCAST* (C-265/16): a service that itself makes the work available cannot rely on its users' private copying ([Kluwer](https://legalblogs.wolterskluwer.com/copyright-blog/up-in-the-cloud-some-reflections-on-the-cjeu-judgment-in-vcast/), [IPKat](https://ipkitten.blogspot.com/2017/11/the-vcast-decision-how-to-turn-private.html)) [V2, sibling note].

Does running the model in the browser change anything? [M, reasoning]

1. **Yes, for the generation step.** The copy is made on the user's device at the user's request, so the user is the maker. Exceptions reserved to private users can apply only if the user is the copier.
2. **No, for "mere tool" status.** A feature whose stated purpose is to imitate the cover's lettering is not a neutral staple article. *Grokster* (inducement) and *Cox v. Sony* (US Supreme Court, 2026: liability for services that induce or are tailored to infringement [M?]) point the same way as the French "rôle actif" reasoning. If the service itself fetches the cover by ISBN and runs the imitation automatically, the user's agency is thin.
3. **No, for regulation.** AI Act provider status and art. 50 marking are unaffected.
4. **It adds an act.** We distribute the weights to every browser in the EU. If weights can embody training works (*GEMA v OpenAI*; contrast *Getty v Stability*, where the UK court held that the Stable Diffusion model is not an "infringing copy"), each download is a reproduction and distribution in the EU, wherever training took place. Recital 106 of the AI Act takes the same territorial line for general-purpose models. A clean training set removes the issue.
5. **Safe harbours are beside the point.** DSA arts. 4–6 protect transmission and hosting of users' information. Nothing is hosted in S1. In S2 and S3 we manufacture.
6. **No logs, no moderation.** We cannot act on outputs we never see. Safeguards must be built into the client (gating, exclusion lists, labelling). That design is what a court would assess.

Output-liability precedents so far put liability on the provider: LG München I (*GEMA*), Guangzhou Internet Court (February 2024, Ultraman images) [M]. All concern server-side services.

### 7.8 Disputes and commentary, 2023–2026

| Matter | Point for us | Conf. |
|---|---|---|
| *Getty Images v Stability AI*, [2025] EWHC 2863 (Ch), 4 Nov 2025 | Training claim dropped (training outside the UK). Model weights are not an "infringing copy". Limited trade-mark infringement for watermarked outputs. Appeal on secondary infringement reportedly allowed [M?] | [M] |
| *GEMA v OpenAI*, LG München I, 11 Nov 2025 | Memorisation is reproduction; the provider is liable for outputs | [M] |
| *Kneschke v LAION*, LG Hamburg 2024; OLG Hamburg Dec 2025 | Research TDM; form of opt-outs | [M] / [M?] |
| *Andersen v. Stability AI* (N.D. Cal.) | Artists' direct and inducement claims survived dismissal (Aug 2024) | [M] |
| *Thomson Reuters v. Ross* (D. Del., Feb 2025) | Fair use rejected for a non-generative AI tool; on appeal | [M] |
| *Bartz v. Anthropic*; *Kadrey v. Meta* (N.D. Cal., June 2025) | Training on lawfully acquired books held fair use; pirated sources not; USD 1.5 bn settlement in *Bartz* | [M] |
| *Disney and Universal v. Midjourney* (C.D. Cal., June 2025); Warner Bros. (Sept 2025) | Output-side claims over recognisable characters | [M] |
| SNE, SGDL, SNAC v. Meta (TJ Paris, March 2025) | The French book trade litigates against AI training; status unknown | [M] |
| *Like Company v Google*, C-250/25 | First CJEU reference on training and outputs | [M?] |
| US Copyright Office AI report, Parts 1–3 (2024–2025) | §7.3; Part 3 (pre-publication, 9 May 2025): training is fair use in some cases and not in others | [M] |
| Monotype "Re:Vision" trends report (2025) and press coverage | Monotype presented AI as an opportunity; type designers objected publicly | [M?] |
| Foundry EULAs with no-AI-training clauses | §7.4 | [M?] |
| Rulings on AI-generated fonts or lettering | None known | [M?] |

For comparison, typeface litigation has almost always been about font software licences, not shapes [M]. China is an exception: courts have protected font libraries and sometimes single glyphs (*Founder v. Procter & Gamble*, Beijing, 2011, decided on implied licence) [M]. Positions of the type-design community (ATypI and others) were not verified.

## 8. Product rules derived

| # | Rule | Reason | See |
|---|---|---|---|
| R1 | The bundle and the font index contain only fonts under OFL 1.1 or Apache 2.0; UFL 1.0 and public-domain-dedicated files after a per-file check. No commercial font, no AGPL font | Only these licences allow shipping to browsers, embedding and training without a contract | §5 |
| R2 | Honour `fsType`. Bundled fonts must be 0x0000. Restricted and bitmap-only user fonts are refused | Industry norm; evidence of good faith | §4.3 |
| R3 | Spine text is drawn as outlines. Other text uses embedded subsets of bundled fonts | No font program travels for the spine; printers' requirements are met | §4.4 |
| R4 | Record each font's family, version and licence, and the spine's provenance, in the PDF metadata | Audit trail; supports the reseller log (clause X.4(e)) | §5.1 |
| R5 | Never read, extract or de-obfuscate fonts from an EPUB or a publisher PDF | Licensed for that file only | §4.2 |
| R6 | Bring-your-own-font runs in the browser only. The file is never uploaded or stored. The user warrants their licence | The only no-licence route for commercial fonts | §4.2 |
| R7 | No commercial font name in the interface, in marketing or in match results. No "free X", no "alternative to X" | Trade marks; comparative advertising | §2.9 |
| R8 | Self-host all fonts and model weights. No third-party font CDN or identification service at run time | GDPR (LG München I); cover images stay on the device | §5.3, §3 |
| R9 | Default spine background is one flat colour sampled from the cover. No cover pixels on the spine | Integrity right; only a colour is free | §6 |
| R10 | Lettered titles and logos get a house face. No imitation | Graphic work; moral rights; trade marks | §6 |
| R11 | AI lettering and outpainting are off by default, available only in "own rights" mode, after explicit user confirmation, with an AI label; in reseller mode only on a per-title publisher opt-in, and never for lettered titles | Highest rungs of the ladder; AI Act art. 50 | §7 |
| R12 | Never train on proprietary fonts or on scraped covers. Publish a training manifest. Ship no third-party checkpoint with undocumented training data | EULA and TDM reservations; weights are distributed in the EU | §7.4–7.5 |
| R13 | A model never outputs a font file. It outputs the title string only, as raster or outlines | Avoids the font-software and "Modified Version" questions | §5.1, §7.1 |
| R14 | Neutral spine layout: title and author. No logo, no imitation of a collection | *Parasitisme* | §2.10 |
| R15 | Ship licence texts and a credits page with every build | OFL condition 2 and termination clause | §5.1 |
| R16 | Remote kill switch for generative features and a per-ISBN exclusion list, effective within 24 hours | No logs or moderation are possible client-side; supports clause X.4(d) | §7.7 |
| R17 | Reseller contract: AI-generated elements are excluded from our warranty unless the publisher opted in | We warrant what we add (clause X.6(ii)) | §7.7 |
| R18 | Catalogue curation: no close copy of a commercial face first published less than 25 years ago | Registered designs are infringed by use, without copying | §5.5 |

## 9. Questions for counsel

Design and copyright in typefaces:

1. Under EU design law (Reg. 6/2002 as amended, art. 19) and CPI L513-4, is printing a title in a protected typeface "use of a product in which the design is incorporated"? Is there any typesetting carve-out comparable to UK CDPA s. 54?
2. Is there French case law on the originality of a typeface under CPI L112-2 8°? Does it distinguish text faces from display faces?
3. Does a title of 10–30 glyphs reproduce "the design" of a typeface for design law, or a substantial part of the work for copyright?
4. What due diligence is expected from a *user* of an open look-alike font if the model it resembles is a registered design? Is catalogue rule R18 sufficient?
5. How would a French court classify a font file (software, database, carrier of the typographic work)? What follows for bring-your-own-font?

Licences:

6. Is a standard desktop EULA breached when the licensee loads the font into a third-party web application that runs only on the licensee's device? Does offering that feature expose us?
7. For rung 3a, is a publisher's desktop licence enough to produce spine outlines for a card printed by a third party?
8. May a font marked `fsType` restricted be outlined if the user confirms that their licence allows it?

Covers, lettering and backgrounds:

9. Is a flat colour sampled from a cover free of any right? Where is the line: a two-colour gradient, a strongly blurred edge strip?
10. With a publisher opt-in, may we crop the title strip from the cover (rung 4)? Is the consent of the illustrator or letterer also needed?
11. After the Cour de cassation judgments of 26 June 2024, when does a spine in the cover's colour, with a similar open font, become *parasitisme* of a collection's look?
12. May we show "similar to [commercial font name]" anywhere? We plan not to.

Generative AI:

13. For AI lettering in "own rights" mode: are we a provider under AI Act art. 50(2)? Does the "standard editing" exception apply? How is a printed object marked? What is the application date after the Digital Omnibus?
14. Is a consumer's warranty of rights meaningful? What wording is advised, and is the letterer's consent needed separately from the publisher's?
15. If the model is trained only on OFL and Apache fonts, is there any residual exposure in distributing the weights? Do the OFL's "Modified Version" terms reach weights or generated glyph images?
16. In scenario S1, how would *complicité* or art. 1240 liability be assessed for an opt-in imitative feature? Which design choices reduce it most?
17. Reseller contract: is the split in clause X.6 sound, and is IP insurance available for generated elements?
18. Which of these answers change for Belgium, Germany, Switzerland and the UK?

## 10. What could not be verified, and sources

### 10.1 To verify first, in priority order

Everything marked [M] or [M?] is unverified. The items that change decisions:

1. **Commercial licence terms and prices** (as of 2026-11): Monotype/MyFonts server and web-to-print licences; the Adobe Fonts wording on servers and product-customisation tools. No price was obtained.
2. **AI/ML clauses in commercial EULAs**: Monotype/MyFonts, Adobe Fonts, Fontshare, and French foundries (Production Type, Typofonderie, 205TF). Wording and dates.
3. **AI Act art. 50 as of November 2026**: effect of the Digital Omnibus on the date; state of the Commission's code of practice on marking and labelling; any reading of the "standard editing" exception.
4. **French case law** on the originality of typefaces. None found or recalled with a checkable citation.
5. **EU design reform**: exact wording and numbering of the new product definition, of art. 19(2)(d) and of the new permitted uses in Reg. 2024/2822; the second application date; French transposition status of Dir. 2024/2823; any decision or EUIPO invalidity case on typeface designs; the current rule on representing a typeface design.
6. ***Laatz v. Zazzle***: procedural history and outcome.
7. ***Mio / konektra***: exact date and holding; whether a German court has since granted copyright to a text typeface.
8. **OFL FAQ**: passages on document embedding, WOFF conversion, subsetting and Reserved Font Names, and any statement on AI training. The licence text itself is verified.
9. **Current licences** of Roboto, Open Sans, Arimo/Tinos/Cousine, Caladea and the URW base 35 set; the upstream copyright line of Bebas Neue.
10. **Status after mid-2026** of: the *Getty v Stability* appeal; the *GEMA v OpenAI* appeal; OLG Hamburg in *LAION*; *Like Company* (C-250/25); *Pelham II* (C-590/23); SNE/SGDL/SNAC v. Meta; *Andersen*; the Midjourney cases; *Thomson Reuters v. Ross*.
11. **US points**: Supreme Court outcome in *Thaler*; exact holding in *Cox v. Sony*; any Copyright Office statement on AI-generated typefaces.
12. **Cour de cassation, 26 June 2024**, on *parasitisme*: numbers and wording.
13. **French law on AI and copyright after 2025**; the exact article for the TDM opt-out (R122-28).
14. **Licences and training data** of candidate models: FontDiffuser, DG-Font, MX-Font, LF-Font, Diff-Font; LaMa and MI-GAN weights; current Stability and Black Forest Labs terms.
15. **Font-identification services**: terms of use, and whether any offers an API usable in a commercial product; licence terms of public font-recognition datasets.
16. **Type-community positions**: ATypI, the Monotype report and reactions; any "AI training licence" offers.
17. **Small facts**: death dates used in §5.5; Ascender's authorship of Liberation and Arimo; browser support of the Local Font Access API; the foreign cases cited for colour (*Founder v. P&G*, the Ultraman cases).

### 10.2 Sources read in this session

- `/home/user/ebook-cadeau/assets/fonts/OFL-Spectral.txt`, `OFL-BebasNeue.txt`, `OFL-CourierPrime.txt`, `OFL-AtkinsonHyperlegible.txt`: full SIL OFL 1.1 text and copyright headers.
- The eight `.ttf` files in `/home/user/ebook-cadeau/assets/fonts/`: OS/2 `fsType` and version; `name` table entries 0, 1, 5, 7, 13, 14. `SHA256SUMS` and `README.md` in the same folder.
- URLs printed in those files (licence headers, README and `name` tables): [https://openfontlicense.org](https://openfontlicense.org), [https://scripts.sil.org/OFL](https://scripts.sil.org/OFL), [https://github.com/google/fonts](https://github.com/google/fonts), [https://github.com/productiontype/Spectral](https://github.com/productiontype/Spectral), [https://github.com/quoteunquoteapps/CourierPrime](https://github.com/quoteunquoteapps/CourierPrime), [https://github.com/dharmatype/Bebas-Neue](https://github.com/dharmatype/Bebas-Neue).
- `/home/user/ebook-cadeau/crates/ebc-text/src/font.rs` and `/home/user/ebook-cadeau/docs/architecture.md`: existing `fsType` decoding and spine design.
- Sibling notes in `/home/user/ebook-cadeau/docs/research/notes/`: `legal-covers-reseller.reseller.md`, `legal-covers-reseller.independent.md`, `print-requirements.prepress.md`, `cover-sources.scraping-law.md`. Web links carried from them are marked "sibling note".

### 10.3 Legal references cited from memory, to look up

- **USA:** 37 CFR § 202.1(a), (e); Compendium of U.S. Copyright Office Practices (Third) §§ 723, 906.4; H.R. Rep. 94-1476; *Eltra Corp. v. Ringer*, 579 F.2d 294 (4th Cir. 1978); *Adobe Systems v. Southern Software* (N.D. Cal. 1998); *Agfa Monotype v. Adobe* (N.D. Ill. 2005); *Laatz v. Zazzle* (N.D. Cal., 5:22-cv-04844); 35 U.S.C. § 171; MPEP § 1504.01(a); 88 Fed. Reg. 16190; Copyright Office, "Copyright and Artificial Intelligence", Parts 1–3; *Thaler v. Perlmutter* (D.C. Cir. 2025); *MGM v. Grokster* (2005); *Cox v. Sony* (2026).
- **EU:** Reg. (EC) 6/2002 arts. 3, 5, 6, 10, 11, 12, 19, 20; Implementing Reg. (EC) 2245/2002 art. 4(4); Reg. (EU) 2024/2822; Dir. (EU) 2024/2823; Dir. 2001/29/EC; Dir. 2009/24/EC; Dir. (EU) 2019/790 arts. 3, 4, 7, recital 18; Dir. 2006/114/EC art. 4(g); Reg. (EU) 2017/1001 art. 14; Reg. (EC) 864/2007 art. 8; Reg. (EU) 2022/2065 arts. 4–6; Reg. (EU) 2024/1689 arts. 2(12), 3(3), 3(4), 3(63), 50, 53, 99, recital 106; Commission guidelines on general-purpose AI obligations (July 2025).
- **CJEU:** C-5/08 *Infopaq*; C-476/17 *Pelham*; C-590/23 *Pelham II*; C-683/17 *Cofemel*; C-833/18 *Brompton*; C-580/23 and C-795/23 *Mio/konektra*; C-487/07 *L'Oréal v Bellure*; C-324/09 *L'Oréal v eBay*; C-265/16 *VCAST*; C-426/21 *Ocilion*; C-30/14 *Ryanair*; C-250/25 *Like Company*.
- **France:** CPI L112-2 7°, 8° and 13°, L121-1, L122-4, L122-5 2°, 4° and 10°, L122-5-3, L122-6-1, R122-28, L335-2, L335-3, L511-1, L513-1, L513-4; Code civil art. 1240; Code pénal art. 121-7; Code de la consommation arts. L122-1 and following; Cass. com., 26 June 2024, n° 23-13.535.
- **Germany:** DesignG §§ 1, 61; SchriftzG 1981; UrhG §§ 14, 23, 44b, 60d, 69a; UWG § 4; BGH I ZR 21/57 and I ZR 143/12; LG Köln 28 O 133/97; LG München I 3 O 17493/20 and 42 O 14139/24; LG Hamburg 310 O 227/23.
- **UK:** CDPA 1988 ss. 4, 54, 55, 84, 178; *Getty Images v Stability AI* [2025] EWHC 2863 (Ch).
- **International:** Vienna Agreement for the Protection of Type Faces and their International Deposit, 12 June 1973.
- **Papers:** FontDiffuser (AAAI 2024); DG-Font (CVPR 2021); LF-Font (AAAI 2021); MX-Font (ICCV 2021); Diff-Font; MC-GAN (CVPR 2018); Lopes et al. (ICCV 2019); DeepFont (2015); Chen et al. (2019).

## Implications for the implementation

Numbers marked "engineering default" are starting values to calibrate, not legal thresholds.

**Spine pipeline**

1. Choose the spine source in this fixed order: (a) spine or flat cover supplied by the publisher or reseller; (b) nearest open font from the index; (c) house face. No other source runs without an explicit opt-in.
2. Place supplied spine artwork with uniform scaling only: 0 % non-uniform scaling, no crop, no recolouring. Fill any remaining area of the 10 mm panel with a flat colour sampled from the supplied spine.
3. Default background: one flat colour, sampled from the binding-edge strip of the cover (engineering default: the outer 2 % of the cover width, median colour). The default pipeline must put no pixel of the cover on the spine panel.
4. The current architecture text describes "a blurred/extended edge strip" for non-uniform edges (`docs/architecture.md`, ebc-spine, "Synthesis"). Move that variant behind the "own rights" gate (item 20). The default for a non-uniform edge is still a flat colour.
5. Run a three-class image classifier on the title region: *typeset*, *lettered*, *logo*. No language model.
6. If the class is *typeset* and the best index match scores at least 0.80 on the 0–1 scale (engineering default), use that open font. Otherwise, and always for *lettered* and *logo*, use a house face by class: Spectral for serif, Bebas Neue for condensed display, Atkinson Hyperlegible for sans.
7. Draw the spine title and author as filled outlines. Fit the cap height to at most 7 mm on the 10 mm spine, inside the spine safe margins, and never set type below 6 pt (printer minimum [V2, sibling note `print-requirements.prepress.md`]). Choose the text colour for a contrast ratio of at least 4.5:1 (engineering default).
8. Spine content is the title and the author as plain text. Never place a publisher logo, a collection monogram or other series device on a generated spine.

**Fonts: bundle, index, manifest**

9. Keep a font manifest with one entry per file: family, style, version, SPDX licence identifier, copyright line, source URL, SHA-256, `fsType`, Reserved Font Name (or "none"), trade mark notice.
10. CI fails if: a font file has no manifest entry; a hash differs; the licence is not in the allowlist {OFL-1.1, Apache-2.0}; `fsType` is not 0x0000; the matching licence text file is missing; a Reserved Font Name is declared and the shipped file differs from the upstream original. Adding UFL-1.0 or a public-domain-dedicated file needs a manual review recorded in the manifest.
11. The index builder refuses any font that is not in the manifest. The index and the classifier labels contain open fonts only.
12. Apply the catalogue rule of §5.5 when adding families: no close copy of a commercial face first published less than 25 years ago. Record the basis (revival, original design, long-distributed clone) in the manifest.
13. Ship the font files unmodified. Subsetting happens only inside the PDF writer.
14. Ship the `OFL-*.txt` files with every build (web, WASM, CLI), and a credits page that lists each font, its authors, its copyright line and its licence. For Bebas Neue, print both copyright lines found in the repository. If an Apache-2.0 font is added, ship its LICENSE and NOTICE files the same way.
15. Serve fonts and weights from our own origin only. Set `font-src 'self'` in the Content-Security-Policy. CI fails if the code or the built site references `fonts.googleapis.com`, `fonts.gstatic.com`, `use.typekit.net` or any font-identification service.

**`fsType` and embedding**

16. Decode `fsType` as in §4.3, with the most restrictive reading for malformed fonts of OS/2 version 3 or later (already implemented). Unit-test each row of the table.
17. Embed only bundled fonts, as subsets. Never embed a user-supplied font: use outlines only.
18. For a user-supplied font: refuse if `fsType` has the restricted bit (0x0002) or the bitmap-only bit (0x0200), or if there is no OS/2 table. Substitute a house face and tell the user why. Log the decision in the PDF metadata.

**Bring-your-own-font**

19. Offer it in standalone mode only, not in reseller checkout. Read the file with the file picker, or with the Local Font Access API where the browser has it. Keep the bytes in memory only: no upload, no request that carries them, no persistent storage. Add a test that fails if font bytes appear in any network request. Require a ticked warranty before use, for example: « Je dispose d'une licence m'autorisant à utiliser cette police pour créer ce document. Le fichier reste sur mon appareil. »

**Generative features**

20. AI lettering, outpainting, edge-strip extension, stretch and mirror are disabled by default and compiled behind a feature flag. They are reachable only in "own rights" mode. In every mode the front cover panel stays pixel-identical to the supplied cover, apart from proportional scaling and colour conversion.
21. "Own rights" mode needs two separate confirmations before each generation: (a) rights: « Je suis l'auteur de cette couverture et de son lettrage, ou je dispose de l'autorisation écrite de leurs auteurs pour les adapter. » (b) AI disclosure: « Le dos sera généré par une IA et la carte le mentionnera. » The mode never starts automatically and never runs on a cover fetched by ISBN.
22. In reseller mode, generative features run only when the feed carries a per-ISBN publisher flag. Three flags, all false by default: `spine_artwork_supplied`, `ai_lettering_allowed`, `outpainting_allowed`. For a title classified *lettered* or *logo*, `ai_lettering_allowed` is ignored.
23. A model outputs the characters of the title and author strings only, as raster or outlines. It never writes a font file and never exposes a reusable glyph set.
24. Every card with a generated element carries: a visible mention on the inside page, at 6 pt or larger (for example « Dos généré avec l'aide d'une IA »), and a machine-readable mark in the PDF metadata (XMP digital source type for AI-generated or composite content; exact vocabulary term to confirm [M]). Record the model version and the hash of the training manifest.
25. Download model weights only after the user enters "own rights" mode, and only from our origin.
26. Fetch a signed feature-flag and exclusion list at the start of each session; cache it for at most 24 hours. It must be able to disable every generative feature and to exclude single ISBNs. Exclude film and TV tie-in covers, licensed-character titles and franchise titles with registered title logos from all generative paths by default.

**Training**

27. Train the classifier and any generative model only on synthetic renders of open fonts taken from a pinned revision of the Google Fonts repository, licence in {OFL-1.1, Apache-2.0, UFL-1.0}, and listed in the training manifest (item 28). No commercial font, no scraped cover, no third-party dataset.
28. Publish a training manifest with each model: font, version, licence, source, hash. Make the weights reproducible from it. CI checks that every training input is in the manifest.
29. Do not ship or fine-tune FontDiffuser, DG-Font, MX-Font, LF-Font, Diff-Font, Stable Diffusion or FLUX checkpoints in production builds.

**Inputs we must not touch**

30. The EPUB reader used for cover extraction ignores every font resource (`font/*`, `application/font-*`, `application/vnd.ms-opentype`) and never de-obfuscates one. PDF import never extracts embedded fonts. Add a test with an EPUB that contains an embedded font.
31. The interface, the match results, the marketing copy and the PDF metadata of default-mode cards never contain a commercial font name. Add a denylist test over the UI strings (Helvetica, Arial, Times New Roman, Futura, Gotham, Frutiger, Univers and similar).

**Metadata and logs**

32. Write to the PDF metadata of every card: spine source (`publisher`, `open-font`, `house-font`, `user-font`, `ai-lettering`), background mode (`flat`, `supplied`, `ai-outpaint`), and for each font used: family, version, SPDX licence. For a user-supplied font write the family, the `fsType` and "licence: user-warranted". The preflight crate checks that these keys exist.
33. In reseller mode, log per ISBN: spine source, font family and licence, background mode, application version, date. Keep the log available for the reseller audit (clause X.4(e)).
34. Add a preflight check that, in default mode, the spine panel of page 1 contains vector fills only: no image object.
