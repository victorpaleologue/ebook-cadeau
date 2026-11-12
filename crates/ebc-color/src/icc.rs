//! A small colour-management module: sRGB → the CMYK space of a printer's ICC profile.
//!
//! # What it does
//!
//! [`CmykProfile::parse`] reads a CMYK output profile (ICC version 2 or 4, class `prtr`,
//! or any class that carries the tables below) whose transforms are stored as
//! `lut16Type` (`mft2`) or `lut8Type` (`mft1`) tables — the form of every ECI/FOGRA
//! offset profile and of the bundled basICColor "ISO Coated v2 300%" (FOGRA39L). It
//! keeps the PCS → CMYK tables (`B2A0` perceptual, `B2A1` relative colorimetric) and the
//! CMYK → PCS tables (`A2B0`, `A2B1`).
//!
//! [`SrgbToCmyk::new`] links sRGB (IEC 61966-2-1, adapted to D50 with the Bradford
//! matrix, as in the ICC's sRGB profiles) to such a profile with one rendering intent,
//! optionally with black-point compensation, and samples the result on a 33 × 33 × 33
//! grid — what Little CMS does for an 8-bit RGB → CMYK transform. Pixels and flat
//! colours are then interpolated tetrahedrally on that grid, so a cover image and a flat
//! ground colour taken from it go through exactly the same transform.
//!
//! The default is the one `docs/research/03-print-requirements.md` §3.4 asks for:
//! **relative colorimetric with black-point compensation** (the black point is detected
//! with the algorithm Little CMS uses for LUT-based CMYK output profiles).
//!
//! # Accuracy
//!
//! Checked against Little CMS 2.19 (through Pillow's `ImageCms`) with the bundled
//! FOGRA39L profile on a 9 × 9 × 9 sRGB lattice and the colours of the test suite: the
//! CMYK values differ by at most a few per cent of a channel, and the two results differ
//! by ΔE2000 well under 1 once rendered back to Lab through the profile's `A2B1` table
//! (`tests/icc_reference.rs`). This is a reimplementation, not a certified CMM.
//!
//! # Not supported
//!
//! * `lutAtoBType` / `lutBtoAType` (`mAB` / `mBA`) tables, used by ICC v4 profiles such
//!   as some PSO v3 builds: [`IccError::UnsupportedTag`]. The caller then falls back to
//!   tagged RGB where its profile allows RGB, and refuses otherwise.
//! * Absolute colorimetric and saturation intents, `gamt`, and embedded profiles of
//!   source images (sources are always sRGB).
//!
//! # Determinism
//!
//! Only `+ − × ÷` are used (the sRGB curve and the Lab cube root are the Newton iterations
//! of this crate), so the same input gives the same bytes natively and in WebAssembly.

use ebc_scene::Color;

use crate::Rgb8;
use crate::lab::{lab_f, lab_f_inv, srgb_eotf, srgb_to_linear};

/// Largest profile accepted, in bytes. FOGRA profiles are 1–3 MB.
pub const MAX_PROFILE_BYTES: usize = 16 * 1024 * 1024;
/// Grid points per axis of the sampled sRGB → CMYK transform (as Little CMS).
pub const GRID_POINTS: usize = 33;

/// The PCS white, D50, as Little CMS writes it.
const D50: [f64; 3] = [0.9642, 1.0, 0.8249];
/// Linear sRGB → XYZ (D50), the colorants of Little CMS's built-in sRGB profile
/// (Bradford-adapted, quantised to `s15Fixed16` as stored in the profile).
const SRGB_TO_XYZ_D50: [[f64; 3]; 3] = [
    [0.436_035_156_25, 0.385_116_577_148_437_5, 0.143_051_147_460_937_5],
    [0.222_488_403_320_312_5, 0.716_903_686_523_437_5, 0.060_607_910_156_25],
    [0.013_916_015_625, 0.097_061_157_226_562_5, 0.713_912_963_867_187_5],
];
/// Largest number of grid points per axis accepted in a profile's colour lookup table.
const MAX_CLUT_POINTS: usize = 255;
/// Largest table (input or output curve) length accepted in a `lut16Type`.
const MAX_CURVE_ENTRIES: usize = 4096;

/// Why a profile cannot be used for a conversion.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum IccError {
    /// The data is not an ICC profile (too short, no `acsp`, inconsistent sizes).
    #[error("not an ICC profile: {0}")]
    Malformed(&'static str),
    /// The profile's data colour space is not CMYK.
    #[error("the profile's colour space is not CMYK")]
    NotCmyk,
    /// The profile's PCS is neither Lab nor XYZ.
    #[error("the profile's connection space is neither Lab nor XYZ")]
    UnsupportedPcs,
    /// A tag needed for the conversion is missing.
    #[error("the profile has no {0} table")]
    MissingTag(&'static str),
    /// A tag needed for the conversion uses a type this module does not read.
    #[error("the profile's {tag} table is of type '{kind}', which is not supported")]
    UnsupportedTag {
        /// The tag signature, e.g. `"B2A1"`.
        tag: &'static str,
        /// The tag type signature found, e.g. `"mBA "`.
        kind: String,
    },
}

/// A rendering intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RenderingIntent {
    /// The profile's perceptual tables (`B2A0`).
    Perceptual,
    /// The profile's colorimetric tables (`B2A1`), relative to the paper white.
    #[default]
    RelativeColorimetric,
}

