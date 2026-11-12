//! Small, defensive helpers over `lopdf`: bounded reference resolution, typed dictionary
//! access, text strings, matrices and rectangles. Nothing here panics on malformed input.

use lopdf::{Dictionary, Document, Object, ObjectId};

/// How many references are followed before a chain is treated as a cycle.
pub(crate) const MAX_DEREF: usize = 32;

/// Points per millimetre.
pub(crate) const PT_PER_MM: f64 = 72.0 / 25.4;

/// Follows references until a direct object is reached.
///
/// Returns `None` for dangling references, reference cycles and `null`.
pub(crate) fn resolve<'a>(doc: &'a Document, object: &'a Object) -> Option<&'a Object> {
    resolve_with_id(doc, object).1
}

/// Like [`resolve`], also returning the id of the last reference followed (if any).
pub(crate) fn resolve_with_id<'a>(
    doc: &'a Document,
    mut object: &'a Object,
) -> (Option<ObjectId>, Option<&'a Object>) {
    let mut id = None;
    for _ in 0..=MAX_DEREF {
        match object {
            Object::Reference(reference) => {
                id = Some(*reference);
                match doc.objects.get(reference) {
                    Some(target) => object = target,
                    None => return (id, None),
                }
            }
            Object::Null => return (id, None),
            other => return (id, Some(other)),
        }
    }
    (id, None)
}

/// The dictionary of a dictionary or stream object (after resolving references).
pub(crate) fn as_dict<'a>(doc: &'a Document, object: &'a Object) -> Option<&'a Dictionary> {
    match resolve(doc, object)? {
        Object::Dictionary(dict) => Some(dict),
        Object::Stream(stream) => Some(&stream.dict),
        _ => None,
    }
}

/// `dict[key]`, resolved; `None` when absent, `null` or dangling.
pub(crate) fn get<'a>(doc: &'a Document, dict: &'a Dictionary, key: &[u8]) -> Option<&'a Object> {
    resolve(doc, dict.get(key).ok()?)
}

/// `dict[key]` as a dictionary (or the dictionary of a stream).
pub(crate) fn get_dict<'a>(
    doc: &'a Document,
    dict: &'a Dictionary,
    key: &[u8],
) -> Option<&'a Dictionary> {
    as_dict(doc, dict.get(key).ok()?)
}

/// `dict[key]` as an array.
pub(crate) fn get_array<'a>(
    doc: &'a Document,
    dict: &'a Dictionary,
    key: &[u8],
) -> Option<&'a [Object]> {
    match get(doc, dict, key)? {
        Object::Array(items) => Some(items.as_slice()),
        _ => None,
    }
}

/// `dict[key]` as a name.
pub(crate) fn get_name<'a>(
    doc: &'a Document,
    dict: &'a Dictionary,
    key: &[u8],
) -> Option<&'a [u8]> {
    match get(doc, dict, key)? {
        Object::Name(name) => Some(name.as_slice()),
        _ => None,
    }
}

/// `dict[key]` as a finite number.
pub(crate) fn get_num(doc: &Document, dict: &Dictionary, key: &[u8]) -> Option<f64> {
    num(get(doc, dict, key)?)
}

/// `dict[key]` as an integer (reals are accepted when they are whole numbers).
pub(crate) fn get_int(doc: &Document, dict: &Dictionary, key: &[u8]) -> Option<i64> {
    int(get(doc, dict, key)?)
}

/// `dict[key]` as a boolean.
pub(crate) fn get_bool(doc: &Document, dict: &Dictionary, key: &[u8]) -> Option<bool> {
    match get(doc, dict, key)? {
        Object::Boolean(value) => Some(*value),
        _ => None,
    }
}

/// `dict[key]` as a text string (PDFDocEncoding or UTF-16), trimmed of trailing NULs.
pub(crate) fn get_text(doc: &Document, dict: &Dictionary, key: &[u8]) -> Option<String> {
    match get(doc, dict, key)? {
        object @ Object::String(bytes, _) => Some(text_string(object, bytes)),
        // Some producers write names where strings are expected (e.g. `/Trapped`-like keys).
        Object::Name(name) => Some(lossy(name)),
        _ => None,
    }
}

/// A direct numeric object as a finite `f64`.
pub(crate) fn num(object: &Object) -> Option<f64> {
    let value = match object {
        // i64 → f64 is exact up to 2^53, far beyond any sane PDF number.
        #[allow(clippy::cast_precision_loss)]
        Object::Integer(value) => *value as f64,
        Object::Real(value) => f64::from(*value),
        _ => return None,
    };
    value.is_finite().then_some(value)
}

