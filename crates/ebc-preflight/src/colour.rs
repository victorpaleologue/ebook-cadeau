//! Colour space resolution and classification.

use lopdf::{Dictionary, Document, Object};

use crate::pdf::{get, get_int, lossy, resolve};

/// How deep colour space definitions may nest (`Indexed` → `Separation` → `ICCBased` …).
const MAX_DEPTH: usize = 8;
/// Colorant names kept from a `DeviceN` space.
const MAX_COLORANTS: usize = 32;

/// A resolved colour space, reduced to what a print check needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ColorSpace {
    DeviceGray,
    DeviceRgb,
    DeviceCmyk,
    CalGray,
    CalRgb,
    Lab,
    /// `ICCBased` with `N` components.
    Icc(u8),
    Indexed(Box<ColorSpace>),
    /// `Separation` with its colorant name and alternate space.
    Separation(String, Box<ColorSpace>),
    /// `DeviceN` with its colorant names and alternate space.
    DeviceN(Vec<String>, Box<ColorSpace>),
    /// `Pattern`, with the underlying space of uncoloured patterns.
    Pattern(Option<Box<ColorSpace>>),
    /// Could not be resolved; the string says why or what was named.
    Unknown(String),
}

/// What a colour space means for a CMYK print job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorClass {
    Gray,
    Cmyk,
    /// RGB-like: `DeviceRGB`, `CalRGB`, `Lab`, three-component ICC.
    Rgb,
    /// A real spot colour.
    Spot,
    /// `Separation /All`.
    Registration,
    /// `Separation /None`, or a pattern space (the pattern's own colours count).
    Neutral,
    Unknown,
}

impl ColorSpace {
    /// Resolves a colour space object: a name (device space or a key of the `ColorSpace`
    /// resource dictionary) or an array.
    pub(crate) fn from_object(
        doc: &Document,
        object: &Object,
        resources: Option<&Dictionary>,
    ) -> ColorSpace {
        Self::resolve_object(doc, object, resources, 0)
    }

    /// Resolves a colour space given by name, as the `cs`/`CS` operators and inline images do.
    pub(crate) fn from_name(doc: &Document, name: &[u8], resources: Option<&Dictionary>) -> Self {
        Self::resolve_name(doc, name, resources, 0)
    }

    fn resolve_name(
        doc: &Document,
        name: &[u8],
        resources: Option<&Dictionary>,
        depth: usize,
    ) -> ColorSpace {
        match name {
            b"DeviceGray" | b"G" => ColorSpace::DeviceGray,
            b"DeviceRGB" | b"RGB" => ColorSpace::DeviceRgb,
            b"DeviceCMYK" | b"CMYK" => ColorSpace::DeviceCmyk,
            b"Pattern" => ColorSpace::Pattern(None),
            b"CalGray" => ColorSpace::CalGray,
            b"CalRGB" => ColorSpace::CalRgb,
            b"Lab" => ColorSpace::Lab,
            other => {
                let named =
                    resources.and_then(|res| get(doc, res, b"ColorSpace")).and_then(|spaces| {
                        match spaces {
                            Object::Dictionary(dict) => dict.get(other).ok(),
                            _ => None,
                        }
                    });
                match named {
                    // A named resource must not be another resource name: resolve it without
                    // resources so that `/CS0 /CS0` cannot loop.
                    Some(object) => Self::resolve_object(doc, object, None, depth + 1),
                    None => ColorSpace::Unknown(lossy(other)),
                }
            }
        }
    }

    fn resolve_object(
        doc: &Document,
        object: &Object,
        resources: Option<&Dictionary>,
        depth: usize,
    ) -> ColorSpace {
        if depth > MAX_DEPTH {
            return ColorSpace::Unknown("nested too deeply".to_string());
        }
        match resolve(doc, object) {
            Some(Object::Name(name)) => Self::resolve_name(doc, name, resources, depth),
            Some(Object::Array(items)) => Self::resolve_array(doc, items, resources, depth),
            _ => ColorSpace::Unknown("not a name or array".to_string()),
        }
    }

