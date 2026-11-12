//! Evaluation harness on synthetic covers (feature `synth`).
//!
//! For each *condition* (a recipe of sizes, backgrounds and degradations) covers are
//! synthesised with every test font in turn and pushed through the pipeline three ways:
//! with the true title box (what a user would draw), with the box found by the
//! detector, and without the true font in the index (hold-out). The numbers are honest
//! about what they are: the true font is in the index unless stated, the index glyphs
//! and the covers come from the same rasteriser (at other sizes, supersampled, shaped
//! as words and degraded), and the covers are synthetic. See
//! `docs/research/fontid-eval.md` for how to read them.

use ebc_text::Font;

use crate::detect::{TextRegion, find_text_regions, find_title_regions};
use crate::identify::{Identification, identify_detailed};
use crate::index::{FontIndex, IndexEntry, IndexEntryMeta};
use crate::rerank::rerank_detailed;
use crate::synth::{Background, CoverSpec, Degradation, Rng, SynthCover, render_cover};

/// Titles used on the synthetic covers, in catalogue case (French and English, with
/// accents, digits and punctuation).
pub const TITLES: [&str; 28] = [
    "Le Rouge et le Noir",
    "Bonjour tristesse",
    "L'Étranger",
    "Voyage au bout de la nuit",
    "Les Misérables",
    "Mémoires d'Hadrien",
    "La Peste",
    "Du côté de chez Swann",
    "Germinal",
    "Vingt mille lieues sous les mers",
    "Le Petit Prince",
    "Zazie dans le métro",
    "Pride and Prejudice",
    "The Great Gatsby",
    "Brave New World",
    "Wuthering Heights",
    "Moby Dick",
    "Fahrenheit 451",
    "Jane Eyre",
    "The Quiet Kingdom",
    "Dracula",
    "Emma",
    "Ulysses",
    "Quatrevingt-treize",
    "Où es-tu ?",
    "1984",
    "Middlemarch",
    "Regain",
];

/// Author lines for the synthetic covers (fictional names).
pub const AUTHORS: [&str; 6] = [
    "Camille Vasseur",
    "J. R. Hartwell",
    "Élise Montfort",
    "Thomas Aldridge",
    "Marguerite Dervaux",
    "N. K. Osei",
];

/// A recipe for synthetic covers: every range is sampled uniformly per cover.
#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    /// Name shown in the table.
    pub name: String,
    /// Height of the title's capital letters, in pixels (for a 600 x 900 cover; scaled
    /// with the cover height).
    pub cap_height: (f32, f32),
    /// Backgrounds to choose from.
    pub backgrounds: Vec<Background>,
    /// Blur standard deviation in pixels.
    pub blur: (f32, f32),
    /// Noise standard deviation in grey levels.
    pub noise: (f32, f32),
    /// Largest rotation in degrees (sampled in `-max..max`).
    pub rotation: f32,
    /// JPEG quality range, if the cover is compressed.
    pub jpeg: Option<(u8, u8)>,
    /// Letter-spacing of the title in em.
    pub spacing: (f32, f32),
    /// Share of covers whose title is set in capitals (the pipeline is always given the
    /// catalogue case).
    pub upper_share: f32,
    /// Break titles of two or more words into two or three lines.
    pub multi_line: bool,
    /// Give the pipeline the title (`false` exercises the unlabelled fallback).
    pub with_text: bool,
}

impl Condition {
    /// Clean covers: flat background, no degradation, 30–70 px capitals.
    #[must_use]
    pub fn clean() -> Self {
        Self {
            name: "clean".into(),
            cap_height: (30.0, 70.0),
            backgrounds: vec![Background::Flat],
            blur: (0.0, 0.0),
            noise: (0.0, 0.0),
            rotation: 0.0,
            jpeg: None,
            spacing: (0.0, 0.0),
            upper_share: 0.3,
            multi_line: false,
            with_text: true,
        }
    }

