## ebc-layout — final report

`ebc-layout` is implemented and all five definition-of-done commands pass, with one contract point not met: for home profiles the MediaBox is not always the full sheet. `ebc_scene::Page` cannot express it, and the fix needs a change in `ebc-scene` and `ebc-pdf` (details under Deviations).

`docs/research/03-print-requirements.md` did not exist at any point during the run (last checked at the end). All numbers are the brief's defaults, cross-read against `docs/research/notes/print-requirements.{prepress,folding}.md`, and they all live in `crates/ebc-layout/src/profile.rs` for when the document lands.

### Files (all under `/home/user/ebook-cadeau/crates/ebc-layout/`)
- `Cargo.toml` — deps `ebc-scene`, `thiserror` (both workspace-inherited, added with `cargo add -p ebc-layout`)
- `src/lib.rs`, `src/error.rs`, `src/format.rs`, `src/profile.rs`, `src/page.rs`, `src/marks.rs`, `src/guides.rs`, `src/image.rs`
- `tests/invariants.rs` — integration and property tests
- `examples/layout_svg.rs` — dev aid: `cargo run -p ebc-layout --example layout_svg -- poche home_a4 out.svg [--all-marks]`

### Definition of done (run in the real workspace)
```
=== cargo fmt -p ebc-layout            (no output; `-- --check` exit 0)
=== cargo build -p ebc-layout
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
=== cargo test -p ebc-layout
     Running unittests src/lib.rs
running 59 tests
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/invariants.rs
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
   Doc-tests ebc_layout
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
=== cargo clippy -p ebc-layout --all-targets -- -D warnings
    Checking ebc-layout v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.76s      (exit 0)
=== cargo build -p ebc-layout --target wasm32-unknown-unknown
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.54s
```
`RUSTDOCFLAGS="-D warnings" cargo doc -p ebc-layout --no-deps` is also clean.

**Number of tests: 70** — 59 unit, 10 integration/property, 1 doctest. All are offline and deterministic (SplitMix64 with fixed seeds, no dev-dependencies).

### Public API
**Formats (`format.rs`)**
- `CardFormat { cover: Size, spine_width: f64 }` (Copy; `Default` = poche).
- Presets `poche()` 110×178, `a6()` 105×148, `digest()` 130×200, `a5()` 148×210, all with a 10 mm spine; also `presets()`, `by_name()`, `PRESET_NAMES`.
- `custom(cover, spine) -> Result` validates cover 50–300 mm and spine 3–40 mm, rejecting NaN and infinities; also `with_spine()`, `validate()`.
- Constants `SPINE_DEFAULT_MM`, `COVER_MIN_MM`, `COVER_MAX_MM`, `SPINE_MIN_MM`, `SPINE_MAX_MM`, `SPINE_TEXT_MIN_MM` (5), plus `spine_can_carry_text()`.
- Geometry: `spread()`, `trim_rect()`, `outside()`, `inside()`, `panels(Face)`, `fold_x()`, `fold_lines()`, `mirror_x(rect)`.
- Named panels: `back()`, `spine()`, `front()`, `inside_front()`, `inside_spine()`, `inside_back()`.
- `Panels { left, spine, right }` with `to_array()`, `bounds()`, `center_x()` and `with_bleed(bleed)` (panels grown into the bleed on outer edges only — the rect a full-bleed cover image must fill).
- `Face { Outside, Inside }` with `BOTH` and `label()`.

**Profiles (`profile.rs`)**
- `PrintProfile` has every contract field, plus `professional_eu()`, `home(sheet)`, `upload_eu()`, `by_name()`, `NAMES`, `without_marks()`, `validate()`, `judge_ppi() -> Resolution { Good, Low, TooLow }`, `target_px(size)`, `min_px(size)`.
- `Marks` has the contract fields plus `style: MarkStyle { Press, Guide }` and `scale_ruler: bool`; constructors `press()`, `guides()`, `none()`; helpers `any()`, `color()`, `stroke()`.
- `Sheet { A4, Letter, A3, Sra3, Custom(Size) }` with `size()`, `landscape()`, `portrait()`, `name()`, `by_name()`, `NAMED`.
- Constants `MAX_MARGIN_MM`, `MAX_MARK_WEIGHT_PT`, `MAX_SHEET_MM`, `GUIDE_GRAY_LEVEL`.