/// The PCS of a profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pcs {
    Lab,
    Xyz,
}

/// One `lut8Type` / `lut16Type` table, normalised to 0–1.
#[derive(Debug, Clone)]
struct Lut {
    inputs: usize,
    outputs: usize,
    /// Applied to XYZ input only (ICC: ignored for Lab input).
    matrix: Option<[[f64; 3]; 3]>,
    input_curves: Vec<Vec<f64>>,
    grid: usize,
    clut: Vec<f64>,
    output_curves: Vec<Vec<f64>>,
    /// `true` for `lut16Type` (16-bit legacy PCS encoding), `false` for `lut8Type`.
    wide: bool,
}

/// A CMYK output profile, ready to convert to and from its PCS.
#[derive(Debug, Clone)]
pub struct CmykProfile {
    pcs: Pcs,
    /// PCS → CMYK, perceptual and relative colorimetric (the second may be the first
    /// when the profile has only `B2A0`).
    to_device: [Lut; 2],
    /// CMYK → PCS, perceptual and relative colorimetric.
    to_pcs: [Lut; 2],
    /// `true` when the profile has its own perceptual tables (`B2A0`).
    has_perceptual: bool,
}

fn be_u16(data: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes([*data.get(at)?, *data.get(at + 1)?]))
}

fn be_u32(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes([
        *data.get(at)?,
        *data.get(at + 1)?,
        *data.get(at + 2)?,
        *data.get(at + 3)?,
    ]))
}

fn be_s15f16(data: &[u8], at: usize) -> Option<f64> {
    be_u32(data, at).map(|v| f64::from(v as i32) / 65536.0)
}

