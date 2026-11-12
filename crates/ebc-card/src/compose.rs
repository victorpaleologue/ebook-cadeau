//! The composer: validation, the rights gate, document information, marks and the
//! instructions printed on a home sheet. The two pages are built in `outside.rs` and
//! `inside.rs`.

use ebc_i18n::{Msg, t, tf};
use ebc_layout::{
    CardFormat, DuplexFlip, Face, GUIDES_LAYER, PageGeometry, PrintProfile, caption_areas,
    check_marks_fit, guides, info_entries, marks, new_page_for, page_geometry, scale_ruler,
};
use ebc_scene::{DocInfo, Document, ImageData, Item, Page, PdfStandard, PixelFormat, Point, Rect};
use ebc_text::fit_size;

use crate::cover::DecodedCover;
use crate::error::{CardError, CardWarning, TextField};
use crate::inspect::{Inspector, LeafKind};
use crate::paint::Paint;
use crate::request::{CardRequest, CoverInput, FontSet, OutputProfile, SpineChoice};
use crate::rights::{self, Placement, RightsBasis};
use crate::text::{Align, MIN_TYPE_PT, ParaSpec, Setter, clean, layout, note_clamped};

/// Name of the layer that holds the instructions printed in the margin of a home sheet
/// and the job line of a press sheet.
pub const NOTES_LAYER: &str = "notes";

/// Tolerance of the composer's own geometric checks, in millimetres.
const EPS_MM: f64 = 1e-6;

/// State shared by the page builders.
pub(crate) struct Composer<'a> {
    pub(crate) req: &'a CardRequest,
    pub(crate) fonts: &'a FontSet,
    pub(crate) doc: Document,
    pub(crate) setter: Setter,
    pub(crate) warnings: Vec<CardWarning>,
    pub(crate) paint: Paint,
    pub(crate) geometry: PageGeometry,
    /// Module size of the QR code once the inside page is built, in millimetres.
    pub(crate) qr_module_mm: Option<f64>,
    /// The visible rectangle of the cover on the outside page, when one is printed.
    pub(crate) cover_rect: Option<Rect>,
}

/// A decoded cover and how the rights gate lets it be placed.
pub(crate) struct CoverPlan {
    pub(crate) image: DecodedCover,
    pub(crate) placement: Placement,
    pub(crate) rights: RightsBasis,
}

/// What a composition is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Purpose {
    /// The print document: colours converted to the output intent where possible.
    Print,
    /// A screen preview: nothing is converted, so that the screen shows the sRGB design
    /// (the cover as it is, grounds through the exact inverse of the naive conversion).
    Preview,
}

