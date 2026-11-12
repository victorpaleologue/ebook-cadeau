# Legal: fonts on covers — generative AI reproducing lettering, and outpainting

Topic: legal-fonts-genai · Angle: genai · Date: 2026-11-12 · Scope: France first, then EU; US and UK for contrast.

## 0. Status of this note — read first

**No web search could be run.** All five searches I sent were refused: the WebSearch budget for this workflow turn (200 calls, shared by all agents) was already used up. Fetching was not an alternative (network allowlist; GitHub access limited to this repository). I did not work around the limit.

Consequences:

- **Nothing here is [V1] or [V2].** No URL is cited, because none appeared in a search result and the rules forbid guessed URLs. Sources are given as formal citations (statute, article, case name, court, date) so they can be checked.
- This is not legal advice; it is a research memo from memory with one locally verified source. It needs a verification pass (list in §9) before any decision relies on it.

Confidence markers used:

| Marker | Meaning |
|---|---|
| [L] | Verified in this session against a file in this repository |
| [S] | Taken from a sibling note in `docs/research/notes/` that cites its own sources; not re-verified by me |
| [M+] | From memory, high confidence (statute wording, landmark decision) |
| [M] | From memory |
| [M?] | From memory, low confidence — details may be wrong |

My knowledge stops around mid-2026; anything after that is unknown to me.

## 1. Bottom line

1. **The title on the spine is the same string as the title on the cover.** An AI "style transfer" therefore re-draws the same letters in the same forms. Legally that is much closer to reproducing the lettering than to borrowing a style. Style is free; these specific letter drawings are not necessarily free.
2. **Typeset titles (a real font):** in the US the glyph shapes are unprotected [M+]. In the EU the shapes can be protected by a registered design (up to 25 years), by the unregistered EU design (3 years, against copying) and, in France, by copyright if original [M+]. An AI copy made from the cover is "copying" by construction. Exposure is real but narrow, and concentrated on recent display faces.
3. **Hand-lettered titles and title logos:** a graphic work in France and the EU. AI re-rendering is an unauthorised adaptation plus a moral-rights problem, and for franchise titles a trade-mark problem. Do not imitate.
4. **Outpainting the artwork onto the spine** is the riskiest operation in the whole product: adaptation right, right of integrity (inalienable in France, so a publisher cannot clear it), AI-labelling duties, and the most hostile stakeholder group (illustrators, authors, publishers).
5. **Training:** only on OFL/Apache/UFL fonts. The OFL 1.1 text has no field-of-use restriction and expressly grants "use, study, copy, merge, embed, modify" [L]. Commercial font EULAs increasingly forbid ML training [M?], and a contractual reservation defeats the EU commercial TDM exception [M+].
6. **Pretrained research models** (FontDiffuser, DG-Font, MX-Font, LF-Font, Diff-Font) were trained on font collections whose licences are not documented in the papers, mostly CJK [M]. Do not ship their weights; retrain on a documented open corpus.
7. **Running in the browser** moves the act of generating the output to the user. It does not make the service a neutral bystander, does not remove "provider" status under the AI Act, and stops helping the moment the PDF is sent to us or a printer.
8. **I know of no court ruling anywhere on AI-generated fonts or lettering** as of mid-2026 [M?]. All of §2 is reasoning from general rules.
9. Recommended default: rungs 1–2 of the ladder (open font, nearest open look-alike) on a flat-colour spine; rungs 5–6 only in an "own rights" mode with a warranty, or on per-title publisher opt-in, and labelled.

## 2. Output side

### 2.1 What protects letter shapes

