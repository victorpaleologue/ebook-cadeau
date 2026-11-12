//! The evaluation harness: synthesises covers, runs the pipeline, prints the tables of
//! `docs/research/fontid-eval.md`.
//!
//! ```sh
//! cargo run -p ebc-fontid --release --example eval --features synth
//! cargo run -p ebc-fontid --release --example eval --features synth -- \
//!     --cases 96 --seed 7 --corpus DIR [--only-bundled-covers]
//! ```
//!
//! * `--cases N`: covers per condition (default 64).
//! * `--seed S`: seed of everything random (default 2026).
//! * `--corpus DIR`: adds the `.ttf`/`.otf` files of `DIR` and of its sub-directories to
//!   the gallery. A font enters only with a licence file next to it — `OFL*.txt` with the
//!   SIL Open Font License or `LICENSE*.txt` with the Apache License 2.0 (the layout of
//!   the `ofl/` and `apache/` directories of github.com/google/fonts) — and only if its
//!   embedding bits allow bundling; the rest is refused and listed on standard error.
//!   The gallery is built with `FontIndex::build_checked`. Keep a downloaded corpus out
//!   of the repository (for example under `target/`).
//! * `--only-bundled-covers`: with a corpus, draw covers with the bundled fonts only
//!   (the corpus fonts are distractors).
//! * `--rerank-top K`: candidates passed to the render-and-compare stage (default 5).
//! * `--only NAMES`: run only the conditions whose name contains one of the
//!   comma-separated `NAMES`.
//! * `--verbose`: one line per cover that went wrong somewhere (on standard error).
//! * `--tables-only`: skip the hold-out and scaling measurements.
//! * `--dump FILE`: write one tab-separated line per cover (condition, case, font,
//!   labelled glyphs, rank of the true face, its score, the best score of another
//!   family, confident, detected) — the data the score calibration was read from.

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use ebc_fontid::eval::{
    CaseReport, Condition, EvalOptions, evaluate, evaluate_family_holdout, evaluate_holdout,
    format_category_holdout, format_holdout, format_table, standard_conditions,
};
use ebc_fontid::synth::{Background, CoverSpec, Degradation, render_cover};
use ebc_fontid::{FontIndex, IndexEntry, IndexEntryMeta, find_title_regions, identify_detailed};
use ebc_text::Font;

fn font_files(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if depth > 0 {
                font_files(&path, depth - 1, out);
            }
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("ttf") || e.eq_ignore_ascii_case("otf"))
        {
            out.push(path);
        }
    }
}

/// The SPDX identifier of the open licence whose text lies next to a font file: an
/// `OFL*.txt` file with the SIL Open Font License, or a `LICENSE*.txt` file with the
/// Apache License 2.0. `None` when there is no such file.
/// The google/fonts category (`SERIF`, `SANS_SERIF`, `DISPLAY`, `HANDWRITING`,
/// `MONOSPACE`) from the `METADATA.pb` file next to a font file.
fn category_next_to(font: &Path) -> Option<String> {
    let text = std::fs::read_to_string(font.parent()?.join("METADATA.pb")).ok()?;
    let line = text.lines().find(|l| l.trim_start().starts_with("category:"))?;
    let value = line.split('"').nth(1)?;
    (!value.is_empty()).then(|| value.to_string())
}

fn licence_next_to(font: &Path) -> Option<&'static str> {
    let dir = font.parent()?;
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    for name in names {
        let upper = name.to_uppercase();
        let (spdx, marker) = if upper.starts_with("OFL") && upper.ends_with(".TXT") {
            ("OFL-1.1", "SIL OPEN FONT LICENSE")
        } else if upper.starts_with("LICENSE") && upper.ends_with(".TXT") {
            ("Apache-2.0", "APACHE LICENSE")
        } else {
            continue;
        };
        let Ok(text) = std::fs::read_to_string(dir.join(&name)) else { continue };
        if text.to_uppercase().contains(marker) {
            return Some(spdx);
        }
    }
    None
}

