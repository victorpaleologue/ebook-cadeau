//! Page 2, the inside of the card: inside-front | inside of the spine | inside-back.
//!
//! The sheet is turned about the axis parallel to its creases, so the **left** panel of
//! this page is the back of the front cover — the first thing seen when the card is
//! opened. It carries the gift itself (headline, "for …", the message or ruled space for
//! one, "from …"). The **right** panel carries what to do: the three steps, the QR code
//! with the address and the code beside it, the line for people without a phone, and the
//! legal lines at the foot — the content of §12 of
//! `docs/research/notes/ereader-ux.ux.md`, through the strings of `ebc-i18n`.
//!
//! The ground is paper white on every profile and the inside of the spine carries
//! nothing (`docs/research/03-print-requirements.md`, implication 17), so no ink lies on
//! the creases. All text is real text in black only; the QR code is one merged path in
//! black on a white box that includes the four-module quiet zone.

use ebc_i18n::{Msg, format_unix_date, t, tf};
use ebc_layout::{Face, safe_rect};
use ebc_model::claim_url;
use ebc_qr::{
    EcLevel, MIN_PRINT_MODULE_MM, QUIET_ZONE_MODULES, QrCode, RECOMMENDED_PRINT_MODULE_MM,
};
use ebc_scene::{Item, Page, Path, Point, Rect, Stroke, pt_to_mm};

use crate::compose::Composer;
use crate::error::{CardError, CardWarning, TextField};
use crate::text::{
    Align, MIN_TYPE_PT, Para, ParaSpec, first_baseline, fit_line, layout, note_clamped, width_of,
};

/// Preferred side of the white box that holds the QR code and its quiet zone, in
/// millimetres (`docs/research/03-print-requirements.md`, implication 31).
pub const QR_BOX_MM: f64 = 33.0;
/// Largest side of that box, in millimetres.
pub const QR_BOX_MAX_MM: f64 = 40.0;
/// Smallest side of the symbol itself (without its quiet zone), in millimetres.
pub const QR_MIN_SYMBOL_MM: f64 = 20.0;
/// Size of the redemption code, in points.
pub const CODE_PT: f64 = 16.0;
/// Size the three steps are designed for, in points.
pub const STEPS_PT: f64 = 12.0;
/// Step sizes tried, largest first, until the panel's content fits.
const STEP_SIZES_PT: [f64; 7] = [12.0, 11.0, 10.0, 9.5, 9.0, 8.5, 8.0];

/// The text the QR code carries for a claim URL.
///
/// A URL of the form `scheme://host[:port]/k/<code>` is upper-cased from end to end
/// (that is [`ebc_model::claim_url_qr`]: the whole payload then fits the compact
/// alphanumeric mode). Any other URL is encoded as given, with only its scheme and host
/// upper-cased by the encoder, because upper-casing a path the server may treat as
/// case-sensitive would break the link; the flag says which of the two happened.
///
/// # Errors
/// [`CardError::InvalidClaimUrl`] for an empty URL, one without an `http(s)` scheme, or
/// one with white space or control characters.
pub fn qr_payload(url: &str, code: &ebc_model::GiftCode) -> Result<(String, bool), CardError> {
    let url = url.trim();
    if url.is_empty() {
        return Err(CardError::InvalidClaimUrl("it is empty"));
    }
    if url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(CardError::InvalidClaimUrl("it contains white space or control characters"));
    }
    let lower = url.to_ascii_lowercase();
    let Some(rest) = lower.strip_prefix("https://").or_else(|| lower.strip_prefix("http://"))
    else {
        return Err(CardError::InvalidClaimUrl("it does not start with http:// or https://"));
    };
    if rest.is_empty() || rest.starts_with('/') {
        return Err(CardError::InvalidClaimUrl("it has no host"));
    }
    // Our own scheme: nothing but a host (and port) before "/k/<code>".
    let base_len = url.len() - rest.len() + rest.find('/').unwrap_or(rest.len());
    let own = url.is_ascii()
        && url.get(..base_len).is_some_and(|base| {
            claim_url(base, code).eq_ignore_ascii_case(url)
        });
    if own { Ok((url.to_ascii_uppercase(), true)) } else { Ok((url.to_owned(), false)) }
}

/// Everything the how-to panel sets, measured at one type scale.
struct HowTo {
    heading: Para,
    steps: Vec<Para>,
    step_pt: f64,
    indent: f64,
    no_phone: Para,
    legal: Para,
    /// Height of everything but the legal lines, gaps included.
    fixed: f64,
    gap: f64,
}

