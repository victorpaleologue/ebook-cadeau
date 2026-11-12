//! Font embedding checks: every font reachable from a page's resources must carry a font
//! program (`FontFile`, `FontFile2` or `FontFile3`), including the descendant fonts of
//! composite (Type 0) fonts.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::codes;
use crate::content::FontKey;
use crate::pdf::{as_dict, get, get_array, get_dict, get_name, lossy, resolve_with_id};
use crate::report::FontSummary;
use crate::sink::Sink;

/// Resource dictionaries examined per page (pages, forms, patterns, Type 3 fonts).
const MAX_RESOURCE_DICTS: usize = 10_000;
/// Fonts reported per page.
const MAX_FONTS: usize = 2_000;

/// Finds every font in the page's resource tree, checks it and returns the summaries sorted
/// by name. `used` marks the fonts a `Tf` operator selected.
pub(crate) fn check_fonts(
    doc: &Document,
    resources: Option<&Dictionary>,
    used: &BTreeSet<FontKey>,
    page: u32,
    sink: &mut Sink,
) -> Vec<FontSummary> {
    let mut fonts: BTreeMap<FontKey, FontSummary> = BTreeMap::new();
    let mut visited: HashSet<ObjectId> = HashSet::new();
    let mut queue: Vec<&Dictionary> = resources.into_iter().collect();
    let mut examined = 0_usize;
    while let Some(resources) = queue.pop() {
        examined += 1;
        if examined > MAX_RESOURCE_DICTS || fonts.len() > MAX_FONTS {
            sink.error(
                codes::LIMIT_EXCEEDED,
                Some(page),
                "too many resource dictionaries or fonts; the font check is partial",
            );
            break;
        }
        if let Some(font_dict) = get_dict(doc, resources, b"Font") {
            for (name, entry) in font_dict {
                let (id, object) = resolve_with_id(doc, entry);
                let key = id.map_or_else(|| FontKey::Direct(name.clone()), FontKey::Id);
                if fonts.contains_key(&key) {
                    continue;
                }
                let Some(dict) = object.and_then(|o| as_dict(doc, o)) else {
                    sink.error(
                        codes::FONT_INVALID,
                        Some(page),
                        format!("font resource /{} is not a dictionary", lossy(name)),
                    );
                    continue;
                };
                match analyse(doc, dict) {
                    Ok(mut summary) => {
                        summary.used = used.contains(&key);
                        report(&summary, page, sink);
                        fonts.insert(key, summary);
                    }
                    Err(reason) => sink.error(
                        codes::FONT_INVALID,
                        Some(page),
                        format!("font resource /{}: {reason}", lossy(name)),
                    ),
                }
                // Type 3 glyph procedures have resources of their own.
                if let Some(own) = get_dict(doc, dict, b"Resources") {
                    if id.is_none_or(|id| visited.insert(id)) {
                        queue.push(own);
                    }
                }
            }
        }
        // Form XObjects and tiling patterns carry nested resources.
        for category in [&b"XObject"[..], b"Pattern"] {
            let Some(entries) = get_dict(doc, resources, category) else { continue };
            for (_, entry) in entries {
                let (id, Some(Object::Stream(stream))) = resolve_with_id(doc, entry) else { continue };
                if let Some(id) = id {
                    if !visited.insert(id) {
                        continue;
                    }
                }
                if let Some(nested) = get_dict(doc, &stream.dict, b"Resources") {
                    queue.push(nested);
                }
            }
        }
    }
    let mut summaries: Vec<FontSummary> = fonts.into_values().collect();
    summaries.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.subtype.cmp(&b.subtype)));
    summaries
}

