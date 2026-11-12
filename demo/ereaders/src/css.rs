//! Just enough CSS reading for the page lint: inline declarations, colours, contrast.
//!
//! This is not a CSS engine. It reads `style="…"` attributes and looks for telltale substrings
//! in `<style>` blocks; selectors, the cascade and external style sheets are out of scope.

/// An sRGB colour.
pub(crate) type Rgb = (u8, u8, u8);

pub(crate) const WHITE: Rgb = (255, 255, 255);
pub(crate) const BLACK: Rgb = (0, 0, 0);

/// Splits a `style` attribute into lower-cased `(property, value)` pairs.
pub(crate) fn declarations(style: &str) -> Vec<(String, String)> {
    style
        .split(';')
        .filter_map(|decl| decl.split_once(':'))
        .map(|(prop, value)| (prop.trim().to_ascii_lowercase(), value.trim().to_ascii_lowercase()))
        .filter(|(prop, value)| !prop.is_empty() && !value.is_empty())
        .take(64)
        .collect()
}

/// Parses `#rgb`, `#rrggbb`, `rgb()/rgba()` with integer channels and a few dozen colour names.
pub(crate) fn parse_color(value: &str) -> Option<Rgb> {
    let v = value.trim().trim_end_matches("!important").trim().to_ascii_lowercase();
    if let Some(hex) = v.strip_prefix('#') {
        return match hex.len() {
            3 => {
                let mut it = hex.chars().map(|c| c.to_digit(16));
                let (r, g, b) = (it.next()??, it.next()??, it.next()??);
                Some(((r * 17) as u8, (g * 17) as u8, (b * 17) as u8))
            }
            6 => {
                let channel = |i: usize| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok();
                Some((channel(0)?, channel(2)?, channel(4)?))
            }
            _ => None,
        };
    }
    if let Some(args) = v.strip_prefix("rgba(").or_else(|| v.strip_prefix("rgb(")) {
        let args = args.strip_suffix(')')?;
        let mut channels =
            args.split([',', ' ', '/']).filter(|s| !s.is_empty()).map(|s| s.trim().parse::<u8>());
        let r = channels.next()?.ok()?;
        let g = channels.next()?.ok()?;
        let b = channels.next()?.ok()?;
        return Some((r, g, b));
    }
    Some(match v.as_str() {
        "black" => (0, 0, 0),
        "white" => (255, 255, 255),
        "gray" | "grey" => (128, 128, 128),
        "silver" => (192, 192, 192),
        "lightgray" | "lightgrey" => (211, 211, 211),
        "darkgray" | "darkgrey" => (169, 169, 169),
        "dimgray" | "dimgrey" => (105, 105, 105),
        "gainsboro" => (220, 220, 220),
        "whitesmoke" => (245, 245, 245),
        "red" => (255, 0, 0),
        "green" => (0, 128, 0),
        "blue" => (0, 0, 255),
        "yellow" => (255, 255, 0),
        "orange" => (255, 165, 0),
        "purple" => (128, 0, 128),
        "navy" => (0, 0, 128),
        "teal" => (0, 128, 128),
        "maroon" => (128, 0, 0),
        "olive" => (128, 128, 0),
        "lime" => (0, 255, 0),
        "aqua" | "cyan" => (0, 255, 255),
        "fuchsia" | "magenta" => (255, 0, 255),
        _ => return None,
    })
}

/// The colour in a `background` shorthand, if one of its tokens is a colour.
pub(crate) fn background_color(value: &str) -> Option<Rgb> {
    parse_color(value).or_else(|| value.split_whitespace().take(8).find_map(parse_color))
}