impl CmykProfile {
    /// Parses a CMYK ICC profile. The bytes are untrusted: every offset and size is
    /// checked, and the tables are bounded ([`MAX_PROFILE_BYTES`], 255 grid points).
    ///
    /// # Errors
    /// See [`IccError`].
    pub fn parse(data: &[u8]) -> Result<Self, IccError> {
        if data.len() > MAX_PROFILE_BYTES {
            return Err(IccError::Malformed("the profile is larger than 16 MiB"));
        }
        if data.len() < 132 || data.get(36..40) != Some(b"acsp") {
            return Err(IccError::Malformed("no profile header"));
        }
        if data.get(16..20) != Some(b"CMYK") {
            return Err(IccError::NotCmyk);
        }
        let pcs = match data.get(20..24) {
            Some(b"Lab ") => Pcs::Lab,
            Some(b"XYZ ") => Pcs::Xyz,
            _ => return Err(IccError::UnsupportedPcs),
        };
        let count = be_u32(data, 128).ok_or(IccError::Malformed("truncated tag table"))?;
        let count = usize::try_from(count).unwrap_or(usize::MAX);
        if count > (data.len() - 132) / 12 {
            return Err(IccError::Malformed("the tag table runs past the end"));
        }
        let mut tags: Vec<([u8; 4], &[u8])> = Vec::with_capacity(count);
        for i in 0..count {
            let at = 132 + 12 * i;
            let sig = [data[at], data[at + 1], data[at + 2], data[at + 3]];
            let offset = be_u32(data, at + 4).ok_or(IccError::Malformed("truncated tag"))?;
            let size = be_u32(data, at + 8).ok_or(IccError::Malformed("truncated tag"))?;
            let (offset, size) = (offset as usize, size as usize);
            let body = offset
                .checked_add(size)
                .and_then(|end| data.get(offset..end))
                .ok_or(IccError::Malformed("a tag lies outside the profile"))?;
            tags.push((sig, body));
        }
        let find = |sig: &[u8; 4]| tags.iter().find(|(s, _)| s == sig).map(|(_, body)| *body);
        let table =
            |name: &'static str, inputs: usize, outputs: usize| -> Result<Option<Lut>, IccError> {
                let Some(body) = find(name.as_bytes().try_into().unwrap_or(&[0; 4])) else {
                    return Ok(None);
                };
                Lut::parse(name, body, inputs, outputs).map(Some)
            };
        let b2a0 = table("B2A0", 3, 4)?;
        let b2a1 = table("B2A1", 3, 4)?;
        let a2b0 = table("A2B0", 4, 3)?;
        let a2b1 = table("A2B1", 4, 3)?;
        let has_perceptual = b2a0.is_some();
        let to_device = match (b2a0, b2a1) {
            (Some(p), Some(r)) => [p, r],
            (Some(p), None) => [p.clone(), p],
            (None, Some(r)) => [r.clone(), r],
            (None, None) => return Err(IccError::MissingTag("B2A0")),
        };
        let to_pcs = match (a2b0, a2b1) {
            (Some(p), Some(r)) => [p, r],
            (Some(p), None) => [p.clone(), p],
            (None, Some(r)) => [r.clone(), r],
            (None, None) => return Err(IccError::MissingTag("A2B0")),
        };
        Ok(Self { pcs, to_device, to_pcs, has_perceptual })
    }

    fn lut_to_device(&self, intent: RenderingIntent) -> &Lut {
        match intent {
            RenderingIntent::Perceptual => &self.to_device[0],
            RenderingIntent::RelativeColorimetric => &self.to_device[1],
        }
    }

    fn lut_to_pcs(&self, intent: RenderingIntent) -> &Lut {
        match intent {
            RenderingIntent::Perceptual => &self.to_pcs[0],
            RenderingIntent::RelativeColorimetric => &self.to_pcs[1],
        }
    }

    /// CIELAB (D50) → CMYK (each 0–1) through the profile's `B2A` table for `intent`.
    #[must_use]
    pub fn lab_to_cmyk(&self, lab: [f64; 3], intent: RenderingIntent) -> [f64; 4] {
        let lut = self.lut_to_device(intent);
        let input = match self.pcs {
            Pcs::Lab => encode_lab(lab, lut.wide),
            Pcs::Xyz => encode_xyz(lab_to_xyz(lab)),
        };
        let out = lut.eval(&input);
        [out[0], out[1], out[2], out[3]]
    }

    /// CMYK (each 0–1) → CIELAB (D50) through the profile's `A2B` table for `intent`
    /// (relative colorimetric: the paper white is L\* 100).
    #[must_use]
    pub fn cmyk_to_lab(&self, cmyk: [f64; 4], intent: RenderingIntent) -> [f64; 3] {
        let lut = self.lut_to_pcs(intent);
        let out = lut.eval(&cmyk);
        let pcs = [out[0], out[1], out[2]];
        match self.pcs {
            Pcs::Lab => decode_lab(pcs, lut.wide),
            Pcs::Xyz => xyz_to_lab(decode_xyz(pcs)),
        }
    }

    /// The darkest colour the profile prints under `intent`, as CIE XYZ (D50): Little
    /// CMS's `cmsDetectDestinationBlackPoint` for a LUT-based CMYK output profile.
    fn destination_black_point(&self, intent: RenderingIntent) -> [f64; 3] {
        let initial = match intent {
            RenderingIntent::RelativeColorimetric => {
                // "Black point using perceptual black": Lab 0 through the perceptual
                // tables and back colorimetrically, L* clipped to 50, made neutral.
                if self.has_perceptual {
                    let cmyk = self.lab_to_cmyk([0.0; 3], RenderingIntent::Perceptual);
                    let lab = self.cmyk_to_lab(cmyk, RenderingIntent::RelativeColorimetric);
                    [lab[0].min(50.0), 0.0, 0.0]
                } else {
                    [0.0; 3]
                }
            }
            RenderingIntent::Perceptual => [0.0; 3],
        };
        // Round trip of a neutral ramp: Lab → CMYK (intent) → Lab (colorimetric).
        let round_trip = |l: f64| {
            let cmyk = self.lab_to_cmyk([l, initial[1], initial[2]], intent);
            self.cmyk_to_lab(cmyk, RenderingIntent::RelativeColorimetric)[0]
        };
        let mut in_ramp = [0.0; 256];
        let mut out_ramp = [0.0; 256];
        for (l, (input, output)) in in_ramp.iter_mut().zip(out_ramp.iter_mut()).enumerate() {
            *input = l as f64 * 100.0 / 255.0;
            *output = round_trip(*input);
        }
        for l in (1..255).rev() {
            out_ramp[l] = out_ramp[l].min(out_ramp[l + 1]);
        }
        if out_ramp[0] >= out_ramp[255] {
            return [0.0; 3];
        }
        let (min_l, max_l) = (out_ramp[0], out_ramp[255]);
        if intent == RenderingIntent::RelativeColorimetric {
            let straight = in_ramp
                .iter()
                .zip(&out_ramp)
                .all(|(&i, &o)| i <= min_l + 0.2 * (max_l - min_l) || (i - o).abs() < 4.0);
            if straight {
                return lab_to_xyz(initial);
            }
        }
        let (lo, hi) = match intent {
            RenderingIntent::RelativeColorimetric => (0.1, 0.5),
            RenderingIntent::Perceptual => (0.03, 0.25),
        };
        let points: Vec<(f64, f64)> = in_ramp
            .iter()
            .zip(&out_ramp)
            .map(|(&i, &o)| (i, (o - min_l) / (max_l - min_l)))
            .filter(|&(_, y)| y >= lo && y < hi)
            .collect();
        if points.len() < 3 {
            return [0.0; 3];
        }
        let l = quadratic_root(&points);
        lab_to_xyz([l.max(0.0), initial[1], initial[2]])
    }
}

/// The vertex search of Little CMS's `RootOfLeastSquaresFitQuadraticCurve`: fits
/// `y = a x² + b x + c` and returns the root nearest the shadows, clipped to 0–50.
fn quadratic_root(points: &[(f64, f64)]) -> f64 {
    if points.len() < 4 {
        return 0.0;
    }
    let (mut sx, mut sx2, mut sx3, mut sx4, mut sy, mut syx, mut syx2) =
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for &(x, y) in points {
        let x2 = x * x;
        sx += x;
        sx2 += x2;
        sx3 += x2 * x;
        sx4 += x2 * x2;
        sy += y;
        syx += y * x;
        syx2 += y * x2;
    }
    let n = points.len() as f64;
    let m = [[n, sx, sx2], [sx, sx2, sx3], [sx2, sx3, sx4]];
    let Some([c, b, a]) = solve3(m, [sy, syx, syx2]) else { return 0.0 };
    if a.abs() < 1.0e-10 {
        // Little CMS returns `min(0, max(50, -c / b))` here, which is always 0.
        return 0.0;
    }
    let d = b * b - 4.0 * a * c;
    if d <= 0.0 {
        return 0.0;
    }
    ((-b + sqrt_newton(d)) / (2.0 * a)).clamp(0.0, 50.0)
}