fn report(font: &FontSummary, page: u32, sink: &mut Sink) {
    if font.subtype == "Type3" {
        sink.warning(
            codes::FONT_TYPE3,
            Some(page),
            format!("font {} is a Type 3 font: its glyphs are drawings or bitmaps of unknown quality", font.name),
        );
    } else if !font.embedded {
        sink.error(
            codes::FONT_NOT_EMBEDDED,
            Some(page),
            format!("font {} ({}) is not embedded", font.name, font.subtype),
        );
    } else if !font.subset {
        sink.info(
            codes::FONT_NOT_SUBSET,
            Some(page),
            format!("font {} ({}) is embedded in full, not as a subset", font.name, font.subtype),
        );
    }
}

/// Reads a font dictionary. `Err` describes why it is malformed.
fn analyse(doc: &Document, font: &Dictionary) -> Result<FontSummary, String> {
    let subtype = get_name(doc, font, b"Subtype").ok_or("the font has no Subtype")?;
    let base_font = get_name(doc, font, b"BaseFont").map(lossy);
    let (subtype_text, embedded, name) = match subtype {
        b"Type3" => {
            let name = get_name(doc, font, b"Name").map(lossy).or(base_font);
            ("Type3".to_string(), true, name.unwrap_or_else(|| "(unnamed Type 3 font)".to_string()))
        }
        b"Type0" => {
            let descendant = get_array(doc, font, b"DescendantFonts")
                .and_then(<[Object]>::first)
                .and_then(|d| as_dict(doc, d))
                .ok_or("the Type0 font has no descendant font")?;
            let descendant_type = get_name(doc, descendant, b"Subtype").map_or_else(|| "?".to_string(), lossy);
            let name = base_font
                .or_else(|| get_name(doc, descendant, b"BaseFont").map(lossy))
                .ok_or("the font has no BaseFont")?;
            (format!("Type0/{descendant_type}"), has_font_program(doc, descendant), name)
        }
        b"Type1" | b"MMType1" | b"TrueType" => {
            (lossy(subtype), has_font_program(doc, font), base_font.ok_or("the font has no BaseFont")?)
        }
        other => return Err(format!("unknown font Subtype {}", lossy(other))),
    };
    let subset_prefix = subset_prefix(&name);
    Ok(FontSummary {
        name,
        subtype: subtype_text,
        embedded,
        subset: subset_prefix.is_some(),
        subset_prefix,
        used: false,
    })
}

/// Whether the font's descriptor references a non-empty font program stream.
fn has_font_program(doc: &Document, font: &Dictionary) -> bool {
    let Some(descriptor) = get_dict(doc, font, b"FontDescriptor") else { return false };
    [&b"FontFile"[..], b"FontFile2", b"FontFile3"].iter().any(|key| {
        matches!(get(doc, descriptor, key), Some(Object::Stream(stream)) if !stream.content.is_empty())
    })
}

