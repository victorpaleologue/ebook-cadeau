# ebc-text — final report

`ebc-text` is implemented to its contract and all five definition-of-done commands pass: 120 tests plus 5 doctests, clippy clean, wasm32 builds. I only touched `crates/ebc-text/` (plus the `Cargo.lock` entries written by `cargo add`); no git operations.

## Things you should know first

- **Hostile fonts can panic the font libraries in debug builds.** `ttf-parser` and `rustybuzz` are panic-free only in release. My fuzz test hit `rustybuzz-0.20.1/src/hb/set_digest.rs:93` ("attempt to subtract with overflow") 15 times in one debug run, and a font collection declaring 2^32−1 faces trips a `debug_assert!` in `ttf-parser`'s `parser.rs`. The release run produced zero library panics.
  - I contain these with `catch_unwind` at every entry point (`src/guard.rs`) and pre-validate the collection header. A contained panic degrades to `Err` or an empty result, and still prints to stderr.
  - Containment cannot work where panics abort: a **dev-profile wasm build will trap** on such a font. Release wasm is unaffected.
  - Recommended fix in the root `Cargo.toml` (not mine to edit): `[profile.dev.package.ttf-parser]` and `[profile.dev.package.rustybuzz]` with `debug-assertions = false` and `overflow-checks = false`.
- **A font with no `OS/2` table is reported as `Restricted`.** This follows `docs/research/08-legal-fonts-and-genai.md` §4.3 ("unknown → refuse"). `fs_type()` returns `None` so callers can tell unknown from forbidden.
- **Ink can rise above the ascender.** "É" in Courier Prime Bold exceeds it by 0.06 em. `ebc-spine` and `ebc-card` should position with `ShapedText::ink_bounds` or cap height, not assume ink ≤ ascender.
- **Bundled-font facts worth knowing:**
  - Atkinson Hyperlegible has no fi/ffi ligatures and lacks U+2011 and U+FB01.
  - Bebas Neue also lacks U+2011 and U+FB01, and its x-height equals its cap height.
  - Spectral has an "ffi" ligature and small caps.
- **Task #12 (migration to skrifa/harfrust):** no `ttf-parser` or `rustybuzz` type appears in the public API, so that migration stays inside this crate.

## Public API

```rust
// Font (cheap Clone, Send + Sync, Debug, PartialEq)
Font::from_bytes(data: impl Into<Arc<[u8]>>, face_index: u32) -> Result<Font, FontError>
font.data() -> &Arc<[u8]>;  font.face_index() -> u32
font.family_name() -> String        // name ID 16, else 1
font.style_name() -> String         // ID 17, else 2
font.full_name() -> String          // ID 4, else family + style
font.postscript_name() / licence_description() / licence_url() / version() -> Option<String>
font.name(name_id: u16) -> Option<String>
font.fs_type() -> Option<u16>
font.embedding() -> EmbeddingPermission
font.embedding_flags() -> EmbeddingFlags { no_subsetting, bitmap_only }
font.allows_bundling() -> bool            // fsType == 0 exactly
font.allows_subset_embedding() -> bool
font.allows_outlines() -> bool            // the user-supplied-font rule
font.metrics() -> FontMetrics
font.glyph_count() -> u16;  font.glyph_id(char) -> Option<u16>
font.outline_format() -> OutlineFormat { TrueType, Cff, Cff2, None }
font.weight() -> u16;  font.is_italic();  font.is_monospaced()
font.is_variable();  font.axes() -> Vec<VariationAxis>;  font.variations() -> Vec<(String, f32)>
font.with_variations(&[(&str, f32)]) -> Font
font.missing_chars(&str) -> Vec<char>

enum EmbeddingPermission { Installable, PreviewAndPrint, Editable, Restricted }
  ::from_fs_type(u16)        // least-restrictive rule, OS/2 v0–2
  ::from_os2(fs_type, ver)   // most-restrictive for malformed v3+
  .allows_embedding()        // plus FS_* bit constants

struct FontMetrics { units_per_em: u16, ascender, descender, line_gap, cap_height, x_height: i16 }
  .ascender_mm(pt) .descender_mm(pt) .line_gap_mm(pt) .cap_height_mm(pt) .x_height_mm(pt)
  .line_height_mm(pt) .units_to_mm(units, pt) .mm_per_unit(pt) .size_for_cap_height(mm)

struct ShapeOptions { letter_spacing_em: f32, language: Option<String>, rtl: bool, features: Vec<(String, u32)> }  // Default
struct ShapedGlyph { id: u16, x: f64, y: f64, advance: f64, text: String }
struct ShapedText { glyphs: Vec<ShapedGlyph>, width: f64, size_pt: f64 }
shape(&Font, &str, size_pt: f64, &ShapeOptions) -> ShapedText
ShapedText::to_glyph_item(FontId, Color, Point) -> Item
ShapedText::to_outline(&Font, Point) -> Path
ShapedText::ink_bounds(&Font, Point) -> Option<Rect>
outline_bounds(&Path) -> Option<Rect>     // exact, unlike Path::control_bounds
const MAX_SIZE_PT: f64 = 1e6

fit_size(&Font, &str, max_width, max_cap_height, min_pt, max_pt, &ShapeOptions) -> Option<f64>
wrap(&Font, &str, size_pt, max_width, &ShapeOptions) -> Vec<String>
wrap_ranges(...same...) -> Vec<Range<usize>>

// feature "raster" (default on)
struct Mask { width: u32, height: u32, data: Vec<u8> }   // + is_empty(), get(x, y), row(y)
struct RasterizedText { mask: Mask, left: i32, top: i32, advance: f32 }
rasterize(&Font, &str, px_per_em: f32, &ShapeOptions) -> Mask
try_rasterize(...same...) -> Result<RasterizedText, FontError>
const MAX_MASK_SIDE: u32 = 8192;  const MAX_MASK_PIXELS: u64 = 1 << 24

#[non_exhaustive] enum FontError { Parse(String), FaceIndex { index, count }, InvalidRasterSize(String),
                                   RasterTooLarge { width, height }, Malformed, NonFiniteOutline }
```

