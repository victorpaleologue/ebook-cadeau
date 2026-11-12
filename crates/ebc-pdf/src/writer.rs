//! Document assembly: validation against the target standard, resource collection and
//! the PDF object graph.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use ebc_scene::{
    DocInfo, Document, FontResource, ImageResource, Item, OutputIntent, PT_PER_MM, Page,
    PdfStandard,
};
use pdf_writer::types::{OutputIntentSubtype, TrappingStatus};
use pdf_writer::writers::ColorSpace;
use pdf_writer::{Filter, Finish, Name, Pdf, Rect, Ref, Settings, TextStr};

use crate::WriteOptions;
use crate::content::{
    ContentContext, MAX_GROUP_DEPTH, PageContent, PageUse, REGISTRATION_SPACE, SRGB_SPACE,
    layer_resource_name, round_pt,
};
use crate::date::DateTime;
use crate::error::PdfError;
use crate::font::{EmbeddedFont, FontUsage};
use crate::image::{
    IccPool, ImageSpace, PreparedImage, SRGB_PROFILE, allows_transparency, max_icc_version,
};
use crate::{icc, md5, xmp};

/// What the writer calls itself in the Info dictionary and the XMP packet.
const PRODUCER: &str = concat!("ebc-pdf ", env!("CARGO_PKG_VERSION"));

/// Info dictionary keys the writer sets itself; [`DocInfo::custom`] must not use them.
const RESERVED_INFO_KEYS: [&str; 11] = [
    "Title",
    "Author",
    "Subject",
    "Keywords",
    "Creator",
    "Producer",
    "CreationDate",
    "ModDate",
    "Trapped",
    "GTS_PDFXVersion",
    "GTS_PDFXConformance",
];

/// Most pages a document may have. The page tree is written flat, and PDF 1.3 consumers
/// only have to support arrays of 8191 elements.
const MAX_PAGES: usize = 8_000;

/// Smallest and largest page side a PDF consumer has to support, in points.
const MIN_PAGE_PT: f64 = 3.0;
const MAX_PAGE_PT: f64 = 14_400.0;

/// Hands out consecutive indirect object numbers.
pub(crate) struct RefAllocator {
    next: i32,
}

impl RefAllocator {
    fn new() -> Self {
        Self { next: 1 }
    }

    /// The next free object reference.
    pub(crate) fn next(&mut self) -> Result<Ref, PdfError> {
        if self.next == i32::MAX {
            return Err(PdfError::TooLarge { what: "the number of objects" });
        }
        let id = Ref::new(self.next);
        self.next += 1;
        Ok(id)
    }
}

/// Zlib-compresses `data` for a `FlateDecode` stream.
pub(crate) fn deflate(data: &[u8]) -> Vec<u8> {
    miniz_oxide::deflate::compress_to_vec_zlib(data, 6)
}

/// Checks that a stream of `len` bytes fits the PDF `/Length` integer.
pub(crate) fn stream_len(len: usize, what: &'static str) -> Result<i32, PdfError> {
    i32::try_from(len).map_err(|_| PdfError::TooLarge { what })
}

/// The sanitised document information.
struct Meta {
    title: String,
    author: Option<String>,
    subject: Option<String>,
    keywords: Option<String>,
    creator: Option<String>,
    language: Option<String>,
    date: Option<DateTime>,
}

fn non_empty(value: Option<&str>) -> Option<String> {
    let clean = xmp::sanitize(value?.trim());
    (!clean.is_empty()).then_some(clean)
}