/// The six upper-case letters before the `+` of a subset font name.
fn subset_prefix(name: &str) -> Option<String> {
    let (prefix, rest) = name.split_once('+')?;
    (prefix.len() == 6 && prefix.bytes().all(|b| b.is_ascii_uppercase()) && !rest.is_empty())
        .then(|| prefix.to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use lopdf::{Stream, dictionary};

    use super::*;
    use crate::report::Severity;

    fn font_file(doc: &mut Document) -> ObjectId {
        doc.add_object(Stream::new(dictionary! {}, vec![0, 1, 0, 0]))
    }

    fn run(doc: &Document, resources: &Dictionary, used: &BTreeSet<FontKey>) -> (Vec<FontSummary>, Vec<(&'static str, Severity)>) {
        let mut sink = Sink::default();
        let fonts = check_fonts(doc, Some(resources), used, 1, &mut sink);
        (fonts, sink.into_sorted().into_iter().map(|f| (f.code, f.severity)).collect())
    }

    #[test]
    fn embedded_subset_fonts_pass_and_are_described() {
        let mut doc = Document::with_version("1.6");
        let file = font_file(&mut doc);
        let descriptor = doc.add_object(dictionary! { "Type" => "FontDescriptor", "FontFile2" => file });
        let cid = doc.add_object(dictionary! {
            "Type" => "Font", "Subtype" => "CIDFontType2", "BaseFont" => "ABCDEF+Spectral-Regular",
            "FontDescriptor" => descriptor,
        });
        let type0 = doc.add_object(dictionary! {
            "Type" => "Font", "Subtype" => "Type0", "BaseFont" => "ABCDEF+Spectral-Regular",
            "Encoding" => "Identity-H", "DescendantFonts" => vec![cid.into()],
        });
        let truetype = doc.add_object(dictionary! {
            "Type" => "Font", "Subtype" => "TrueType", "BaseFont" => "GHIJKL+BebasNeue",
            "FontDescriptor" => dictionary! { "FontFile2" => file },
        });
        let resources = dictionary! { "Font" => dictionary! { "F1" => type0, "F2" => truetype } };
        let used: BTreeSet<FontKey> = [FontKey::Id(type0)].into_iter().collect();
        let (fonts, findings) = run(&doc, &resources, &used);
        assert!(findings.is_empty(), "{findings:?}");
        assert_eq!(fonts.len(), 2);
        assert_eq!(fonts[0].name, "ABCDEF+Spectral-Regular");
        assert_eq!(fonts[0].subtype, "Type0/CIDFontType2");
        assert!(fonts[0].embedded && fonts[0].subset && fonts[0].used);
        assert_eq!(fonts[0].subset_prefix.as_deref(), Some("ABCDEF"));
        assert_eq!(fonts[1].subtype, "TrueType");
        assert!(fonts[1].embedded && !fonts[1].used);
    }

    #[test]
    fn missing_font_programs_are_errors() {
        let mut doc = Document::with_version("1.6");
        let empty_file = doc.add_object(Stream::new(dictionary! {}, Vec::new()));
        // The standard 14: no descriptor at all.
        let helvetica = doc.add_object(dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica" });
        // A descriptor without a font file.
        let arial = doc.add_object(dictionary! {
            "Type" => "Font", "Subtype" => "TrueType", "BaseFont" => "ArialMT",
            "FontDescriptor" => dictionary! { "Type" => "FontDescriptor", "FontName" => "ArialMT" },
        });
        // A font file that is empty, one that dangles, and one that is not a stream.
        let empty = doc.add_object(dictionary! {
            "Subtype" => "Type1", "BaseFont" => "Empty", "FontDescriptor" => dictionary! { "FontFile" => empty_file },
        });
        let dangling = doc.add_object(dictionary! {
            "Subtype" => "Type1", "BaseFont" => "Dangling",
            "FontDescriptor" => dictionary! { "FontFile3" => Object::Reference((4242, 0)) },
        });
        let not_stream = doc.add_object(dictionary! {
            "Subtype" => "Type1", "BaseFont" => "NotStream", "FontDescriptor" => dictionary! { "FontFile" => 5 },
        });
        // A composite font whose descendant is not embedded.
        let cid = doc.add_object(dictionary! { "Subtype" => "CIDFontType0", "BaseFont" => "Kozuka" });
        let type0 = doc.add_object(dictionary! {
            "Subtype" => "Type0", "BaseFont" => "Kozuka-Identity-H", "DescendantFonts" => vec![cid.into()],
        });
        let resources = dictionary! { "Font" => dictionary! {
            "F1" => helvetica, "F2" => arial, "F3" => empty, "F4" => dangling, "F5" => not_stream, "F6" => type0,
        } };
        let (fonts, findings) = run(&doc, &resources, &BTreeSet::new());
        assert_eq!(fonts.len(), 6);
        assert!(fonts.iter().all(|f| !f.embedded));
        assert_eq!(findings, vec![(codes::FONT_NOT_EMBEDDED, Severity::Error); 6]);
    }

    #[test]
    fn type3_full_embedding_and_malformed_fonts() {
        let mut doc = Document::with_version("1.6");
        let file = font_file(&mut doc);
        let type3 = doc.add_object(dictionary! { "Subtype" => "Type3", "Name" => "Bitmap" });
        let full = doc.add_object(dictionary! {
            "Subtype" => "Type1", "BaseFont" => "Courier+Prime",
            "FontDescriptor" => dictionary! { "FontFile3" => file },
        });
        let no_subtype = doc.add_object(dictionary! { "BaseFont" => "X" });
        let no_descendant = doc.add_object(dictionary! { "Subtype" => "Type0", "BaseFont" => "Y" });
        let weird = doc.add_object(dictionary! { "Subtype" => "OpenType", "BaseFont" => "Z" });
        let resources = dictionary! { "Font" => dictionary! {
            "A" => type3, "B" => full, "C" => no_subtype, "D" => no_descendant, "E" => weird,
            "F" => 12, "G" => Object::Reference((777, 0)),
        } };
        let (fonts, findings) = run(&doc, &resources, &BTreeSet::new());
        assert_eq!(fonts.len(), 2);
        let full_font = fonts.iter().find(|f| f.name == "Courier+Prime").unwrap();
        assert!(full_font.embedded && !full_font.subset, "a + without a six-letter prefix is not a subset tag");
        assert_eq!(
            findings,
            vec![
                (codes::FONT_TYPE3, Severity::Warning),
                (codes::FONT_NOT_SUBSET, Severity::Info),
                (codes::FONT_INVALID, Severity::Error),
                (codes::FONT_INVALID, Severity::Error),
                (codes::FONT_INVALID, Severity::Error),
                (codes::FONT_INVALID, Severity::Error),
                (codes::FONT_INVALID, Severity::Error),
            ]
        );
    }

    #[test]
    fn fonts_inside_forms_patterns_and_type3_resources_are_found_once() {
        let mut doc = Document::with_version("1.6");
        let helvetica = doc.add_object(dictionary! { "Subtype" => "Type1", "BaseFont" => "Helvetica" });
        let times = doc.add_object(dictionary! { "Subtype" => "Type1", "BaseFont" => "Times-Roman" });
        let symbol = doc.add_object(dictionary! { "Subtype" => "Type1", "BaseFont" => "Symbol" });
        // Two forms that reference each other, each with a font.
        let form_a = doc.new_object_id();
        let form_b = doc.add_object(Stream::new(
            dictionary! { "Subtype" => "Form", "Resources" => dictionary! {
                "Font" => dictionary! { "T" => times },
                "XObject" => dictionary! { "A" => form_a },
            } },
            Vec::new(),
        ));
        doc.objects.insert(
            form_a,
            Object::Stream(Stream::new(
                dictionary! { "Subtype" => "Form", "Resources" => dictionary! {
                    "Font" => dictionary! { "H" => helvetica },
                    "XObject" => dictionary! { "B" => form_b },
                } },
                Vec::new(),
            )),
        );
        let pattern = doc.add_object(Stream::new(
            dictionary! { "PatternType" => 1, "Resources" => dictionary! { "Font" => dictionary! { "S" => symbol } } },
            Vec::new(),
        ));
        let resources = dictionary! {
            "XObject" => dictionary! { "A" => form_a, "A2" => form_a },
            "Pattern" => dictionary! { "P" => pattern },
            "Font" => dictionary! { "H2" => helvetica },
        };
        let (fonts, findings) = run(&doc, &resources, &BTreeSet::new());
        let names: Vec<&str> = fonts.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["Helvetica", "Symbol", "Times-Roman"]);
        assert_eq!(findings.len(), 3);
    }

    #[test]
    fn subset_prefixes() {
        assert_eq!(subset_prefix("ABCDEF+Name").as_deref(), Some("ABCDEF"));
        assert_eq!(subset_prefix("ABCDE+Name"), None);
        assert_eq!(subset_prefix("abcdef+Name"), None);
        assert_eq!(subset_prefix("ABCDEF+"), None);
        assert_eq!(subset_prefix("Name"), None);
        assert_eq!(subset_prefix("ABCDEFG+Name"), None);
    }
}
