//! Builds a font index from a directory of font files and a manifest.
//!
//! ```sh
//! # Regenerates crates/ebc-fontid/data/bundled-index.bin from assets/fonts:
//! cargo run -p ebc-fontid --example build-index
//! # Any other gallery:
//! cargo run -p ebc-fontid --example build-index -- --fonts DIR --manifest FILE --out FILE
//! ```
//!
//! The manifest is a text file with one font per line, fields separated by tabs:
//! `file name`, `family`, `style`, `licence (SPDX)`, `source URL`. Empty family or style
//! fields are read from the font. Lines starting with `#` are comments. A font file that
//! is not in the manifest is **refused**, and so is a licence outside
//! `ebc_fontid::OPEN_LICENCES` or a font whose embedding bits forbid bundling: the index
//! may only contain fonts the product is allowed to typeset with.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ebc_fontid::{BUNDLED_FONT_FILES, FontIndex, IndexEntryMeta};
use ebc_text::Font;

struct ManifestLine {
    file: String,
    family: String,
    style: String,
    licence: String,
    source_url: String,
}

fn parse_manifest(text: &str) -> Result<Vec<ManifestLine>, String> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').map(str::trim).collect();
        if fields.len() != 5 {
            return Err(format!("manifest line {}: expected 5 tab-separated fields", n + 1));
        }
        out.push(ManifestLine {
            file: fields[0].to_string(),
            family: fields[1].to_string(),
            style: fields[2].to_string(),
            licence: fields[3].to_string(),
            source_url: fields[4].to_string(),
        });
    }
    Ok(out)
}

/// The manifest of the fonts bundled in `assets/fonts` (all SIL OFL 1.1, see
/// `assets/fonts/README.md`).
fn bundled_manifest() -> Vec<ManifestLine> {
    BUNDLED_FONT_FILES
        .iter()
        .map(|file| ManifestLine {
            file: (*file).to_string(),
            family: String::new(),
            style: String::new(),
            licence: "OFL-1.1".to_string(),
            source_url: format!("assets/fonts/{file}"),
        })
        .collect()
}

fn run() -> Result<(), String> {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut fonts_dir: PathBuf = crate_dir.join("../../assets/fonts");
    let mut manifest_path: Option<PathBuf> = None;
    let mut out_path: PathBuf = crate_dir.join("data/bundled-index.bin");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
        match arg.as_str() {
            "--fonts" => fonts_dir = PathBuf::from(value()?),
            "--manifest" => manifest_path = Some(PathBuf::from(value()?)),
            "--out" => out_path = PathBuf::from(value()?),
            other => return Err(format!("unknown argument {other} (see the file header)")),
        }
    }
    let manifest = match &manifest_path {
        Some(path) => parse_manifest(
            &std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?,
        )?,
        None => bundled_manifest(),
    };

    // Refuse font files the manifest does not know.
    let mut unlisted = Vec::new();
    for entry in std::fs::read_dir(&fonts_dir).map_err(|e| format!("{}: {e}", fonts_dir.display()))? {
        let name = entry.map_err(|e| e.to_string())?.file_name().to_string_lossy().into_owned();
        let lower = name.to_lowercase();
        let is_font = lower.ends_with(".ttf") || lower.ends_with(".otf") || lower.ends_with(".ttc");
        if is_font && !manifest.iter().any(|m| m.file == name) {
            unlisted.push(name);
        }
    }
    if !unlisted.is_empty() {
        unlisted.sort();
        return Err(format!("font files without a manifest entry: {}", unlisted.join(", ")));
    }

    let mut fonts = Vec::with_capacity(manifest.len());
    for line in &manifest {
        let path = fonts_dir.join(&line.file);
        let data = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let font = Font::from_bytes(data, 0).map_err(|e| format!("{}: {e}", path.display()))?;
        let meta = IndexEntryMeta {
            family: if line.family.is_empty() { font.family_name() } else { line.family.clone() },
            style: if line.style.is_empty() { font.style_name() } else { line.style.clone() },
            licence: line.licence.clone(),
            source_url: line.source_url.clone(),
        };
        fonts.push((meta, font));
    }
    let index = FontIndex::build_checked(&fonts).map_err(|e| e.to_string())?;
    let bytes = index.to_bytes();
    std::fs::write(&out_path, &bytes).map_err(|e| format!("{}: {e}", out_path.display()))?;

    let mut out = std::io::stdout().lock();
    for entry in index.entries() {
        writeln!(
            out,
            "{:<24} {:<10} {:<8} {:>2}/{} characters  {}",
            entry.family,
            entry.style,
            entry.licence,
            entry.coverage(),
            index.chars().len(),
            entry.source_url
        )
        .map_err(|e| e.to_string())?;
    }
    writeln!(out, "{} fonts, {} bytes -> {}", index.len(), bytes.len(), out_path.display())
        .map_err(|e| e.to_string())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            let _ = writeln!(std::io::stderr(), "build-index: {message}");
            ExitCode::FAILURE
        }
    }
}