**Pages (`page.rs`)**
- `new_page(format, profile) -> Result<Page, LayoutError>` and `new_page_for(face, …)` (sets `Page::label`).
- `page_geometry(format, profile) -> Result<PageGeometry, _>` returns trim, bleed, media, mark area, bleed/slug/mark reach in mm, and `sheet: Option<SheetPlacement { sheet, printable, landscape, media_is_sheet }>`.

**Marks (`marks.rs`)**
- `marks(format, profile) -> Item` is the layer `"marks"` (`MARKS_LAYER`), built from the enabled generators.
- Generators, each callable on its own: `crop_marks`, `fold_marks`, `registration_targets` (with `registration_target_centers`), `color_bar` (`COLOR_BAR_PATCHES`: 4 solids, 3 overprints, 4 tints at 50 %), `scale_ruler -> Option<ScaleRuler>`.
- `caption_areas -> Option<[Rect; 2]>` gives the two margin rectangles no mark ever uses, for small print.
- Constants `MIN_MARK_LENGTH_MM`, `FOLD_GUIDE_DASH_MM`, `FOLD_GUIDE_MAX_MM`, `FURNITURE_BAND_MAX_MM`, `CAPTION_MIN_HEIGHT_MM`.

**Safe areas, guides, images**
- `safe_rect(panel, profile)`, `spine_safe_rect(format, profile)`, `safe_panels(format, profile)`.
- `guides(format, profile) -> Item` is the layer `"guides"` (`GUIDES_LAYER`): bleed, trim, creases, safe areas in CMYK.
- `info_entries(format, profile)` produces `EBC.*` pairs for `DocInfo::custom`.
- `Fit { Cover, Contain }`, `place_image`, `place_image_aligned(…, align_x, align_y)`, `effective_ppi`.

**Errors**
- `LayoutError` (`#[non_exhaustive]`, thiserror): `CoverOutOfRange`, `SpineOutOfRange`, `InvalidProfile { field, value }`, `InvalidSheet`, `DoesNotFit { needed, sheet }`.

### Behaviour worth knowing
- **Professional page:** poche gives TrimBox 230×178, BleedBox +3 mm, MediaBox +13 mm. Crop marks are 5 mm long, start at the bleed edge, 0.25 pt, registration colour. Fold marks are solid ticks at both `fold_x` above and below. Registration targets are centred on all four sides.
- **Marks never enter the bleed:** the effective offset is `max(marks.offset, bleed)`. Marks shrink to the available margin and are omitted below 1 mm.
- **Invalid input never panics:** `marks()` and `guides()` return an empty layer whenever `new_page` would fail.
- **Home page:** the spread is centred on the sheet, landscape preferred, portrait only if landscape does not fit. `DoesNotFit` is returned when spread + bleed + printable margin exceeds the sheet in both orientations; A5 and digest are rejected on A4 and Letter, poche and A6 fit.
- **Home marks:** light grey `Color::Gray(0.55)`, 0.5 pt. Fold ticks are continued by a dashed guide to the printable edge (capped at 20 mm). A 20/50/100 mm scale ruler is drawn. Everything stays inside the printable area.
- **Inside vs outside:** `inside()` returns `left` = inside-front, `right` = inside-back. Numerically it equals `outside()` because the spread is symmetric; `mirror_x` maps a rect to what lies behind it.