Behaviour worth knowing when calling it:

- **Metrics sign:** `descender` and `descender_mm` are negative (y-up, as in the font file); the `*_mm` helpers are plain unit conversions.
- **Metric fallbacks:**
  - cap height: `OS/2`, else the outline of H/I/X/E, else 0.7 em;
  - x-height: `OS/2`, else the outline of x/z/v, else 0.5 em;
  - ascender/descender: font bounding box, else 0.8/−0.2 em.
- **Glyph text:** the concatenated `text` of all glyphs equals the input exactly. For an RTL run, iterate the glyphs in reverse.
- **Letter spacing:** added between clusters only, never after the last. A non-zero value disables `liga`/`clig` unless the caller sets them.
- **Control characters and U+2028/2029** are set as spaces; a run is one line.
- **`fit_size`:** the returned size satisfies both limits exactly, with the same arithmetic `shape` uses. `f64::INFINITY` disables a limit.
- **`wrap`:**
  - trailing whitespace is trimmed, leading whitespace kept, blank lines kept;
  - a soft hyphen becomes a visible `-` only where a line breaks;
  - over-long words are cut between grapheme clusters;
  - every returned line is verified by measuring it whole;
  - cost is linear in the text for any width.
- **`rasterize`:** the mask is cropped to the ink; `left`/`top` place it relative to the baseline origin. It returns an empty mask on invalid or oversized requests; `try_rasterize` reports why.

## Deviations from the contract

1. **Embedding flags are a separate struct.** The contract wrote the flags as a comment on a fieldless enum, which cannot carry them. `embedding()` returns the four-variant enum exactly as specified; the flags come from `embedding_flags()`.
2. **No-`OS/2` fonts map to `Restricted`** (see above); the contract did not say.
3. **`wrap` is not a naive greedy loop.** It scans forward on per-segment widths, then verifies whole lines backwards. The result is greedy and exact, as two tests with non-additive fake measures check; the reason is that measuring from the line start at every opportunity is quadratic on long lines.
4. **`rasterize` swallows errors** because the contract signature returns `Mask`; `try_rasterize` is the fallible form.
5. **Additions beyond the contract:** the policy helpers `allows_bundling`, `allows_subset_embedding`, `allows_outlines`; `style_name`, `version`, `name`, `axes`, `variations`, `glyph_id`, `glyph_count`, `outline_format`, `weight`; `ink_bounds`, `outline_bounds`, `wrap_ranges`, `RasterizedText`.
6. **Dependencies:** `rustybuzz` 0.20.1, `ttf-parser` 0.25.1 (one copy in the tree), `unicode-linebreak` 0.1.5, `unicode-segmentation` 1.13 (for grapheme-safe cuts), `zeno` 0.3.3 (optional, rasteriser). All MIT/Apache.

## Assumptions

- Conflicting permission bits in an OS/2 v3+ font are read most-restrictively; research 08 confirms this.
- Family name prefers the typographic family: Spectral SemiBold reports family "Spectral", style "SemiBold".
- Name records are chosen in this order: Windows en-US, other Windows English, Unicode platform, other Windows, Mac Roman.
- An empty text wraps to `[]`; a lone `"\n"` wraps to `[""]`.
- Doc examples and the spine test use the research 08 values: cap height ≤ 7 mm, minimum 6 pt.