/// A direct numeric object as an integer.
pub(crate) fn int(object: &Object) -> Option<i64> {
    match object {
        Object::Integer(value) => Some(*value),
        Object::Real(value) if value.is_finite() && value.fract() == 0.0 => {
            // Truncation is intended and safe: the value is whole and within f32 range.
            #[allow(clippy::cast_possible_truncation)]
            Some(*value as i64)
        }
        _ => None,
    }
}

/// Decodes a PDF text string.
fn text_string(object: &Object, bytes: &[u8]) -> String {
    let decoded = lopdf::decode_text_string(object).unwrap_or_else(|_| latin1(bytes));
    decoded.trim_end_matches('\0').to_string()
}

/// ISO 8859-1 decoding: every byte maps to the code point of the same value.
fn latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| char::from(b)).collect()
}

/// A name or other byte string for display: UTF-8 when valid, Latin-1 otherwise, with
/// control characters replaced and the length capped so hostile names cannot bloat reports.
pub(crate) fn lossy(bytes: &[u8]) -> String {
    const MAX_CHARS: usize = 120;
    let text = match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => latin1(bytes),
    };
    let mut out: String =
        text.chars().take(MAX_CHARS).map(|c| if c.is_control() { '\u{FFFD}' } else { c }).collect();
    if text.chars().count() > MAX_CHARS {
        out.push('…');
    }
    out
}

/// Trims a text value and cuts it to at most `max_chars` characters (with an ellipsis), so
/// that hostile metadata cannot bloat the report.
pub(crate) fn clip(text: &str, max_chars: usize) -> String {
    let text = text.trim();
    match text.char_indices().nth(max_chars) {
        Some((end, _)) => format!("{}…", &text[..end]),
        None => text.to_string(),
    }
}

/// A rectangle in PDF default user space, in points, normalised so that `x0 <= x1`, `y0 <= y1`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RectPt {
    pub(crate) x0: f64,
    pub(crate) y0: f64,
    pub(crate) x1: f64,
    pub(crate) y1: f64,
}

impl RectPt {
    /// Largest coordinate magnitude accepted (about 35 km): anything beyond is hostile or
    /// broken, and bounding it keeps later arithmetic finite.
    const MAX_ABS: f64 = 1.0e8;

    /// Parses a PDF rectangle array `[x0 y0 x1 y1]` (elements may be references).
    pub(crate) fn from_object(doc: &Document, object: &Object) -> Option<Self> {
        let Object::Array(items) = resolve(doc, object)? else { return None };
        if items.len() != 4 {
            return None;
        }
        let mut values = [0.0_f64; 4];
        for (slot, item) in values.iter_mut().zip(items) {
            let value = num(resolve(doc, item)?)?;
            if value.abs() > Self::MAX_ABS {
                return None;
            }
            *slot = value;
        }
        let [a, b, c, d] = values;
        Some(Self { x0: a.min(c), y0: b.min(d), x1: a.max(c), y1: b.max(d) })
    }

    pub(crate) fn width(&self) -> f64 {
        self.x1 - self.x0
    }

    pub(crate) fn height(&self) -> f64 {
        self.y1 - self.y0
    }

    /// Whether `other` overlaps this rectangle by more than `tolerance` in both directions.
    pub(crate) fn overlaps(&self, other: &RectPt, tolerance: f64) -> bool {
        other.x1 > self.x0 + tolerance
            && other.x0 < self.x1 - tolerance
            && other.y1 > self.y0 + tolerance
            && other.y0 < self.y1 - tolerance
    }
}

/// A PDF transformation matrix `[a b c d e f]` (row-vector convention:
/// `x' = a·x + c·y + e`, `y' = b·x + d·y + f`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Matrix {
    pub(crate) a: f64,
    pub(crate) b: f64,
    pub(crate) c: f64,
    pub(crate) d: f64,
    pub(crate) e: f64,
    pub(crate) f: f64,
}

impl Matrix {
    pub(crate) const IDENTITY: Self = Self { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 };

    pub(crate) fn from_slice(v: &[f64]) -> Option<Self> {
        match v {
            [a, b, c, d, e, f] => Some(Self { a: *a, b: *b, c: *c, d: *d, e: *e, f: *f }),
            _ => None,
        }
    }

    /// Parses a matrix array of six numbers.
    pub(crate) fn from_object(doc: &Document, object: &Object) -> Option<Self> {
        let Object::Array(items) = resolve(doc, object)? else { return None };
        if items.len() != 6 {
            return None;
        }
        let mut values = [0.0_f64; 6];
        for (slot, item) in values.iter_mut().zip(items) {
            *slot = num(resolve(doc, item)?)?;
        }
        Self::from_slice(&values)
    }

    /// `self × other`: applies `self` first, then `other` (the PDF `cm` rule is
    /// `CTM' = M × CTM`).
    pub(crate) fn then(&self, other: &Matrix) -> Matrix {
        Matrix {
            a: self.a * other.a + self.b * other.c,
            b: self.a * other.b + self.b * other.d,
            c: self.c * other.a + self.d * other.c,
            d: self.c * other.b + self.d * other.d,
            e: self.e * other.a + self.f * other.c + other.e,
            f: self.e * other.b + self.f * other.d + other.f,
        }
    }