### Deviations from the contract
1. **Home MediaBox is not always the sheet.** `ebc_scene::Page` has one scalar `slug` for all four sides (and `ebc-pdf` uses `page.bleed + page.slug` symmetrically). A poche spread centred on A4 landscape has 33.5 mm left/right but 16 mm top/bottom, which the type cannot express.
   - What `new_page` returns instead: trim = spread (TrimBox and BleedBox are right) and the largest media box centred on the spread that fits the sheet. That is 262×210 for poche on A4 and 267.9×215.9 on Letter. It equals the sheet exactly only when both margins are equal (tested for A4 and Letter).
   - Practical effect: I expect a viewer printing at 100 % to centre the page, giving the same sheet; this was not tested on a real viewer or printer.
   - Proposed fix: give `ebc_scene::Page` a per-axis slug or an explicit media rect and have `ebc-pdf` honour it. `page_geometry().sheet.sheet` already holds the exact sheet rect, so `new_page` would change by about three lines.
2. **Extra fields on `Marks`:** `style` and `scale_ruler`. This is allowed by "add what the implementation needs", but a struct literal written from the contract text will not compile; use `Marks::press()`, `Marks::guides()` or `..` update syntax.
3. **Narrow spines change the top and bottom registration targets.** They sit between the two fold marks, so they shrink with the spine and are dropped below about 4.1 mm; left and right targets are always centred. The default 10 mm spine has four full-size targets.
4. **Additions not in the contract:** `upload_eu()`, portrait fallback, `page_geometry`, `caption_areas`, `scale_ruler`, `info_entries`, `place_image_aligned`, `effective_ppi`, `guides`, the SVG example. `upload_eu()` exists because the research notes found most web-to-print portals reject marks.

### Assumptions
- **Home profile values the brief did not fix:** safe margin 5 mm, spine safe margin 2 mm, mark weight 0.5 pt, `PdfStandard::Plain`, target 200 ppi, minimum 100 ppi, `slug` field 0 (ignored when a sheet is set), scale ruler on.
- **Professional profile:** colour bar off by default. Pro fold marks are solid; only home gets the dashed guide.
- **Mark geometry is trade convention.** The research notes say no surveyed online printer publishes crop-mark geometry.
- **`EBC.*` keys:** `EBC.CoverWidthMm`, `EBC.CoverHeightMm`, `EBC.SpineWidthMm`, `EBC.FoldXMm` ("110,120", measured from the left trim edge), `EBC.BleedMm`, `EBC.Profile`. `ebc-preflight` already references `EBC.SpineWidthMm` and `EBC.FoldXMm`, but its parser was not written when I looked, so the list format and origin of `FoldXMm` need confirming with that crate.
- **Validation bounds are mine:** margins 0–100 mm, mark weight above 0 and at most 2 pt, sheet edge at most 5080 mm, total ink above 0 and at most 400 %.

### Known limitations
- No multi-up imposition, although the crate description says "imposition". Only one card centred on one sheet is supported.
- The crate sets no text. The ruler label and any "cut here / fold here" captions are for the composer to place, using `ScaleRuler::label_origin` and `caption_areas`.
- Marks are identical on both faces. The colour bar and ruler are not mirror-symmetric (harmless); crop marks, fold marks and targets are, and that is tested.
- The `guides` layer draws across the artwork and is for proofs only. Whether it is non-printing is up to the PDF writer or composer.
- `Panels::with_bleed`: the grown left panel's right edge matches the crease only to floating-point rounding. The un-bled panels tile bit-exactly.
- No serde support, because `ebc_scene::PdfStandard` has no serde derive. `by_name` lookups are provided instead.
- Nothing was validated against a real printer or a rendered PDF. I only looked at the SVG output of the example for poche with the professional and home-A4 profiles.

### Process notes
- The workspace `Cargo.toml` was broken by another agent for about ten minutes (most `[workspace.dependencies]` entries removed). I did not touch it; I iterated in a scratch mirror workspace and re-ran everything in the real workspace once it was repaired.
- I ran one read-only `git log` by mistake inside a grep command. Nothing was modified.
- Twenty-three hand-written mutants (e.g. fold mark off by 1e-7, marks entering the bleed, portrait fallback disabled) were each killed by the integration tests alone; that harness lived in the scratchpad and is not in the repo.
