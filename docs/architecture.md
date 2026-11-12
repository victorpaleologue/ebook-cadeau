# eBook Cadeau — architecture

## 1. Product in one paragraph

A gifted eBook has nothing to wrap. eBook Cadeau produces a **print-ready PDF** of a
book-shaped card: the outside is a wraparound jacket — **back | 10 mm spine | front** — where
the front is the book's real cover and the spine carries the title so the object can stand on
a shelf next to real books; the inside carries a **QR code**, a **short code** and three
steps that get the eBook onto the recipient's e-reader. The same card can be printed at home
on A4 or ordered from a professional printer.

Two modes:

- **Standalone** — an individual, author or publisher creates a card for a file they are
  entitled to distribute (own work, public domain, open licence). Since v0.3 of the
  contract this goes through a **shelf** service (below), never through the core.
- **Reseller-integrated** — an eBook store offers an "eBook cadeau" option at checkout. The
  store keeps the content, the DRM and the customer account; eBook Cadeau issues the code,
  renders the card and routes the redemption back to the store's fulfilment.

### 1.1 The two-service model (owner decision, contract v0.3)

```
            signed partner API (docs/api.md §4)                 recipient's device
 store ─┐   create gift, cover, card.pdf, print, cancel  ┌────────┐  /k/CODE, /e, /d/…   ┌──────────┐
        ├──────────────────────────────────────────────►│  core  │◄─────────────────────│ e-reader │
 shelf ─┘◄──────────────────────────────────────────────│ (codes,│  303 to the partner's │  phone   │
   ▲        §4.5: signed "give me a link" on each claim  │ cards, │  short-lived link     └────┬─────┘
   │                                                     │ pages) │                          │
   └─────────────────── the file itself, from the partner's own host ────────────────────────┘
```

- The **core** (`crates/ebc-server`) issues codes, renders cards, serves the redemption
  pages, pairs devices, sends to Kindle, forwards print orders and keeps **gift records
  only**. It never stores a book file: there is no upload route and no creator API.
- A **file is only ever served by a registered partner**: a store (Septante in the demo
  serves its own stamped EPUBs), or a **shelf** — a separate service (next step) that
  hosts files for their rights holders and carries the hosting duties: uploader identity
  and rights declaration, DRM refusal (`crates/ebc-epub`), notice and action (DSA art. 16),
  statements of reasons, repeat-infringer policy. The shelf registers with the core exactly
  like a store (`EBC_PARTNERS`, same signed API) and creates `partner_file` gifts for its
  users.
- `partner_file` delivery: on each claim the core sends the partner a signed request (the
  parameters of the redemption redirect), gets back a short-lived download link on one of
  the partner's `redirect_hosts`, and redirects the device to it; Send to Kindle fetches
  the link into memory only. OPDS acquisition links are the core's session URLs.
- Why: a service storing users' book files is a hosting provider whose liability exemption
  is fragile when it knows which work each file is (the card names it), and which must run
  notice and action (`docs/research/09-legal-gifting-and-commerce.md` §2.6, §3.3). Keeping
  files out of the core keeps the core a code-and-card service with a contact point
  (`EBC_CONTACT_EMAIL`) and a short retention of personal data (names and message deleted
  12 months after the first claim).

## 2. Principles

1. **Browser first.** Anything CPU-heavy (image analysis, font matching, spine synthesis, PDF
   generation, QR) is a pure-Rust library compiled to WebAssembly and run on the user's
   device. The server stores gifts and serves tiny pages.
2. **Sans-IO libraries.** Libraries never open sockets or files. Catalogue and cover
   providers *describe* HTTP requests and *parse* responses (`ebc_model::http`); the host
   (server, CLI, browser) performs them. This is what makes every test offline.
3. **Deterministic output.** Same input → same bytes. Time and randomness are injected.
4. **Print correctness is a tested property.** Every generated PDF is preflighted by an
   independent crate that parses it back (`ebc-preflight`), and by external tools in CI.
5. **e-ink pages need no JavaScript.** Redemption pages are plain HTML forms that work in
   decade-old WebKit browsers on e-readers.