    fn resolve_array(
        doc: &Document,
        items: &[Object],
        resources: Option<&Dictionary>,
        depth: usize,
    ) -> ColorSpace {
        let Some(Object::Name(kind)) = items.first().and_then(|first| resolve(doc, first)) else {
            return ColorSpace::Unknown("array without a family name".to_string());
        };
        let nested = |index: usize| -> Box<ColorSpace> {
            Box::new(match items.get(index) {
                Some(object) => Self::resolve_object(doc, object, resources, depth + 1),
                None => ColorSpace::Unknown("missing base colour space".to_string()),
            })
        };
        match kind.as_slice() {
            b"ICCBased" => {
                let components = items
                    .get(1)
                    .and_then(|profile| match resolve(doc, profile) {
                        Some(Object::Stream(stream)) => get_int(doc, &stream.dict, b"N"),
                        _ => None,
                    })
                    .and_then(|n| u8::try_from(n).ok());
                match components {
                    Some(n @ (1 | 3 | 4)) => ColorSpace::Icc(n),
                    _ => ColorSpace::Unknown("ICCBased without a valid N".to_string()),
                }
            }
            b"Indexed" | b"I" => ColorSpace::Indexed(nested(1)),
            b"Separation" => {
                let name = match items.get(1).and_then(|n| resolve(doc, n)) {
                    Some(Object::Name(name)) => lossy(name),
                    _ => String::from("?"),
                };
                ColorSpace::Separation(name, nested(2))
            }
            b"DeviceN" => {
                let names = match items.get(1).and_then(|n| resolve(doc, n)) {
                    Some(Object::Array(names)) => names
                        .iter()
                        .take(MAX_COLORANTS)
                        .map(|n| match resolve(doc, n) {
                            Some(Object::Name(name)) => lossy(name),
                            _ => String::from("?"),
                        })
                        .collect(),
                    _ => Vec::new(),
                };
                ColorSpace::DeviceN(names, nested(2))
            }
            b"Pattern" => ColorSpace::Pattern(items.get(1).map(|_| nested(1))),
            // `[/DeviceRGB]` and friends, and CIE-based spaces with their parameter dictionary.
            other => Self::resolve_name(doc, other, None, depth),
        }
    }

    /// A short, stable description, e.g. `DeviceCMYK`, `ICCBased(3)`, `Separation(All)`.
    pub(crate) fn describe(&self) -> String {
        match self {
            Self::DeviceGray => "DeviceGray".to_string(),
            Self::DeviceRgb => "DeviceRGB".to_string(),
            Self::DeviceCmyk => "DeviceCMYK".to_string(),
            Self::CalGray => "CalGray".to_string(),
            Self::CalRgb => "CalRGB".to_string(),
            Self::Lab => "Lab".to_string(),
            Self::Icc(n) => format!("ICCBased({n})"),
            Self::Indexed(base) => format!("Indexed({})", base.describe()),
            Self::Separation(name, _) => format!("Separation({name})"),
            Self::DeviceN(names, _) => format!("DeviceN({})", names.join(",")),
            Self::Pattern(None) => "Pattern".to_string(),
            Self::Pattern(Some(base)) => format!("Pattern({})", base.describe()),
            Self::Unknown(what) => format!("Unknown({what})"),
        }
    }

    /// Classifies the space for a CMYK print job.
    pub(crate) fn class(&self) -> ColorClass {
        match self {
            Self::DeviceGray | Self::CalGray | Self::Icc(1) => ColorClass::Gray,
            Self::DeviceCmyk | Self::Icc(4) => ColorClass::Cmyk,
            Self::DeviceRgb | Self::CalRgb | Self::Lab | Self::Icc(_) => ColorClass::Rgb,
            Self::Indexed(base) => base.class(),
            Self::Separation(name, _) => match name.as_str() {
                "All" => ColorClass::Registration,
                "None" => ColorClass::Neutral,
                name if is_process_colorant(name) => ColorClass::Cmyk,
                _ => ColorClass::Spot,
            },
            Self::DeviceN(names, _) => {
                if names.iter().all(|n| n == "None" || is_process_colorant(n)) {
                    ColorClass::Cmyk
                } else {
                    ColorClass::Spot
                }
            }
            Self::Pattern(None) => ColorClass::Neutral,
            Self::Pattern(Some(base)) => base.class(),
            Self::Unknown(_) => ColorClass::Unknown,
        }
    }

    /// Whether colour components in this space are C, M, Y, K ink amounts.
    pub(crate) fn is_cmyk_ink(&self) -> bool {
        matches!(self, Self::DeviceCmyk | Self::Icc(4))
    }
}

fn is_process_colorant(name: &str) -> bool {
    matches!(name, "Cyan" | "Magenta" | "Yellow" | "Black")
}

/// The signature and component count read from an ICC profile header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IccHeader {
    /// Data colour space signature, trimmed (`CMYK`, `RGB`, `GRAY`, `Lab` …).
    pub(crate) color_space: String,
    /// Number of components implied by the signature, when known.
    pub(crate) components: Option<u8>,
    /// Profile/device class signature (`prtr`, `mntr`, `scnr` …).
    pub(crate) class: String,
    /// Major version of the ICC specification the profile follows (2 or 4).
    pub(crate) version_major: u8,
}