    /// "Typical web cover": flat, graded or textured background, 26–60 px capitals,
    /// blur 0.4–0.9, noise 2–5, rotation up to 1°, JPEG quality 60–85, letter-spacing
    /// −0.01..0.05 em, half of the multi-word titles on several lines.
    #[must_use]
    pub fn typical() -> Self {
        Self {
            name: "typical (degraded)".into(),
            cap_height: (26.0, 60.0),
            backgrounds: vec![Background::Flat, Background::Gradient, Background::Textured],
            blur: (0.4, 0.9),
            noise: (2.0, 5.0),
            rotation: 1.0,
            jpeg: Some((60, 85)),
            spacing: (-0.01, 0.05),
            ..Self::clean()
        }
    }

    /// Hard covers: textured background, 16–30 px capitals, blur 0.8–1.4, noise 6–12,
    /// rotation up to 2°, JPEG quality 35–60, letter-spacing 0..0.12 em.
    #[must_use]
    pub fn hard() -> Self {
        Self {
            name: "hard (degraded)".into(),
            cap_height: (16.0, 30.0),
            backgrounds: vec![Background::Gradient, Background::Textured],
            blur: (0.8, 1.4),
            noise: (6.0, 12.0),
            rotation: 2.0,
            jpeg: Some((35, 60)),
            spacing: (0.0, 0.12),
            ..Self::clean()
        }
    }

    fn named(mut self, name: &str) -> Self {
        self.name = name.into();
        self
    }

    /// A number that identifies how covers are drawn — everything but the name and
    /// whether the title is given, so that two conditions that differ only in what the
    /// pipeline is told are measured on the same covers.
    fn drawing_key(&self) -> u64 {
        let recipe = Self { name: String::new(), with_text: true, ..self.clone() };
        hash(&format!("{recipe:?}"))
    }
}

/// The conditions of the standard table: clean, typical, hard, then one factor at a
/// time on top of clean.
#[must_use]
pub fn standard_conditions() -> Vec<Condition> {
    let clean = Condition::clean;
    vec![
        clean(),
        Condition::typical(),
        Condition::hard(),
        Condition { blur: (1.0, 1.4), ..clean() }.named("clean + blur 1.0–1.4"),
        Condition { noise: (8.0, 12.0), ..clean() }.named("clean + noise 8–12"),
        Condition { rotation: 2.0, ..clean() }.named("clean + rotation ±2°"),
        Condition { jpeg: Some((30, 45)), ..clean() }.named("clean + JPEG q30–45"),
        Condition { cap_height: (14.0, 20.0), ..clean() }.named("clean + small text 14–20 px"),
        Condition { backgrounds: vec![Background::Textured], ..clean() }
            .named("clean + textured background"),
        Condition { spacing: (0.10, 0.20), ..clean() }.named("clean + tracking 0.10–0.20 em"),
        Condition { spacing: (-0.06, -0.03), ..clean() }.named("clean + tight tracking −0.06..−0.03"),
        Condition { multi_line: true, ..clean() }.named("clean + multi-line"),
        Condition { upper_share: 1.0, ..clean() }.named("clean + ALL CAPS on cover"),
        Condition { with_text: false, ..clean() }.named("clean, title unknown"),
        Condition { with_text: false, ..Condition::typical() }.named("typical, title unknown"),
    ]
}

/// How an evaluation is run.
#[derive(Clone, Copy)]
pub struct EvalOptions<'a> {
    /// Covers per condition (fonts are used in turn).
    pub cases: usize,
    /// Seed of everything random.
    pub seed: u64,
    /// Cover width in pixels.
    pub width: u32,
    /// Cover height in pixels.
    pub height: u32,
    /// How many of the best descriptor matches are passed to the render-and-compare
    /// stage.
    pub rerank_top: usize,
    /// A clock in milliseconds, for timings (`None`: timings stay 0). The library never
    /// reads the time itself.
    pub clock: Option<&'a dyn Fn() -> f64>,
    /// Called once per cover with what happened to it (for inspecting failures).
    pub observer: Option<&'a dyn Fn(&CaseReport)>,
}