6. **Legally conservative defaults.** Only openly licensed fonts are bundled; generative
   features are opt-in; covers come from the user or the reseller unless a source's terms
   allow more. See `docs/research/07…09`.

## 3. Crate map

```
ebc-scene      geometry, colour, paths, Document/Page/Item  (the contract; no deps)
ebc-isbn       ISBN parsing/validation/hyphenation          (no deps)
ebc-model      BookMeta, Locale, HttpRequest/Response, GiftCode   -> ebc-isbn
ebc-i18n       strings per locale, spine direction               -> ebc-model
ebc-color      palettes, contrast, naive CMYK                    -> ebc-scene
ebc-qr         QR for print (vector)                              -> ebc-scene
ebc-text       fonts: load, shape, measure, fit, outline, raster  -> ebc-scene
ebc-layout     card format, panels, bleed/safe, marks, imposition -> ebc-scene
ebc-pdf        Document -> PDF bytes (PDF/X)                      -> ebc-scene
ebc-preflight  PDF bytes -> findings                              (independent)
ebc-catalog    ISBN database providers (sans-IO), merge           -> ebc-model
ebc-covers     cover cascade, scraping helpers, sniffing          -> ebc-model
ebc-epub       eBook file checks for a hosting service: EPUB/PDF sniffing, DRM refusal,
               rights declaration, safe file names (used by the shelf, not the core)
ebc-fontid     identify the cover's title font (image-based)      -> ebc-scene, ebc-text
ebc-spine      adapt or synthesise the 10 mm spine                -> scene, text, color, layout, i18n, model
ebc-card       compose the whole card                             -> all of the above print crates
ebc-wasm       wasm-bindgen facade                                -> card, catalog, covers, fontid, qr, isbn
ebc-cli        `ebc` command line                                 -> everything
ebc-server     the core: axum service, no file hosting            -> model, i18n, isbn (+ card for server-side render)
demo/stores    two fictional eBook stores                         -> server's partner API (HTTP)
demo/printer   fictional print-on-demand API                      -> ebc-preflight
demo/ereaders  simulated e-readers                                (HTTP clients)
```

All library crates except `ebc-server`, `ebc-cli` and the demos must build for
`wasm32-unknown-unknown`.

## 4. Shared conventions

- Lengths: millimetres (`f64`). Font sizes and stroke widths: points.
- Page coordinates: origin at the **top-left of the trim box**, x right, y down.
- Outside spread, left to right: **back cover | spine | front cover** (left-to-right books).
  Inside spread (the reverse side, sheet flipped about its vertical axis), left to right:
  **inside-front | inside-spine | inside-back**.
- Colours: `Color::Cmyk`/`Gray` for text, QR and marks (`Color::BLACK` = K only);
  `Color::Registration` only for printer's marks.
- Errors: one `thiserror` enum per crate, `#[non_exhaustive]`.
- Public API items are documented; examples in docs where useful.

## 5. Contracts per crate

Signatures below are the **minimum**; add what the implementation needs. Names given here
are relied on by other crates, so keep them.

### ebc-isbn

Keep the existing `Isbn13` API. Add:

```rust
pub struct Isbn10;                      // Copy, Eq, Hash, Display (10 chars, 'X' upper-case), FromStr
impl Isbn10 { pub fn parse(&str) -> Result<Self, IsbnError>; pub fn to_isbn13(&self) -> Isbn13; }
impl Isbn13 {
    pub fn to_isbn10(&self) -> Result<Isbn10, IsbnError>;      // Err(NoIsbn10) for 979
    pub fn hyphenated(&self) -> String;                         // "978-2-07-036002-4"; falls back to plain digits if the range is unknown
    pub fn registration_group(&self) -> Option<RegistrationGroup>; // e.g. prefix "978-2", name "French language"
    pub fn gs1_prefix(&self) -> u16;                            // 978 or 979
}
pub fn extract_all(text: &str) -> Vec<Isbn13>;                  // every valid ISBN-10/13 found in free text, deduplicated, in order
pub fn ean13_check_digit(first12: &[u8; 12]) -> u8;
```