## Known limitations

- **One run is one font, one script, one direction.** There is no font fallback (missing characters become `.notdef`), no script itemisation, no bidi reordering, and no vertical layout. `rtl` only flips the whole run.
- **No hyphenation dictionary.** Thai, Lao and Khmer get no word breaks, so they fall back to cuts between clusters.
- **Soft hyphens:** a forced cut that lands just before one keeps it, invisible, on the cut line and adds no visible hyphen.
- **`to_glyph_item` on a variable-font instance is wrong by design:** a PDF viewer would draw the default instance. Use `to_outline` for instances.
- **Glyphs without outlines draw nothing** (bitmap, SVG, colour emoji). WOFF/WOFF2 are not decoded.
- **Named instances are not exposed** — axes only.
- **CFF coverage is thin.** Cubic outlines are tested with a synthetic CFF font (one circle, one rectangle) built in the test helpers. No real `.otf` is bundled, and CFF2 is untested.
- **`MAX_MASK_SIDE` is 8192 px** to keep zeno's 32-bit fixed-point arithmetic in range, so very large previews must be tiled by the caller.
- **zeno has one `unsafe`** (`to_int_unchecked`). I guarantee its precondition by checking that every coordinate is finite and within the mask before the call.
- **`wrap` could be slow with a hostile font and a pathological text together.** The half-em estimation margin is an engineering default; if cross-word kerning ever exceeded it, the exact check still guarantees every line fits and only greediness would suffer.
- **`from_bytes(&'static [u8])` copies the bytes once** into an `Arc`.
- **Each call re-parses the face** (table directory and layout accelerators). This is cheap, but nothing is cached across calls.

## Tests

120 tests plus 5 doctests, all offline and deterministic:

| Suite | Count | What it covers |
|---|---|---|
| unit (`src/`) | 35 | fsType exhaustively over all 65 536 values, Mac Roman and UTF-16 decoding, cluster-to-text mapping, outline sink, exact Bézier bounds, line breaker with fake measures (including adversarial ones and three linear-time guards) |
| `tests/font.rs` | 26 | names, licence, fsType table of research 08 row by row, metrics and fallbacks, coverage, garbage input, collections, variations |
| `tests/shaping.rs` | 25 | French text in all 8 bundled fonts, ligatures, multi-glyph clusters, kerning, letter spacing, features, RTL, glyph items, outlines |
| `tests/fit_wrap.rs` | 17 | fit bounds and maximality, closed form, spine scenario, wrap invariants, French punctuation, soft hyphens, degenerate widths |
| `tests/raster.rs` | 12 | pixel-exact rectangles, coverage equals area, shaping positions, non-zero fill on overlaps, cubic disc, size limits |
| `tests/hostile.rs` | 5 | truncations, about 1 150 seeded corruptions per run, pathological text and options |

Variable-font tests use a synthetic TrueType font with `fvar`/`gvar` built in `tests/common/mod.rs`, since no bundled font is variable. With `--no-default-features` the suite is 106 tests plus 5 doctests.

## Definition of done

```
$ cargo fmt -p ebc-text                     (exit 0; `-- --check` exit 0)
$ cargo build -p ebc-text
   Compiling ebc-text v0.1.0 (/home/user/ebook-cadeau/crates/ebc-text)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.32s
$ cargo test -p ebc-text
     Running unittests src/lib.rs
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
     Running tests/fit_wrap.rs
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
     Running tests/font.rs
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/hostile.rs
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.50s
     Running tests/raster.rs
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running tests/shaping.rs
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
   Doc-tests ebc_text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
$ cargo clippy -p ebc-text --all-targets -- -D warnings
    Checking ebc-text v0.1.0 (/home/user/ebook-cadeau/crates/ebc-text)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.70s
$ cargo build -p ebc-text --target wasm32-unknown-unknown
   Compiling ebc-text v0.1.0 (/home/user/ebook-cadeau/crates/ebc-text)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.68s
```

Also run and passing: clippy and the wasm32 build with `--no-default-features`; `cargo test -p ebc-text --release` (same 120 + 5); `RUSTDOCFLAGS="-D warnings" cargo doc -p ebc-text --no-deps`.

## Files

Everything is under `/home/user/ebook-cadeau/crates/ebc-text/`:

- `Cargo.toml`
- `src/lib.rs`
- `src/error.rs`
- `src/font.rs`
- `src/shape.rs`
- `src/fit.rs`
- `src/wrap.rs`
- `src/raster.rs`
- `src/guard.rs`
- `tests/common/mod.rs`
- `tests/font.rs`
- `tests/shaping.rs`
- `tests/fit_wrap.rs`
- `tests/raster.rs`
- `tests/hostile.rs`