/// What happened to one synthetic cover.
#[derive(Debug, Clone, PartialEq)]
pub struct CaseReport {
    /// Name of the condition.
    pub condition: String,
    /// Number of the cover within the condition.
    pub case: usize,
    /// Family and style of the font the title was set in.
    pub font: String,
    /// The title in catalogue case.
    pub title: String,
    /// Height of the capitals as drawn, in pixels.
    pub cap_height_px: f32,
    /// The true title box.
    pub truth: TextRegion,
    /// The detector's first region.
    pub found: Option<TextRegion>,
    /// Whether `found` counts as the title.
    pub detected: bool,
    /// Glyphs labelled with the true box.
    pub labelled_glyphs: usize,
    /// Rank of the true face with the true box (0 = first).
    pub id_rank: Option<usize>,
    /// Rank of the true face after render-and-compare.
    pub rerank_rank: Option<usize>,
    /// Rank of the true face with the detector's box.
    pub auto_rank: Option<usize>,
    /// Best match with the true box: family, style and score.
    pub best: String,
    /// Score of the true face with the true box.
    pub true_score: f32,
    /// The identification with the true box was flagged confident.
    pub confident: bool,
}

impl std::fmt::Debug for EvalOptions<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvalOptions")
            .field("cases", &self.cases)
            .field("seed", &self.seed)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("rerank_top", &self.rerank_top)
            .field("clock", &self.clock.is_some())
            .field("observer", &self.observer.is_some())
            .finish()
    }
}

impl Default for EvalOptions<'_> {
    fn default() -> Self {
        Self {
            cases: 64,
            seed: 2026,
            width: 600,
            height: 900,
            rerank_top: 5,
            clock: None,
            observer: None,
        }
    }
}

/// Counts for one condition. "Face" means the exact font (family and style), "family"
/// the family alone.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Row {
    /// Name of the condition.
    pub name: String,
    /// Covers drawn.
    pub cases: usize,
    /// Whether the title was given to the pipeline.
    pub with_text: bool,
    /// True box: glyphs were labelled with the title's characters.
    pub labelled: usize,
    /// True box, descriptor stage: the true face is first.
    pub id_top1: usize,
    /// True box, descriptor stage: the true face is among the first three.
    pub id_top3: usize,
    /// True box, descriptor stage: the first match is of the true family.
    pub id_family: usize,
    /// True box, after render-and-compare: the true face is first.
    pub rr_top1: usize,
    /// True box, after render-and-compare: the true face is among the first three.
    pub rr_top3: usize,
    /// True box, after render-and-compare: the first match is of the true family.
    pub rr_family: usize,
    /// The detector's first region is the title (see [`is_detected`]).
    pub detected: usize,
    /// Detector's box, descriptor stage: the true face is first.
    pub auto_top1: usize,
    /// Detector's box, descriptor stage: the true face is among the first three.
    pub auto_top3: usize,
    /// Detector's box, descriptor stage: the first match is of the true family.
    pub auto_family: usize,
    /// True box: the identification was flagged confident.
    pub confident: usize,
    /// True box: flagged confident and the first match is of the true family.
    pub confident_right: usize,
    /// Total time in the detector, in milliseconds.
    pub ms_detect: f64,
    /// Total time in `identify` (true box), in milliseconds.
    pub ms_identify: f64,
    /// Total time in `rerank` (true box), in milliseconds.
    pub ms_rerank: f64,
}

/// Result of the hold-out run: the true face is removed from the index; does the best
/// remaining match belong to the true family (a sibling weight or style)?
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Holdout {
    /// Name of the condition.
    pub name: String,
    /// Covers evaluated (only fonts whose family has another face in the index).
    pub cases: usize,
    /// The best match is a sibling of the removed face.
    pub sibling_top1: usize,
    /// A sibling is among the first three.
    pub sibling_top3: usize,
    /// Expected `sibling_top1 / cases` of a random ranking.
    pub chance: f32,
}

/// Is `found` the title? Yes when it overlaps the true box with an
/// intersection-over-union of at least 0.5, or when it lies inside the true box and
/// spans at least 30 % of it (one line of a multi-line title).
#[must_use]
pub fn is_detected(found: &TextRegion, truth: &TextRegion) -> bool {
    let (fx1, fy1) = (found.x.saturating_add(found.w), found.y.saturating_add(found.h));
    let (tx1, ty1) = (truth.x.saturating_add(truth.w), truth.y.saturating_add(truth.h));
    let iw = fx1.min(tx1).saturating_sub(found.x.max(truth.x)) as f32;
    let ih = fy1.min(ty1).saturating_sub(found.y.max(truth.y)) as f32;
    let inter = iw * ih;
    let (fa, ta) = (found.w as f32 * found.h as f32, truth.w as f32 * truth.h as f32);
    if inter <= 0.0 || fa <= 0.0 || ta <= 0.0 {
        return false;
    }
    inter / (fa + ta - inter) >= 0.5 || (inter / fa >= 0.9 && inter / ta >= 0.3)
}