Hyphenation needs the ISBN range table (registration groups + registrant ranges). Embed a
compact Rust table generated from the International ISBN Agency's public `RangeMessage`
data; note its date. Property-test: parse(hyphenated(x)) == x; ISBN-10 ↔ 13 round trip.

### ebc-model

Keep `book`, `http`, `locale`. Add `code`:

```rust
/// The human-typed redemption code printed on the card.
pub struct GiftCode;   // Display = grouped "XXXX-XXXX-XXXX"; FromStr is lenient
impl GiftCode {
    pub const ALPHABET: &'static str = "BCDFGHJKLMNPQRSTVWXZ"; // 20 consonants: letters only, no vowels
    pub const DATA_SYMBOLS: usize = 11;      // 20^11 ≈ 47.5 bits of entropy
    pub const LEN: usize = 12;               // 11 data + 1 check
    pub fn from_random_bytes(bytes: &[u8]) -> Result<Self, CodeError>; // caller supplies >= 8 CSPRNG bytes; unbiased (rejection sampling or wide reduction)
    pub fn parse(input: &str) -> Result<Self, CodeError>; // case-insensitive; ignores '-', ' ', '.', '_'
    pub fn compact(&self) -> String;         // 12 symbols, no separator
    pub fn grouped(&self) -> String;         // "XXXX-XXXX-XXXX"
}
/// The short code that pairs a phone with an e-reader browser: 6 symbols of the same alphabet, "XXX-XXX", no check symbol.
pub struct PairCode;   // from_random_bytes, parse (lenient), compact, grouped
/// The URL printed on the card, e.g. base "https://ebk.example" -> "https://ebk.example/k/BCDFGHJKLMNP".
pub fn claim_url(base_url: &str, code: &GiftCode) -> String;
/// The same URL fully upper-cased for the QR code (alphanumeric mode): "HTTPS://EBK.EXAMPLE/K/BCDFGHJKLMNP".
pub fn claim_url_qr(base_url: &str, code: &GiftCode) -> String;
```

Why this alphabet (see `docs/research/05-ereader-distribution-ux.md`): digits cost a layer
switch on phone and e-ink on-screen keyboards and Shift on AZERTY keyboards, and bring the
0/O, 1/I/L, 5/S, 8/B confusions; consonants only cannot spell a word on a gift.

The check symbol must detect **every single-symbol substitution and every adjacent
transposition**: use a Damm-style scheme over a totally anti-symmetric quasigroup of order
20, e.g. the direct product of GF(4) and ℤ/5 with x∘y = a·x + y in each component (a = 2 in
ℤ/5; a = a generator of GF(4)* with modulus x²+x+1), the check symbol being the one that
brings the running state to 0. Prove both properties with EXHAUSTIVE tests over the
quasigroup and randomised whole-code tests. Parsing must distinguish `CodeError::Check`
(typo — never counted as a brute-force attempt) from well-formed codes.