/// Solves `m · x = v` by Cramer's rule; `None` for a singular system.
fn solve3(m: [[f64; 3]; 3], v: [f64; 3]) -> Option<[f64; 3]> {
    let det = |m: &[[f64; 3]; 3]| {
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    };
    let d = det(&m);
    if d.abs() < 1.0e-300 || !d.is_finite() {
        return None;
    }
    let mut out = [0.0; 3];
    for (col, slot) in out.iter_mut().enumerate() {
        let mut mc = m;
        for row in 0..3 {
            mc[row][col] = v[row];
        }
        *slot = det(&mc) / d;
    }
    Some(out)
}

/// Square root by Newton's method (basic operations only, see the module docs).
fn sqrt_newton(x: f64) -> f64 {
    if x.is_nan() || x <= 0.0 {
        return 0.0;
    }
    let mut y = x.max(1.0);
    for _ in 0..2000 {
        let next = 0.5 * (y + x / y);
        if next >= y {
            break;
        }
        y = next;
    }
    y
}

impl Lut {
    fn parse(
        tag: &'static str,
        body: &[u8],
        inputs: usize,
        outputs: usize,
    ) -> Result<Self, IccError> {
        let kind = body.get(0..4).unwrap_or_default();
        let wide = match kind {
            b"mft2" => true,
            b"mft1" => false,
            other => {
                return Err(IccError::UnsupportedTag {
                    tag,
                    kind: String::from_utf8_lossy(other).into_owned(),
                });
            }
        };
        let bad = |why: &'static str| IccError::Malformed(why);
        let (Some(&ins), Some(&outs), Some(&grid)) = (body.get(8), body.get(9), body.get(10))
        else {
            return Err(bad("truncated lookup table"));
        };
        if usize::from(ins) != inputs || usize::from(outs) != outputs {
            return Err(bad("a lookup table has the wrong number of channels"));
        }
        let grid = usize::from(grid);
        if !(2..=MAX_CLUT_POINTS).contains(&grid) {
            return Err(bad("a lookup table has fewer than two grid points"));
        }
        let mut matrix = [[0.0; 3]; 3];
        for (i, slot) in matrix.iter_mut().flatten().enumerate() {
            *slot = be_s15f16(body, 12 + 4 * i).ok_or(bad("truncated lookup table"))?;
        }
        let identity = matrix == [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let (in_entries, out_entries, mut at, sample) = if wide {
            let n_in = usize::from(be_u16(body, 48).ok_or(bad("truncated lookup table"))?);
            let n_out = usize::from(be_u16(body, 50).ok_or(bad("truncated lookup table"))?);
            if !(2..=MAX_CURVE_ENTRIES).contains(&n_in) || !(2..=MAX_CURVE_ENTRIES).contains(&n_out)
            {
                return Err(bad("a lookup table curve has fewer than two entries"));
            }
            (n_in, n_out, 52usize, 2usize)
        } else {
            (256, 256, 48usize, 1usize)
        };
        let read = |at: usize| -> Option<f64> {
            if wide {
                be_u16(body, at).map(|v| f64::from(v) / 65535.0)
            } else {
                body.get(at).map(|&v| f64::from(v) / 255.0)
            }
        };
        let curves =
            |count: usize, entries: usize, at: &mut usize| -> Result<Vec<Vec<f64>>, IccError> {
                let mut out = Vec::with_capacity(count);
                for _ in 0..count {
                    let mut curve = Vec::with_capacity(entries);
                    for _ in 0..entries {
                        curve.push(read(*at).ok_or(bad("truncated lookup table"))?);
                        *at += sample;
                    }
                    out.push(curve);
                }
                Ok(out)
            };
        let input_curves = curves(inputs, in_entries, &mut at)?;
        let cells = (0..inputs)
            .try_fold(1usize, |acc, _| acc.checked_mul(grid))
            .and_then(|n| n.checked_mul(outputs))
            .ok_or(bad("the lookup table is too large"))?;
        if cells.checked_mul(sample).is_none_or(|bytes| bytes > body.len()) {
            return Err(bad("truncated lookup table"));
        }
        let mut clut = Vec::with_capacity(cells);
        for _ in 0..cells {
            clut.push(read(at).ok_or(bad("truncated lookup table"))?);
            at += sample;
        }
        let output_curves = curves(outputs, out_entries, &mut at)?;
        Ok(Self {
            inputs,
            outputs,
            matrix: (!identity).then_some(matrix),
            input_curves,
            grid,
            clut,
            output_curves,
            wide,
        })
    }

    /// Evaluates the table on normalised input (0–1 per channel).
    fn eval(&self, input: &[f64]) -> Vec<f64> {
        let mut x: Vec<f64> = input.iter().map(|&v| clamp01(v)).collect();
        if let (Some(m), 3) = (self.matrix, self.inputs) {
            // Only meaningful for XYZ input; Lab tables carry the identity.
            let v = [x[0], x[1], x[2]];
            for (row, slot) in m.iter().zip(x.iter_mut()) {
                *slot = clamp01(row[0] * v[0] + row[1] * v[1] + row[2] * v[2]);
            }
        }
        for (value, curve) in x.iter_mut().zip(&self.input_curves) {
            *value = curve_eval(curve, *value);
        }
        let mut out = vec![0.0; self.outputs];
        clut_eval(&self.clut, self.grid, self.inputs, self.outputs, &x, &mut out);
        for (value, curve) in out.iter_mut().zip(&self.output_curves) {
            *value = curve_eval(curve, clamp01(*value));
        }
        out
    }
}

fn clamp01(v: f64) -> f64 {
    if v.is_nan() { 0.0 } else { v.clamp(0.0, 1.0) }
}

/// A tabulated curve, linearly interpolated.
fn curve_eval(curve: &[f64], x: f64) -> f64 {
    let last = curve.len().saturating_sub(1);
    if last == 0 {
        return curve.first().copied().unwrap_or(x);
    }
    let pos = clamp01(x) * last as f64;
    let i = (pos as usize).min(last - 1);
    let f = pos - i as f64;
    curve[i] + (curve[i + 1] - curve[i]) * f
}

/// Multilinear table lookup: tetrahedral interpolation on the last three inputs, linear
/// on any input before them (as Little CMS does for 3 and 4 inputs).
fn clut_eval(clut: &[f64], grid: usize, inputs: usize, outputs: usize, x: &[f64], out: &mut [f64]) {
    let max = (grid - 1) as f64;
    match inputs {
        3 => {
            tetrahedral(clut, grid, outputs, 0, [x[0] * max, x[1] * max, x[2] * max], out);
        }
        _ => {
            // First input linear, the rest recursively (inputs >= 4).
            let pos = x[0] * max;
            let i = (pos as usize).min(grid - 2);
            let f = pos - i as f64;
            let stride = grid.pow(u32::try_from(inputs - 1).unwrap_or(0)) * outputs;
            let mut lo = vec![0.0; outputs];
            let mut hi = vec![0.0; outputs];
            let (a, b) = (i * stride, (i + 1) * stride);
            let sub = |base: usize, out: &mut [f64]| {
                if let Some(slice) = clut.get(base..base + stride) {
                    clut_eval(slice, grid, inputs - 1, outputs, &x[1..], out);
                }
            };
            sub(a, &mut lo);
            sub(b, &mut hi);
            for ((o, l), h) in out.iter_mut().zip(&lo).zip(&hi) {
                *o = l + (h - l) * f;
            }
        }
    }
}

/// Tetrahedral interpolation in a 3-input table (`offset` into `clut`), positions in
/// grid units.
fn tetrahedral(
    clut: &[f64],
    grid: usize,
    outputs: usize,
    offset: usize,
    pos: [f64; 3],
    out: &mut [f64],
) {
    let cell = |p: f64| {
        let i = if p.is_nan() { 0 } else { (p.max(0.0) as usize).min(grid - 2) };
        (i, (p - i as f64).clamp(0.0, 1.0))
    };
    let ((x0, rx), (y0, ry), (z0, rz)) = (cell(pos[0]), cell(pos[1]), cell(pos[2]));
    let sx = grid * grid * outputs;
    let sy = grid * outputs;
    let sz = outputs;
    let base = offset + x0 * sx + y0 * sy + z0 * sz;
    let at = |dx: usize, dy: usize, dz: usize, k: usize| {
        clut.get(base + dx * sx + dy * sy + dz * sz + k).copied().unwrap_or(0.0)
    };
    for (k, slot) in out.iter_mut().enumerate().take(outputs) {
        let c0 = at(0, 0, 0, k);
        let (c1, c2, c3);
        if rx >= ry && ry >= rz {
            c1 = at(1, 0, 0, k) - c0;
            c2 = at(1, 1, 0, k) - at(1, 0, 0, k);
            c3 = at(1, 1, 1, k) - at(1, 1, 0, k);
        } else if rx >= rz && rz >= ry {
            c1 = at(1, 0, 0, k) - c0;
            c2 = at(1, 1, 1, k) - at(1, 0, 1, k);
            c3 = at(1, 0, 1, k) - at(1, 0, 0, k);
        } else if rz >= rx && rx >= ry {
            c1 = at(1, 0, 1, k) - at(0, 0, 1, k);
            c2 = at(1, 1, 1, k) - at(1, 0, 1, k);
            c3 = at(0, 0, 1, k) - c0;
        } else if ry >= rx && rx >= rz {
            c1 = at(1, 1, 0, k) - at(0, 1, 0, k);
            c2 = at(0, 1, 0, k) - c0;
            c3 = at(1, 1, 1, k) - at(1, 1, 0, k);
        } else if ry >= rz && rz >= rx {
            c1 = at(1, 1, 1, k) - at(0, 1, 1, k);
            c2 = at(0, 1, 0, k) - c0;
            c3 = at(0, 1, 1, k) - at(0, 1, 0, k);
        } else {
            c1 = at(1, 1, 1, k) - at(0, 1, 1, k);
            c2 = at(0, 1, 1, k) - at(0, 0, 1, k);
            c3 = at(0, 0, 1, k) - c0;
        }
        *slot = c0 + c1 * rx + c2 * ry + c3 * rz;
    }
}

/// CIELAB (D50) → the legacy (ICC v2) PCS encoding of `lut16Type`, or the 8-bit one.
fn encode_lab([l, a, b]: [f64; 3], wide: bool) -> [f64; 3] {
    if wide {
        [l * 652.80 / 65535.0, (a + 128.0) * 256.0 / 65535.0, (b + 128.0) * 256.0 / 65535.0]
    } else {
        [l / 100.0, (a + 128.0) / 255.0, (b + 128.0) / 255.0]
    }
}

fn decode_lab([l, a, b]: [f64; 3], wide: bool) -> [f64; 3] {
    if wide {
        [l * 65535.0 / 652.80, a * 65535.0 / 256.0 - 128.0, b * 65535.0 / 256.0 - 128.0]
    } else {
        [l * 100.0, a * 255.0 - 128.0, b * 255.0 - 128.0]
    }
}

/// XYZ → the `u1Fixed15` PCS encoding (1.0 = 0x8000), normalised.
fn encode_xyz(xyz: [f64; 3]) -> [f64; 3] {
    xyz.map(|v| v * 32768.0 / 65535.0)
}

fn decode_xyz(pcs: [f64; 3]) -> [f64; 3] {
    pcs.map(|v| v * 65535.0 / 32768.0)
}

/// CIE XYZ → CIELAB, both relative to D50.
#[must_use]
pub fn xyz_to_lab(xyz: [f64; 3]) -> [f64; 3] {
    let f = [lab_f(xyz[0] / D50[0]), lab_f(xyz[1] / D50[1]), lab_f(xyz[2] / D50[2])];
    [116.0 * f[1] - 16.0, 500.0 * (f[0] - f[1]), 200.0 * (f[1] - f[2])]
}

/// CIELAB → CIE XYZ, both relative to D50.
#[must_use]
pub fn lab_to_xyz([l, a, b]: [f64; 3]) -> [f64; 3] {
    let fy = (l + 16.0) / 116.0;
    let fx = fy + a / 500.0;
    let fz = fy - b / 200.0;
    [lab_f_inv(fx) * D50[0], lab_f_inv(fy) * D50[1], lab_f_inv(fz) * D50[2]]
}

/// sRGB (0–1, gamma-encoded) → CIE XYZ (D50).
fn srgb_to_xyz(rgb: [f64; 3]) -> [f64; 3] {
    let lin = rgb.map(|v| srgb_eotf(clamp01(v)));
    let m = &SRGB_TO_XYZ_D50;
    [
        m[0][0] * lin[0] + m[0][1] * lin[1] + m[0][2] * lin[2],
        m[1][0] * lin[0] + m[1][1] * lin[1] + m[1][2] * lin[2],
        m[2][0] * lin[0] + m[2][1] * lin[1] + m[2][2] * lin[2],
    ]
}

/// The CIELAB (D50) value of an 8-bit sRGB colour, as an ICC colour-managed workflow
/// sees it (Bradford-adapted to D50). For checks and previews.
#[must_use]
pub fn srgb8_to_lab_d50(c: Rgb8) -> [f64; 3] {
    let lin = [srgb_to_linear(c.r), srgb_to_linear(c.g), srgb_to_linear(c.b)];
    let m = &SRGB_TO_XYZ_D50;
    xyz_to_lab([
        m[0][0] * lin[0] + m[0][1] * lin[1] + m[0][2] * lin[2],
        m[1][0] * lin[0] + m[1][1] * lin[1] + m[1][2] * lin[2],
        m[2][0] * lin[0] + m[2][1] * lin[1] + m[2][2] * lin[2],
    ])
}

/// A sampled sRGB → CMYK transform for one profile and intent. Cheap to clone (the
/// grid is about 0.6 MB).
#[derive(Debug, Clone)]
pub struct SrgbToCmyk {
    /// `GRID_POINTS³` CMYK values (0–1), red slowest, blue fastest.
    grid: std::sync::Arc<[[f32; 4]]>,
    intent: RenderingIntent,
    black_point_compensation: bool,
}

impl SrgbToCmyk {
    /// Links sRGB to `profile` with `intent`, with or without black-point compensation.
    #[must_use]
    pub fn new(profile: &CmykProfile, intent: RenderingIntent, bpc: bool) -> Self {
        // Black-point compensation from sRGB, whose black is XYZ 0, to the profile's
        // black: XYZ' = XYZ · (1 − BP / D50) + BP, per component (Little CMS's
        // `ComputeBlackPointCompensation` with a zero source black point).
        let scale_offset = if bpc {
            let bp = profile.destination_black_point(intent);
            Some([0, 1, 2].map(|i| (1.0 - bp[i] / D50[i], bp[i])))
        } else {
            None
        };
        let n = GRID_POINTS;
        let mut grid = Vec::with_capacity(n * n * n);
        let step = 1.0 / (n - 1) as f64;
        for r in 0..n {
            for g in 0..n {
                for b in 0..n {
                    let rgb = [r as f64 * step, g as f64 * step, b as f64 * step];
                    let mut xyz = srgb_to_xyz(rgb);
                    if let Some(so) = scale_offset {
                        for (v, (s, o)) in xyz.iter_mut().zip(so) {
                            *v = *v * s + o;
                        }
                    }
                    let cmyk = profile.lab_to_cmyk(xyz_to_lab(xyz), intent);
                    grid.push(cmyk.map(|v| clamp01(v) as f32));
                }
            }
        }
        Self { grid: grid.into(), intent, black_point_compensation: bpc }
    }