fn check_info(info: &DocInfo, standard: PdfStandard) -> Result<Meta, PdfError> {
    let is_x = standard != PdfStandard::Plain;
    let title = xmp::sanitize(info.title.trim());
    if is_x && title.is_empty() {
        return Err(PdfError::MissingTitle { standard });
    }
    let date = match info.created_unix {
        Some(seconds) => Some(DateTime::from_unix(seconds).ok_or(PdfError::InvalidDate(seconds))?),
        None if is_x => return Err(PdfError::MissingCreationDate { standard }),
        None => None,
    };
    let keywords: Vec<String> =
        info.keywords.iter().filter_map(|k| non_empty(Some(k.as_str()))).collect();

    let mut seen = BTreeSet::new();
    for (key, _) in &info.custom {
        let reason = if key.is_empty() {
            Some("the key is empty")
        } else if key.len() > 127 {
            Some("the key is longer than 127 bytes")
        } else if !key.bytes().all(|b| b.is_ascii_graphic() && !b"()<>[]{}/%#".contains(&b)) {
            Some("keys must be printable ASCII without spaces or PDF delimiters")
        } else if RESERVED_INFO_KEYS.contains(&key.as_str()) {
            Some("the key is reserved for the writer")
        } else if !seen.insert(key.as_str()) {
            Some("the key is given more than once")
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err(PdfError::InvalidCustomKey { key: key.clone(), reason });
        }
    }

    Ok(Meta {
        title,
        author: non_empty(info.author.as_deref()),
        subject: non_empty(info.subject.as_deref()),
        keywords: (!keywords.is_empty()).then(|| keywords.join(", ")),
        creator: non_empty(info.creator.as_deref()),
        language: non_empty(info.language.as_deref()),
        date,
    })
}

fn check_output_intent(doc: &Document) -> Result<Option<&OutputIntent>, PdfError> {
    let standard = doc.standard;
    let is_x = standard != PdfStandard::Plain;
    let Some(intent) = &doc.output_intent else {
        return if is_x { Err(PdfError::MissingOutputIntent { standard }) } else { Ok(None) };
    };
    if intent.condition_identifier.trim().is_empty() {
        return Err(PdfError::UnsupportedOutputIntent {
            reason: "the output condition identifier is empty",
        });
    }
    if is_x && intent.components != 4 {
        return Err(PdfError::UnsupportedOutputIntent {
            reason: "only CMYK (4-component) printing conditions are supported for PDF/X",
        });
    }
    if is_x && intent.registry_name.trim().is_empty() && intent.info.trim().is_empty() {
        return Err(PdfError::UnsupportedOutputIntent {
            reason: "a printing condition that is not in a registry needs a description \
                     (OutputIntent::info)",
        });
    }
    match &intent.icc_profile {
        Some(profile) => {
            let context = || "the output intent".to_owned();
            let info = icc::inspect(profile)
                .map_err(|reason| PdfError::InvalidIccProfile { context: context(), reason })?;
            if info.components != intent.components {
                return Err(PdfError::InvalidIccProfile {
                    context: context(),
                    reason: "the profile's colour space does not match OutputIntent::components",
                });
            }
            if info.version_major > max_icc_version(standard) {
                return Err(PdfError::InvalidIccProfile {
                    context: context(),
                    reason: "the ICC version is newer than the target PDF version supports",
                });
            }
        }
        None => {
            // PDF/X-4 always wants the profile; the older parts accept a bare reference
            // to a characterisation registered with the ICC.
            if standard == PdfStandard::X4 || (is_x && intent.registry_name.trim().is_empty()) {
                return Err(PdfError::MissingOutputProfile { standard });
            }
        }
    }
    Ok(Some(intent))
}

/// The page boxes in PDF points: media (= crop), bleed and trim.
fn page_boxes(page: &Page, number: usize) -> Result<[Rect; 3], PdfError> {
    let invalid = |reason| PdfError::InvalidPage { page: number, reason };
    let (w, h) = (page.trim.w, page.trim.h);
    if !(w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0) {
        return Err(invalid("the trim size must be positive and finite"));
    }
    if !(page.bleed.is_finite() && page.bleed >= 0.0) {
        return Err(invalid("the bleed must be zero or positive"));
    }
    if !(page.slug.is_finite() && page.slug >= 0.0) {
        return Err(invalid("the slug must be zero or positive"));
    }
    let margin = page.bleed + page.slug;
    let media_w = (w + 2.0 * margin) * PT_PER_MM;
    let media_h = (h + 2.0 * margin) * PT_PER_MM;
    // A hair of tolerance so that exactly 200 inches survives the unit conversion.
    let range = (MIN_PAGE_PT - 1.0e-6)..=(MAX_PAGE_PT + 1.0e-6);
    if !(range.contains(&media_w) && range.contains(&media_h)) {
        return Err(invalid("the sheet must be between 3 pt and 14400 pt (5080 mm) on each side"));
    }
    let pt = |mm: f64| round_pt(mm * PT_PER_MM);
    Ok([
        Rect::new(0.0, 0.0, pt(w + 2.0 * margin), pt(h + 2.0 * margin)),
        Rect::new(
            pt(page.slug),
            pt(page.slug),
            pt(page.slug + 2.0 * page.bleed + w),
            pt(page.slug + 2.0 * page.bleed + h),
        ),
        Rect::new(pt(margin), pt(margin), pt(margin + w), pt(margin + h)),
    ])
}