impl Composer<'_> {
    pub(crate) fn inside_page(&mut self) -> Result<Page, CardError> {
        let format = self.req.format;
        let profile = self.req.profile.clone();
        let mut page = self.new_page(Face::Inside)?;

        let gift = safe_rect(format.inside_front(), &profile);
        let items = self.gift_panel(gift)?;
        page.items.extend(items);

        let how_to = safe_rect(format.inside_back(), &profile);
        let items = self.how_to_panel(how_to)?;
        page.items.extend(items);

        self.finish_page(&mut page, Face::Inside)?;
        Ok(page)
    }

    /// Inside-front: headline, "for …", the message (or ruled space), "from …", and the
    /// book's title and author at the foot.
    fn gift_panel(&mut self, area: Rect) -> Result<Vec<Item>, CardError> {
        let locale = self.req.locale;
        let ink = self.paint.ink();
        let fonts = self.fonts.clone();
        let scale = (area.w / 100.0).clamp(0.6, 1.3);
        let mut items = Vec::new();
        let mut y = area.y;

        // Headline and a short rule.
        let headline = t(locale, Msg::CardHeadline);
        let spec = ParaSpec::new(21.0 * scale, 12.0, 1.15, 3);
        let para = layout(&fonts.title, headline, area.w, 0.25 * area.h, &spec);
        items.extend(self.setter.para(
            &mut self.doc,
            &fonts.title,
            &para,
            ink,
            area.x,
            y,
            area.w,
            Align::Center,
        )?);
        y += para.height() + 3.0 * scale;
        let cx = area.center().x;
        items.push(Item::stroke(
            Path::line(Point::new(cx - 9.0 * scale, y), Point::new(cx + 9.0 * scale, y)),
            Stroke::solid(ink, 0.75),
        ));
        y += 7.0 * scale;

        // Foot: the book.
        let mut foot = area.bottom();
        let book = self.req.book.clone();
        let author = book.author_line().map(|a| self.clean_line(&fonts.body, &a, TextField::Author));
        if let Some(author) = author.filter(|a| !a.is_empty()) {
            let (text, size, cut) = fit_line(&fonts.body, &author, area.w, 8.5, 7.0);
            if cut {
                note_clamped(TextField::Author, &mut self.warnings);
            }
            items.extend(self.setter.aligned(
                &mut self.doc,
                &fonts.body,
                &text,
                size,
                ink,
                area.x,
                area.w,
                foot - 0.8,
                Align::Left,
            )?);
            foot -= pt_to_mm(size * 1.35);
        }
        let title = self.clean_line(&fonts.title, &book.title, TextField::Title);
        let spec = ParaSpec::fixed(10.5, 1.25, 2);
        let para = layout(&fonts.title, &title, area.w, f64::INFINITY, &spec);
        foot -= para.height();
        items.extend(self.setter.para(
            &mut self.doc,
            &fonts.title,
            &para,
            ink,
            area.x,
            foot,
            area.w,
            Align::Left,
        )?);
        foot -= 2.5;
        items.push(Item::stroke(
            Path::line(Point::new(area.x, foot), Point::new(area.x + 12.0, foot)),
            Stroke::solid(ink, 0.75),
        ));
        foot -= 5.0;

        // "For …".
        let recipient = self.req.recipient.as_deref().map(str::trim).filter(|s| !s.is_empty());
        if let Some(name) = recipient {
            let name = self.clean_line(&fonts.body_bold, name, TextField::Recipient);
            if !name.is_empty() {
                let text = tf(locale, Msg::CardTo, &[("name", &name)]);
                let (text, size, cut) = fit_line(&fonts.body_bold, &text, area.w, 13.0 * scale, 9.0);
                if cut {
                    note_clamped(TextField::Recipient, &mut self.warnings);
                }
                let line = pt_to_mm(size * 1.3);
                items.extend(self.setter.aligned(
                    &mut self.doc,
                    &fonts.body_bold,
                    &text,
                    size,
                    ink,
                    area.x,
                    area.w,
                    y + first_baseline(&fonts.body_bold, size, line),
                    Align::Left,
                )?);
                y += line + 4.0 * scale;
            }
        }

        // "From …", right-aligned above the foot.
        let giver = self.req.giver.as_deref().map(str::trim).filter(|s| !s.is_empty());
        if let Some(name) = giver {
            let name = self.clean_line(&fonts.body_bold, name, TextField::Giver);
            if !name.is_empty() {
                let text = tf(locale, Msg::CardFrom, &[("name", &name)]);
                let (text, size, cut) = fit_line(&fonts.body_bold, &text, area.w, 12.0 * scale, 9.0);
                if cut {
                    note_clamped(TextField::Giver, &mut self.warnings);
                }
                let line = pt_to_mm(size * 1.3);
                foot -= line;
                items.extend(self.setter.aligned(
                    &mut self.doc,
                    &fonts.body_bold,
                    &text,
                    size,
                    ink,
                    area.x,
                    area.w,
                    foot + first_baseline(&fonts.body_bold, size, line),
                    Align::Right,
                )?);
                foot -= 4.0 * scale;
            }
        }

        // The message, or ruled space for one.
        let room = foot - y;
        let message = self.req.message.as_deref().map(str::trim).filter(|s| !s.is_empty());
        match message {
            Some(message) if room > 4.0 => {
                let text = self.clean_block(&fonts.body, message, TextField::Message);
                let spec = ParaSpec::new(12.0 * scale, 8.5, 1.45, 80);
                let para = layout(&fonts.body, &text, area.w, room, &spec);
                if para.clamped {
                    note_clamped(TextField::Message, &mut self.warnings);
                }
                items.extend(self.setter.para(
                    &mut self.doc,
                    &fonts.body,
                    &para,
                    ink,
                    area.x,
                    y,
                    area.w,
                    Align::Left,
                )?);
            }
            Some(_) => note_clamped(TextField::Message, &mut self.warnings),
            None => {
                // Lines to write on by hand, 9 mm apart, in a light black-only grey.
                let mut rules = Path::new();
                let mut line_y = y + 9.0;
                while line_y <= foot - 1.0 {
                    rules.move_to(area.x, line_y).line_to(area.right(), line_y);
                    line_y += 9.0;
                }
                if !rules.is_empty() {
                    items.push(Item::stroke(rules, Stroke::solid(self.paint.grey(0.6), 0.5)));
                }
            }
        }
        Ok(items)
    }

    /// The legal and information lines: what the card gives, single use, the time limit,
    /// help, then the caller's own lines.
    fn legal_text(&mut self) -> String {
        let locale = self.req.locale;
        let font = self.fonts.body.clone();
        let book = self.req.book.clone();
        let title = self.clean_line(&font, &book.title, TextField::Title);
        let title = if title.chars().count() > 90 {
            let cut: String = title.chars().take(89).collect();
            format!("{}…", cut.trim_end())
        } else {
            title
        };
        let author = book.author_line().map(|a| self.clean_line(&font, &a, TextField::Author));
        let mut text = match author.filter(|a| !a.is_empty()) {
            Some(author) => {
                let author: String = author.chars().take(80).collect();
                tf(locale, Msg::CardLegalLine, &[("title", &title), ("author", &author)])
            }
            None => tf(locale, Msg::CardLegalLineNoAuthor, &[("title", &title)]),
        };
        text.push(' ');
        text.push_str(t(locale, Msg::CardSingleUse));
        text.push(' ');
        match self.req.expires_unix {
            Some(unix) => {
                let date = format_unix_date(locale, unix);
                text.push_str(&tf(locale, Msg::CardValidUntil, &[("date", &date)]));
            }
            None => text.push_str(t(locale, Msg::CardNoExpiry)),
        }
        if let Some(help) = self.req.help_url.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            let help = self.clean_line(&font, help, TextField::Legal);
            text.push(' ');
            text.push_str(&tf(locale, Msg::CardHelp, &[("url", &help)]));
        }
        let extra = self.req.legal_lines.clone();
        for line in extra.iter().take(12) {
            let line = self.clean_line(&font, line, TextField::Legal);
            if !line.is_empty() {
                text.push('\n');
                text.push_str(&line);
            }
        }
        text
    }

    /// The texts of the how-to panel, laid out for one step size.
    #[allow(clippy::too_many_arguments)]
    fn measure_how_to(
        &self,
        area: Rect,
        step_pt: f64,
        heading: &str,
        steps: &[String],
        no_phone: &str,
        legal: &str,
        qr_block: f64,
    ) -> HowTo {
        let fonts = self.fonts;
        let s = step_pt / STEPS_PT;
        let indent = 6.5 * s;
        let gap = 4.0 * s;
        let heading =
            layout(&fonts.body_bold, heading, area.w, f64::INFINITY, &ParaSpec::fixed(11.0 * s.max(0.8), 1.3, 2));
        let steps: Vec<Para> = steps
            .iter()
            .map(|step| {
                layout(&fonts.body, step, area.w - indent, f64::INFINITY, &ParaSpec::fixed(step_pt, 1.28, 8))
            })
            .collect();
        let no_phone_pt = (9.0 * s).max(7.0);
        let no_phone =
            layout(&fonts.body, no_phone, area.w, f64::INFINITY, &ParaSpec::fixed(no_phone_pt, 1.32, 8));
        let legal = layout(&fonts.body, legal, area.w, f64::INFINITY, &ParaSpec::fixed(7.0, 1.3, 30));
        let fixed = heading.height()
            + 2.0 * s
            + steps.iter().map(Para::height).sum::<f64>()
            + 1.5 * s * steps.len().saturating_sub(1) as f64
            + gap * 1.25
            + qr_block
            + gap
            + no_phone.height()
            + gap;
        HowTo { heading, steps, step_pt, indent, no_phone, legal, fixed, gap }
    }

    /// Inside-back: the three steps, the QR code with the address and the code, the line
    /// for people without a phone and the legal lines.
    fn how_to_panel(&mut self, area: Rect) -> Result<Vec<Item>, CardError> {
        let locale = self.req.locale;
        let ink = self.paint.ink();
        let fonts = self.fonts.clone();
        let mut items = Vec::new();

        // ---- The QR code ---------------------------------------------------------------
        let (payload, upper_cased) = qr_payload(&self.req.claim_url, &self.req.code)?;
        let qr = if upper_cased {
            QrCode::encode(&payload, EcLevel::M)?
        } else {
            self.warnings.push(CardWarning::QrPayloadNotUpperCased);
            QrCode::encode_url(&payload, EcLevel::M)?
        };
        if qr.version() > 4 {
            self.warnings.push(CardWarning::QrVersionHigh { version: qr.version() });
        }
        let modules = f64::from(qr.size());
        let with_quiet = modules + 2.0 * f64::from(QUIET_ZONE_MODULES);
        // At least 20 mm of symbol, and modules of the recommended size when the panel
        // allows it; never more than the panel is wide.
        let min_box = QR_MIN_SYMBOL_MM * with_quiet / modules;
        let comfortable = RECOMMENDED_PRINT_MODULE_MM * with_quiet;
        let qr_box = QR_BOX_MM.max(min_box).max(comfortable.min(QR_BOX_MAX_MM)).min(area.w);
        if qr_box + 1e-9 < min_box {
            return Err(CardError::FormatTooSmall {
                panel: "inside-back",
                needed_mm: min_box,
                available_mm: area.w,
            });
        }
        let module_mm = qr_box / with_quiet;
        if module_mm < MIN_PRINT_MODULE_MM {
            self.warnings.push(CardWarning::QrModuleBelowMinimum {
                module_mm,
                minimum_mm: MIN_PRINT_MODULE_MM,
            });
        } else if module_mm < RECOMMENDED_PRINT_MODULE_MM {
            self.warnings.push(CardWarning::QrModuleSmall {
                module_mm,
                recommended_mm: RECOMMENDED_PRINT_MODULE_MM,
            });
        }
        self.qr_module_mm = Some(module_mm);

        // ---- Address and code, beside the QR code or under it --------------------------
        let display_url = self.display_url();
        let code = self.req.code.grouped();
        let label = t(locale, Msg::CodeLabel);
        let column_gap = 5.0;
        let beside_w = area.w - qr_box - column_gap;
        let beside = beside_w >= width_of(&fonts.code, &code, CODE_PT);
        let column_w = if beside { beside_w } else { area.w };
        let (code_text, code_pt, code_cut) = fit_line(&fonts.code, &code, column_w, CODE_PT, 10.0);
        if code_cut {
            // Never print a cut code: the panel cannot carry this card.
            return Err(CardError::FormatTooSmall {
                panel: "inside-back",
                needed_mm: width_of(&fonts.code, &code, 10.0),
                available_mm: column_w,
            });
        }
        let (url_text, url_pt, url_cut) = fit_line(&fonts.body_bold, &display_url, column_w, 12.0, 7.0);
        if url_cut {
            note_clamped(TextField::DisplayUrl, &mut self.warnings);
        }
        let label_pt = 8.5;
        let url_line = if url_text.is_empty() { 0.0 } else { pt_to_mm(url_pt * 1.3) };
        let label_line = pt_to_mm(label_pt * 1.3);
        let code_line = pt_to_mm(code_pt * 1.25);
        let column_h = url_line + 2.5 + label_line + 0.5 + code_line;
        let qr_block = if beside { qr_box.max(column_h) } else { qr_box + 3.0 + column_h };

        // ---- Texts, at the largest step size that fits -----------------------------------
        let heading = t(locale, Msg::CardHowTo).to_owned();
        let store = self.req.store_name.as_deref().map(str::trim).filter(|s| !s.is_empty());
        let steps: Vec<String> = match store {
            Some(store) => {
                let store = self.clean_line(&fonts.body, store, TextField::StoreName);
                vec![
                    t(locale, Msg::CardStep1).to_owned(),
                    tf(locale, Msg::CardStep2Reseller, &[("store", &store)]),
                    t(locale, Msg::CardStep3Reseller).to_owned(),
                ]
            }
            None => vec![
                t(locale, Msg::CardStep1).to_owned(),
                t(locale, Msg::CardStep2).to_owned(),
                t(locale, Msg::CardStep3).to_owned(),
            ],
        };
        let no_phone = tf(locale, Msg::CardNoPhone, &[("url", &display_url)]);
        let legal = self.legal_text();

        let mut chosen = None;
        for step_pt in STEP_SIZES_PT {
            let plan = self.measure_how_to(area, step_pt, &heading, &steps, &no_phone, &legal, qr_block);
            let fits = plan.fixed + plan.legal.height() <= area.h + 1e-9;
            chosen = Some(plan);
            if fits {
                break;
            }
        }
        // `STEP_SIZES_PT` is not empty, so a plan exists; the fallback keeps this total.
        let mut plan = match chosen {
            Some(plan) => plan,
            None => self.measure_how_to(area, 8.0, &heading, &steps, &no_phone, &legal, qr_block),
        };
        if plan.step_pt < STEPS_PT {
            self.warnings.push(CardWarning::StepsReduced { size_pt: plan.step_pt });
        }
        if plan.fixed + plan.legal.height() > area.h + 1e-9 {
            // Still too high at the smallest size: cut the legal lines first, then the
            // no-phone line; the steps, the QR code and the code are never cut.
            let room = area.h - plan.fixed;
            let legal_spec = ParaSpec::new(7.0, MIN_TYPE_PT, 1.25, 30);
            if room >= pt_to_mm(MIN_TYPE_PT * 1.25) {
                plan.legal = layout(&fonts.body, &legal, area.w, room, &legal_spec);
            } else {
                plan.legal = Para::empty();
                let without = plan.fixed - plan.no_phone.height() - plan.gap;
                let room = area.h - without - plan.gap;
                let line = pt_to_mm(plan.no_phone.size_pt * 1.32);
                if room >= line {
                    let spec = ParaSpec::fixed(plan.no_phone.size_pt, 1.32, 8);
                    plan.no_phone = layout(&fonts.body, &no_phone, area.w, room, &spec);
                    plan.fixed = without + plan.no_phone.height() + plan.gap;
                } else if without - plan.gap <= area.h + 1e-9 {
                    plan.no_phone = Para::empty();
                    plan.fixed = without;
                    note_clamped(TextField::NoPhone, &mut self.warnings);
                } else {
                    return Err(CardError::FormatTooSmall {
                        panel: "inside-back",
                        needed_mm: without - plan.gap,
                        available_mm: area.h,
                    });
                }
                if plan.no_phone.clamped {
                    note_clamped(TextField::NoPhone, &mut self.warnings);
                }
            }
            note_clamped(TextField::Legal, &mut self.warnings);
        }

        // ---- Drawing ---------------------------------------------------------------------
        let s = plan.step_pt / STEPS_PT;
        let mut y = area.y;
        items.extend(self.setter.para(
            &mut self.doc,
            &fonts.body_bold,
            &plan.heading,
            ink,
            area.x,
            y,
            area.w,
            Align::Left,
        )?);
        y += plan.heading.height() + 2.0 * s;
        for (number, step) in plan.steps.iter().enumerate() {
            let digit = (number + 1).to_string();
            items.extend(self.setter.line(
                &mut self.doc,
                &fonts.body_bold,
                &digit,
                step.size_pt,
                ink,
                Point::new(area.x, y + first_baseline(&fonts.body, step.size_pt, step.line_mm)),
            )?);
            items.extend(self.setter.para(
                &mut self.doc,
                &fonts.body,
                step,
                ink,
                area.x + plan.indent,
                y,
                area.w - plan.indent,
                Align::Left,
            )?);
            y += step.height() + 1.5 * s;
        }
        y += plan.gap * 1.25 - 1.5 * s;

        // The QR code on its white box.
        let box_x = if beside { area.x } else { area.x + (area.w - qr_box) / 2.0 };
        let qr_y = if beside { y + (qr_block - qr_box) / 2.0 } else { y };
        let qr_rect = Rect::new(box_x, qr_y, qr_box, qr_box);
        items.push(Item::filled_rect(qr_rect, self.paint.paper()));
        let placement = qr.fit_in(qr_rect, QUIET_ZONE_MODULES);
        items.push(Item::fill(qr.to_path(placement.origin, placement.module_mm), ink));

        // Address, "Code", the code.
        let (column_x, mut column_y, align) = if beside {
            (qr_rect.right() + column_gap, y + (qr_block - column_h) / 2.0, Align::Left)
        } else {
            (area.x, qr_rect.bottom() + 3.0, Align::Center)
        };
        if !url_text.is_empty() {
            items.extend(self.setter.aligned(
                &mut self.doc,
                &fonts.body_bold,
                &url_text,
                url_pt,
                ink,
                column_x,
                column_w,
                column_y + first_baseline(&fonts.body_bold, url_pt, url_line),
                align,
            )?);
            column_y += url_line;
        }
        column_y += 2.5;
        items.extend(self.setter.aligned(
            &mut self.doc,
            &fonts.body,
            label,
            label_pt,
            ink,
            column_x,
            column_w,
            column_y + first_baseline(&fonts.body, label_pt, label_line),
            align,
        )?);
        column_y += label_line + 0.5;
        items.extend(self.setter.aligned(
            &mut self.doc,
            &fonts.code,
            &code_text,
            code_pt,
            ink,
            column_x,
            column_w,
            column_y + first_baseline(&fonts.code, code_pt, code_line),
            align,
        )?);
        y += qr_block + plan.gap;

        // The line for people without a phone.
        if !plan.no_phone.is_empty() {
            items.extend(self.setter.para(
                &mut self.doc,
                &fonts.body,
                &plan.no_phone,
                ink,
                area.x,
                y,
                area.w,
                Align::Left,
            )?);
        }

        // Legal lines, at the foot.
        if !plan.legal.is_empty() {
            let top = area.bottom() - plan.legal.height();
            items.extend(self.setter.para(
                &mut self.doc,
                &fonts.body,
                &plan.legal,
                ink,
                area.x,
                top,
                area.w,
                Align::Left,
            )?);
        }
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ebc_model::{GiftCode, claim_url_qr};

    #[test]
    fn the_payload_is_the_upper_case_claim_url_for_our_own_scheme() {
        let code = GiftCode::from_payload(123_456_789).expect("valid payload");
        for base in ["https://ebk.example", "http://ebk.example:8080", "HTTPS://Ebk.Example/"] {
            let url = claim_url(base, &code);
            assert_eq!(qr_payload(&url, &code).expect("valid"), (claim_url_qr(base, &code), true));
        }
        // A path before /k/, another code, or another route: encoded as given.
        let other = GiftCode::from_payload(987_654_321).expect("valid payload");
        for url in [
            claim_url("https://store.example/Gifts", &code),
            claim_url("https://ebk.example", &other),
            "https://store.example/redeem?c=AbC".to_owned(),
        ] {
            assert_eq!(qr_payload(&url, &code).expect("valid"), (url.clone(), false));
        }
    }

    #[test]
    fn unusable_claim_urls_are_refused() {
        let code = GiftCode::from_payload(1).expect("valid payload");
        for url in ["", "   ", "ebk.example/k/x", "ftp://ebk.example", "https://", "https:///k/x",
            "https://ebk.example/a b", "https://ebk.example/\u{7}"]
        {
            assert!(matches!(qr_payload(url, &code), Err(CardError::InvalidClaimUrl(_))), "{url:?}");
        }
    }
}