    pub(crate) fn apply(&self, x: f64, y: f64) -> (f64, f64) {
        (self.a * x + self.c * y + self.e, self.b * x + self.d * y + self.f)
    }

    /// Length of the transformed unit x vector.
    pub(crate) fn scale_x(&self) -> f64 {
        self.a.hypot(self.b)
    }

    /// Length of the transformed unit y vector.
    pub(crate) fn scale_y(&self) -> f64 {
        self.c.hypot(self.d)
    }

    /// Whether every coefficient is finite and of sane magnitude.
    pub(crate) fn is_sane(&self) -> bool {
        [self.a, self.b, self.c, self.d, self.e, self.f]
            .iter()
            .all(|v| v.is_finite() && v.abs() < 1.0e12)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn matrix_composition_follows_the_pdf_rule() {
        // Scale by 2, then translate by (10, 20).
        let scale = Matrix { a: 2.0, b: 0.0, c: 0.0, d: 2.0, e: 0.0, f: 0.0 };
        let translate = Matrix { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 10.0, f: 20.0 };
        let m = scale.then(&translate);
        assert_eq!(m.apply(1.0, 1.0), (12.0, 22.0));
        // The other order translates first.
        let m = translate.then(&scale);
        assert_eq!(m.apply(1.0, 1.0), (22.0, 42.0));
        // A quarter turn keeps lengths; a shear lengthens the x axis.
        let rot = Matrix { a: 0.0, b: 1.0, c: -1.0, d: 0.0, e: 0.0, f: 0.0 };
        assert_eq!((rot.scale_x(), rot.scale_y()), (1.0, 1.0));
        assert_eq!(rot.apply(1.0, 0.0), (0.0, 1.0));
        let skew = Matrix { a: 1.0, b: 0.5, c: 0.0, d: 1.0, e: 0.0, f: 0.0 };
        assert!((skew.scale_x() - 1.25_f64.sqrt()).abs() < 1e-12);
        assert!(skew.is_sane());
        assert!(!Matrix { a: f64::NAN, ..skew }.is_sane());
        assert!(!Matrix { e: 1.0e13, ..skew }.is_sane());
    }

    #[test]
    fn reference_cycles_resolve_to_none() {
        let mut doc = Document::with_version("1.4");
        doc.objects.insert((1, 0), Object::Reference((2, 0)));
        doc.objects.insert((2, 0), Object::Reference((1, 0)));
        doc.objects.insert((3, 0), Object::Reference((9, 0)));
        doc.objects.insert((4, 0), Object::Integer(7));
        assert!(resolve(&doc, &Object::Reference((1, 0))).is_none());
        assert!(resolve(&doc, &Object::Reference((3, 0))).is_none());
        assert!(resolve(&doc, &Object::Null).is_none());
        assert_eq!(resolve(&doc, &Object::Reference((4, 0))), Some(&Object::Integer(7)));
    }

    #[test]
    fn rectangles_are_normalised_and_bounded() {
        let doc = Document::with_version("1.4");
        let rect = Object::Array(vec![100.into(), 50.into(), 0.into(), 0.into()]);
        let r = RectPt::from_object(&doc, &rect).expect("valid rectangle");
        assert_eq!((r.x0, r.y0, r.x1, r.y1), (0.0, 0.0, 100.0, 50.0));
        let short = Object::Array(vec![0.into(), 0.into(), 1.into()]);
        assert!(RectPt::from_object(&doc, &short).is_none());
        let huge = Object::Array(vec![0.into(), 0.into(), Object::Real(1.0e30), 1.into()]);
        assert!(RectPt::from_object(&doc, &huge).is_none());
        let nan = Object::Array(vec![0.into(), 0.into(), Object::Real(f32::NAN), 1.into()]);
        assert!(RectPt::from_object(&doc, &nan).is_none());
    }

    #[test]
    fn clipped_text() {
        assert_eq!(clip("  Le Petit Prince \n", 100), "Le Petit Prince");
        assert_eq!(clip("abcdef", 3), "abc…");
        assert_eq!(clip("abc", 3), "abc");
        assert_eq!(clip("ééééé", 2), "éé…");
        assert_eq!(clip("", 5), "");
    }

    #[test]
    fn lossy_names_are_capped_and_printable() {
        assert_eq!(lossy(b"Helvetica"), "Helvetica");
        assert_eq!(lossy(b"a\x00b"), "a\u{FFFD}b");
        let long = vec![b'x'; 500];
        assert_eq!(lossy(&long).chars().count(), 121);
        assert_eq!(lossy(&[0xE9]), "é");
    }
}