fn linear(channel: u8) -> f64 {
    let c = f64::from(channel) / 255.0;
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn luminance((r, g, b): Rgb) -> f64 {
    0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
}

/// WCAG 2.x contrast ratio, 1.0..=21.0.
pub(crate) fn contrast_ratio(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// A length in CSS pixels (`640px`, or a bare number as in `width="640"`).
pub(crate) fn px(value: &str) -> Option<u32> {
    let v = value.trim().trim_end_matches("!important").trim();
    let digits = v.strip_suffix("px").unwrap_or(v).trim();
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
        return None;
    }
    let number: f64 = digits.parse().ok()?;
    (number.is_finite() && (0.0..1.0e7).contains(&number)).then_some(number as u32)
}

/// `true` when declarations hide the element (`display:none`, `visibility:hidden`).
pub(crate) fn hides(declarations: &[(String, String)]) -> bool {
    declarations.iter().any(|(prop, value)| {
        (prop == "display" && value.starts_with("none"))
            || (prop == "visibility" && value.starts_with("hidden"))
    })
}

/// Every `url(...)` target in a piece of CSS (bounded).
pub(crate) fn urls(css: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lower = css.to_ascii_lowercase();
    let mut from = 0;
    while let Some(at) = lower.get(from..).and_then(|rest| rest.find("url(")) {
        let start = from + at + 4;
        let Some(len) = lower.get(start..).and_then(|rest| rest.find(')')) else { break };
        if let Some(raw) = css.get(start..start + len) {
            let target = raw.trim().trim_matches(['"', '\'']).trim();
            if !target.is_empty() {
                out.push(target.to_owned());
            }
        }
        from = start + len;
        if out.len() >= 64 {
            break;
        }
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn colours_parse_in_the_common_notations() {
        assert_eq!(parse_color("#fff"), Some(WHITE));
        assert_eq!(parse_color(" #1A2b3C "), Some((0x1a, 0x2b, 0x3c)));
        assert_eq!(parse_color("rgb(1, 2, 3)"), Some((1, 2, 3)));
        assert_eq!(parse_color("rgba(10,20,30,0.5)"), Some((10, 20, 30)));
        assert_eq!(parse_color("GREY"), Some((128, 128, 128)));
        assert_eq!(parse_color("#ccc !important"), Some((204, 204, 204)));
        for bad in ["", "#", "#12", "#gggggg", "rgb(", "rgb(1,2)", "rgb(300,0,0)", "inherit", "#ééé", "#€€"] {
            assert_eq!(parse_color(bad), None, "{bad:?}");
        }
        assert_eq!(background_color("url(x.png) no-repeat #000"), Some(BLACK));
        assert_eq!(background_color("none"), None);
    }

    #[test]
    fn contrast_matches_wcag_reference_points() {
        assert!((contrast_ratio(BLACK, WHITE) - 21.0).abs() < 1e-9);
        assert!((contrast_ratio(WHITE, WHITE) - 1.0).abs() < 1e-9);
        // #767676 on white is the classic "just passes AA" grey: 4.54:1.
        let ratio = contrast_ratio((0x76, 0x76, 0x76), WHITE);
        assert!((4.5..4.6).contains(&ratio), "{ratio}");
        assert!(contrast_ratio((204, 204, 204), WHITE) < 2.0);
        assert_eq!(contrast_ratio(BLACK, WHITE), contrast_ratio(WHITE, BLACK));
    }

    #[test]
    fn declarations_lengths_and_urls() {
        let decls = declarations("Color: RED; background-color:#000;;broken; width : 640PX");
        assert_eq!(decls.len(), 3);
        assert_eq!(decls[0], ("color".to_owned(), "red".to_owned()));
        assert_eq!(px(&decls[2].1), Some(640));
        assert_eq!(px("100%"), None);
        assert_eq!(px("12.5px"), Some(12));
        assert_eq!(px(""), None);
        assert_eq!(px("1e99"), None);
        assert_eq!(px("99999999999999999999"), None);
        assert!(hides(&declarations("display: none")));
        assert!(!hides(&declarations("display: block")));
        assert_eq!(
            urls("a{background:url( 'https://cdn.example/x.png' )} @font-face{src:url(/f.woff)} url("),
            vec!["https://cdn.example/x.png".to_owned(), "/f.woff".to_owned()]
        );
        assert!(urls("url(").is_empty());
        assert!(urls("é url(é) ü").len() == 1);
    }
}