/// The fonts of a directory tree that may enter an index, and the files refused (with
/// the reason).
fn load(dir: &Path) -> (Vec<(IndexEntryMeta, Font)>, Vec<String>) {
    let mut files = Vec::new();
    font_files(dir, 2, &mut files);
    files.sort();
    let mut fonts: Vec<(IndexEntryMeta, Font)> = Vec::new();
    let mut refused = Vec::new();
    for path in files {
        let shown = path.strip_prefix(dir).unwrap_or(&path).display().to_string();
        let Some(licence) = licence_next_to(&path) else {
            refused.push(format!("{shown}: no OFL or Apache licence file next to it"));
            continue;
        };
        let Ok(data) = std::fs::read(&path) else { continue };
        let font = match Font::from_bytes(data, 0) {
            Ok(font) => font,
            Err(e) => {
                refused.push(format!("{shown}: {e}"));
                continue;
            }
        };
        if !font.allows_bundling() {
            refused.push(format!("{shown}: embedding bits forbid bundling"));
            continue;
        }
        let mut meta = IndexEntryMeta::from_font(&font, shown.clone());
        meta.licence = licence.to_string();
        // Family and style must identify a face.
        if fonts.iter().any(|(m, _)| m.family == meta.family && m.style == meta.style) {
            refused.push(format!("{shown}: duplicate of {} {}", meta.family, meta.style));
            continue;
        }
        fonts.push((meta, font));
    }
    (fonts, refused)
}