    /// The default link of `docs/research/03-print-requirements.md` §3.4: relative
    /// colorimetric with black-point compensation.
    #[must_use]
    pub fn relative_bpc(profile: &CmykProfile) -> Self {
        Self::new(profile, RenderingIntent::RelativeColorimetric, true)
    }

    /// The rendering intent.
    #[must_use]
    pub fn intent(&self) -> RenderingIntent {
        self.intent
    }

    /// Whether black-point compensation is applied.
    #[must_use]
    pub fn black_point_compensation(&self) -> bool {
        self.black_point_compensation
    }

    fn sample(&self, pos: [f64; 3]) -> [f64; 4] {
        let n = GRID_POINTS;
        let cell = |p: f64| {
            let i = (p as usize).min(n - 2);
            (i, (p - i as f64).clamp(0.0, 1.0))
        };
        let ((x0, rx), (y0, ry), (z0, rz)) = (cell(pos[0]), cell(pos[1]), cell(pos[2]));
        let at = |dx: usize, dy: usize, dz: usize| {
            let i = ((x0 + dx) * n + (y0 + dy)) * n + (z0 + dz);
            self.grid.get(i).copied().unwrap_or([0.0; 4]).map(f64::from)
        };
        let c0 = at(0, 0, 0);
        let (p1, p2, p3);
        // The same six tetrahedra as `tetrahedral`, on whole CMYK values.
        let (a, b, c) = if rx >= ry && ry >= rz {
            p1 = at(1, 0, 0);
            p2 = at(1, 1, 0);
            p3 = at(1, 1, 1);
            ((p1, c0), (p2, p1), (p3, p2))
        } else if rx >= rz && rz >= ry {
            p1 = at(1, 0, 0);
            p2 = at(1, 1, 1);
            p3 = at(1, 0, 1);
            ((p1, c0), (p2, p3), (p3, p1))
        } else if rz >= rx && rx >= ry {
            p1 = at(1, 0, 1);
            p2 = at(1, 1, 1);
            p3 = at(0, 0, 1);
            ((p1, p3), (p2, p1), (p3, c0))
        } else if ry >= rx && rx >= rz {
            p1 = at(1, 1, 0);
            p2 = at(0, 1, 0);
            p3 = at(1, 1, 1);
            ((p1, p2), (p2, c0), (p3, p1))
        } else if ry >= rz && rz >= rx {
            p1 = at(1, 1, 1);
            p2 = at(0, 1, 0);
            p3 = at(0, 1, 1);
            ((p1, p3), (p2, c0), (p3, p2))
        } else {
            p1 = at(1, 1, 1);
            p2 = at(0, 1, 1);
            p3 = at(0, 0, 1);
            ((p1, p2), (p2, p3), (p3, c0))
        };
        [0, 1, 2, 3].map(|k| {
            c0[k] + (a.0[k] - a.1[k]) * rx + (b.0[k] - b.1[k]) * ry + (c.0[k] - c.1[k]) * rz
        })
    }