| Jurisdiction | Typeface design (shapes) | Font file | Hand lettering / title logo |
|---|---|---|---|
| **US** | Not copyrightable: 37 C.F.R. § 202.1(e) "typeface as typeface"; § 202.1(a) excludes "mere variations of typographic ornamentation, lettering or coloring"; *Eltra Corp. v. Ringer* (4th Cir. 1978); Compendium (Third) § 906.4 [M+]. Design patents on type fonts exist, 15 years from grant [M+]. Font names are trade marks [M+]. | Protected as a computer program: Copyright Office 1992 rule; *Adobe v. Southern Software* (N.D. Cal. 1998) [M+] | Usually refused as "mere lettering" unless there is separable pictorial authorship [M] |
| **EU** | "Typographic typefaces" are named as products in art. 3(b) of Regulation 6/2002 [M+]. Registered: 25 years. Unregistered: 3 years from disclosure in the EU, only against copying (art. 11, 19(2)) [M+]. Scope: no "different overall impression" on the informed user, taking the designer's freedom into account (art. 10) [M+]. Copyright: only originality counts, no extra threshold for applied art (*Cofemel* C-683/17; *Brompton* C-833/18; *Mio/konektra* C-580/23 and C-795/23, Dec. 2025) [M+ / M for the last]. | Computer program (Directive 2009/24) to the extent it is code [M] | Graphic work if original |
| **France** | CPI L. 112-2 8° lists "œuvres graphiques et typographiques" [M+]; CPI L. 511-1 lists "caractères typographiques" for dessins et modèles [M+]. Copyright needs originality; term life + 70. No French typeface judgment verified. | Logiciel, L. 112-2 13° [M+] | Graphic work; moral rights L. 121-1 perpetual and inalienable [M+] |
| **UK** | Copyright exists but CDPA s. 54 allows using a typeface "in the ordinary course of typing, composing text, typesetting or printing", even with an infringing font; s. 55 limits protection to 25 years from first marketing; s. 54(2) keeps liability for articles "specifically designed or adapted for producing material in a particular typeface" [M+] | Program | Artistic work |
| **Germany** | Design law; copyright only for exceptional faces (BGH *Candida*, 1958); the 1981 Schriftzeichengesetz was repealed in 2004 [M] | Program (LG Köln, 2000) [M?] | Work if original |

EU design reform: Regulation (EU) 2024/2822 applies in two steps (1 May 2025 and 1 July 2026) [M]. It keeps typefaces in the product definition, adds as infringement the creating, downloading, copying or sharing of "any medium or software which records the design" for the purpose of making a product (new art. 19(2)(d)), and adds permitted uses for referential use and for comment, critique or parody [M]. Directive (EU) 2024/2823 must be transposed by 9 December 2027 [M].

The 1973 Vienna Agreement on type faces never entered into force [M+].

### 2.2 AI re-rendering of a typeset title

- **US:** shapes are free and we output raster or outlines, not font software. The font-software cases do not apply. Residual: design patents (rare, short) and font-name trade marks (never display a commercial font name as ours) [M+].
- **EU design right:** a few-shot model that reads the cover and regenerates the glyphs derives them from the design. That satisfies "copying" for the unregistered right. The private, non-commercial exemption (art. 20(1)(a)) covers a consumer printing at home, not a service or a reseller [M+].
- **Open point:** whether 10–30 glyphs of a title are a use of "the design" of a whole typeface. I know of no case. The limited design freedom in text faces narrows the scope of protection; display faces have more freedom and therefore broader protection [M, reasoning].
- **French copyright:** partial reproduction is enough if the part carries the originality (*Infopaq* C-5/08; *Pelham* C-476/17: a recognisable fragment counts, and member states cannot add exceptions) [M+]. *Mio/konektra* frames the copyright test as recognisable reproduction of creative elements, not "overall impression" [M].
- **Why this is worse than cropping (rung 4):** the glyphs printed on the cover come from the publisher's licensed use of the font, and reproducing the licensed cover carries that chain. An AI re-drawing has no licence chain at all.
- **Where the exposure concentrates:** recent display faces on new releases (inside the 3-year unregistered window), faces from foundries that register designs, and distinctive faces by living French or EU designers. Classic text faces are practically safe.
- **French precedent on tone:** in 2010 Hadopi had to withdraw its logo because it used "Bienvenüe", a typeface exclusive to France Télécom [M+]. Font misuse in France has a reputational cost independent of damages.

### 2.3 AI re-rendering of hand-lettered title art