/// Everything the pages reference, gathered before any object is written.
#[derive(Default)]
struct Usage {
    fonts: BTreeMap<u32, FontUsage>,
    images: BTreeSet<u32>,
    /// Layer name → index in order of first appearance.
    layers: BTreeMap<String, usize>,
}

fn collect(
    doc: &Document,
    items: &[Item],
    depth: usize,
    page: usize,
    usage: &mut Usage,
) -> Result<(), PdfError> {
    for item in items {
        match item {
            Item::Path { .. } => {}
            Item::Image { image, .. } => {
                if doc.image(*image).is_none() {
                    return Err(PdfError::UnknownImage(*image));
                }
                usage.images.insert(image.0);
            }
            Item::Glyphs { font, glyphs, glyph_text, .. } => {
                if glyphs.is_empty() {
                    continue;
                }
                if doc.font(*font).is_none() {
                    return Err(PdfError::UnknownFont(*font));
                }
                let font_usage = usage.fonts.entry(font.0).or_default();
                for (i, glyph) in glyphs.iter().enumerate() {
                    font_usage.record(glyph.id, glyph_text.get(i).map_or("", String::as_str));
                }
            }
            Item::Group { layer, items, .. } => {
                if depth >= MAX_GROUP_DEPTH {
                    return Err(PdfError::NestingTooDeep { page, max: MAX_GROUP_DEPTH });
                }
                if let Some(name) = layer {
                    let next = usage.layers.len();
                    usage.layers.entry(name.clone()).or_insert(next);
                }
                collect(doc, items, depth + 1, page, usage)?;
            }
        }
    }
    Ok(())
}

fn find_font(doc: &Document, id: u32) -> Option<&FontResource> {
    doc.fonts.iter().find(|f| f.id.0 == id)
}

fn find_image(doc: &Document, id: u32) -> Option<&ImageResource> {
    doc.images.iter().find(|i| i.id.0 == id)
}