fn run() -> Result<(), String> {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut cases = 64usize;
    let mut seed = 2026u64;
    let mut rerank_top = 5usize;
    let mut corpus: Option<PathBuf> = None;
    let mut only_bundled_covers = false;
    let mut only: Option<String> = None;
    let mut verbose = false;
    let mut tables_only = false;
    let mut dump_path: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
        match arg.as_str() {
            "--cases" => cases = value()?.parse().map_err(|_| "--cases: not a number")?,
            "--seed" => seed = value()?.parse().map_err(|_| "--seed: not a number")?,
            "--rerank-top" => {
                rerank_top = value()?.parse().map_err(|_| "--rerank-top: not a number")?;
            }
            "--corpus" => corpus = Some(PathBuf::from(value()?)),
            "--only-bundled-covers" => only_bundled_covers = true,
            "--only" => only = Some(value()?),
            "--verbose" => verbose = true,
            "--tables-only" => tables_only = true,
            "--dump" => dump_path = Some(PathBuf::from(value()?)),
            other => return Err(format!("unknown argument {other} (see the file header)")),
        }
    }

    let (bundled, refused) = load(&crate_dir.join("../../assets/fonts"));
    if bundled.is_empty() || !refused.is_empty() {
        return Err(format!("the bundled fonts in assets/fonts do not load: {refused:?}"));
    }
    let mut gallery = bundled.clone();
    // Family -> category, from the `METADATA.pb` files of a google/fonts corpus.
    let mut categories: HashMap<String, String> = HashMap::new();
    if let Some(dir) = &corpus {
        let (fonts, refused) = load(dir);
        for reason in &refused {
            let _ = writeln!(std::io::stderr(), "refused: {reason}");
        }
        for (meta, _) in &fonts {
            if let Some(category) = category_next_to(&dir.join(&meta.source_url)) {
                categories.entry(meta.family.clone()).or_insert(category);
            }
        }
        for (meta, font) in fonts {
            if !gallery.iter().any(|(m, _)| m.family == meta.family && m.style == meta.style) {
                gallery.push((meta, font));
            }
        }
    }
    let cover_fonts = if only_bundled_covers { bundled.clone() } else { gallery.clone() };

    let start = Instant::now();
    let clock = move || start.elapsed().as_secs_f64() * 1000.0;
    let t0 = clock();
    let index = FontIndex::build_checked(&gallery).map_err(|e| e.to_string())?;
    let build_ms = clock() - t0;
    let bytes = index.to_bytes();
    let families = {
        let mut f: Vec<&str> = index.entries().iter().map(|e| e.family.as_str()).collect();
        f.sort_unstable();
        f.dedup();
        f.len()
    };

    let mut out = std::io::stdout().lock();
    let mut line = |s: String| writeln!(out, "{s}").map_err(|e| e.to_string());
    line(format!(
        "Gallery: {} faces in {} families ({} bundled); covers drawn with {} faces; index {} bytes, built in {:.0} ms ({:.1} ms per face).",
        index.len(),
        families,
        bundled.len(),
        cover_fonts.len(),
        bytes.len(),
        build_ms,
        build_ms / index.len() as f64
    ))?;
    let ofl = index.entries().iter().filter(|e| e.licence == "OFL-1.1").count();
    line(format!(
        "Licences (from the licence file next to each font): {ofl} OFL-1.1, {} Apache-2.0.",
        index.len() - ofl
    ))?;
    line(format!(
        "Covers: 600 x 900 px, {cases} per condition, seed {seed}; rerank on the best {rerank_top}.\n"
    ))?;

    let dump = std::cell::RefCell::new(String::new());
    let observer = |r: &CaseReport| {
        if dump_path.is_some() {
            dump.borrow_mut().push_str(&format!(
                "{}\t{}\t{}\t{}\t{:?}\t{:.4}\t{:.4}\t{}\t{}\n",
                r.condition,
                r.case,
                r.font,
                r.labelled_glyphs,
                r.id_rank,
                r.true_score,
                r.other_family_score,
                r.confident,
                r.detected,
            ));
        }
        let wrong = !r.detected || r.id_rank != Some(0) || r.auto_rank != Some(0);
        if verbose && wrong {
            let _ = writeln!(
                std::io::stderr(),
                "[{}] #{} {} \"{}\" cap {:.0}px: detected={} truth={:?} found={:?} labelled={} id_rank={:?} rerank={:?} auto_rank={:?} best={} true_score={:.2} confident={}",
                r.condition,
                r.case,
                r.font,
                r.title,
                r.cap_height_px,
                r.detected,
                (r.truth.x, r.truth.y, r.truth.w, r.truth.h),
                r.found.map(|f| (f.x, f.y, f.w, f.h)),
                r.labelled_glyphs,
                r.id_rank,
                r.rerank_rank,
                r.auto_rank,
                r.best,
                r.true_score,
                r.confident,
            );
        }
    };
    let opts = EvalOptions {
        cases,
        seed,
        width: 600,
        height: 900,
        rerank_top,
        clock: Some(&clock),
        observer: Some(&observer),
    };
    let conditions: Vec<Condition> = standard_conditions()
        .into_iter()
        .filter(|c| only.as_ref().is_none_or(|o| o.split(',').any(|part| c.name.contains(part))))
        .collect();
    let rows = evaluate(&cover_fonts, &index, &conditions, &opts);
    line(format_table(&rows))?;
    if let Some(path) = &dump_path {
        std::fs::write(path, dump.borrow().as_bytes())
            .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    if tables_only {
        return Ok(());
    }

    let holdout: Vec<_> = [Condition::clean(), Condition::typical()]
        .iter()
        .map(|c| evaluate_holdout(&cover_fonts, &index, c, &opts))
        .collect();
    line("Hold-out (true face removed from the index; title box given):\n".to_string())?;
    line(format_holdout(&holdout))?;
    if !categories.is_empty() {
        let category = |family: &str| categories.get(family).cloned();
        let rows: Vec<_> = [Condition::clean(), Condition::typical()]
            .iter()
            .map(|c| evaluate_family_holdout(&cover_fonts, &index, c, &opts, &category))
            .collect();
        line(
            "Family hold-out (every face of the true family removed; title box given; categories from google/fonts METADATA.pb):\n"
                .to_string(),
        )?;
        line(format_category_holdout(&rows))?;
    }

    // Scaling: one typical cover against galleries of growing size (entries repeated).
    let spec = CoverSpec {
        width: 600,
        height: 900,
        title_lines: vec!["Voyage au bout".into(), "de la nuit".into()],
        cap_height_px: 44.0,
        letter_spacing_em: 0.02,
        light_on_dark: false,
        background: Background::Gradient,
        author: Some("Camille Vasseur".into()),
        ornaments: true,
        degradation: Degradation {
            rotation_degrees: 0.7,
            blur_sigma: 0.6,
            noise_sigma: 3.0,
            jpeg_quality: Some(75),
        },
        seed,
    };
    let (_, title_font) = &bundled[bundled.len().min(4) - 1];
    let (_, author_font) = &bundled[0];
    if let Some(cover) = render_cover(&spec, title_font, author_font) {
        let title = "Voyage au bout de la nuit";
        line("Scaling of identify() on one 600 x 900 cover (title box given; entries repeated to reach the size):\n".to_string())?;
        line("| Faces in the index | detect ms | identify ms |\n|---|---|---|".to_string())?;
        for size in [index.len(), 100, 300, 1000] {
            let entries: Vec<IndexEntry> =
                index.entries().iter().cycle().take(size).cloned().collect();
            let big = FontIndex::from_entries(entries);
            let runs = 10;
            let t0 = clock();
            for _ in 0..runs {
                let _ = find_title_regions(&cover.image, title);
            }
            let detect = (clock() - t0) / f64::from(runs);
            let t0 = clock();
            for _ in 0..runs {
                let _ = identify_detailed(&cover.image, &cover.title_region, Some(title), &big);
            }
            let identify = (clock() - t0) / f64::from(runs);
            line(format!("| {size} | {detect:.1} | {identify:.1} |"))?;
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            let _ = writeln!(std::io::stderr(), "eval: {message}");
            ExitCode::FAILURE
        }
    }
}