/// One synthetic test case.
struct Case {
    cover: SynthCover,
    title: String,
    font: usize,
}

/// Draws case `i` of a condition. `None` if no title can be drawn with the font.
fn make_case(
    fonts: &[(IndexEntryMeta, Font)],
    condition: &Condition,
    opts: &EvalOptions<'_>,
    i: usize,
) -> Option<Case> {
    let n = fonts.len();
    let font = i % n;
    let mut rng =
        Rng::new(opts.seed ^ (i as u64).wrapping_mul(0x9e37_79b9) ^ condition.drawing_key());
    let (_, title_font) = &fonts[font];
    let (_, author_font) = &fonts[(font + 1 + rng.below(n.max(2) - 1)) % n];
    // A title the font can draw.
    let mut title = None;
    for _ in 0..8 {
        let candidate = TITLES[rng.below(TITLES.len())];
        if title_font.missing_chars(candidate).is_empty() {
            title = Some(candidate);
            break;
        }
    }
    let title = title?;
    let shown =
        if rng.chance(condition.upper_share) { title.to_uppercase() } else { title.to_string() };
    let words: Vec<&str> = shown.split_whitespace().collect();
    let title_lines: Vec<String> = if condition.multi_line && words.len() >= 2 {
        let lines = if words.len() >= 4 { 2 + rng.below(2) } else { 2 };
        let per = words.len().div_ceil(lines);
        words.chunks(per).map(|c| c.join(" ")).collect()
    } else {
        vec![shown.clone()]
    };
    let scale = opts.height as f32 / 900.0;
    let jpeg_quality =
        condition.jpeg.map(|(lo, hi)| rng.range(f32::from(lo), f32::from(hi) + 1.0) as u8);
    let author = AUTHORS[rng.below(AUTHORS.len())];
    let spec = CoverSpec {
        width: opts.width,
        height: opts.height,
        title_lines,
        cap_height_px: rng.range(condition.cap_height.0, condition.cap_height.1) * scale,
        letter_spacing_em: rng.range(condition.spacing.0, condition.spacing.1),
        light_on_dark: rng.chance(0.5),
        background: condition.backgrounds[rng.below(condition.backgrounds.len())],
        author: author_font.missing_chars(author).is_empty().then(|| author.to_string()),
        ornaments: rng.chance(0.5),
        degradation: Degradation {
            rotation_degrees: rng.range(-condition.rotation, condition.rotation),
            blur_sigma: rng.range(condition.blur.0, condition.blur.1),
            noise_sigma: rng.range(condition.noise.0, condition.noise.1),
            jpeg_quality,
        },
        seed: rng.next_u64(),
    };
    let cover = render_cover(&spec, title_font, author_font)?;
    Some(Case { cover, title: title.to_string(), font })
}

fn hash(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3))
}

fn is_entry(entry: &IndexEntry, meta: &IndexEntryMeta) -> bool {
    entry.family == meta.family && entry.style == meta.style
}

/// Position of the true face in a result (`None` if absent).
fn rank_of(result: &Identification, meta: &IndexEntryMeta) -> Option<usize> {
    result.matches.iter().position(|m| is_entry(&m.entry, meta))
}

fn family_first(result: &Identification, meta: &IndexEntryMeta) -> bool {
    result.matches.first().is_some_and(|m| m.entry.family == meta.family)
}