/// Writes `doc` as a PDF file. See [`crate::write_pdf`].
pub(crate) fn write(doc: &Document, opts: &WriteOptions) -> Result<Vec<u8>, PdfError> {
    let standard = doc.standard;
    let compress = opts.compress;

    // ----- validation ------------------------------------------------------------------
    if doc.pages.is_empty() {
        return Err(PdfError::NoPages);
    }
    if doc.pages.len() > MAX_PAGES {
        return Err(PdfError::TooLarge { what: "the number of pages" });
    }
    let meta = check_info(&doc.info, standard)?;
    let intent = check_output_intent(doc)?;
    let mut boxes = Vec::with_capacity(doc.pages.len());
    for (i, page) in doc.pages.iter().enumerate() {
        boxes.push(page_boxes(page, i + 1)?);
    }

    // ----- resources -------------------------------------------------------------------
    let mut usage = Usage::default();
    for (i, page) in doc.pages.iter().enumerate() {
        collect(doc, &page.items, 0, i + 1, &mut usage)?;
    }

    let mut fonts = BTreeMap::new();
    for (index, (id, font_usage)) in usage.fonts.iter().enumerate() {
        if let Some(resource) = find_font(doc, *id) {
            fonts.insert(*id, EmbeddedFont::prepare(resource, font_usage, index)?);
        }
    }

    let mut pool = IccPool::default();
    let mut images = BTreeMap::new();
    for (index, id) in usage.images.iter().enumerate() {
        if let Some(resource) = find_image(doc, *id) {
            images.insert(
                *id,
                PreparedImage::prepare(resource, index, standard, compress, &mut pool)?,
            );
        }
    }
    let image_names: BTreeMap<u32, Vec<u8>> =
        images.iter().map(|(id, image)| (*id, image.resource_name.clone())).collect();

    let keep_layers = opts.keep_layers && allows_transparency(standard);

    // ----- page content ----------------------------------------------------------------
    let ctx = ContentContext {
        standard,
        pretty: !compress,
        fonts: &fonts,
        images: &image_names,
        layers: keep_layers.then_some(&usage.layers),
    };
    let mut contents = Vec::with_capacity(doc.pages.len());
    let mut used = PageUse::default();
    for (i, page) in doc.pages.iter().enumerate() {
        let (bytes, page_use) = PageContent::render(&ctx, page, i + 1)?;
        used.fonts.extend(&page_use.fonts);
        used.images.extend(&page_use.images);
        used.layers.extend(&page_use.layers);
        used.srgb |= page_use.srgb;
        contents.push((bytes, page_use));
    }

    let srgb_index = used.srgb.then(|| pool.intern(SRGB_PROFILE, 3));
    let intent_profile_index = intent
        .and_then(|i| i.icc_profile.as_deref().map(|profile| pool.intern(profile, i.components)));

    // ----- objects ---------------------------------------------------------------------
    let mut refs = RefAllocator::new();
    let catalog_ref = refs.next()?;
    let info_ref = refs.next()?;
    let metadata_ref = refs.next()?;
    let pages_ref = refs.next()?;

    let mut pdf = Pdf::with_settings(Settings { pretty: !compress });
    match standard {
        PdfStandard::X1a2001 | PdfStandard::X3_2002 => pdf.set_version(1, 3),
        PdfStandard::X4 => pdf.set_version(1, 6),
        PdfStandard::Plain => pdf.set_version(1, 7),
    }

    // Only profiles that something in the file refers to are written.
    let mut used_profiles: BTreeSet<usize> =
        srgb_index.into_iter().chain(intent_profile_index).collect();
    for (id, image) in &images {
        if let (true, ImageSpace::Icc(index)) = (used.images.contains(id), image.space) {
            used_profiles.insert(index);
        }
    }
    let icc_refs = pool.write(&mut pdf, &mut refs, compress, &used_profiles)?;
    let profile_ref = |index: Option<usize>| index.and_then(|i| icc_refs.get(i).copied().flatten());

    let mut font_refs = BTreeMap::new();
    for (id, font) in &fonts {
        if used.fonts.contains(id) {
            font_refs.insert(*id, font.write(&mut pdf, &mut refs, compress)?);
        }
    }
    let mut image_refs = BTreeMap::new();
    for (id, image) in &images {
        if used.images.contains(id) {
            image_refs.insert(*id, image.write(&mut pdf, &mut refs, &icc_refs)?);
        }
    }

    // Optional-content groups, in order of first appearance.
    let mut layer_names: Vec<(&str, usize)> =
        usage.layers.iter().map(|(name, index)| (name.as_str(), *index)).collect();
    layer_names.sort_by_key(|(_, index)| *index);
    let mut layer_refs = BTreeMap::new();
    for (name, index) in layer_names {
        if used.layers.contains(&index) {
            let id = refs.next()?;
            let clean = xmp::sanitize(name);
            let mut ocg = pdf.indirect(id).dict();
            ocg.pair(Name(b"Type"), Name(b"OCG"));
            ocg.pair(Name(b"Name"), TextStr(&clean));
            ocg.finish();
            layer_refs.insert(index, id);
        }
    }

    // Pages.
    let mut page_refs = Vec::with_capacity(doc.pages.len());
    for ((bytes, page_use), [media, bleed, trim]) in contents.iter().zip(&boxes) {
        let page_ref = refs.next()?;
        let content_ref = refs.next()?;
        page_refs.push(page_ref);

        let data: Cow<'_, [u8]> =
            if compress { Cow::Owned(deflate(bytes)) } else { Cow::Borrowed(bytes) };
        stream_len(data.len(), "a page content stream")?;
        let mut stream = pdf.stream(content_ref, &data);
        if compress {
            stream.filter(Filter::FlateDecode);
        }
        stream.finish();

        let mut page = pdf.page(page_ref);
        page.parent(pages_ref)
            .media_box(*media)
            .crop_box(*media)
            .bleed_box(*bleed)
            .trim_box(*trim)
            .contents(content_ref);
        let mut resources = page.resources();
        resources.proc_sets_all();
        if !page_use.fonts.is_empty() {
            let mut dict = resources.fonts();
            for id in &page_use.fonts {
                if let (Some(font), Some(font_ref)) = (fonts.get(id), font_refs.get(id)) {
                    dict.pair(font.resource_name(), *font_ref);
                }
            }
        }
        if !page_use.images.is_empty() {
            let mut dict = resources.x_objects();
            for id in &page_use.images {
                if let (Some(name), Some(image_ref)) = (image_names.get(id), image_refs.get(id)) {
                    dict.pair(Name(name), *image_ref);
                }
            }
        }
        if page_use.srgb || page_use.registration {
            let mut dict = resources.color_spaces();
            if page_use.registration {
                // The registration colour: 100 % on every separation.
                let mut separation = dict
                    .insert(Name(REGISTRATION_SPACE))
                    .start::<ColorSpace>()
                    .separation(Name(b"All"));
                separation.alternate_color_space().device_cmyk();
                let mut tint = separation.tint_exponential();
                tint.domain([0.0, 1.0]);
                tint.c0([0.0, 0.0, 0.0, 0.0]);
                tint.c1([1.0, 1.0, 1.0, 1.0]);
                tint.n(1.0);
            }
            if let (true, Some(srgb_ref)) = (page_use.srgb, profile_ref(srgb_index)) {
                dict.insert(Name(SRGB_SPACE)).start::<ColorSpace>().icc_based(srgb_ref);
            }
        }
        if !page_use.layers.is_empty() {
            let mut dict = resources.insert(Name(b"Properties")).dict();
            for index in &page_use.layers {
                if let Some(layer_ref) = layer_refs.get(index) {
                    dict.pair(Name(&layer_resource_name(*index)), *layer_ref);
                }
            }
        }
        resources.finish();
        page.finish();
    }
    let page_count = i32::try_from(page_refs.len())
        .map_err(|_| PdfError::TooLarge { what: "the number of pages" })?;
    pdf.pages(pages_ref).kids(page_refs.iter().copied()).count(page_count);

    // ----- metadata --------------------------------------------------------------------
    // Identifiers are derived from everything written so far plus the document
    // information, so identical input gives identical files and different input
    // (almost certainly) different identifiers.
    let date_xmp = meta.date.map(DateTime::to_xmp);
    let mut hasher = md5::Md5::new();
    hasher.update(pdf.as_bytes());
    let hashed = pdf.len();
    let body_digest = {
        let mut body = hasher.clone();
        body.update(meta.title.as_bytes())
            .update(&[0])
            .update(meta.author.as_deref().unwrap_or("").as_bytes())
            .update(&[0])
            .update(date_xmp.as_deref().unwrap_or("").as_bytes());
        body.finish()
    };
    let document_id = format!("uuid:{}", md5::uuid(md5::digest(&[b"document", &body_digest])));
    let instance_id = format!("uuid:{}", md5::uuid(md5::digest(&[b"instance", &body_digest])));

    let packet = xmp::packet(&xmp::XmpFields {
        standard,
        title: &meta.title,
        author: meta.author.as_deref(),
        subject: meta.subject.as_deref(),
        keywords: meta.keywords.as_deref(),
        creator_tool: meta.creator.as_deref(),
        producer: PRODUCER,
        language: meta.language.as_deref(),
        date: date_xmp.as_deref(),
        document_id: &document_id,
        instance_id: &instance_id,
    });
    stream_len(packet.len(), "the XMP metadata")?;
    // Deliberately uncompressed so that tools can find the packet by scanning the file.
    pdf.metadata(metadata_ref, &packet).finish();

    let mut info = pdf.document_info(info_ref);
    if !meta.title.is_empty() {
        info.title(TextStr(&meta.title));
    }
    if let Some(author) = &meta.author {
        info.author(TextStr(author));
    }
    if let Some(subject) = &meta.subject {
        info.subject(TextStr(subject));
    }
    if let Some(keywords) = &meta.keywords {
        info.keywords(TextStr(keywords));
    }
    if let Some(creator) = &meta.creator {
        info.creator(TextStr(creator));
    }
    info.producer(TextStr(PRODUCER));
    if let Some(date) = meta.date {
        info.creation_date(date.to_pdf());
        info.modified_date(date.to_pdf());
    }
    info.trapped(TrappingStatus::NotTrapped);
    match standard {
        PdfStandard::Plain => {}
        PdfStandard::X1a2001 => {
            info.pair(Name(b"GTS_PDFXVersion"), TextStr("PDF/X-1:2001"));
            info.pair(Name(b"GTS_PDFXConformance"), TextStr("PDF/X-1a:2001"));
        }
        PdfStandard::X3_2002 => {
            info.pair(Name(b"GTS_PDFXVersion"), TextStr("PDF/X-3:2002"));
        }
        PdfStandard::X4 => {
            info.pair(Name(b"GTS_PDFXVersion"), TextStr("PDF/X-4"));
        }
    }
    for (key, value) in &doc.info.custom {
        info.pair(Name(key.as_bytes()), TextStr(&xmp::sanitize(value)));
    }
    info.finish();

    // ----- catalog ---------------------------------------------------------------------
    let mut catalog = pdf.catalog(catalog_ref);
    catalog.pages(pages_ref);
    catalog.metadata(metadata_ref);
    if let Some(language) = &meta.language {
        catalog.lang(TextStr(language));
    }
    if let Some(intent) = intent {
        let mut intents = catalog.output_intents();
        let mut dict = intents.push();
        dict.subtype(OutputIntentSubtype::PDFX);
        dict.output_condition_identifier(TextStr(&xmp::sanitize(
            intent.condition_identifier.trim(),
        )));
        let registry = xmp::sanitize(intent.registry_name.trim());
        if !registry.is_empty() {
            dict.registry_name(TextStr(&registry));
        }
        let description = xmp::sanitize(intent.info.trim());
        if !description.is_empty() {
            dict.info(TextStr(&description));
        }
        if let Some(output_profile_ref) = profile_ref(intent_profile_index) {
            dict.dest_output_profile(output_profile_ref);
        }
    }
    if !layer_refs.is_empty() {
        let mut properties = catalog.insert(Name(b"OCProperties")).dict();
        properties.insert(Name(b"OCGs")).array().items(layer_refs.values().copied());
        let mut default = properties.insert(Name(b"D")).dict();
        default.pair(Name(b"Name"), TextStr("Layers"));
        default.insert(Name(b"Order")).array().items(layer_refs.values().copied());
    }
    if doc.pages.iter().any(|p| p.label.as_deref().is_some_and(|l| !l.trim().is_empty())) {
        let mut labels = catalog.insert(Name(b"PageLabels")).dict();
        let mut nums = labels.insert(Name(b"Nums")).array();
        for (i, page) in doc.pages.iter().enumerate() {
            let Ok(index) = i32::try_from(i) else { break };
            nums.item(index);
            let mut label = nums.push().dict();
            match page.label.as_deref().map(str::trim).filter(|l| !l.is_empty()) {
                Some(text) => {
                    label.pair(Name(b"P"), TextStr(&xmp::sanitize(text)));
                }
                None => {
                    label.pair(Name(b"S"), Name(b"D"));
                    label.pair(Name(b"St"), index.saturating_add(1));
                }
            }
        }
    }
    catalog.finish();

    // ----- trailer ---------------------------------------------------------------------
    // The identifier is the digest of every object in the file.
    hasher.update(pdf.as_bytes().get(hashed..).unwrap_or_default());
    let file_id = hasher.finish().to_vec();
    pdf.set_file_id((file_id.clone(), file_id));
    Ok(pdf.finish())
}