- A hand-drawn title is a graphic work. CPI L. 122-4 prohibits any reproduction, "adaptation, transformation, arrangement" without consent [M+].
- The output re-draws the same word in the same forms, rotated and resized: an adaptation, not an independent creation.
- **Moral rights:** integrity (a degraded or "almost right" redraw is the textbook complaint) and attribution. They belong to the lettering artist; the publisher cannot waive them [S: `legal-covers-reseller.reseller.md` §4].
- **Exceptions do not fit.** Pastiche (L. 122-5 4°; CJEU reference *Pelham II*, C-590/23, AG opinion June 2025, judgment status unknown to me [M?]) requires some artistic dialogue; a functional spine label has none. Private copying covers the home user at most, and whether a gift is "usage privé du copiste" is unsettled [S: `legal-covers-reseller.independent.md`].
- **Trade marks:** franchise title logos are often registered figurative marks [M]. In *Getty v Stability* the model provider was held liable for trade-mark infringement where outputs reproduced a watermark [M+]. A tool that redraws a registered title logo is in the same position. Plain retyped text is referential use; an imitated logo is not.

### 2.4 Outpainting the cover onto the spine

- Adaptation right (L. 122-4) and integrity (L. 121-1). French courts treat cropping, mirroring, colourisation and cut-outs as violations [S: reseller note §4, with Livres Hebdo and a €15,000 award]. Adding invented content contiguous to the work, presented as part of it, is a stronger alteration than a crop.
- It also attributes to the illustrator content they did not make (UK: false attribution, CDPA s. 84 [M]; France: integrity and art. 1240 C. civ.).
- Germany: §§ 14 and 23 UrhG (distortion; adaptations need consent for exploitation) [M+].
- Stretching or mirroring edge pixels is non-AI but legally the same alteration [S].
- A **flat colour sampled from the cover** is not a reproduction: a colour is not a work [M, reasoning]. This is the only background treatment that is clean without consent.
- Generic inpainting models bring their own baggage: training-data litigation (§5) and model licences (§3.5).

### 2.5 AI output: authorship and labelling

- **US Copyright Office:** registration guidance of 16 March 2023 (88 Fed. Reg. 16190); Report "Copyright and Artificial Intelligence", Part 1 Digital Replicas (July 2024), Part 2 Copyrightability (29 Jan. 2025: prompts alone do not make the user an author; human-authored elements and arrangements remain protectable), Part 3 Generative AI Training (pre-publication version, 9 May 2025: training is fair use in some cases and not others) [M+]. *Thaler v. Perlmutter* (D.C. Cir., 18 March 2025) confirmed the human-authorship requirement [M+]; certiorari reportedly denied in 2026 [M?].
- I know of **no Copyright Office statement specific to AI font generation** [M?]. By inference: an AI-generated glyph set is doubly unprotected in the US (typeface rule plus no human author).
- **EU:** no harmonised rule, but "author's own intellectual creation" presupposes a human; the Prague Municipal Court (2023) refused copyright in an AI image [M]. We cannot own the generated spine. That is harmless for us.
- **AI Act (Regulation 2024/1689), art. 50(2):** providers of AI systems generating synthetic image content must mark outputs in a machine-readable, detectable way; applicable from 2 August 2026 [M+]. Exception for systems with "an assistive function for standard editing" or that "do not substantially alter the input data" [M+]. Whether spine lettering or outpainting fits the exception is untested. Fines for art. 50 breaches: up to €15 million or 3 % of turnover (art. 99(4)), with the lower cap for SMEs [M]. The open-source exclusion does not cover art. 50 systems (art. 2(12)) [M+]. The 2025 "Digital Omnibus" proposed adjustments to these dates; outcome unknown to me [M?].

## 3. Training side

### 3.1 Commercial font EULAs

- Font licences are contracts over the font software. Monotype's current EULAs are reported to prohibit using the fonts to train AI/ML [M?]; several independent foundries have added similar clauses since 2023 [M?]; Adobe's general terms contain a no-AI-training clause for its services and software [M?]. Exact wording not checked.
- A EULA binds licensees only. Someone who never licensed the font and learns from rendered images is not bound by it, but still needs a copyright basis in the EU (§3.2).
- Contract can restrict beyond copyright where no mandatory exception applies (*Ryanair v PR Aviation*, C-30/14) [M+].