/// Runs the conditions. `fonts` are the fonts covers are drawn with; every one of them
/// must be in `index` under the same family and style (else it can never be found and
/// counts as a miss). Fonts are matched with index entries by family and style.
#[must_use]
pub fn evaluate(
    fonts: &[(IndexEntryMeta, Font)],
    index: &FontIndex,
    conditions: &[Condition],
    opts: &EvalOptions<'_>,
) -> Vec<Row> {
    let now = || opts.clock.map_or(0.0, |c| c());
    let mut rows = Vec::with_capacity(conditions.len());
    if fonts.is_empty() {
        return rows;
    }
    for condition in conditions {
        let mut row =
            Row { name: condition.name.clone(), with_text: condition.with_text, ..Row::default() };
        for i in 0..opts.cases {
            let Some(case) = make_case(fonts, condition, opts, i) else { continue };
            let (meta, _) = &fonts[case.font];
            let image = &case.cover.image;
            let text = condition.with_text.then_some(case.title.as_str());
            row.cases += 1;

            // --- the true box ------------------------------------------------------------
            let t0 = now();
            let id = identify_detailed(image, &case.cover.title_region, text, index);
            row.ms_identify += now() - t0;
            let rank = rank_of(&id, meta);
            let mut report = CaseReport {
                condition: condition.name.clone(),
                case: i,
                font: format!("{} {}", meta.family, meta.style),
                title: case.title.clone(),
                cap_height_px: case.cover.cap_height_px,
                truth: case.cover.title_region,
                found: None,
                detected: false,
                labelled_glyphs: id.labelled_glyphs,
                id_rank: rank,
                rerank_rank: None,
                auto_rank: None,
                best: id.matches.first().map_or_else(String::new, |m| {
                    format!("{} {} {:.2}", m.entry.family, m.entry.style, m.score)
                }),
                true_score: rank.map_or(0.0, |r| id.matches[r].score),
                confident: id.confident,
            };
            row.labelled += usize::from(id.labelled_glyphs > 0);
            row.id_top1 += usize::from(rank == Some(0));
            row.id_top3 += usize::from(rank.is_some_and(|r| r < 3));
            let family_right = family_first(&id, meta);
            row.id_family += usize::from(family_right);
            row.confident += usize::from(id.confident);
            row.confident_right += usize::from(id.confident && family_right);

            // --- render-and-compare on the best descriptor matches ---------------------------
            if let Some(text) = text {
                let candidates: Vec<(IndexEntry, Font)> = id
                    .matches
                    .iter()
                    .take(opts.rerank_top)
                    .filter_map(|m| {
                        fonts
                            .iter()
                            .find(|(meta, _)| is_entry(&m.entry, meta))
                            .map(|(_, font)| (m.entry.clone(), font.clone()))
                    })
                    .collect();
                let t0 = now();
                let rr = rerank_detailed(image, &case.cover.title_region, text, &candidates);
                row.ms_rerank += now() - t0;
                let rank = rank_of(&rr, meta);
                report.rerank_rank = rank;
                row.rr_top1 += usize::from(rank == Some(0));
                row.rr_top3 += usize::from(rank.is_some_and(|r| r < 3));
                row.rr_family += usize::from(family_first(&rr, meta));
            }

            // --- the detector's box -------------------------------------------------------
            let t0 = now();
            let regions = match text {
                Some(text) => find_title_regions(image, text),
                None => find_text_regions(image),
            };
            row.ms_detect += now() - t0;
            if let Some(found) = regions.first() {
                report.found = Some(*found);
                if is_detected(found, &case.cover.title_region) {
                    row.detected += 1;
                    report.detected = true;
                }
                let id = identify_detailed(image, found, text, index);
                let rank = rank_of(&id, meta);
                report.auto_rank = rank;
                row.auto_top1 += usize::from(rank == Some(0));
                row.auto_top3 += usize::from(rank.is_some_and(|r| r < 3));
                row.auto_family += usize::from(family_first(&id, meta));
            }
            if let Some(observer) = opts.observer {
                observer(&report);
            }
        }
        rows.push(row);
    }
    rows
}

/// Runs the hold-out experiment for one condition: for every cover the true face is
/// taken out of the index before identifying (true box, title known).
#[must_use]
pub fn evaluate_holdout(
    fonts: &[(IndexEntryMeta, Font)],
    index: &FontIndex,
    condition: &Condition,
    opts: &EvalOptions<'_>,
) -> Holdout {
    let mut out = Holdout { name: condition.name.clone(), ..Holdout::default() };
    if fonts.is_empty() {
        return out;
    }
    let mut chance = 0.0f32;
    for i in 0..opts.cases {
        let (meta, _) = &fonts[i % fonts.len()];
        let rest: Vec<IndexEntry> =
            index.entries().iter().filter(|e| !is_entry(e, meta)).cloned().collect();
        let siblings = rest.iter().filter(|e| e.family == meta.family).count();
        if siblings == 0 || rest.len() < 2 {
            continue;
        }
        let Some(case) = make_case(fonts, condition, opts, i) else { continue };
        let reduced = FontIndex::from_entries(rest.iter().cloned());
        let id =
            identify_detailed(&case.cover.image, &case.cover.title_region, Some(&case.title), &reduced);
        out.cases += 1;
        chance += siblings as f32 / rest.len() as f32;
        out.sibling_top1 += usize::from(family_first(&id, meta));
        out.sibling_top3 +=
            usize::from(id.matches.iter().take(3).any(|m| m.entry.family == meta.family));
    }
    out.chance = if out.cases > 0 { chance / out.cases as f32 } else { 0.0 };
    out
}