Also add `gift`: `GiftId`, `Gift { id, code, book: BookMeta, giver_name, recipient_name,
message, locale, created_unix, expires_unix, delivery: Delivery }`,
`Delivery::{PartnerFile { format: FileFormat (epub | pdf), url }, ResellerRedirect { url }, ResellerLicence { format: "lcpl" | "acsm" | … , url }}`,
`PartnerFileLink { download_url, file_name?, media_type?, size_bytes?, sha256?, expires_at? }`
(the partner's answer of `docs/api.md` §4.5, with `validate(format, now)`), `GiftStatus`.
(`HostedFile` was removed in contract v0.3: the core holds no file.)

### ebc-i18n

```rust
pub use ebc_model::Locale;
pub enum Msg { ProductName, CheckoutOption, CheckoutOptionHelp, CardHeadline, CardFrom, CardTo,
               CardStep1, CardStep2, CardStep3, CardNoPhone, RedeemCta, EbookWord, EreaderWord,
               CodeLabel, FoldHere, CutHere, PrintedWith, /* web + server strings as needed */ }
pub fn t(locale: Locale, msg: Msg) -> &'static str;             // never panics; falls back to English then French
pub fn tf(locale: Locale, msg: Msg, args: &[(&str, &str)]) -> String; // replaces "{name}" placeholders
pub enum SpineDirection { TopToBottom, BottomToTop }
pub fn spine_direction(locale: Locale) -> SpineDirection;       // from docs/research/11-spine-design.md
pub fn spine_direction_for_language(bcp47: &str) -> SpineDirection; // by the BOOK's language, default by convention
```

Strings come from `docs/research/06-terminology.md` when available; until then use sensible
drafts. Test that every `Msg` has a non-empty string in every `Locale::ALL`.

### ebc-color

```rust
pub struct Rgb8 { pub r: u8, pub g: u8, pub b: u8 }
pub struct Swatch { pub color: Rgb8, pub share: f32 }            // share of pixels, 0..=1
pub fn palette(img: &Pixmap, k: usize) -> Vec<Swatch>;           // k-means++ in CIELAB on a downsample, deterministic seed, sorted by share
pub enum Edge { Left, Right, Top, Bottom }
pub fn edge_palette(img: &Pixmap, edge: Edge, strip_fraction: f32, k: usize) -> Vec<Swatch>;
pub fn edge_uniformity(img: &Pixmap, edge: Edge, strip_fraction: f32) -> f32; // 1.0 = flat colour, 0.0 = busy
pub fn has_uniform_border(img: &Pixmap) -> Option<Rgb8>;         // covers with a white/solid frame
pub fn contrast_ratio(a: Rgb8, b: Rgb8) -> f64;                  // WCAG 2.x, 1..=21
pub fn pick_text_color(background: Rgb8, candidates: &[Rgb8], min_ratio: f64) -> Rgb8; // falls back to black/white
pub fn delta_e2000(a: Rgb8, b: Rgb8) -> f64;
pub fn to_cmyk_naive(c: Rgb8) -> ebc_scene::Color;               // with grey-component replacement; documented as non-colour-managed
pub fn to_scene_rgb(c: Rgb8) -> ebc_scene::Color;
```

### ebc-qr

Wrap a maintained encoder (`qrcodegen` or `qrcode`). Round-trip test every encoder path by
decoding a rasterised matrix with `rqrr` (dev-dependency).

```rust
pub enum EcLevel { L, M, Q, H }
pub struct QrCode;  // immutable module matrix
impl QrCode {
    pub fn encode(text: &str, min_ec: EcLevel) -> Result<Self, QrError>;   // picks the smallest version; boosts EC if free
    pub fn encode_url(url: &str, min_ec: EcLevel) -> Result<Self, QrError>; // upper-cases scheme+host (and the path when it is [A-Z0-9/-.:$%*+ ] only) to use alphanumeric mode
    pub fn size(&self) -> u32;             // modules per side, without quiet zone
    pub fn version(&self) -> u8;
    pub fn module(&self, x: u32, y: u32) -> bool;
    pub fn rects(&self) -> Vec<(u32, u32, u32, u32)>; // merged dark runs (x, y, w, h) in modules
    pub fn to_path(&self, top_left: Point, module_mm: f64) -> ebc_scene::Path; // dark modules only; caller draws the white quiet zone
    pub fn side_mm(&self, module_mm: f64, quiet_modules: u32) -> f64;
    pub fn to_svg(&self, module_px: u32, quiet_modules: u32) -> String;
}
pub const QUIET_ZONE_MODULES: u32 = 4;
pub const MIN_PRINT_MODULE_MM: f64 = 0.4;  // adjust to docs/research/03
```

### ebc-text

Use `ttf-parser` + `rustybuzz`. Variable fonts: support selecting an instance by axis values
for measuring/outlining.

```rust
pub struct Font;   // cheap to clone (Arc)
impl Font {
    pub fn from_bytes(data: impl Into<Arc<[u8]>>, face_index: u32) -> Result<Self, FontError>;
    pub fn data(&self) -> &Arc<[u8]>;  pub fn face_index(&self) -> u32;
    pub fn family_name(&self) -> String; pub fn full_name(&self) -> String; pub fn postscript_name(&self) -> Option<String>;
    pub fn licence_description(&self) -> Option<String>; pub fn licence_url(&self) -> Option<String>; // name IDs 13/14
    pub fn embedding(&self) -> EmbeddingPermission;   // from OS/2 fsType
    pub fn metrics(&self) -> FontMetrics;             // units_per_em, ascender, descender, line_gap, cap_height, x_height (font units) + *_mm(size_pt) helpers
    pub fn is_variable(&self) -> bool; pub fn with_variations(&self, axes: &[(&str, f32)]) -> Self;
    pub fn missing_chars(&self, text: &str) -> Vec<char>;
}
pub enum EmbeddingPermission { Installable, PreviewAndPrint, Editable, Restricted }  // + flags no_subsetting, bitmap_only
pub struct ShapeOptions { pub letter_spacing_em: f32, pub language: Option<String>, pub rtl: bool, pub features: Vec<(String, u32)> }
pub struct ShapedGlyph { pub id: u16, pub x: f64, pub y: f64, pub advance: f64, pub text: String } // mm, relative to run origin
pub struct ShapedText { pub glyphs: Vec<ShapedGlyph>, pub width: f64, pub size_pt: f64 }           // mm
pub fn shape(font: &Font, text: &str, size_pt: f64, opts: &ShapeOptions) -> ShapedText;
impl ShapedText {
    pub fn to_glyph_item(&self, font: FontId, color: Color, origin: Point) -> Item;   // Item::Glyphs
    pub fn to_outline(&self, font: &Font, origin: Point) -> Path;                      // filled outlines (y-down mm)
}
pub fn fit_size(font: &Font, text: &str, max_width: f64, max_cap_height: f64, min_pt: f64, max_pt: f64, opts: &ShapeOptions) -> Option<f64>;
pub fn wrap(font: &Font, text: &str, size_pt: f64, max_width: f64, opts: &ShapeOptions) -> Vec<String>; // greedy, Unicode line-break aware
pub struct Mask { pub width: u32, pub height: u32, pub data: Vec<u8> }   // 8-bit coverage
pub fn rasterize(font: &Font, text: &str, px_per_em: f32, opts: &ShapeOptions) -> Mask;  // feature "raster" (default on)
```

### ebc-layout

```rust
pub struct CardFormat { pub cover: Size, pub spine_width: f64 }   // cover = one panel
impl CardFormat {
    pub const SPINE_DEFAULT_MM: f64 = 10.0;
    pub fn poche() -> Self;  /* 110 x 178 */  pub fn a6() -> Self;  pub fn digest() -> Self; /* 130 x 200 */  pub fn a5() -> Self;
    pub fn custom(cover: Size, spine_width: f64) -> Result<Self, LayoutError>;  // validates ranges
    pub fn spread(&self) -> Size;                 // 2*cover.w + spine_width, cover.h
    pub fn outside(&self) -> Panels;              // back, spine, front
    pub fn inside(&self) -> Panels;               // back = inside-front (left), spine, front = inside-back (right) — document clearly
    pub fn fold_x(&self) -> [f64; 2];             // x of the two creases
}
pub struct Panels { pub left: Rect, pub spine: Rect, pub right: Rect }
pub struct Marks { pub crop: bool, pub fold: bool, pub registration: bool, pub color_bar: bool,
                   pub offset: f64, pub length: f64, pub weight_pt: f64 }
pub enum Sheet { A4, Letter, A3, Sra3, Custom(Size) }
pub struct PrintProfile { pub name: String, pub bleed: f64, pub safe_margin: f64, pub spine_safe_margin: f64,
                          pub slug: f64, pub marks: Marks, pub standard: PdfStandard,
                          pub target_ppi: f64, pub min_ppi: f64, pub max_total_ink_percent: f32,
                          pub sheet: Option<Sheet>, pub printable_margin: f64 }
impl PrintProfile {
    pub fn professional_eu() -> Self;   // values from docs/research/03 (bleed 3 mm, …)
    pub fn home(sheet: Sheet) -> Self;  // spread centred on a sheet, light cut/fold guides, no registration colour
}
pub fn new_page(format: &CardFormat, profile: &PrintProfile) -> Result<Page, LayoutError>; // errors if the spread cannot fit the sheet
pub fn marks(format: &CardFormat, profile: &PrintProfile) -> Item;   // a layer "marks": crop marks, fold marks top+bottom at fold_x, registration targets, colour bar
pub fn safe_rect(panel: Rect, profile: &PrintProfile) -> Rect;
pub enum Fit { Cover, Contain }
pub fn place_image(image_px: (u32, u32), target: Rect, fit: Fit) -> Rect;  // destination rect; caller clips to `target`
```

For `home` profiles the PDF page **is** the sheet (MediaBox = A4 landscape, TrimBox = the
spread centred in it) and marks must stay inside the printable area.

### ebc-pdf

```rust
pub struct WriteOptions { pub compress: bool, pub keep_layers: bool }
pub fn write_pdf(doc: &Document, opts: &WriteOptions) -> Result<Vec<u8>, PdfError>;
```

Requirements: `MediaBox`/`BleedBox`/`TrimBox` (and `CropBox` = MediaBox) per page from
`Page::{trim, bleed, slug}`; y-flip; paths with fill/stroke/dash; clipping groups;
transforms; images (JPEG passthrough via `DCTDecode`; raw → `FlateDecode`; RGBA → colour +
`SMask` only where the standard allows transparency, else error); fonts embedded as
subsets (TrueType → CIDFontType2 / Identity-H; CFF → CIDFontType0C) with `ToUnicode`;
refuse fonts whose `fsType` forbids embedding; `OutputIntent` (`GTS_PDFX`) with embedded ICC
profile; Info dictionary + XMP (`pdfx:GTS_PDFXVersion` / `pdfxid:GTS_PDFXVersion`),
`Trapped`, file ID; the standard's constraints enforced with errors, not silently
(e.g. RGB content in X-1a). Output must be byte-for-byte deterministic. Validate in tests
with `qpdf --check`, `pdfinfo -box`, `pdffonts`, `pdftoppm` (skip gracefully if a tool is
missing) and by parsing back with `lopdf` (dev-dependency).

### ebc-preflight

Independent of `ebc-pdf` (parse with `lopdf`).

```rust
pub struct Expectations { pub bleed_mm: f64, pub min_ppi: f64, pub require_output_intent: bool,
                          pub allow_rgb: bool, pub allow_transparency: bool, pub max_total_ink_percent: Option<f32>,
                          pub trim_mm: Option<Size-like (w,h)>, pub spine_mm: Option<f64>, pub min_qr_module_mm: Option<f64> }
pub enum Severity { Error, Warning, Info }
pub struct Finding { pub severity: Severity, pub code: &'static str, pub page: Option<u32>, pub message: String }
pub struct Report { pub findings: Vec<Finding>, pub pages: Vec<PageSummary> }   // PageSummary: boxes in mm, fonts, images with effective ppi, colour spaces
impl Report { pub fn passed(&self) -> bool; pub fn to_json(&self) -> String; }
pub fn check(pdf: &[u8], expect: &Expectations) -> Result<Report, PreflightError>;
```

Checks: parses; not encrypted; no JavaScript/actions/annotations/forms; page boxes present and
nested (Trim ⊆ Bleed ⊆ Media) with bleed ≥ expected; all fonts embedded (and subset);
image effective resolution (from CTM) ≥ `min_ppi`; colour spaces used (DeviceRGB flagged
unless allowed); transparency (SMask, `ca`/`CA` < 1, blend modes); OutputIntent present and
well-formed; PDF/X version markers consistent; Info `Title` present; `Trapped` set; total
ink of CMYK vector fills ≤ limit, and of the decoded pixels of losslessly stored CMYK images
(99.5th percentile; CMYK JPEGs are not decoded); heavy ink (> 180 %) within 2 mm of a fold
listed in `EBC.FoldXMm` (a warning advising lamination); the `EBC.*` Info keys (spine
width, fold positions) match the page geometry. Must never panic on malformed PDFs (fuzz-ish tests with truncated input).

### ebc-catalog

```rust
pub struct ProviderInfo { pub id: &'static str, pub name: &'static str, pub homepage: &'static str,
                          pub data_licence: &'static str, pub terms_url: &'static str, pub requires_key: bool,
                          pub browser_cors: bool, pub commercial_use: Permission, pub attribution: Option<&'static str>,
                          pub min_interval_ms: u32 }
pub trait Provider {
    fn info(&self) -> &ProviderInfo;
    fn lookup(&self, isbn: &Isbn13) -> Vec<HttpRequest>;                         // first request(s)
    fn parse(&self, isbn: &Isbn13, request: &HttpRequest, response: &HttpResponse) -> Result<Parsed, CatalogError>;
}
pub struct Parsed { pub records: Vec<BookMeta>, pub follow_up: Vec<HttpRequest> }   // follow-ups for multi-step APIs
pub fn merge(records: &[BookMeta]) -> Option<BookMeta>;   // field-wise, provider priority + agreement; keeps all sources/covers
pub fn default_providers(config: &Config) -> Vec<Box<dyn Provider>>;   // ordered per docs/research/01
#[cfg(feature = "client")] pub mod client;   // reqwest-based runner with per-host rate limiting, caching, User-Agent
```

Providers to implement first: Open Library, Google Books, BnF (SRU), DNB (SRU), Library of
Congress (SRU), Wikidata (SPARQL); then whatever `docs/research/01-isbn-databases.md`
ranks as allowed and useful. Each provider has fixture-based tests (realistic recorded
payload shapes, including "not found", rate-limit and malformed responses).

### ebc-covers

```rust
pub enum Risk { Low, Medium, High }
pub struct CoverCandidate { pub request: HttpRequest, pub source: &'static str, pub kind: CoverKind, pub risk: Risk, pub note: &'static str }
pub fn candidates(book: &BookMeta, policy: &Policy) -> Vec<CoverCandidate>;   // ordered cascade from docs/research/02, filtered by max risk
pub mod scrape { pub fn extract(html: &str, base_url: &str) -> Vec<Found> }   // og:image, twitter:image, JSON-LD Book/Product image, link rel=image_src; plus extract_all ISBNs to confirm the page is about the right book
pub mod robots { pub struct Robots; impl Robots { pub fn parse(txt: &str) -> Self; pub fn allowed(&self, user_agent: &str, path: &str) -> bool; pub fn crawl_delay(&self, ua: &str) -> Option<f32>; } }  // RFC 9309
pub mod sniff { pub struct ImageInfo { pub format: Format, pub width: u32, pub height: u32 } pub fn probe(bytes: &[u8]) -> Option<ImageInfo>; }  // JPEG/PNG/WebP/GIF/AVIF headers, no full decode
pub fn is_placeholder(info: &ImageInfo, bytes: &[u8]) -> bool;      // 1x1 gifs, known "no image" hashes, uniform images, tiny files
pub fn print_quality(info: &ImageInfo, target_mm: Size) -> Quality;  // effective ppi and a verdict vs min/target
```

### ebc-fontid

Image-based only (no language models). Design per `docs/research/10-font-identification-tech.md`.

```rust
pub struct FontIndex;                       // serialisable; entries: family, style, licence, source URL, descriptor
pub struct IndexEntry { pub family: String, pub style: String, pub licence: String, pub source_url: String, /* descriptor */ }
impl FontIndex { pub fn build(fonts: &[(IndexEntryMeta, Font)]) -> Self; pub fn to_bytes(&self) -> Vec<u8>; pub fn from_bytes(&[u8]) -> Result<Self, FontIdError>; }
pub struct TextRegion { pub x: u32, pub y: u32, pub w: u32, pub h: u32, pub score: f32 }
pub fn find_text_regions(img: &Pixmap) -> Vec<TextRegion>;            // classical detector; the UI may let the user adjust the box
pub struct Match { pub entry: IndexEntry, pub score: f32 }            // score 0..=1, higher is closer
pub fn identify(img: &Pixmap, region: &TextRegion, known_text: Option<&str>, index: &FontIndex) -> Vec<Match>; // descriptor stage (fast)
pub fn rerank(img: &Pixmap, region: &TextRegion, known_text: &str, candidates: &[(IndexEntry, Font)]) -> Vec<Match>; // render-and-compare stage
```

### ebc-spine

```rust
pub enum SpineSource<'a> { Synthesize, Provided(&'a Pixmap), FullWrap { image: &'a Pixmap, spine_px: (u32, u32) } }
pub struct SpineRequest<'a> { pub book: &'a BookMeta, pub cover: Option<&'a Pixmap>, pub source: SpineSource<'a>,
                              pub rect: Rect /* spine panel incl. bleed top/bottom */, pub bleed: f64,
                              pub title_font: &'a Font, pub author_font: &'a Font, pub direction: SpineDirection, pub safe_margin: f64 }
pub struct SpineDesign { pub items: Vec<Item>, pub background: Rgb8, pub text_color: Rgb8, pub title_size_pt: f64, pub notes: Vec<String> }
pub fn design(req: &SpineRequest, doc: &mut Document) -> Result<SpineDesign, SpineError>;  // may register images in `doc`
```

Synthesis: background from the cover's binding edge (uniform → solid colour; otherwise a
blurred/extended edge strip darkened or lightened for contrast), text colour by WCAG
contrast, title (and author when it fits) set along the spine as **outlines**, auto-fitted
between min and max sizes, truncated with an ellipsis only as a last resort. Text drawn as
outlines so that any font (incl. variable) works and nothing is embedded.

