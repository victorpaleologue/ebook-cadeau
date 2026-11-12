//! Colours as they are written for a given profile.
//!
//! The card is designed in sRGB ([`Rgb8`]) and painted according to the output:
//!
//! * **PDF/X profiles** — flat grounds are converted to the output intent's CMYK with the
//!   ICC transform of [`ebc_color::SrgbToCmyk`] (relative colorimetric with black-point
//!   compensation — the very transform the cover image goes through, so that the ground
//!   and the cover's own frame meet on press), then cleaned of tints under 5 % and capped
//!   at the profile's ink limit ([`ebc_color::finish_flat`]). A near-black ground is
//!   K 100 alone. Text and the QR code are 0/0/0/100, white is 0/0/0/0. When the output
//!   profile cannot be read by the colour-management module, the naive conversion of
//!   [`ebc_color::to_cmyk_ink_safe`] is the fallback (see [`Paint::link`]).
//! * **Plain-PDF (home) profiles** — chromatic grounds stay sRGB, neutrals and all text
//!   are `DeviceGray`, as `docs/research/03-print-requirements.md` (implication 29) asks.
//!
//! Text, QR code and rules are never RGB in either case.

use ebc_color::{
    NEAR_BLACK_LSTAR, Rgb8, SrgbToCmyk, black_or_white, finish_flat, to_cmyk_ink_safe,
    to_scene_rgb,
};
use ebc_layout::PrintProfile;
use ebc_scene::{Color, Item, PdfStandard};

/// How colours are written for one profile.
#[derive(Debug, Clone)]
pub(crate) struct Paint {
    cmyk: bool,
    max_ink_percent: f32,
    /// The ICC transform to the output intent, for PDF/X profiles whose profile could be
    /// read.
    link: Option<SrgbToCmyk>,
}

impl Paint {
    /// Paint for `profile`; `link` is the sRGB → output-intent transform (PDF/X only).
    pub(crate) fn for_profile(profile: &PrintProfile, link: Option<SrgbToCmyk>) -> Self {
        let max = profile.max_total_ink_percent;
        let cmyk = profile.standard != PdfStandard::Plain;
        Self {
            cmyk,
            max_ink_percent: if max.is_finite() && max > 0.0 { max.min(400.0) } else { 300.0 },
            link: if cmyk { link } else { None },
        }
    }

    /// The ICC transform, when there is one.
    pub(crate) fn link(&self) -> Option<&SrgbToCmyk> {
        self.link.as_ref()
    }

    /// A flat ground or tint.
    pub(crate) fn flat(&self, c: Rgb8) -> Color {
        if self.cmyk {
            match &self.link {
                // A near-black ground prints as K 100 alone, not as a four-colour build
                // (and never as rich black across a crease, §4.3).
                Some(_) if c.to_lab().l < NEAR_BLACK_LSTAR => Color::BLACK,
                Some(link) => finish_flat(link.convert_color(c), self.max_ink_percent),
                None => to_cmyk_ink_safe(c, self.max_ink_percent),
            }
        } else if c.r == c.g && c.g == c.b {
            Color::Gray(f32::from(c.r) / 255.0)
        } else {
            to_scene_rgb(c)
        }
    }

    /// Black for text, rules and the QR code: K only.
    pub(crate) fn ink(&self) -> Color {
        if self.cmyk { Color::BLACK } else { Color::Gray(0.0) }
    }

    /// Paper white (no ink).
    pub(crate) fn paper(&self) -> Color {
        if self.cmyk { Color::WHITE } else { Color::Gray(1.0) }
    }

    /// A black-only grey for secondary marks; `level` is the grey level (0 = black).
    pub(crate) fn grey(&self, level: f32) -> Color {
        let level = level.clamp(0.0, 1.0);
        if self.cmyk { Color::cmyk(0.0, 0.0, 0.0, 1.0 - level) } else { Color::Gray(level) }
    }

    /// The text colour for a ground: black or white, whichever contrasts more. The flag
    /// is `true` for white ("reversed") text, which must not be set too small.
    pub(crate) fn text_on(&self, ground: Rgb8) -> (Color, bool) {
        if black_or_white(ground) == Rgb8::WHITE {
            (self.paper(), true)
        } else {
            (self.ink(), false)
        }
    }