fn percent(count: usize, total: usize) -> String {
    if total == 0 { "–".to_string() } else { format!("{:.0} %", 100.0 * count as f64 / total as f64) }
}

/// The rows as a Markdown table (accuracy in percent of the covers drawn, mean
/// timings in milliseconds per cover).
#[must_use]
pub fn format_table(rows: &[Row]) -> String {
    let mut out = String::from(
        "| Condition | n | Labelled | Box: top-1 | Box: top-3 | Box: family | Rerank: top-1 | Rerank: top-3 | Rerank: family | Detected | Auto: top-1 | Auto: top-3 | Auto: family | Confident (right) | detect ms | identify ms | rerank ms |\n\
         |---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n",
    );
    for r in rows {
        let n = r.cases;
        let rr = |count: usize| if r.with_text { percent(count, n) } else { "–".to_string() };
        let ms = |total: f64| if n == 0 { "–".to_string() } else { format!("{:.1}", total / n as f64) };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} ({}) | {} | {} | {} |\n",
            r.name,
            n,
            percent(r.labelled, n),
            percent(r.id_top1, n),
            percent(r.id_top3, n),
            percent(r.id_family, n),
            rr(r.rr_top1),
            rr(r.rr_top3),
            rr(r.rr_family),
            percent(r.detected, n),
            percent(r.auto_top1, n),
            percent(r.auto_top3, n),
            percent(r.auto_family, n),
            percent(r.confident, n),
            percent(r.confident_right, r.confident),
            ms(r.ms_detect),
            ms(r.ms_identify),
            if r.with_text { ms(r.ms_rerank) } else { "–".to_string() },
        ));
    }
    out
}

/// The hold-out results as a Markdown table.
#[must_use]
pub fn format_holdout(rows: &[Holdout]) -> String {
    let mut out = String::from(
        "| Condition | n | Sibling is top-1 | Sibling in top-3 | Chance (top-1) |\n|---|---|---|---|---|\n",
    );
    for r in rows {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {:.0} % |\n",
            r.name,
            r.cases,
            percent(r.sibling_top1, r.cases),
            percent(r.sibling_top3, r.cases),
            100.0 * r.chance,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detection_criterion() {
        let truth = TextRegion { x: 100, y: 100, w: 300, h: 100, score: 1.0 };
        let r = |x, y, w, h| TextRegion { x, y, w, h, score: 0.5 };
        assert!(is_detected(&truth, &truth));
        assert!(is_detected(&r(95, 95, 310, 110), &truth));
        assert!(is_detected(&r(100, 100, 300, 45), &truth), "one line of two");
        assert!(!is_detected(&r(100, 100, 60, 45), &truth), "a single word of one line");
        assert!(!is_detected(&r(100, 300, 300, 100), &truth));
        assert!(!is_detected(&r(0, 0, 600, 900), &truth), "the whole cover is not a detection");
        assert!(!is_detected(&r(0, 0, 0, 0), &truth));
        assert!(!is_detected(&r(u32::MAX, u32::MAX, u32::MAX, u32::MAX), &truth));
    }

    #[test]
    fn tables_render_without_cases() {
        let table = format_table(&[Row { name: "empty".into(), ..Row::default() }]);
        assert!(table.contains("| empty | 0 | – |"));
        assert!(format_holdout(&[Holdout::default()]).contains("| 0 | – | – |"));
        assert!(evaluate(&[], &FontIndex::default(), &standard_conditions(), &EvalOptions::default())
            .is_empty());
    }
}
