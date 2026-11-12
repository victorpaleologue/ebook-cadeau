//! Page 1, the outside of the card: back cover | spine | front cover.
//!
//! # Layout decisions
//!
//! * **One flat ground.** Back, spine and front share one flat colour that fills the
//!   bleed (`docs/research/03-print-requirements.md`, implications 14–15): a neutral when
//!   a cover is printed (paper white, or a dark grey when the cover's own edges are
//!   light, so that the cover always stands out), a colour picked from a fixed palette by
//!   a hash of the title for the typographic card. No colour change falls on a crease
//!   unless a spine style with its own colour is chosen.
//! * **The cover is never altered** (`docs/research/07-…`, implications 5–10): uniform
//!   scaling, no rotation, no crop — letterboxed in the panel's safe area, at least 3 mm
//!   inside the trim — unless the request says `trim_authorised`, in which case it fills
//!   the panel and its bleed with its spine-side edge exactly on the crease. Nothing is
//!   painted over it; the composer checks that on the finished page.
//! * **`promo_all_media`**: the cover at no more than 60 % of the panel's width and
//!   height, at the top of the panel, with title, author and publisher retyped *below*
//!   it (the panel is portrait: there is no room beside a cover of that size).
//! * **Spine**: `ebc-spine`'s flat-colour typeset spine on the shared ground by default.
//!   A spine with its own colour continues 2 mm past the front crease (never as far as
//!   the cover), so that a crease that drifts does not show a sliver of the other colour.
//! * **Back**: a short text, then a block at the foot with the product line, the address,
//!   the imprint and — when a cover is printed — its credit line. No barcode, no logo.

use ebc_color::{Edge, Rgb8, edge_palette, relative_luminance};
use ebc_i18n::{Msg, spine_direction_for_book, t};
use ebc_layout::{Face, Fit, place_image, place_image_aligned, safe_rect};
use ebc_scene::{Color, Item, Page, Path, Pixmap, Point, Rect, Stroke};
use ebc_spine::{
    BackgroundPaint, BackgroundStyle, SpineDesign, SpineNote, SpineOptions, SpineRequest,
    SpineSource, design_with,
};
use ebc_text::Font;

use crate::compose::{Composer, CoverPlan};
use crate::error::{CardError, CardWarning, TextField};
use crate::request::SpineChoice;
use crate::rights::{Placement, RightsBasis};
use crate::strings;
use crate::text::{
    Align, MIN_REVERSED_PT, MIN_TYPE_PT, Para, ParaSpec, fit_line, layout, note_clamped,
};

/// The smallest inset of a letterboxed cover from the trim and the crease, in
/// millimetres, whatever the profile's safe margin.
pub const MIN_COVER_INSET_MM: f64 = 3.0;
/// Largest share of the front panel's width and height a `promo_all_media` cover takes.
pub const REDUCED_COVER_SHARE: f64 = 0.6;
/// How far a spine with its own colour continues past the front crease, in millimetres.
const SPINE_OVERLAP_MM: f64 = 2.0;
/// Longest side of the cover copy handed to the spine designer when it samples pixels.
const SPINE_SAMPLE_SIDE_PX: u32 = 2_400;

/// Grounds of the typographic card. Deep colours that carry white type and two light
/// ones that carry black; all original, none tied to a publisher's collection.
const PALETTE: [Rgb8; 10] = [
    Rgb8::new(31, 58, 95),    // navy
    Rgb8::new(122, 31, 43),   // burgundy
    Rgb8::new(30, 89, 69),    // forest
    Rgb8::new(59, 47, 92),    // aubergine
    Rgb8::new(156, 74, 30),   // rust
    Rgb8::new(15, 93, 115),   // petrol
    Rgb8::new(47, 62, 70),    // slate
    Rgb8::new(94, 39, 80),    // plum
    Rgb8::new(201, 162, 39),  // ochre
    Rgb8::new(232, 220, 194), // cream
];
/// Neutral ground behind a cover whose edges are dark or mid-toned: paper white.
const NEUTRAL_LIGHT: Rgb8 = Rgb8::new(255, 255, 255);
/// Neutral ground behind a cover whose edges are light: a dark grey (black plate only).
const NEUTRAL_DARK: Rgb8 = Rgb8::new(51, 51, 51);
/// Mean luminance of a cover's edges above which the dark neutral is used.
const LIGHT_EDGE_LUMINANCE: f64 = 0.6;

