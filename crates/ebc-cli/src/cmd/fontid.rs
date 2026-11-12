//! `ebc fontid`: which font of an index looks most like the title lettering of a cover.
//!
//! The answer is always *the nearest font of the index*, never "the font of the cover":
//! cover lettering is mostly commercial or hand-drawn, and the index holds openly
//! licensed fonts only.

use std::path::Path;

use ebc_fontid::{
    CaseUsed, FontIndex, Identification, IndexEntry, TextRegion, bundled_index, find_title_regions,
    identify_detailed, rerank_detailed,
};
use ebc_scene::Pixmap;
use ebc_text::Font;
use serde_json::{Value, json};

use crate::args::FontidArgs;
use crate::error::{CliError, Status};
use crate::util::{Streams, read_input};

/// Largest image file read.
const MAX_IMAGE_BYTES: u64 = 64 * 1024 * 1024;
/// Largest image side decoded, in pixels.
const MAX_IMAGE_SIDE: u32 = 16_384;
/// Largest font index or font file read.
const MAX_FONT_BYTES: u64 = 64 * 1024 * 1024;
/// Most candidates rendered by the second pass.
const MAX_RERANKED: usize = 20;

/// Decodes the cover into RGBA pixels, with bounds on what a hostile file may ask for.
fn decode(path: &Path) -> Result<Pixmap, CliError> {
    let bytes = read_input(path, MAX_IMAGE_BYTES)?;
    let refuse = |reason: String| CliError::failure(format!("{}: {reason}", path.display()));
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| refuse(e.to_string()))?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_SIDE);
    limits.max_image_height = Some(MAX_IMAGE_SIDE);
    limits.max_alloc = Some(512 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|e| refuse(format!("not a readable JPEG, PNG or WebP image ({e})")))?
        .to_rgba8();
    let (width, height) = image.dimensions();
    Pixmap::from_rgba8(width, height, image.into_raw())
        .ok_or_else(|| refuse("the image has no pixels".to_owned()))
}

/// Parses `X,Y,W,H`.
fn parse_region(text: &str) -> Result<TextRegion, CliError> {
    let numbers: Vec<u32> =
        text.split(',').map(|part| part.trim().parse::<u32>()).collect::<Result<_, _>>().map_err(
            |_| CliError::usage(format!("--region {text:?}: expected X,Y,WIDTH,HEIGHT")),
        )?;
    match numbers.as_slice() {
        [x, y, w, h] if *w > 0 && *h > 0 => {
            Ok(TextRegion { x: *x, y: *y, w: *w, h: *h, score: 1.0 })
        }
        _ => Err(CliError::usage(format!(
            "--region {text:?}: expected X,Y,WIDTH,HEIGHT in pixels, with a non-empty size"
        ))),
    }
}

/// The file of an index entry in `dir`: the last path segment of its source URL.
fn font_for(entry: &IndexEntry, dir: &Path) -> Result<Font, String> {
    let name = entry
        .source_url
        .rsplit(['/', '\\'])
        .next()
        .filter(|name| !name.is_empty() && *name != "." && *name != "..")
        .ok_or_else(|| format!("{} {}: no file name in its source", entry.family, entry.style))?;
    let path = dir.join(name);
    let bytes = read_input(&path, MAX_FONT_BYTES).map_err(|e| e.to_string())?;
    Font::from_bytes(bytes, 0).map_err(|e| format!("{}: {e}", path.display()))
}

fn case_name(case: CaseUsed) -> &'static str {
    match case {
        CaseUsed::AsGiven => "as given",
        CaseUsed::Upper => "upper",
        CaseUsed::Lower => "lower",
        CaseUsed::Title => "title",
        _ => "unknown",
    }
}

fn round3(value: f32) -> f64 {
    (f64::from(value) * 1000.0).round() / 1000.0
}

