//! Draws the page boxes, the guides overlay and the printer's marks of a card as an SVG
//! file — a quick way to *look* at what `ebc-layout` computes.
//!
//! ```sh
//! cargo run -p ebc-layout --example layout_svg -- poche professional_eu /tmp/layout.svg
//! cargo run -p ebc-layout --example layout_svg -- poche home_a4 /tmp/layout.svg --all-marks
//! ```
//!
//! Formats: `poche`, `a6`, `digest`, `a5`. Profiles: `professional_eu`, `upload_eu`,
//! `home_a4`, `home_letter`, `home_a3`, `home_sra3`. One SVG user unit is one millimetre.
//! The media box is white, the bleed pale red, the trimmed card pale yellow; for home
//! profiles the physical sheet is outlined in grey with its printable area dotted.

use std::fmt::Write as _;
use std::process::ExitCode;

use ebc_layout::{
    CardFormat, PrintProfile, caption_areas, guides, marks, page_geometry, scale_ruler,
};
use ebc_scene::{Color, Item, Path, PathSeg, Rect, pt_to_mm};

fn rgb(color: &Color) -> String {
    let [r, g, b] = color.to_srgb_approx();
    let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", c(r), c(g), c(b))
}

fn path_data(path: &Path) -> String {
    let mut d = String::new();
    for seg in &path.segs {
        // Writing to a String cannot fail.
        let _ = match *seg {
            PathSeg::MoveTo(p) => write!(d, "M{} {} ", p.x, p.y),
            PathSeg::LineTo(p) => write!(d, "L{} {} ", p.x, p.y),
            PathSeg::QuadTo(c, p) => write!(d, "Q{} {} {} {} ", c.x, c.y, p.x, p.y),
            PathSeg::CubicTo(a, b, p) => {
                write!(d, "C{} {} {} {} {} {} ", a.x, a.y, b.x, b.y, p.x, p.y)
            }
            PathSeg::Close => write!(d, "Z "),
        };
    }
    d
}

fn draw(item: &Item, out: &mut String) {
    match item {
        Item::Path { path, fill, stroke } => {
            let fill = fill.as_ref().map_or_else(|| "none".to_owned(), |(c, _)| rgb(c));
            let stroke = stroke.as_ref().map_or_else(String::new, |s| {
                let dash: Vec<String> = s.dash_mm.iter().map(f64::to_string).collect();
                format!(
                    " stroke=\"{}\" stroke-width=\"{}\" stroke-dasharray=\"{}\"",
                    rgb(&s.color),
                    pt_to_mm(s.width_pt),
                    if dash.is_empty() { "none".to_owned() } else { dash.join(" ") }
                )
            });
            let _ = writeln!(out, "<path d=\"{}\" fill=\"{fill}\"{stroke}/>", path_data(path));
        }
        Item::Group { items, .. } => items.iter().for_each(|i| draw(i, out)),
        // Layout layers contain neither images nor text.
        Item::Image { .. } | Item::Glyphs { .. } => {}
    }
}

fn rect(r: &Rect, style: &str, out: &mut String) {
    let _ = writeln!(
        out,
        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" {style}/>",
        r.x, r.y, r.w, r.h
    );
}

fn render(format: &CardFormat, profile: &PrintProfile) -> Result<String, String> {
    let g = page_geometry(format, profile).map_err(|e| e.to_string())?;
    // Show the whole sheet when there is one, with a little air around it.
    let view = g.sheet.map_or(g.media, |s| s.sheet.union(&g.media)).inflate(4.0);
    let mut out = String::new();
    let _ = writeln!(
        out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{} {} {} {}\" \
         width=\"{}mm\" height=\"{}mm\">",
        view.x, view.y, view.w, view.h, view.w, view.h
    );
    rect(&view, "fill=\"#9a9a9a\"", &mut out);
    if let Some(s) = &g.sheet {
        rect(&s.sheet, "fill=\"#e4e4e4\" stroke=\"#555\" stroke-width=\"0.2\"", &mut out);
    }
    rect(&g.media, "fill=\"#ffffff\"", &mut out);
    if let Some(s) = &g.sheet {
        let dotted =
            "fill=\"none\" stroke=\"#555\" stroke-width=\"0.15\" stroke-dasharray=\"0.5 1\"";
        rect(&s.printable, dotted, &mut out);
    }
    rect(&g.bleed, "fill=\"#f6d5d5\"", &mut out);
    rect(&g.trim, "fill=\"#fbf7dc\"", &mut out);
    if let Some(areas) = caption_areas(format, profile) {
        for a in &areas {
            rect(a, "fill=\"#e3eefc\"", &mut out);
        }
    }
    draw(&guides(format, profile), &mut out);
    draw(&marks(format, profile), &mut out);
    if let Some(ruler) = scale_ruler(format, profile).filter(|_| profile.marks.scale_ruler) {
        let _ = writeln!(
            out,
            "<text x=\"{}\" y=\"{}\" font-size=\"2.4\" font-family=\"sans-serif\">{} mm</text>",
            ruler.label_origin.x, ruler.label_origin.y, ruler.length_mm
        );
    }
    out.push_str("</svg>\n");
    Ok(out)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(format_name), Some(profile_name), Some(path)) =
        (args.first(), args.get(1), args.get(2))
    else {
        eprintln!("usage: layout_svg <format> <profile> <out.svg> [--all-marks]");
        return ExitCode::from(2);
    };
    let Some(format) = CardFormat::by_name(format_name) else {
        eprintln!("unknown format {format_name:?}; try one of {:?}", CardFormat::PRESET_NAMES);
        return ExitCode::from(2);
    };
    let Some(mut profile) = PrintProfile::by_name(profile_name) else {
        eprintln!("unknown profile {profile_name:?}; try one of {:?}", PrintProfile::NAMES);
        return ExitCode::from(2);
    };
    if args.iter().any(|a| a == "--all-marks") {
        let m = &mut profile.marks;
        (m.crop, m.fold, m.registration, m.color_bar, m.scale_ruler) =
            (true, true, true, true, true);
        if profile.sheet.is_none() {
            profile.slug = profile.slug.max(10.0);
        }
    }
    match render(&format, &profile)
        .and_then(|svg| std::fs::write(path, svg).map_err(|e| format!("cannot write {path}: {e}")))
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