### 3.2 EU text-and-data-mining exceptions

- Directive 2019/790 art. 3: research organisations and cultural heritage institutions, scientific research, no opt-out, cannot be overridden by contract (art. 7(1)) [M+]. Not us.
- Art. 4: anyone with lawful access, **unless the rightholder has expressly reserved the use "in an appropriate manner"**, machine-readable for online content; recital 18 accepts contractual terms or unilateral declarations in other cases [M+]. A EULA clause against ML training is such a reservation. Training a commercial model on EULA-restricted fonts is therefore outside the exception and a breach of contract.
- France: CPI L. 122-5 10° and L. 122-5-3 (ordonnance of 24 Nov. 2021); the opt-out need not be reasoned and may be expressed by any means, including site terms (R. 122-28) [M / M? for the article number].
- Case law: LG Hamburg, 27 Sept. 2024, *Kneschke v LAION* (dataset building covered by the research exception; a natural-language reservation may count as machine-readable, obiter) [M+]; appeal decided by OLG Hamburg in Dec. 2025, reportedly upheld [M?]. LG München I, 11 Nov. 2025, *GEMA v OpenAI*: memorised works inside model weights are reproductions, not covered by TDM, and the provider, not the user, answers for outputs [M+]. First CJEU reference on generative AI: *Like Company v Google*, C-250/25, pending at my last information [M?].
- French publishers (SNE) have promoted TDM opt-outs since 2023–24 [M]. Scraped French cover images are therefore a poor training source.

### 3.3 Open-licensed fonts

Verified against `assets/fonts/OFL-Spectral.txt` (the file names `https://openfontlicense.org` as the home of the licence and FAQ) [L]:

- Grant: "to use, study, copy, merge, embed, modify, redistribute, and sell modified and unmodified copies of the Font Software". No field-of-use limit, so training is within "use" and "study".
- Conditions: not sold by itself; copyright notice and licence travel with each copy; no Reserved Font Name on a Modified Version; authors' names not used to promote modified versions; derivatives stay under the OFL.
- "Modified Version" is "any derivative made by adding to, deleting, or substituting — in part or in whole — any of the components of the Original Version, by changing formats or by porting".
- "The requirement for fonts to remain under this license does not apply to any document created using the Font Software."

Reading: a rendered title, a PDF or a raster spine is a "document" and is free of the OFL. Model weights are not obviously a "Modified Version"; a generated installable font that closely reproduces one OFL font arguably is. Mitigation: never output font files; publish the training manifest with licences and attribution. Whether the OFL FAQ addresses AI training: not checked.

Apache-2.0 and Ubuntu Font Licence fonts are likewise permissive [M]. Google Fonts holds on the order of 1,800 families, mostly OFL [M].

### 3.4 AI Act: GPAI duties versus a small specialised model

- Art. 53 (technical documentation, copyright policy honouring art. 4(3) opt-outs, public training-content summary) binds providers of general-purpose AI models, since 2 August 2025 [M+]. A GPAI model shows "significant generality" (art. 3(63)); the Commission guidelines of July 2025 use an indicative threshold around 10^23 FLOP of training compute plus text, text-to-image or text-to-video generation [M]. A font-style model trained on a few thousand fonts is orders of magnitude below. Not GPAI.
- If we embed someone else's GPAI model (a Stable Diffusion or FLUX inpainting checkpoint), the upstream provider carries art. 53; light fine-tuning does not make us a GPAI provider (guidelines: about one third of the original compute) [M].
- We are still the **provider of an AI system** (art. 3(3)) wherever inference runs, so art. 50(2) is ours. A consumer using it privately is not a "deployer" (art. 3(4)) [M+].

### 3.5 Datasets and weights in the literature