### ebc-card

```rust
pub struct FontSet { pub body: Font, pub body_bold: Font, pub code: Font, pub title: Font, pub title_alt: Option<Font> }
pub fn bundled_fonts() -> FontSet;      // feature "bundled-fonts": include_bytes! from assets/fonts (OFL)
pub enum CoverInput { Encoded(Vec<u8>) /* jpeg|png|webp */, Pixels(Pixmap), None /* typographic fallback cover */ }
pub struct CardRequest { pub book: BookMeta, pub cover: CoverInput, pub back_text: Option<String>,
                         pub spine: SpineChoice, pub claim_url: String, pub code: GiftCode, pub display_url: String,
                         pub giver: Option<String>, pub recipient: Option<String>, pub message: Option<String>,
                         pub locale: Locale, pub format: CardFormat, pub profile: PrintProfile, pub created_unix: Option<i64> }
pub fn compose(req: &CardRequest, fonts: &FontSet) -> Result<(Document, Vec<String> /* warnings */), CardError>;
pub fn render_pdf(req: &CardRequest, fonts: &FontSet) -> Result<Vec<u8>, CardError>;
pub fn preview_svg(req: &CardRequest, fonts: &FontSet) -> Result<String, CardError>;  // optional, for the web UI
```

Page 1 = outside (back | spine | front), page 2 = inside (QR, code, steps, message).
A typographic fallback cover (title/author on a coloured ground) is used when no cover
image is allowed or available.

### ebc-wasm, ebc-cli, ebc-server, web, demos

Specified in their own briefs once the libraries exist. The server's redemption and
delivery design follows `docs/research/05-ereader-distribution-ux.md`; its HTTP contract
is `docs/api.md` (v0.3: the two-service model of §1.1).

## 6. Testing strategy

| Level | What | Where |
|---|---|---|
| Unit / property | pure logic per crate (ISBN, codes, geometry, QR round-trip) | `crates/*/src`, `crates/*/tests` |
| Golden | PDF structure parsed back; rendered pages compared loosely | `crates/ebc-pdf`, `crates/ebc-card` |
| Preflight | generated PDFs through `ebc-preflight` + qpdf/poppler/ghostscript | CI job `print-check` |
| Integration | server + fictional stores + fictional printer + simulated e-readers over HTTP | `tests/`, `demo/` |
| End-to-end | Playwright: checkout with "eBook cadeau" → card PDF → QR decoded from the rendered PDF → redemption on each simulated device | `e2e/` |
| Wasm | `wasm-bindgen-test` in headless Chromium + size budget | `crates/ebc-wasm` |