/// Reads the fixed 128-byte ICC header. `None` when the data is not an ICC profile.
pub(crate) fn parse_icc_header(data: &[u8]) -> Option<IccHeader> {
    let header = data.get(..128)?;
    if header.get(36..40)? != b"acsp" {
        return None;
    }
    let signature = |range: std::ops::Range<usize>| -> Option<String> {
        let bytes = header.get(range)?;
        let text: String = bytes
            .iter()
            .map(|&b| if b.is_ascii_graphic() || b == b' ' { char::from(b) } else { '?' })
            .collect();
        Some(text.trim_end().to_string())
    };
    let color_space = signature(16..20)?;
    let components = match color_space.as_str() {
        "GRAY" => Some(1),
        "RGB" | "Lab" | "XYZ" | "Luv" | "YCbr" | "Yxy" | "HSV" | "HLS" | "CMY" => Some(3),
        "CMYK" => Some(4),
        other => other
            .strip_suffix("CLR")
            .and_then(|n| u8::from_str_radix(n, 16).ok())
            .filter(|n| (2..=15).contains(n)),
    };
    Some(IccHeader { color_space, components, class: signature(12..16)?, version_major: header[8] })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use lopdf::{Stream, dictionary};

    use super::*;

    fn name(n: &str) -> Object {
        Object::Name(n.as_bytes().to_vec())
    }

    #[test]
    fn device_and_named_spaces() {
        let mut doc = Document::with_version("1.6");
        let icc = doc.add_object(Stream::new(dictionary! { "N" => 3 }, vec![0; 4]));
        let resources = dictionary! {
            "ColorSpace" => dictionary! {
                "CS0" => vec![name("ICCBased"), Object::Reference(icc)],
                "Spot" => vec![name("Separation"), name("PANTONE 485 C"), name("DeviceCMYK"), Object::Null],
                "Reg" => vec![name("Separation"), name("All"), name("DeviceCMYK"), Object::Null],
                "Idx" => vec![name("Indexed"), name("DeviceRGB"), 1.into(), Object::string_literal("abcdef")],
                "Loop" => name("Loop"),
                "DN" => vec![name("DeviceN"), Object::Array(vec![name("Cyan"), name("Black")]), name("DeviceCMYK"), Object::Null],
                "DN2" => vec![name("DeviceN"), Object::Array(vec![name("Cyan"), name("Gold")]), name("DeviceCMYK"), Object::Null],
            },
        };
        let get = |n: &str| ColorSpace::from_name(&doc, n.as_bytes(), Some(&resources));
        assert_eq!(get("DeviceCMYK"), ColorSpace::DeviceCmyk);
        assert_eq!(get("CS0"), ColorSpace::Icc(3));
        assert_eq!(get("CS0").class(), ColorClass::Rgb);
        assert_eq!(get("CS0").describe(), "ICCBased(3)");
        assert_eq!(get("Spot").class(), ColorClass::Spot);
        assert_eq!(get("Spot").describe(), "Separation(PANTONE 485 C)");
        assert_eq!(get("Reg").class(), ColorClass::Registration);
        assert_eq!(get("Idx").class(), ColorClass::Rgb);
        assert_eq!(get("Idx").describe(), "Indexed(DeviceRGB)");
        assert_eq!(get("DN").class(), ColorClass::Cmyk);
        assert_eq!(get("DN2").class(), ColorClass::Spot);
        assert_eq!(get("DN2").describe(), "DeviceN(Cyan,Gold)");
        // A resource that names itself must not loop.
        assert_eq!(get("Loop").class(), ColorClass::Unknown);
        assert_eq!(get("Missing").describe(), "Unknown(Missing)");
        assert_eq!(get("Pattern").class(), ColorClass::Neutral);
    }

    #[test]
    fn self_referential_arrays_terminate() {
        let mut doc = Document::with_version("1.6");
        let id = doc.new_object_id();
        doc.objects
            .insert(id, Object::Array(vec![name("Indexed"), Object::Reference(id), 1.into()]));
        let space = ColorSpace::from_object(&doc, &Object::Reference(id), None);
        assert_eq!(space.class(), ColorClass::Unknown);
        assert!(space.describe().starts_with("Indexed(Indexed("));
    }

    #[test]
    fn icc_header() {
        let mut data = vec![0_u8; 128];
        data[0..4].copy_from_slice(&128_u32.to_be_bytes());
        data[12..16].copy_from_slice(b"prtr");
        data[16..20].copy_from_slice(b"CMYK");
        data[36..40].copy_from_slice(b"acsp");
        let header = parse_icc_header(&data).unwrap();
        assert_eq!(header.color_space, "CMYK");
        assert_eq!(header.components, Some(4));
        assert_eq!(header.class, "prtr");
        assert_eq!(header.version_major, 0);
        data[8] = 4;
        assert_eq!(parse_icc_header(&data).unwrap().version_major, 4);
        data[16..20].copy_from_slice(b"RGB ");
        assert_eq!(parse_icc_header(&data).unwrap().components, Some(3));
        data[16..20].copy_from_slice(b"6CLR");
        assert_eq!(parse_icc_header(&data).unwrap().components, Some(6));
        data[36] = b'x';
        assert!(parse_icc_header(&data).is_none());
        assert!(parse_icc_header(&data[..100]).is_none());
        assert!(parse_icc_header(&[]).is_none());
    }
}