| Work | Training data as I recall it | Reuse issue | Conf. |
|---|---|---|---|
| FontDiffuser (AAAI 2024) | ~400 Chinese fonts × ~800 characters collected by the authors | Font licences undocumented; repository licence not checked | [M] |
| DG-Font (CVPR 2021) | ~400 Chinese fonts | Same | [M] |
| LF-Font, MX-Font (NAVER Clova, 2021) | Chinese and Korean font collections; code MIT | Code licence does not clear the data | [M] |
| Diff-Font (2022–24) | Chinese and Korean fonts | Same | [M] |
| MC-GAN "Capitals64" (CVPR 2018) | ~10,000 Latin fonts gathered from the web, capitals only | Scraped; includes commercial faces | [M] |
| SVG-Fonts / "Glyphazzn" (Lopes et al., 2019), used by DeepVecFont | ~14 million glyphs from fonts found online | Scraped | [M] |
| AdobeVFR (DeepFont, 2015) | 2,383 Adobe font classes | Adobe-owned | [M+] |
| MyFonts tag dataset (Chen et al., 2019) | ~18,800 MyFonts fonts | Monotype catalogue | [M] |
| Stable Diffusion 1.5 / SDXL | LAION subsets; OpenRAIL-M / OpenRAIL++-M | Use restrictions; data under litigation | [M+] |
| FLUX.1 schnell / dev | Undisclosed; Apache-2.0 / non-commercial | dev unusable commercially | [M+] |
| LaMa, MI-GAN (small inpainting, browser-capable) | Places2 and similar; Apache-2.0 / MIT | Dataset research terms | [M / M?] |

Conclusion: no off-the-shelf font-generation checkpoint has a clean, documented data lineage for commercial use in the EU. A model trained from scratch on Google Fonts (Latin-only suits a French market) is the defensible route.

## 4. Who is liable when the model runs in the browser

