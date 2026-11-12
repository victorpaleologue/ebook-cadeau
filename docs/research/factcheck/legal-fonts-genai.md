# Fact-check report — legal-fonts-genai

_Produced by an adversarial fact-checker on 2026-11-12. Read this next to the matching document: where that document's status is still "draft", these corrections were NOT applied._

# Fact-check of `/home/user/ebook-cadeau/docs/research/08-legal-fonts-and-genai.md`

## Limits — read first

**No fresh web verification was possible.** I sent 5 WebSearch queries (AI Act omnibus, Laatz v. Zazzle, Cox v. Sony, Mio/konektra, EU design reform). All 5 were refused: the shared budget of 200 searches for this turn is spent. I did not work around it.

So every statute, case, licence-term and price claim stays **UNSUPPORTED** by default, and I have no new URLs. What I could check:

- **(A)** the repository itself (font binaries, licence files, code), re-run by me;
- **(B)** sibling notes and documents that were web-verified earlier today. Their URLs are relayed as recorded there, not re-fetched.
- **"Recollection"** lines are my own memory (to mid-2026). They are leads for the next search pass, not verification.

Seven real errors or inconsistencies came out of (A) and (B): claims 2 to 7 below, and the catalogue-rule gap in the omissions.

## 1. The 20 most load-bearing claims

**Checked against the repository (A) or sibling notes (B)**

1. **OFL 1.1 clauses and consequences (§5.1).** CONFIRMED (A). Every quoted clause is in `/home/user/ebook-cadeau/assets/fonts/OFL-Spectral.txt`. The four licence bodies are word-identical; only a trailing newline differs.

2. **Audit of the eight bundled fonts (§5.2, finding 6).** CONFIRMED (A) for `fsType` 0x0000 on all eight, no Reserved Font Name, versions, copyright lines, trade mark notices, the Bebas Neue copyright mismatch, and `SHA256SUMS`.
   - PARTLY for "shipped unmodified … (they are today [L])" in §1.4. `SHA256SUMS` only proves the files match the repo's own list; no upstream hash was compared.
   - Corrected: "self-consistent; identity with upstream not verified".

3. **`fsType` rules and "the decoding itself is already implemented" (§4.3, §1.4, R2, items 16–18).** PARTLY (A).
   - The code reports a font with **no OS/2 table as Installable**, and `allows_subset_embedding()` is then true (`/home/user/ebook-cadeau/crates/ebc-text/src/font.rs` lines 412–421, 440–443). The doc's table says "Reject / Refuse".
   - "Draw nothing with it in a PDF" exists only for 0x0002 (line 33). For 0x0200 the code says only "do not embed" (lines 110–111), which does not support refusing outlines.
   - The version rule (v0–2 least restrictive, v3+ exclusive) is in the code and matches an independent implementation (`/root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ttf-parser-0.25.1/src/tables/os2.rs` lines 427–447).
   - Corrected: "bit decoding is implemented; refusing fonts without OS/2 and bitmap-only fonts is new policy that needs code and tests".

4. **"An outlined spine title plus embedded, subset body fonts satisfies all of them" (§4.4, R3).** WRONG as worded (B).
   - The sibling note and document 03 both say embedding satisfies every printer **except Printoclock**, which wants all text outlined. Document 03 adds a per-printer `outline_all_text` flag (`/home/user/ebook-cadeau/docs/research/03-print-requirements.md` lines 107, 414).
   - Source as recorded: https://www.printoclock.com/bien-realiser-son-fichier

5. **"Cap height is about 5–7 mm" (finding 13); "at most 7 mm … never below 6 pt" (item 7).** WRONG against the project's own geometry (B).
   - Document 03 fixes 2 mm clear each side and a 6 mm text band (lines 30, 49, 530, 559). The ceiling is all-caps at about 24 pt, a cap height of about 5.9 mm without accents.
   - The recommended setting is 9–13 pt, a cap height of about 2.2–3.2 mm (`notes/spine-design.conventions.md` lines 25, 147–153).
   - Document 03 line 418 sets a floor of **8 pt for outlined text**, not 6 pt.
   - The smaller size strengthens finding 13, but item 7 as written breaks the 2 mm margin.