    /// The CMYK value (each 0–1) of an 8-bit sRGB colour.
    #[must_use]
    pub fn convert(&self, c: Rgb8) -> [f64; 4] {
        let scale = (GRID_POINTS - 1) as f64 / 255.0;
        self.sample([f64::from(c.r) * scale, f64::from(c.g) * scale, f64::from(c.b) * scale])
    }

    /// The CMYK value of an sRGB colour as a [`Color::Cmyk`] (no cleaning, no ink cap).
    #[must_use]
    pub fn convert_color(&self, c: Rgb8) -> Color {
        let [cy, m, y, k] = self.convert(c);
        Color::cmyk(cy as f32, m as f32, y as f32, k as f32)
    }

    /// Converts interleaved 8-bit RGB pixels to interleaved 8-bit CMYK (255 = 100 %
    /// ink). A trailing partial pixel is ignored.
    #[must_use]
    pub fn convert_pixels(&self, rgb: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(rgb.len() / 3 * 4);
        // Photographs repeat colours: remember the last one.
        let mut last: Option<([u8; 3], [u8; 4])> = None;
        for px in rgb.chunks_exact(3) {
            let key = [px[0], px[1], px[2]];
            let value = match last {
                Some((k, v)) if k == key => v,
                _ => {
                    let cmyk = self.convert(Rgb8::new(px[0], px[1], px[2]));
                    // 0..=255 after the clamp; `as` cannot overflow.
                    let v = cmyk.map(|v| (clamp01(v) * 255.0 + 0.5) as u8);
                    last = Some((key, v));
                    v
                }
            };
            out.extend_from_slice(&value);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lab_and_xyz_round_trip() {
        for lab in [[50.0, 20.0, -30.0], [0.0, 0.0, 0.0], [100.0, 0.0, 0.0], [20.0, -60.0, 70.0]] {
            let back = xyz_to_lab(lab_to_xyz(lab));
            for i in 0..3 {
                assert!((back[i] - lab[i]).abs() < 1e-9, "{lab:?} -> {back:?}");
            }
        }
        let white = srgb8_to_lab_d50(Rgb8::WHITE);
        assert!((white[0] - 100.0).abs() < 0.01 && white[1].abs() < 0.1 && white[2].abs() < 0.1);
        assert_eq!(srgb8_to_lab_d50(Rgb8::BLACK), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn lab_encodings_round_trip() {
        for wide in [true, false] {
            let lab = [62.5, -12.0, 40.0];
            let back = decode_lab(encode_lab(lab, wide), wide);
            for i in 0..3 {
                assert!((back[i] - lab[i]).abs() < 1e-9);
            }
        }
        // ICC v2 legacy 16-bit encoding: L* 100 is 0xFF00, a* 0 is 0x8000.
        let e = encode_lab([100.0, 0.0, 0.0], true);
        assert!((e[0] * 65535.0 - 65280.0).abs() < 1e-6);
        assert!((e[1] * 65535.0 - 32768.0).abs() < 1e-6);
    }

    #[test]
    fn helpers_are_exact_enough() {
        assert!((sqrt_newton(2.0) - std::f64::consts::SQRT_2).abs() < 1e-15);
        assert_eq!(sqrt_newton(-1.0), 0.0);
        assert_eq!(sqrt_newton(f64::NAN), 0.0);
        assert_eq!(
            solve3([[1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 4.0]], [1.0, 2.0, 4.0]),
            Some([1.0, 1.0, 1.0])
        );
        assert_eq!(solve3([[0.0; 3]; 3], [1.0; 3]), None);
        // y = (x - 10)² - 4 has its shadow root at 12 (the larger one).
        let pts: Vec<(f64, f64)> = (0..20)
            .map(|i| {
                let x = f64::from(i);
                (x, (x - 10.0) * (x - 10.0) - 4.0)
            })
            .collect();
        assert!((quadratic_root(&pts) - 12.0).abs() < 1e-6);
        assert_eq!(quadratic_root(&pts[..3]), 0.0);
        assert_eq!(curve_eval(&[0.0, 1.0], 0.25), 0.25);
        assert_eq!(curve_eval(&[0.0, 0.5, 1.0], 2.0), 1.0);
        assert_eq!(curve_eval(&[0.3], 0.9), 0.3);
    }

    #[test]
    fn hostile_profiles_are_refused_without_panicking() {
        assert!(matches!(CmykProfile::parse(&[]), Err(IccError::Malformed(_))));
        assert!(matches!(CmykProfile::parse(&[0u8; 200]), Err(IccError::Malformed(_))));
        let mut header = vec![0u8; 132];
        header[36..40].copy_from_slice(b"acsp");
        header[16..20].copy_from_slice(b"RGB ");
        assert_eq!(CmykProfile::parse(&header).unwrap_err(), IccError::NotCmyk);
        header[16..20].copy_from_slice(b"CMYK");
        assert_eq!(CmykProfile::parse(&header).unwrap_err(), IccError::UnsupportedPcs);
        header[20..24].copy_from_slice(b"Lab ");
        assert_eq!(CmykProfile::parse(&header).unwrap_err(), IccError::MissingTag("B2A0"));
        // A tag count far beyond the data.
        header[128..132].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(matches!(CmykProfile::parse(&header), Err(IccError::Malformed(_))));
        // A B2A0 tag pointing outside the profile, then one of an unsupported type.
        let mut p = header.clone();
        p[128..132].copy_from_slice(&1u32.to_be_bytes());
        p.extend_from_slice(b"B2A0");
        p.extend_from_slice(&1000u32.to_be_bytes());
        p.extend_from_slice(&64u32.to_be_bytes());
        assert!(matches!(CmykProfile::parse(&p), Err(IccError::Malformed(_))));
        p[136..140].copy_from_slice(&144u32.to_be_bytes());
        p[140..144].copy_from_slice(&12u32.to_be_bytes());
        p.extend_from_slice(b"mBA \0\0\0\0\0\0\0\0");
        assert!(matches!(
            CmykProfile::parse(&p),
            Err(IccError::UnsupportedTag { tag: "B2A0", .. })
        ));
        // An mft2 header announcing a huge table in a few bytes.
        let len = p.len();
        p.truncate(144);
        let mut lut = b"mft2\0\0\0\0".to_vec();
        lut.extend_from_slice(&[3, 4, 255, 0]);
        lut.extend_from_slice(&[0u8; 36]);
        lut.extend_from_slice(&4096u16.to_be_bytes());
        lut.extend_from_slice(&4096u16.to_be_bytes());
        p.extend_from_slice(&lut);
        p[140..144].copy_from_slice(&(lut.len() as u32).to_be_bytes());
        assert!(len > 0);
        assert!(matches!(CmykProfile::parse(&p), Err(IccError::Malformed(_))));
    }
}