/// Runs `ebc fontid`.
pub(crate) fn run(args: &FontidArgs, streams: &mut Streams<'_>) -> Result<Status, CliError> {
    if args.title.trim().is_empty() {
        return Err(CliError::usage("--title is empty"));
    }
    let given_region = args.region.as_deref().map(parse_region).transpose()?;
    let index: FontIndex = match &args.index {
        Some(path) => FontIndex::from_bytes(&read_input(path, MAX_FONT_BYTES)?)
            .map_err(|e| CliError::failure(format!("{}: {e}", path.display())))?,
        None => bundled_index()
            .map_err(|e| CliError::failure(format!("the bundled font index is unusable: {e}")))?,
    };
    if index.is_empty() {
        return Err(CliError::failure("the font index is empty"));
    }
    let cover = decode(&args.image)?;

    let region = match given_region {
        Some(region) => region,
        None => match find_title_regions(&cover, &args.title).into_iter().next() {
            Some(region) => region,
            None => {
                streams.note(format!(
                    "no lettering found in {}: point at the title with --region X,Y,W,H",
                    args.image.display()
                ));
                return Ok(Status::NotFound);
            }
        },
    };

    let mut result: Identification = identify_detailed(&cover, &region, Some(&args.title), &index);
    let mut reranked = false;
    if let Some(dir) = &args.fonts {
        let wanted = (args.top as usize).clamp(10, MAX_RERANKED);
        let mut candidates: Vec<(IndexEntry, Font)> = Vec::new();
        for found in result.matches.iter().take(wanted) {
            match font_for(&found.entry, dir) {
                Ok(font) => candidates.push((found.entry.clone(), font)),
                Err(reason) => streams.note(format!("note: not re-ranked: {reason}")),
            }
        }
        if !candidates.is_empty() {
            let second = rerank_detailed(&cover, &region, &args.title, &candidates);
            if !second.matches.is_empty() {
                result = second;
                reranked = true;
            }
        }
    }

    let top = &result.matches[..result.matches.len().min(args.top as usize)];
    if args.json {
        let rows: Vec<Value> = top
            .iter()
            .map(|m| {
                json!({
                    "family": m.entry.family,
                    "style": m.entry.style,
                    "score": round3(m.score),
                    "licence": m.entry.licence,
                    "source": m.entry.source_url,
                })
            })
            .collect();
        streams.json(&json!({
            "region": { "x": region.x, "y": region.y, "w": region.w, "h": region.h,
                        "detected": given_region.is_none() },
            "index_fonts": index.len(),
            "reranked": reranked,
            "glyphs": result.glyphs,
            "labelled_glyphs": result.labelled_glyphs,
            "lines": result.lines,
            "case": case_name(result.case_used),
            "text_coverage": round3(result.text_coverage),
            "light_on_dark": result.light_on_dark,
            "skew_degrees": round3(result.skew_degrees),
            "confident": result.confident,
            "family_margin": round3(result.family_margin),
            "matches": rows,
        }))?;
    } else {
        streams.line(format!(
            "region {},{} {}x{} px ({}): {} glyph(s), {} matched with the title, {} line(s), case {}",
            region.x,
            region.y,
            region.w,
            region.h,
            if given_region.is_some() { "given" } else { "detected" },
            result.glyphs,
            result.labelled_glyphs,
            result.lines,
            case_name(result.case_used)
        ))?;
        for (rank, found) in top.iter().enumerate() {
            streams.line(format!(
                "{:>2}. {:.3}  {} {}  [{}]  {}",
                rank + 1,
                found.score,
                found.entry.family,
                found.entry.style,
                found.entry.licence,
                found.entry.source_url
            ))?;
        }
        if !top.is_empty() {
            streams.line(format!(
                "confident: {} (lead over the next family: {:.3}{})",
                if result.confident { "yes" } else { "no" },
                result.family_margin,
                if reranked { "; re-ranked by rendering" } else { "" }
            ))?;
        }
    }
    if top.is_empty() {
        streams.note("nothing in the region looks like text: no font can be suggested");
        return Ok(Status::NotFound);
    }
    streams.note(format!(
        "note: this is the nearest of the {} font(s) of the index, not necessarily the font of \
         the cover",
        index.len()
    ));
    Ok(Status::Success)
}