| Scenario | User | Service (us) | Reseller | Printer |
|---|---|---|---|---|
| **S1** Standalone, print at home, generation fully client-side | Makes the copy. Private-copy exception and the private non-commercial design exemption are available in principle; the "gift" point is unsettled. Practically untouchable. | Tool supplier. France: complicité par fourniture de moyens (C. pén. 121-7), faute (C. civ. 1240), parasitisme [M]. Exposure depends on how the tool is designed and marketed. | — | — |
| **S2** Standalone, professional print order | Warrants rights | **Primary**: we have the copy made and distributed (CPI L. 335-2, L. 335-3). Good faith is irrelevant in civil contrefaçon [M+]. No hosting safe harbour: we play an active role (*L'Oréal v eBay*, C-324/09) [M+]. | — | Co-liable; will demand a warranty |
| **S3** Reseller checkout option | Buyer: none in practice | Primary or joint for generated elements; the sibling note's draft clause X.6(ii) has us **warranting** fonts, backgrounds and generated elements to the reseller [S] | Primary: commercial offer, and breach of its publisher licence if the cover is altered | Co-liable |

Does in-browser execution change anything?

1. **Yes, for the generation step:** the copy is made on the user's device at the user's request, so the user is the maker (logic of the BGH online-recorder cases and, a contrario, *VCAST* C-265/16 and *Ocilion* C-426/21, where the service made the copies) [M]. Exceptions reserved to private users can only apply if the user is the copier.
2. **No, for "mere tool" status:** a feature whose stated purpose is to imitate the cover's lettering is not a neutral staple article. *Grokster* (inducement) and *Cox v. Sony* (US Supreme Court, 2026: liability for services induced or tailored to infringement [M?]) point the same way as the French "rôle actif" reasoning. If the service itself fetches the cover by ISBN and auto-runs the imitation, the user's agency is thin.
3. **No, for regulation:** AI Act provider status and art. 50 marking are unaffected.
4. **It adds an act:** we distribute the weights to every browser in the EU. If weights can embody training works (*GEMA v OpenAI*; contrast *Getty v Stability*, where the UK court held the Stable Diffusion model is not an "infringing copy" [M+]), each download is a reproduction and distribution in the EU, wherever training took place. Recital 106 of the AI Act takes the same territorial line for GPAI [M]. A clean training set removes the issue.
5. **Safe harbours are beside the point:** DSA arts. 4–6 protect transmission and hosting of users' information [M+]. Nothing is hosted in S1. In S2/S3 we manufacture.
6. **No logs, no moderation:** we cannot notice-and-act on outputs we never see. Safeguards must be built into the client (gating, exclusion lists, labelling), and that design is what a court would assess.

Output-liability precedents so far put it on the provider: LG München I (*GEMA*), Guangzhou Internet Court (Feb. 2024, Ultraman images) [M]. All concern server-side services.

## 5. Disputes and commentary, 2023–2026

| Matter | Point for us | Conf. |
|---|---|---|
| *Getty Images v Stability AI*, [2025] EWHC 2863 (Ch), 4 Nov. 2025 | Training claim dropped (training outside the UK). Model weights not an "infringing copy"; an intangible "article" is possible. Limited trade-mark infringement for watermarked outputs. Appeal on secondary infringement reportedly allowed [M?]. US case refiled in N.D. Cal., Aug. 2025 [M]. | [M+] |
| *GEMA v OpenAI*, LG München I, 11 Nov. 2025 | Memorisation = reproduction; provider liable for outputs | [M+] |
| *Kneschke v LAION*, LG Hamburg 2024; OLG Hamburg Dec. 2025 | Research TDM; form of opt-outs | [M+] / [M?] |
| *Andersen v. Stability AI* (N.D. Cal.) | Artists' direct and inducement claims survived dismissal (Aug. 2024) | [M] |
| *Thomson Reuters v. Ross* (D. Del., Feb. 2025) | Fair use rejected for a non-generative AI tool; on appeal | [M+] |
| *Bartz v. Anthropic*; *Kadrey v. Meta* (N.D. Cal., June 2025) | Training on lawfully acquired books held fair use; pirated sources not; USD 1.5 bn settlement in *Bartz* | [M+] |
| *Disney and Universal v. Midjourney* (C.D. Cal., June 2025); Warner Bros. (Sept. 2025) | Output-side claims over recognisable characters | [M+] |
| SNE, SGDL, SNAC v. Meta (TJ Paris, March 2025) | The French book trade litigates against AI training; status unknown | [M+] |
| *Like Company v Google*, C-250/25 | First CJEU reference on training and outputs | [M?] |
| US Copyright Office AI report, Parts 1–3 | §2.5 | [M+] |
| Monotype "Re:Vision" trends report (2025) and press coverage | Monotype presented AI as an opportunity; type designers objected publicly | [M?] |
| Foundry EULAs with no-AI-training clauses | §3.1 | [M?] |
| Rulings on AI-generated fonts or lettering | None known to me | [M?] |

For comparison, typeface litigation has almost always been about **font software licences**, not shapes (e.g. *Font Bureau v. NBC*, 2009; *Berthold v. Target*, 2017) [M]. China is an exception: courts have protected font libraries and sometimes single glyphs (*Founder v. Procter & Gamble*, Beijing, 2011, decided on implied licence) [M].

Positions of the type-design community (ATypI and similar): not verified.

## 6. Risk ladder

Risk is for the service in France/EU with a commercial print run; "typeset" = title set in a font, "lettered" = hand-drawn title or logo.

| # | Option | What is taken | Risk (typeset / lettered) | Main hooks | Mitigations |
|---|---|---|---|---|---|
| 0 | Genuine spine from the publisher's print cover file | Publisher's own art | Lowest, with consent | Scope of publisher's rights | Ask for the flat cover ("couverture à plat") in the reseller feed; scale only |
| 1 | Identified open-licensed font | Nothing from third parties | Very low / n.a. | OFL conditions [L] | Ship licence texts and notices; fonts from a curated source with checksums (as in `assets/fonts`); keep originals unmodified because format conversion or subsetting makes a "Modified Version" and triggers Reserved Font Names; never name a commercial font in the UI |
| 2 | Nearest open look-alike | General appearance only | Low / low–medium | Parasitisme if the spine mimics a collection's trade dress [S]; font-name trade marks | Match by visual embedding over open fonts only, so no commercial font data is needed; neutral layout; no logos; for lettered titles choose a house face rather than a near-imitation |
| 3 | Commercial font licensed to the publisher or reseller | Font software | Low if the licence fits; medium otherwise | Licence is personal to the licensee; web-to-print, server and app use are usually separate licences; sending the font to the browser is distribution [M] | Licensee renders the spine title to outlines and supplies SVG/PDF; we never receive the font file; written confirmation of scope |
| 4 | Crop the lettering pixels from the cover | Part of the cover as printed | Medium / medium–high | Partial reproduction; cut-out and recomposition engage integrity [S]; font layer is covered by the publisher's licence chain | Only with a cover licence that covers the card; no recolouring or redrawing; keep the title with its own background strip rather than cutting it out; credit; publisher opt-in flag |
| 5 | AI style transfer of the lettering | Letterforms re-drawn | Medium–high / **high** | EU unregistered and registered design; French copyright; adaptation and integrity for lettering; trade marks for title logos; AI Act art. 50; training lineage | Model trained only on open fonts with a published manifest; output raster or outlines, title string only, never a font; a typeset-versus-lettered classifier that refuses lettered titles and logos; available only in "own rights" mode (author or publisher uploads own cover and warrants) or per-title publisher opt-in; off by default in reseller mode; machine-readable AI marking in the PDF metadata and a visible mention; fallback to rung 2 |
| 6 | AI outpainting of the artwork | The illustration, extended | **High** / **high** | Adaptation; integrity (not waivable by the publisher); false attribution; AI Act art. 50; generic-model data and licences; relations with publishers and illustrators | Default spine = flat colour sampled from the cover; no stretch or mirror either; outpainting only when the uploader is the artist or holds written consent from the artist; keep the cover panel pixel-identical; label; exclude from the warranty given to resellers |

Product consequence: a 10 mm spine leaves a cap height of roughly 5–7 mm. The visual gain from exact lettering over a good open look-alike is small; the legal delta between rungs 2 and 5 is large.

## 7. Guardrails to build

- Decision order: publisher spine → open font match → house font. AI paths are opt-in, gated and logged in the PDF metadata.
- Classifier (image model, not a language model) for "typeset / lettered / logo"; lettered and logo titles never go to imitation.
- Exclusion list for franchise titles with registered logos.
- Training manifest: font, version, licence, source, hash; reproducible build of the weights.
- Reseller contract: generated lettering and outpainting excluded from our warranty unless the publisher opted in.
- Provenance: XMP/IPTC digital-source marking for any generated element.

## 8. Sources

No web sources: no search could be executed (§0). Verified locally:

- SIL Open Font License 1.1, full text, as bundled in `/home/user/ebook-cadeau/assets/fonts/OFL-Spectral.txt` (and the three sibling `OFL-*.txt` files); the file itself points to `https://openfontlicense.org` for the licence and FAQ.
- Sibling notes: `/home/user/ebook-cadeau/docs/research/notes/legal-covers-reseller.reseller.md` (§4 moral rights, §5 trade dress, §7 risk table, clause X.6); `/home/user/ebook-cadeau/docs/research/notes/legal-covers-reseller.independent.md` (private copying, DSA, parasitisme).

Cited from memory, to be looked up by citation:

- US: 37 C.F.R. § 202.1(a), (e); Compendium of U.S. Copyright Office Practices (Third) §§ 723, 906.4; *Eltra Corp. v. Ringer*, 579 F.2d 294 (4th Cir. 1978); *Adobe Systems v. Southern Software* (N.D. Cal. 1998); 88 Fed. Reg. 16190 (16 March 2023); Copyright Office, "Copyright and Artificial Intelligence", Parts 1–3 (2024–2025); *Thaler v. Perlmutter* (D.C. Cir. 2025); *Sony v. Universal* (1984); *MGM v. Grokster* (2005); *Cox v. Sony* (2026).
- EU: Regulation (EC) 6/2002, arts. 3, 10, 11, 19, 20; Regulation (EU) 2024/2822; Directive (EU) 2024/2823; Directive 2001/29; Directive 2009/24; Directive (EU) 2019/790, arts. 3, 4, 7, recital 18; Regulation (EU) 2022/2065, arts. 4–6; Regulation (EU) 2024/1689, arts. 2(12), 3(3), 3(4), 3(63), 50, 53, 99, recital 106; Commission guidelines on GPAI obligations (July 2025).
- CJEU: C-5/08 *Infopaq*; C-476/17 *Pelham*; C-590/23 *Pelham II*; C-683/17 *Cofemel*; C-833/18 *Brompton*; C-580/23 and C-795/23 *Mio/konektra*; C-324/09 *L'Oréal v eBay*; C-682/18 *YouTube/Cyando*; C-265/16 *VCAST*; C-426/21 *Ocilion*; C-30/14 *Ryanair*; C-250/25 *Like Company*.
- France: CPI L. 112-2 8° and 13°, L. 121-1, L. 122-4, L. 122-5 2°, 4°, 10°, L. 122-5-3, R. 122-28, L. 335-2, L. 335-3, L. 511-1; Code pénal art. 121-7; Code civil art. 1240.
- UK: CDPA 1988 ss. 54, 55, 84; *Getty Images v Stability AI* [2025] EWHC 2863 (Ch).
- Germany: UrhG §§ 14, 23, 44b, 60d; LG Hamburg 310 O 227/23; LG München I 42 O 14139/24.
- Papers: FontDiffuser (AAAI 2024); DG-Font (CVPR 2021); LF-Font (AAAI 2021); MX-Font (ICCV 2021); Diff-Font; MC-GAN (CVPR 2018); Lopes et al. (ICCV 2019); DeepFont (2015); Chen et al. (2019).

## 9. Could not verify

Everything except the OFL text. In priority order for a follow-up pass:

1. **Current text of commercial EULAs on AI/ML training**: Monotype/MyFonts, Adobe Fonts, Fontshare, and two or three French foundries (Production Type, Typofonderie, 205TF). Wording and dates.
2. **AI Act art. 50 as of November 2026**: whether the Digital Omnibus changed the application date or added a grace period; state of the Commission's code of practice on marking and labelling; whether the "standard editing" exception has been interpreted.
3. **OFL FAQ**: any statement on AI training, on rasterised output, and on subsetting versus Reserved Font Names.
4. **Any decision or EUIPO invalidity case on typeface designs**: the informed user for a typeface; whether a short text infringes; number of registered EU designs in Locarno class 18-03.
5. **French case law on copyright in typefaces** (originality of a text face versus a display face). None found from memory.
6. **Status after mid-2026** of: *Getty v Stability* appeal; *GEMA v OpenAI* appeal; OLG Hamburg in *LAION* (holding and any BGH appeal); *Like Company* (C-250/25); *Pelham II* (C-590/23); SNE/SGDL/SNAC v. Meta; *Andersen*; the Midjourney cases; *Thomson Reuters v. Ross* (3d Cir.).
7. **US points**: Supreme Court outcome in *Thaler* and the exact holding in *Cox v. Sony*; any Copyright Office statement on AI-generated typefaces.
8. **French legislation** on AI and copyright after 2025 (Senate bill on a presumption of use; CSPLA reports), and the exact article for the TDM opt-out (R. 122-28).
9. **Licences and training data of candidate models**: FontDiffuser, DG-Font, MX-Font, LF-Font, Diff-Font repositories; LaMa and MI-GAN weights; current Stability and Black Forest Labs licence terms.
10. **Type-community positions**: ATypI, Monotype's Re:Vision report and reactions, foundry statements; any "AI training licence" offers.
11. **EU design reform details**: exact numbering and wording of the new art. 19(2)(d) and the new permitted uses in Regulation 2024/2822; second application date.
12. **Chinese and other foreign cases** cited for colour (Founder v. P&G; Ultraman cases; Prague Municipal Court 2023).