pub(crate) fn compose(
    req: &CardRequest,
    fonts: &FontSet,
    purpose: Purpose,
) -> Result<(Document, Vec<CardWarning>), CardError> {
    req.format.validate()?;
    req.profile.validate()?;
    let geometry = page_geometry(&req.format, &req.profile)?;
    // Without cut and fold marks nobody can finish a home sheet: refuse rather than
    // leave them out silently.
    check_marks_fit(&req.format, &req.profile)?;
    if req.book.title.trim().is_empty() {
        return Err(CardError::EmptyTitle);
    }
    let pdfx = req.profile.standard != PdfStandard::Plain;
    if pdfx && req.created_unix.is_none() {
        return Err(CardError::MissingCreationDate);
    }

    let mut warnings = Vec::new();

    // ---- The rights gate -------------------------------------------------------------
    let route = rights::route_of(&req.profile);
    let basis = req.cover.rights();
    let placement = match &req.cover {
        CoverInput::None => None,
        _ if basis == RightsBasis::None => {
            // No code path decodes, let alone places, a cover without a rights basis.
            warnings.push(CardWarning::CoverWithoutRights);
            None
        }
        _ => Some(rights::placement(basis, route)?),
    };
    let effective_rights = if placement.is_some() { basis } else { RightsBasis::None };
    if req.spine.requires_authorisation()
        && !(req.adaptation_authorised && rights::allows_adaptation(effective_rights))
    {
        return Err(CardError::AdaptationNotAuthorised {
            style: req.spine.name(),
            basis: effective_rights,
            authorised: req.adaptation_authorised,
        });
    }
    // ---- The output profile and the colour transform -----------------------------------
    let output_profile = if pdfx {
        let profile = match &req.output_profile {
            Some(profile) => profile.clone(),
            None => OutputProfile::default_for_build()?,
        };
        if profile.is_fallback() {
            warnings.push(CardWarning::FallbackOutputProfile);
        } else if !profile.is_printer_class() {
            warnings.push(CardWarning::OutputProfileNotPrinterClass {
                class: profile.device_class().to_owned(),
            });
        }
        Some(profile)
    } else {
        None
    };
    // Covers, images and flat colours of a PDF/X file go through one ICC transform to
    // the output intent (docs/research/03-print-requirements.md §3.4), built under the
    // printer's total ink limit by grey-component replacement (`ebc_color::ink`).
    let link = match (&output_profile, purpose) {
        (Some(profile), Purpose::Print) => {
            match profile.colour_link_with_ink_limit(req.profile.max_total_ink_percent) {
                Ok(link) => Some(link),
                Err(error) => {
                    // The fallback profile is already reported as such.
                    if !profile.is_fallback() {
                        warnings.push(CardWarning::ColourNotManaged { reason: error.to_string() });
                    }
                    None
                }
            }
        }
        _ => None,
    };

    let cover = match placement {
        Some(placement) => DecodedCover::decode(&req.cover)?.map(|image| CoverPlan {
            image,
            placement,
            rights: basis,
        }),
        None => None,
    };
    if let Some(plan) = &cover {
        if basis == RightsBasis::UserPrivateCopy {
            warnings.push(CardWarning::OwnPrinterOnly);
        }
        if pdfx && purpose == Purpose::Print && link.is_none() && !plan.image.is_cmyk_jpeg() {
            // `render_pdf` refuses the file unless the profile allows RGB.
            warnings.push(CardWarning::CoverNotConverted);
        }
    } else if matches!(req.spine, SpineChoice::CoverColour) {
        warnings.push(CardWarning::SpineChoiceIgnored { style: req.spine.name() });
    }
    if req.format.spine_width > CardFormat::SPINE_TEXT_MAX_MM {
        warnings.push(CardWarning::SpineOutsideTextRange { spine_mm: req.format.spine_width });
    }

    // ---- The document ------------------------------------------------------------------
    // A proof with the guides overlay is not a print file: it keeps its layers (so that a
    // viewer can hide the guides) and therefore drops the PDF/X label and output intent,
    // which a RIP would take as "print everything" (§3.2: no layers in print files).
    let (standard, output_intent) = if req.guides {
        (PdfStandard::Plain, None)
    } else {
        (req.profile.standard, output_profile.as_ref().map(OutputProfile::to_output_intent))
    };
    let doc = Document {
        info: DocInfo::default(),
        standard,
        output_intent,
        pages: Vec::new(),
        images: Vec::new(),
        fonts: Vec::new(),
    };

    let mut composer = Composer {
        req,
        fonts,
        doc,
        setter: Setter::default(),
        warnings,
        paint: Paint::for_profile(&req.profile, link),
        geometry,
        qr_module_mm: None,
        cover_rect: None,
    };

    let outside = composer.outside_page(cover)?;
    composer.doc.pages.push(outside);
    let inside = composer.inside_page()?;
    composer.doc.pages.push(inside);
    composer.doc.info = composer.doc_info(effective_rights);
    composer.check_cover_is_untouched()?;
    composer.convert_images();
    if pdfx && purpose == Purpose::Print {
        composer.advise_on_creases();
    }

    Ok((composer.doc, composer.warnings))
}