6. **French integrity case law "[V2]" behind "only a flat colour is clean" (finding 9, §1.4, §6.2).** PARTLY (B). It is carried accurately from `notes/legal-covers-reseller.reseller.md` lines 112–114, but:
   - the two sources are a trade-press column and a diocesan practical guide; no decision is identified;
   - doc 08 dropped the counter-example on line 115 of the same note: TJ Bordeaux, 29 Feb 2024, claimants invoking cropping and missing credit lost on all claims (https://www.dalloz-actualite.fr/document/tj-bordeaux-29-fevr-2024-n-2308086);
   - it also dropped the caveat that TGI Paris 5 Oct 2007 was "not read" (`notes/legal-covers-reseller.independent.md` line 249).
   - Corrected: "secondary sources report such case law; the outcome is not automatic".

7. **§7.5 dataset table and "about 1,800 families" (§7.4).** OUTDATED against sibling notes verified today (B).
   - FontDiffuser: README says "can only be used for non-commercial research purposes" (https://github.com/yeungchenwa/FontDiffuser). Doc 08 says "licence not checked".
   - DG-Font and Diff-Font: no licence file; trained on Founder fonts, "free … for non-commercial users" (https://github.com/ecnuycxie/DG-Font).
   - MX-Font and LF-Font: MIT except FUNIT-derived modules; training fonts not released (https://github.com/clovaai/mxfont).
   - Google Fonts: **1,908 families**; licence records OFL 1,828, Apache 36, UFL 5 (https://registry.npmjs.org/google-font-metadata).
   - MI-GAN: MIT code and MIT weights, 27 MB.
   - Internal conflict: R12 bans third-party checkpoints with undocumented data, item 29's list omits MI-GAN and LaMa, and `notes/font-id-tech.generation.md` line 167 recommends MI-GAN outpainting as an opt-in. This needs an explicit decision.

**Web-dependent claims: all UNSUPPORTED (not re-verified)**

8. **EU design basics (§2.2, §2.4): typefaces are products, 25 and 3 years, "using" is covered, private exemption.** UNSUPPORTED. Recollection (high): consistent. Since 1 May 2025 the copying-only rule for unregistered designs sits in art. 19(4), not 19(2); the doc mixes old and new numbering.

9. **Reform Reg. 2024/2822: dates, new art. 19(2)(d), product definition, new permitted uses.** UNSUPPORTED. Recollection (high): consistent. The typeface representation rule cited (Reg. 2245/2002 art. 4(4)) may have been replaced in phase 2 on 1 July 2026; unknown.

10. **Mio/konektra (C-580/23, C-795/23), 4 Dec 2025, no stricter test for applied art.** UNSUPPORTED. Recollection (medium-high): date and holding consistent.

11. **AI Act art. 50(2): 2 Aug 2026, Digital Omnibus "outcome not verified", €15 m / 3 % fines, no open-source exclusion (§7.3, §1.4).** UNSUPPORTED, likely OUTDATED.
    - Recollection (medium): a provisional agreement was reached on 7 May 2026. It gives systems placed on the market before 2 Aug 2026 until 2 Dec 2026 for art. 50(2), and moves high-risk dates to 2 Dec 2027 and 2 Aug 2028. Formal adoption not confirmed.
    - If right, a new system gets no grace period, so "plan as if it applies now" stands.

12. **General-purpose model thresholds: 10^23 FLOP and one third of compute (§7.6).** UNSUPPORTED. Recollection (high): consistent with the July 2025 guidelines.

13. **TDM: art. 4 opt-out, art. 7(1), a EULA clause is a valid reservation, CPI R122-28 "par tout moyen" (§3, §7.4).** UNSUPPORTED. Recollection (medium-high): consistent. The sibling note records that OLG Hamburg (5 U 104/24, 10 Dec 2025) held a natural-language reservation not machine-readable; doc 08 dropped the case number and that point. "Site terms are a valid opt-out" is France-specific.

14. **Licence categories: desktop and Adobe Fonts do not cover a rendering service; server licence needed, per family, on quote (§4.1).** UNSUPPORTED. I obtained no price either.

15. **Bring-your-own-font: "The user's own licence covers it" (finding 5, R6).** UNSUPPORTED and overstated. The doc's own question 6 asks counsel whether this is true. State it as a hypothesis.

16. **Laatz v. Zazzle (§4.5).** UNSUPPORTED. Outcome still unknown.

17. **Getty v Stability: weights are not an infringing copy; provider "held liable for trade-mark infringement where outputs reproduced a watermark" (§7.2, §7.8).** UNSUPPORTED. Recollection (medium-high): the trade-mark finding was narrow, confined to specific early versions, and the s. 10(3) claim failed. The doc's wording is too broad.

18. **GEMA v OpenAI (11 Nov 2025) and Kneschke v LAION (§7.4).** UNSUPPORTED. Recollection (high for GEMA, medium for the appeal): consistent.

19. **Cox v. Sony and Thaler certiorari (§7.3, §7.7).** UNSUPPORTED.
    - Recollection (medium-high): Cox (25 March 2026) **reversed** liability. Contributory liability needs inducement or a service tailored to infringement; knowledge alone is not enough.
    - The doc cites only the half that hurts us. That is fair for a feature built to imitate, but it is US law, and the case narrowed provider liability.
    - Thaler: certiorari denied 2 March 2026 (medium).

20. **Cass. com., 26 June 2024: "three judgments", "tightened the test" (§2.10).** UNSUPPORTED. Recollection (medium): two judgments carrying three appeal numbers (n° 23-13.535; n° 22-17.647 and 22-21.497 joined). *Parasitisme* was upheld in the first and rejected in the second. "Restated the criteria" is safer than "tightened".

**Other points, from memory only**

- *Kadrey v. Meta* (§7.8) is mischaracterised (high). Meta won because the plaintiffs showed no market harm, although Meta had used shadow-library copies. The row's summary fits *Bartz* only.
- FLUX.1 dev is not "unusable commercially"; it needs a paid commercial licence (medium-high).
- Consistent with my memory: LG München I on Google Fonts, the Hadopi logo, the Vienna Agreement, the 15-year US design patent term, CDPA s. 54–55, *Agfa Monotype v. Adobe*, the death dates in §5.5.

## 2. Omissions

Found in the repository (A/B):

- The TJ Bordeaux counter-example and the "decision not read" caveat (claim 6).
- The verified licences of FontDiffuser, DG-Font and MX-Font, and the 1,908-family count (claim 7).
- The MI-GAN conflict between this document and `notes/font-id-tech.generation.md` (claim 7).
- Document 03's 8 pt floor for outlined text and the `outline_all_text` flag (claims 4–5).
- The code's treatment of fonts without an OS/2 table (claim 3).
- **Catalogue rule R18 contradicts §2.6 and §5.5.** A 25-year cut-off tracks design right only. French copyright (life + 70) in an original face by a living designer is not covered. By the doc's own dates, a close copy of Gotham (2000) is already admissible under R18.

From recollection, to verify:

- Digital Omnibus agreement (claim 11).
- The Commission's code of practice on marking AI content: drafts since Dec 2025 ask for layered marking of images (metadata plus imperceptible watermark). XMP alone (item 24) may not be enough (medium).
- AI Act art. 50(4), the deployer's duty to disclose "deep fakes" (content resembling existing objects that would falsely appear authentic), is never discussed. An outpainted cover is arguable, and in reseller mode the deployer is a professional.
- A French Senate bill adopted in April 2026 creating a presumption that an AI provider used a protected work (medium). It would raise the value of the training manifest (R12, item 28).
- Art. 19(2)(d) of the reformed design regulation: a font file, or weights that reproduce a typeface, is "software which records the design". Distributing an open clone of a registered design may itself be a listed act. R18 addresses "use" only.
- Registered typeface designs (Locarno 18-03) can be searched in EUIPO DesignView and at INPI. R18 uses age as a proxy and never proposes the search.
- US Copyright Office practice of questioning registration of fonts built with GUI editors, raised in *Laatz* (low-medium). If right, §2.3 overstates how settled font-software copyright is.
- IPTC terms for item 24: `trainedAlgorithmicMedia`, `compositeWithTrainedAlgorithmicMedia`, and `algorithmicMedia` for procedural output (high).
- The EU–US Data Privacy Framework (2023, upheld by the General Court in Sept 2025) weakens the 2022 Munich reasoning. Self-hosting stays right for other reasons (medium-high).

## 3. Statements to downgrade to "unverified"

- §1.4: "they are [unmodified] today [L]".
- §4.3: "the decoding itself is already implemented", as far as the no-OS/2 and 0x0200 rows go.
- §4.4: "satisfies all of them".
- Finding 13 and item 7: 5–7 mm, 7 mm, 6 pt.
- Finding 9, §1.4, §6.2: the "[V2]" tag on French cropping and mirroring case law. Use "secondary sources only".
- Finding 5 and R6: "the user's own licence covers it".
- Finding 12: "no court ruling on AI-generated fonts is known", and §2.6 "no French judgment". Neither absence was searched.
- §7.2: Getty trade-mark wording. §7.7: Cox "points the same way". §7.8: the Bartz/Kadrey row. §2.10: "three judgments".
- §7.5: every row where the sibling note has verified data.
- §2.4: the typeface representation rule (pre-reform text).
- §7.3: the AI Act date paragraph, until the omnibus is checked.
- The Findings, Product rules and Implications sections read as settled although §0 says nothing is web-verified. Repeat the banner there.

## 4. First searches when budget returns

1. Digital Omnibus final text and art. 50(2) dates.
2. Code of practice on marking and labelling.
3. Laatz v. Zazzle outcome.
4. Monotype and Adobe Fonts terms on servers and AI training, and server licence prices.
5. Cox v. Sony holding.
6. Cass. com. 26 June 2024 numbers.
7. French typeface originality case law.
8. French Senate AI presumption bill.
9. Getty appeal.
10. OFL FAQ on subsetting and AI.