    /// Re-expresses the colours of items produced by another crate (the spine) the way
    /// this profile writes them: sRGB fills go through [`Paint::flat`], and for plain
    /// PDF black-only CMYK becomes grey. Images are left alone.
    pub(crate) fn repaint(&self, item: Item) -> Item {
        match item {
            Item::Path { path, fill, stroke } => Item::Path {
                path,
                fill: fill.map(|(c, rule)| (self.convert(c), rule)),
                stroke: stroke.map(|mut s| {
                    s.color = self.convert(s.color);
                    s
                }),
            },
            Item::Glyphs { font, size_pt, color, origin, glyphs, glyph_text } => Item::Glyphs {
                font,
                size_pt,
                color: self.convert(color),
                origin,
                glyphs,
                glyph_text,
            },
            Item::Group { transform, clip, layer, items } => Item::Group {
                transform,
                clip,
                layer,
                items: items.into_iter().map(|i| self.repaint(i)).collect(),
            },
            image @ Item::Image { .. } => image,
        }
    }

    fn convert(&self, color: Color) -> Color {
        match color {
            Color::Rgb { .. } => self.flat(ebc_color::from_scene_color(&color)),
            Color::Cmyk { c, m, y, k } if !self.cmyk && c == 0.0 && m == 0.0 && y == 0.0 => {
                Color::Gray(1.0 - k)
            }
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ebc_layout::Sheet;
    use ebc_scene::{Path, Rect};

    #[test]
    fn professional_paint_is_cmyk_and_home_paint_is_grey_or_rgb() {
        let pro = Paint::for_profile(&PrintProfile::professional_eu(), None);
        let home = Paint::for_profile(&PrintProfile::home(Sheet::A4), None);
        let navy = Rgb8::new(31, 58, 95);
        assert!(matches!(pro.flat(navy), Color::Cmyk { .. }));
        assert!(pro.flat(navy).total_ink_percent().is_some_and(|ink| ink <= 300.0));
        assert!(matches!(home.flat(navy), Color::Rgb { .. }));
        assert_eq!(home.flat(Rgb8::new(128, 128, 128)), Color::Gray(128.0 / 255.0));
        assert_eq!(pro.ink(), Color::BLACK);
        assert_eq!(home.ink(), Color::Gray(0.0));
        assert_eq!(pro.paper(), Color::WHITE);
        assert_eq!(pro.text_on(navy), (Color::WHITE, true));
        assert_eq!(home.text_on(Rgb8::new(240, 240, 230)), (Color::Gray(0.0), false));
        assert_eq!(pro.grey(0.5), Color::cmyk(0.0, 0.0, 0.0, 0.5));
    }

    #[test]
    fn repainting_converts_rgb_fills_and_keeps_k_only_text() {
        let pro = Paint::for_profile(&PrintProfile::professional_eu(), None);
        let home = Paint::for_profile(&PrintProfile::home(Sheet::A4), None);
        let rect = Path::rect(Rect::new(0.0, 0.0, 1.0, 1.0));
        let rgb = Item::group(
            ebc_scene::Transform::IDENTITY,
            vec![Item::fill(rect.clone(), Color::rgb8(31, 58, 95))],
        );
        let Item::Group { items, .. } = pro.repaint(rgb) else { panic!("group expected") };
        assert_eq!(items, [Item::fill(rect.clone(), pro.flat(Rgb8::new(31, 58, 95)))]);
        assert_eq!(
            home.repaint(Item::fill(rect.clone(), Color::BLACK)),
            Item::fill(rect.clone(), Color::Gray(0.0))
        );
        assert_eq!(
            pro.repaint(Item::fill(rect.clone(), Color::BLACK)),
            Item::fill(rect, Color::BLACK)
        );
    }

    #[cfg(feature = "bundled-fogra39")]
    #[test]
    fn colour_managed_grounds_go_through_the_output_profile_and_are_cleaned() {
        let profile = crate::OutputProfile::bundled_fogra39().expect("bundled profile");
        let link = ebc_color::CmykProfile::parse(profile.icc())
            .map(|p| SrgbToCmyk::relative_bpc(&p))
            .expect("the bundled profile is readable");
        let pro = Paint::for_profile(&PrintProfile::upload_eu(), Some(link.clone()));
        assert!(pro.link().is_some());
        let cream = Rgb8::new(250, 240, 220);
        let Color::Cmyk { c, m, y, k } = pro.flat(cream) else { panic!("CMYK expected") };
        for v in [c, m, y, k] {
            assert!(v == 0.0 || v >= ebc_color::MIN_TINT, "{v}");
        }
        assert_eq!(pro.flat(Rgb8::new(8, 8, 10)), Color::BLACK, "near-black is K only");
        assert!(pro.flat(Rgb8::new(0, 40, 160)).total_ink_percent().is_some_and(|i| i <= 300.01));
        // A home sheet never uses the transform.
        let home = Paint::for_profile(&PrintProfile::home(Sheet::A4), Some(link));
        assert!(home.link().is_none());
        assert!(matches!(home.flat(cream), Color::Rgb { .. }));
    }
}