/// The typographic card's ground for a title: FNV-1a over the lower-cased title.
pub(crate) fn palette_ground(title: &str) -> Rgb8 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in title.trim().to_lowercase().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    // Truncation is the point: an index into the palette.
    PALETTE[(hash % PALETTE.len() as u64) as usize]
}

/// The neutral ground for a cover: white, or dark grey when the cover's edges are light.
fn neutral_ground(cover: &Pixmap) -> Rgb8 {
    let mut sum = 0.0;
    let mut weight = 0.0;
    for edge in Edge::ALL {
        for swatch in edge_palette(cover, edge, 0.04, 3) {
            sum += relative_luminance(swatch.color) * f64::from(swatch.share);
            weight += f64::from(swatch.share);
        }
    }
    if weight > 0.0 && sum / weight > LIGHT_EDGE_LUMINANCE { NEUTRAL_DARK } else { NEUTRAL_LIGHT }
}

impl Composer<'_> {
    pub(crate) fn outside_page(&mut self, cover: Option<CoverPlan>) -> Result<Page, CardError> {
        let format = self.req.format;
        let profile = self.req.profile.clone();
        let bleed = self.geometry.bleed_mm;
        let panels = format.outside();
        let mut page = self.new_page(Face::Outside)?;

        let analysis = cover.as_ref().map(|plan| plan.image.analysis());
        let ground = self.req.ground.unwrap_or_else(|| match &analysis {
            Some(pixels) => neutral_ground(pixels),
            None => palette_ground(&self.req.book.title),
        });
        let full_bleed = cover
            .as_ref()
            .is_some_and(|plan| plan.placement == Placement::Full && self.req.trim_authorised);

        // ---- Spine (designed first: it may decide the colour of the back) --------------
        let sample = match (&self.req.spine, &cover) {
            (SpineChoice::EdgeContinuation, Some(plan)) => {
                Some(plan.image.scaled_pixmap(SPINE_SAMPLE_SIDE_PX))
            }
            _ => None,
        };
        let spine = self.design_spine(ground, analysis.as_ref(), sample.as_ref())?;
        let own_spine_colour = !matches!(self.req.spine, SpineChoice::Typeset) && cover.is_some();
        let back_ground = if own_spine_colour { spine.background } else { ground };

        // ---- Grounds ---------------------------------------------------------------------
        page.items.push(Item::filled_rect(self.geometry.bleed, self.paint.flat(ground)));
        if back_ground != ground {
            let overlap = if full_bleed { 0.0 } else { SPINE_OVERLAP_MM };
            let right = panels.spine.right() + overlap;
            let area = Rect::new(
                self.geometry.bleed.x,
                self.geometry.bleed.y,
                right - self.geometry.bleed.x,
                self.geometry.bleed.h,
            );
            page.items.push(Item::filled_rect(area, self.paint.flat(back_ground)));
        }
        for item in spine.items {
            page.items.push(self.paint.repaint(item));
        }

        // ---- Front -------------------------------------------------------------------
        let front_safe = safe_rect(panels.right, &profile);
        let credit = match cover {
            Some(plan) => {
                let rights = plan.rights;
                let px = plan.image.size_px();
                let inset = profile.safe_margin.max(MIN_COVER_INSET_MM);
                let (rect, visible, clip) = match plan.placement {
                    Placement::Full if full_bleed => {
                        // Authorised trim: the cover fills the panel and its bleed on the
                        // three outer edges; its spine-side edge lies on the crease.
                        let clip = panels.with_bleed(bleed).right;
                        let rect = place_image_aligned(px, clip, Fit::Cover, 0.0, 0.5);
                        self.warn_heavy_crop(rect, clip, bleed);
                        (rect, clip, Some(clip))
                    }
                    Placement::Full => {
                        let rect = place_image(px, panels.right.inflate(-inset), Fit::Contain);
                        (rect, rect, None)
                    }
                    Placement::ReducedSize => {
                        let w = panels.right.w * REDUCED_COVER_SHARE;
                        let h = panels.right.h * REDUCED_COVER_SHARE;
                        let area = Rect::new(
                            panels.right.center().x - w / 2.0,
                            panels.right.y + inset,
                            w,
                            h.min(panels.right.h - 2.0 * inset),
                        );
                        let rect = place_image_aligned(px, area, Fit::Contain, 0.5, 0.0);
                        (rect, rect, None)
                    }
                };
                let data = plan.image.into_image_data(rect.size(), &profile, &mut self.warnings);
                let image = self.doc.add_image(data, None);
                let item = Item::Image { image, rect };
                page.items.push(match clip {
                    Some(clip) => Item::clipped(clip, vec![item]),
                    None => item,
                });
                self.cover_rect = Some(visible);
                if plan.placement == Placement::ReducedSize {
                    let gap = 6.0;
                    let below = Rect::new(
                        front_safe.x,
                        visible.bottom() + gap,
                        front_safe.w,
                        (front_safe.bottom() - visible.bottom() - gap).max(0.0),
                    );
                    let items = self.retyped_title(below, ground)?;
                    page.items.extend(items);
                }
                Some(self.credit_line(rights))
            }
            None => {
                let items = self.typographic_front(front_safe, ground)?;
                page.items.extend(items);
                None
            }
        };

        // ---- Back --------------------------------------------------------------------
        let back_safe = safe_rect(panels.left, &profile);
        let items = self.back_panel(back_safe, back_ground, credit)?;
        page.items.extend(items);

        self.finish_page(&mut page, Face::Outside)?;
        Ok(page)
    }

    fn design_spine(
        &mut self,
        ground: Rgb8,
        analysis: Option<&Pixmap>,
        sample: Option<&Pixmap>,
    ) -> Result<SpineDesign, CardError> {
        let format = self.req.format;
        let bleed = self.geometry.bleed_mm;
        let spine = format.spine();
        let rect = Rect::new(spine.x, spine.y - bleed, spine.w, spine.h + 2.0 * bleed);
        let (background, source, cover) = match (&self.req.spine, analysis) {
            (SpineChoice::CoverColour, Some(pixels)) => {
                (BackgroundStyle::Auto, SpineSource::Synthesize, Some(pixels))
            }
            (SpineChoice::EdgeContinuation, Some(pixels)) => (
                BackgroundStyle::EdgeContinuation,
                SpineSource::Synthesize,
                Some(sample.unwrap_or(pixels)),
            ),
            (SpineChoice::Provided(image), Some(pixels)) => {
                (BackgroundStyle::Auto, SpineSource::Provided(image), Some(pixels))
            }
            (SpineChoice::FullWrap { image, spine_px }, Some(pixels)) => (
                BackgroundStyle::Auto,
                SpineSource::FullWrap { image, spine_px: *spine_px },
                Some(pixels),
            ),
            // The default, and every style that needs a cover when there is none.
            _ => (BackgroundStyle::Solid(ground), SpineSource::Synthesize, None),
        };
        let flat_default = matches!(background, BackgroundStyle::Solid(_));
        let options = SpineOptions {
            background,
            // sRGB here; the fills are re-expressed for the profile by `Paint::repaint`,
            // exactly like the card's own ground, so that the two never differ.
            background_paint: BackgroundPaint::Rgb,
            fallback_font: self.fonts.title_alt.clone(),
            ..SpineOptions::default()
        };
        let request = SpineRequest {
            book: &self.req.book,
            cover,
            source,
            rect,
            bleed,
            title_font: &self.fonts.title,
            author_font: &self.fonts.body,
            direction: spine_direction_for_book(&self.req.book, self.req.locale),
            safe_margin: self.req.profile.spine_safe_margin,
        };
        let design = design_with(&request, &options, &mut self.doc)?;
        if design.requires_authorisation
            && !(self.req.adaptation_authorised
                && crate::rights::allows_adaptation(self.req.cover.rights()))
        {
            // Unreachable through `compose` (the request was gated before any work was
            // done); kept so that no spine with artwork can ever leave without it.
            return Err(CardError::AdaptationNotAuthorised {
                style: self.req.spine.name(),
                basis: self.req.cover.rights(),
                authorised: self.req.adaptation_authorised,
            });
        }
        for note in &design.details {
            // With the flat default the ground was chosen here: "no cover" is no news.
            if flat_default && matches!(note, SpineNote::NoCover) {
                continue;
            }
            self.warnings.push(CardWarning::Spine(note.clone()));
        }
        Ok(design)
    }

    fn warn_heavy_crop(&mut self, image: Rect, clip: Rect, bleed: f64) {
        // What the finished panel shows of each dimension, the trimmed bleed excluded.
        let shown_w = (clip.w - bleed).max(0.0);
        let shown_h = (clip.h - 2.0 * bleed).max(0.0);
        for (dimension, shown, full) in [("width", shown_w, image.w), ("height", shown_h, image.h)]
        {
            if full > 0.0 {
                let cropped = (1.0 - shown / full) * 100.0;
                if cropped > 10.0 {
                    self.warnings.push(CardWarning::HeavyCrop { percent: cropped, dimension });
                }
            }
        }
    }

    /// The credit line for a printed cover: the caller's credit, else the publisher,
    /// else a generic line (with a warning).
    fn credit_line(&mut self, rights: RightsBasis) -> String {
        let locale = self.req.locale;
        let supplied = self.req.cover_credit.as_deref().map(str::trim).filter(|c| !c.is_empty());
        let publisher = self.req.book.publisher.as_deref().map(str::trim).filter(|p| !p.is_empty());
        match (supplied, rights, publisher) {
            (Some(credit), _, _) => strings::cover_credit(locale, credit),
            (None, RightsBasis::PublicDomain, _) => strings::cover_credit_public_domain(locale),
            (None, _, Some(publisher)) => strings::cover_credit_publisher(locale, publisher),
            (None, _, None) => {
                self.warnings.push(CardWarning::CoverCreditMissing);
                strings::cover_credit_unknown(locale)
            }
        }
    }

    /// One centred paragraph in `color`, top at `top`; returns its height.
    #[allow(clippy::too_many_arguments)]
    fn centred(
        &mut self,
        items: &mut Vec<Item>,
        font: &Font,
        para: &Para,
        color: Color,
        area: Rect,
        top: f64,
    ) -> Result<f64, CardError> {
        items.extend(self.setter.para(
            &mut self.doc,
            font,
            para,
            color,
            area.x,
            top,
            area.w,
            Align::Center,
        )?);
        Ok(para.height())
    }

    /// The typographic cover: author, a thin rule, the title, the subtitle and, at the
    /// foot, the publisher — all centred, as plain text in the card's own fonts.
    fn typographic_front(&mut self, area: Rect, ground: Rgb8) -> Result<Vec<Item>, CardError> {
        let (color, reversed) = self.paint.text_on(ground);
        let floor = if reversed { MIN_REVERSED_PT } else { MIN_TYPE_PT };
        // Type scales with the panel: 1.0 on the 100 mm wide safe area of the poche.
        let scale = (area.w / 100.0).clamp(0.55, 1.4);
        let fonts = self.fonts.clone();
        let book = self.req.book.clone();
        let mut items = Vec::new();

        // Publisher at the foot.
        let mut foot = area.bottom();
        if let Some(publisher) = book.publisher.as_deref() {
            let text = self.clean_line(&fonts.body, publisher, TextField::Publisher);
            if !text.is_empty() {
                let (text, size, cut) =
                    fit_line(&fonts.body, &text, area.w, 10.5 * scale.max(0.8), floor.max(8.0));
                if cut {
                    note_clamped(TextField::Publisher, &mut self.warnings);
                }
                let baseline = area.bottom() - 1.0;
                items.extend(self.setter.aligned(
                    &mut self.doc,
                    &fonts.body,
                    &text,
                    size,
                    color,
                    area.x,
                    area.w,
                    baseline,
                    Align::Center,
                )?);
                foot = baseline - ebc_scene::pt_to_mm(size) - 4.0;
            }
        }

        let mut y = area.y + 0.07 * area.h;
        // Author.
        if let Some(author) = book.author_line() {
            let text = self.clean_line(&fonts.body_bold, &author, TextField::Author);
            let spec = ParaSpec::new(13.0 * scale, floor.max(9.0 * scale), 1.25, 2);
            let para = layout(&fonts.body_bold, &text, area.w, 0.14 * area.h, &spec);
            if para.clamped {
                note_clamped(TextField::Author, &mut self.warnings);
            }
            if !para.is_empty() {
                y += self.centred(&mut items, &fonts.body_bold, &para, color, area, y)?;
                y += 3.0 * scale;
                let half = 9.0 * scale;
                let cx = area.center().x;
                items.push(Item::stroke(
                    Path::line(Point::new(cx - half, y), Point::new(cx + half, y)),
                    Stroke::solid(color, 0.75),
                ));
            }
        }
        y = y.max(area.y + 0.2 * area.h) + 6.0 * scale;

        // Title.
        let title = self.clean_line(&fonts.title, &book.title, TextField::Title);
        let room = (foot - y).max(0.0);
        let spec = ParaSpec::new(32.0 * scale, floor.max(12.0 * scale), 1.12, 8);
        let para = layout(&fonts.title, &title, area.w, (0.62 * room).max(6.0), &spec);
        if para.clamped {
            note_clamped(TextField::Title, &mut self.warnings);
        }
        y += self.centred(&mut items, &fonts.title, &para, color, area, y)?;

        // Subtitle.
        if let Some(subtitle) = book.subtitle.as_deref() {
            let text = self.clean_line(&fonts.body, subtitle, TextField::Subtitle);
            y += 4.0 * scale;
            let room = foot - y;
            let spec = ParaSpec::new(12.5 * scale, floor.max(8.5), 1.3, 4);
            if !text.is_empty() && room > ebc_scene::pt_to_mm(spec.min_pt * spec.leading) {
                let inset = Rect::new(area.x + 0.05 * area.w, area.y, 0.9 * area.w, area.h);
                let para = layout(&fonts.body, &text, inset.w, room, &spec);
                if para.clamped {
                    note_clamped(TextField::Subtitle, &mut self.warnings);
                }
                self.centred(&mut items, &fonts.body, &para, color, inset, y)?;
            }
        }
        Ok(items)
    }

    /// Title, author and publisher retyped below a reduced-size cover.
    fn retyped_title(&mut self, area: Rect, ground: Rgb8) -> Result<Vec<Item>, CardError> {
        let (color, reversed) = self.paint.text_on(ground);
        let floor = if reversed { MIN_REVERSED_PT } else { MIN_TYPE_PT };
        let scale = (area.w / 100.0).clamp(0.55, 1.4);
        let fonts = self.fonts.clone();
        let book = self.req.book.clone();
        let mut items = Vec::new();
        if area.h < 6.0 {
            note_clamped(TextField::Title, &mut self.warnings);
            return Ok(items);
        }
        let mut y = area.y;
        let title = self.clean_line(&fonts.title, &book.title, TextField::Title);
        let spec = ParaSpec::new(20.0 * scale, floor.max(10.0 * scale), 1.15, 4);
        let para = layout(&fonts.title, &title, area.w, 0.55 * area.h, &spec);
        if para.clamped {
            note_clamped(TextField::Title, &mut self.warnings);
        }
        y += self.centred(&mut items, &fonts.title, &para, color, area, y)?;
        y += 2.0;

        let mut small = |this: &mut Self,
                         text: Option<String>,
                         font: &Font,
                         size: f64,
                         field: TextField,
                         y: &mut f64|
         -> Result<(), CardError> {
            let Some(text) = text else { return Ok(()) };
            let text = this.clean_line(font, &text, field);
            let spec = ParaSpec::new(size, floor.max(8.0), 1.25, 2);
            let room = area.bottom() - *y;
            if text.is_empty() || room < ebc_scene::pt_to_mm(spec.min_pt * spec.leading) {
                return Ok(());
            }
            let para = layout(font, &text, area.w, room, &spec);
            if para.clamped {
                note_clamped(field, &mut this.warnings);
            }
            *y += this.centred(&mut items, font, &para, color, area, *y)? + 1.0;
            Ok(())
        };
        small(self, book.author_line(), &fonts.body_bold, 11.0 * scale, TextField::Author, &mut y)?;
        small(
            self,
            book.publisher.clone(),
            &fonts.body,
            9.0 * scale,
            TextField::Publisher,
            &mut y,
        )?;
        Ok(items)
    }

    /// The back panel: a short text at the top; at the foot the product line, the
    /// address, the imprint and the cover credit.
    fn back_panel(
        &mut self,
        area: Rect,
        ground: Rgb8,
        credit: Option<String>,
    ) -> Result<Vec<Item>, CardError> {
        let locale = self.req.locale;
        let (color, reversed) = self.paint.text_on(ground);
        let small = if reversed { MIN_REVERSED_PT } else { 7.0 };
        let credit_pt = if reversed { MIN_REVERSED_PT } else { 6.5 };
        let fonts = self.fonts.clone();
        let mut items = Vec::new();

        // ---- Foot block, measured first ---------------------------------------------
        let display_url = self.display_url();
        let imprint = match self.req.store_name.as_deref().map(str::trim).filter(|s| !s.is_empty())
        {
            Some(store) => {
                let store = self.clean_line(&fonts.body, store, TextField::StoreName);
                format!("{store} · {}", t(locale, Msg::PrintedWith))
            }
            None => t(locale, Msg::PrintedWith).to_owned(),
        };
        let credit = match credit {
            Some(text) => self.clean_line(&fonts.body, &text, TextField::Credit),
            None => String::new(),
        };
        let lines: [(&Font, String, ParaSpec, TextField); 5] = [
            (
                &fonts.body_bold,
                t(locale, Msg::ProductName).to_owned(),
                ParaSpec::fixed(9.5, 1.3, 1),
                TextField::Imprint,
            ),
            (
                &fonts.body,
                t(locale, Msg::ProductTagline).to_owned(),
                ParaSpec::fixed(small.max(7.5), 1.3, 2),
                TextField::Imprint,
            ),
            (
                &fonts.body_bold,
                display_url,
                ParaSpec::new(9.0, small, 1.5, 1),
                TextField::DisplayUrl,
            ),
            (&fonts.body, imprint, ParaSpec::fixed(small, 1.3, 2), TextField::Imprint),
            (&fonts.body, credit, ParaSpec::fixed(credit_pt, 1.3, 2), TextField::Credit),
        ];
        let mut block: Vec<(&Font, Para)> = Vec::new();
        for (font, text, spec, field) in lines {
            let para = layout(font, &text, area.w, f64::INFINITY, &spec);
            if para.clamped {
                note_clamped(field, &mut self.warnings);
            }
            if !para.is_empty() {
                block.push((font, para));
            }
        }
        let gap = 1.2;
        let height: f64 = block.iter().map(|(_, p)| p.height()).sum::<f64>()
            + gap * block.len().saturating_sub(1) as f64;
        let block_top = (area.bottom() - height).max(area.y);
        let mut y = block_top;
        for (font, para) in &block {
            items.extend(self.setter.para(
                &mut self.doc,
                font,
                para,
                color,
                area.x,
                y,
                area.w,
                Align::Left,
            )?);
            y += para.height() + gap;
        }
        // A short rule separates the block from the text above.
        let rule_y = block_top - 3.0;
        if rule_y > area.y + 6.0 {
            items.push(Item::stroke(
                Path::line(Point::new(area.x, rule_y), Point::new(area.x + 12.0, rule_y)),
                Stroke::solid(color, 0.75),
            ));
        }

        // ---- Text -------------------------------------------------------------------
        let text = match self.req.back_text.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            Some(text) => self.clean_block(&fonts.body, text, TextField::BackText),
            None => t(locale, Msg::CardBackDefault).to_owned(),
        };
        let room = rule_y - 5.0 - area.y;
        let spec = ParaSpec::new(11.0, 9.0, 1.42, 60);
        if room > ebc_scene::pt_to_mm(spec.min_pt * spec.leading) {
            let para = layout(&fonts.body, &text, area.w, room, &spec);
            if para.clamped {
                note_clamped(TextField::BackText, &mut self.warnings);
            }
            items.extend(self.setter.para(
                &mut self.doc,
                &fonts.body,
                &para,
                color,
                area.x,
                area.y,
                area.w,
                Align::Left,
            )?);
        } else {
            note_clamped(TextField::BackText, &mut self.warnings);
        }
        Ok(items)
    }

    /// The address printed in clear: cleaned, in lower case. When the request gives none,
    /// the host of the claim URL is printed.
    pub(crate) fn display_url(&mut self) -> String {
        let font = self.fonts.body_bold.clone();
        let url = self.req.display_url.to_lowercase();
        let url = self.clean_line(&font, &url, TextField::DisplayUrl);
        if !url.is_empty() {
            return url;
        }
        let claim = self.req.claim_url.trim().to_lowercase();
        let rest = claim.split_once("://").map_or(claim.as_str(), |(_, rest)| rest);
        let host = rest.split(['/', '?', '#', ':']).next().unwrap_or_default().to_owned();
        self.clean_line(&font, &host, TextField::DisplayUrl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ebc_color::contrast_ratio;

    #[test]
    fn palette_is_deterministic_and_legible() {
        assert_eq!(palette_ground("Candide"), palette_ground("  candide "));
        let distinct: std::collections::HashSet<_> =
            ["Candide", "Germinal", "Le Horla", "Bel-Ami", "Nana", "Salammbô", "La Peste"]
                .into_iter()
                .map(|title| palette_ground(title).to_hex())
                .collect();
        assert!(distinct.len() >= 3, "{distinct:?}");
        for ground in PALETTE.into_iter().chain([NEUTRAL_LIGHT, NEUTRAL_DARK]) {
            let text = ebc_color::black_or_white(ground);
            assert!(contrast_ratio(text, ground) >= 4.5, "{ground}");
        }
    }

    #[test]
    fn the_neutral_ground_contrasts_with_the_cover_edges() {
        let dark = Pixmap::filled(60, 90, [20, 30, 60, 255]);
        let light = Pixmap::filled(60, 90, [250, 248, 240, 255]);
        assert_eq!(neutral_ground(&dark), NEUTRAL_LIGHT);
        assert_eq!(neutral_ground(&light), NEUTRAL_DARK);
    }
}
