//! Colours as they are written for a given profile.
//!
//! The card is designed in sRGB ([`Rgb8`]) and painted according to the output:
//!
//! * **PDF/X profiles** — flat grounds become device CMYK through
//!   [`ebc_color::to_cmyk_ink_safe`] (not colour-managed: the printed tint of a chromatic
//!   ground will differ from the screen), text and the QR code are 0/0/0/100, white is
//!   0/0/0/0.
//! * **Plain-PDF (home) profiles** — chromatic grounds stay sRGB, neutrals and all text
//!   are `DeviceGray`, as `docs/research/03-print-requirements.md` (implication 29) asks.
//!
//! Text, QR code and rules are never RGB in either case.

use ebc_color::{Rgb8, black_or_white, to_cmyk_ink_safe, to_scene_rgb};
use ebc_layout::PrintProfile;
use ebc_scene::{Color, Item, PdfStandard};

/// How colours are written for one profile.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Paint {
    cmyk: bool,
    max_ink_percent: f32,
}

impl Paint {
    pub(crate) fn for_profile(profile: &PrintProfile) -> Self {
        let max = profile.max_total_ink_percent;
        Self {
            cmyk: profile.standard != PdfStandard::Plain,
            max_ink_percent: if max.is_finite() && max > 0.0 { max.min(400.0) } else { 300.0 },
        }
    }

    /// A flat ground or tint.
    pub(crate) fn flat(&self, c: Rgb8) -> Color {
        if self.cmyk {
            to_cmyk_ink_safe(c, self.max_ink_percent)
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
        let pro = Paint::for_profile(&PrintProfile::professional_eu());
        let home = Paint::for_profile(&PrintProfile::home(Sheet::A4));
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
        let pro = Paint::for_profile(&PrintProfile::professional_eu());
        let home = Paint::for_profile(&PrintProfile::home(Sheet::A4));
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
}