impl Composer<'_> {
    pub(crate) fn format(&self) -> &CardFormat {
        &self.req.format
    }

    pub(crate) fn profile(&self) -> &PrintProfile {
        &self.req.profile
    }

    /// An empty page for one face.
    pub(crate) fn new_page(&self, face: Face) -> Result<Page, CardError> {
        Ok(new_page_for(face, self.format(), self.profile())?)
    }

    /// Cleans an untrusted single-line string for `font`.
    pub(crate) fn clean_line(
        &mut self,
        font: &ebc_text::Font,
        text: &str,
        field: TextField,
    ) -> String {
        clean(font, text, field, false, &mut self.warnings)
    }

    /// Cleans an untrusted multi-line string for `font`.
    pub(crate) fn clean_block(
        &mut self,
        font: &ebc_text::Font,
        text: &str,
        field: TextField,
    ) -> String {
        clean(font, text, field, true, &mut self.warnings)
    }

    /// What every page gets on top of its artwork: the printer's marks, the notes in the
    /// margin (duplex and cutting instructions on a home sheet, a job line on a press
    /// sheet with marks, the ruler's label) and, when asked for, the proofing overlay.
    pub(crate) fn finish_page(&mut self, page: &mut Page, face: Face) -> Result<(), CardError> {
        let (format, profile) = (self.req.format, self.req.profile.clone());
        // One object per mark, clear of the bleed box: `ebc-layout` sees to both.
        page.items.push(marks(&format, &profile));
        let notes = self.margin_notes(face)?;
        if !notes.is_empty() {
            page.items.push(Item::layer(NOTES_LAYER, notes));
        }
        if self.req.guides {
            page.items.push(guides(&format, &profile));
            if !self.warnings.contains(&CardWarning::GuidesIncluded) {
                self.warnings.push(CardWarning::GuidesIncluded);
            }
        }
        Ok(())
    }

    fn margin_notes(&mut self, face: Face) -> Result<Vec<Item>, CardError> {
        let (format, profile) = (self.req.format, self.req.profile.clone());
        let locale = self.req.locale;
        let font = self.fonts.body.clone();
        // On a home sheet a dark grey on the black plate only: readable, and clearly not
        // part of the card. The job line of a press sheet is 100 % K, like the marks of a
        // digital job, so that it survives on the black plate alone.
        let color = if profile.sheet.is_some() { self.paint.grey(0.25) } else { self.paint.ink() };
        let mut items = Vec::new();
        if !profile.marks.any() {
            return Ok(items);
        }

        if let Some([above, below]) = caption_areas(&format, &profile) {
            let texts: [String; 2] = if profile.sheet.is_some() {
                // The inside page is laid out for a sheet turned about the axis parallel
                // to the creases, like the page of a book: the inside-front panel (left
                // of page 2) must land behind the front cover (right of page 1). The
                // creases run across the short dimension of a landscape sheet, so that
                // turn is "flip on the short edge" in a print dialog — not the usual
                // default, "long edge", which would print the inside upside down behind
                // the wrong cover. `duplex_flip()` gives the edge for the sheet in use
                // (the long edge only for tall custom cards on a portrait sheet).
                let duplex = match self.geometry.duplex_flip() {
                    DuplexFlip::ShortEdge => Msg::PrintDuplexShortEdge,
                    DuplexFlip::LongEdge => Msg::PrintDuplexLongEdge,
                };
                [t(locale, duplex).to_owned(), t(locale, Msg::PrintCutFold).to_owned()]
            } else {
                // Press sheet with marks: a job line in the bottom slug — product, title,
                // format and face, in the card's language.
                let title = self.clean_line(&font, &self.req.book.title, TextField::Title);
                let title: String = title.chars().take(60).collect();
                let size = tf(
                    locale,
                    Msg::CardFormatLine,
                    &[
                        ("width", &trim_mm(format.cover.w)),
                        ("height", &trim_mm(format.cover.h)),
                        ("spine", &trim_mm(format.spine_width)),
                    ],
                );
                let face = match face {
                    Face::Outside => Msg::OutsideLabel,
                    Face::Inside => Msg::InsideLabel,
                };
                // The crease positions, from the left edge of the trim, as the printer
                // must set its creasing tool (they are also in the EBC.FoldXMm key).
                let [crease_a, crease_b] = format.fold_x();
                let job = format!(
                    "{} · {title} · {size} · {} · {}: x = {} / {} mm",
                    t(locale, Msg::ProductName),
                    t(locale, face),
                    t(locale, Msg::FoldHere),
                    trim_mm(crease_a),
                    trim_mm(crease_b),
                );
                [String::new(), job]
            };
            for (area, text) in [above, below].into_iter().zip(texts) {
                let text = self.clean_line(&font, &text, TextField::SheetHint);
                let para =
                    layout(&font, &text, area.w, area.h, &ParaSpec::new(7.0, MIN_TYPE_PT, 1.2, 2));
                if para.is_empty() {
                    continue;
                }
                if para.clamped {
                    note_clamped(TextField::SheetHint, &mut self.warnings);
                }
                let top = area.y + (area.h - para.height()).max(0.0) / 2.0;
                items.extend(self.setter.para(
                    &mut self.doc,
                    &font,
                    &para,
                    color,
                    area.x,
                    top,
                    area.w,
                    Align::Left,
                )?);
            }
        }

        if profile.marks.scale_ruler
            && let Some(ruler) = scale_ruler(&format, &profile)
        {
            let label = format!("{} mm", trim_mm(ruler.length_mm));
            let area = ruler.label_area;
            let opts = ebc_text::ShapeOptions::default();
            if let Some(size) = fit_size(&font, &label, area.w, area.h, 4.5, 6.5, &opts) {
                items.extend(self.setter.line(
                    &mut self.doc,
                    &font,
                    &label,
                    size,
                    color,
                    Point::new(ruler.label_origin.x, ruler.label_origin.y),
                )?);
            }
        }
        Ok(items)
    }

    fn doc_info(&self, rights: RightsBasis) -> DocInfo {
        let req = self.req;
        let locale = req.locale;
        let title: String = req.book.title.split_whitespace().collect::<Vec<_>>().join(" ");
        let mut custom = info_entries(&req.format, &req.profile);
        let format_name = CardFormat::presets()
            .into_iter()
            .find(|(_, preset)| *preset == req.format)
            .map_or("custom", |(name, _)| name);
        custom.push(("EBC.Format".to_owned(), format_name.to_owned()));
        if let Some(module) = self.qr_module_mm {
            custom.push(("EBC.QrModuleMm".to_owned(), format!("{module:.3}")));
        }
        // Lets a printed card be traced to the authorisation its cover was printed
        // under. The redemption code is never written to the metadata.
        custom.push(("EBC.RightsBasis".to_owned(), rights.as_str().to_owned()));
        DocInfo {
            title: format!("{title} — {}", t(locale, Msg::ProductName)),
            author: req.book.author_line(),
            subject: Some(t(locale, Msg::ProductTagline).to_owned()),
            keywords: Vec::new(),
            creator: Some(format!("ebc-card {}", env!("CARGO_PKG_VERSION"))),
            created_unix: req.created_unix,
            language: Some(locale.bcp47().to_owned()),
            custom,
        }
    }

    /// Converts the RGB images still in a PDF/X document (the spine's generated or
    /// adapted backgrounds; the cover is converted when it is placed) to the output
    /// intent's CMYK, with the same transform as the cover and the grounds. Without a
    /// transform they stay RGB (and `render_pdf` refuses the file unless the profile
    /// allows RGB).
    fn convert_images(&mut self) {
        let Some(link) = self.paint.link().cloned() else { return };
        for resource in &mut self.doc.images {
            let ImageData::Raw { pixels, width_px, height_px, format: PixelFormat::Rgb8 } =
                &resource.data
            else {
                continue;
            };
            resource.data = ImageData::Raw {
                pixels: link.convert_pixels(pixels).into(),
                width_px: *width_px,
                height_px: *height_px,
                format: PixelFormat::Cmyk8,
            };
            resource.icc_profile = None;
        }
    }

    /// Adds [`CardWarning::DarkAcrossCreases`] when dark or heavy ink lies on a crease of
    /// the outside (see `crease.rs`). After [`convert_images`](Self::convert_images), so
    /// that images are measured as they are printed.
    fn advise_on_creases(&mut self) {
        let spine = self.req.format.outside().spine;
        let profile = self.paint.link().and_then(ebc_color::SrgbToCmyk::profile);
        if let Some(advice) = crate::crease::advice(
            &self.doc,
            0,
            &[spine.x, spine.right()],
            self.geometry.trim.h,
            profile,
        ) {
            self.warnings.push(advice);
        }
    }

    /// Verifies the cover-integrity rule on the finished outside page: nothing that is
    /// painted after the cover may intersect its visible rectangle (the ground lies
    /// under it; the proofing overlay is exempt).
    fn check_cover_is_untouched(&self) -> Result<(), CardError> {
        let Some(cover) = self.cover_rect else { return Ok(()) };
        let leaves = Inspector::new(&self.doc).leaves(0);
        let Some(index) = leaves.iter().position(|leaf| {
            leaf.kind == LeafKind::Image
                && leaf.bounds.inflate(EPS_MM).contains_rect(&cover, EPS_MM)
        }) else {
            return Err(CardError::CoverOverlap("the cover image was not found".to_owned()));
        };
        let inner = cover.inflate(-EPS_MM);
        for leaf in &leaves[index + 1..] {
            if leaf.layer.as_deref() == Some(GUIDES_LAYER) {
                continue;
            }
            if leaf.bounds.intersect(&inner).is_some() {
                return Err(CardError::CoverOverlap(format!(
                    "{:?} at ({:.2}, {:.2}) {:.2} x {:.2} mm",
                    leaf.kind, leaf.bounds.x, leaf.bounds.y, leaf.bounds.w, leaf.bounds.h
                )));
            }
        }
        Ok(())
    }
}

/// How far beyond the bleed edge the cut and fold marks start, in millimetres: an alias
/// of [`ebc_layout::MARK_BLEED_GAP_MM`], where the rule now lives (the marks are drawn
/// by `ebc-layout` as they come, one object per mark).
pub const MARK_CLEARANCE_MM: f64 = ebc_layout::MARK_BLEED_GAP_MM;

/// A length for a printed note: at most one decimal, no trailing zero.
fn trim_mm(value: f64) -> String {
    let s = format!("{value:.1}");
    s.strip_suffix(".0").map(str::to_owned).unwrap_or(s)
}
