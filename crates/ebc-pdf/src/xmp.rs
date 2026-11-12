//! XMP metadata packet.
//!
//! The packet mirrors the Info dictionary (PDF/X-4 requires both to agree) and carries the
//! PDF/X identification: `pdfxid:GTS_PDFXVersion` for PDF/X-4, and the Info-dictionary
//! mirror `pdfx:GTS_PDFXVersion` / `pdfx:GTS_PDFXConformance` for PDF/X-1a and PDF/X-3.

use ebc_scene::PdfStandard;

/// The values written to the XMP packet. All strings are already sanitised with
/// [`sanitize`].
pub(crate) struct XmpFields<'a> {
    pub(crate) standard: PdfStandard,
    pub(crate) title: &'a str,
    pub(crate) author: Option<&'a str>,
    pub(crate) subject: Option<&'a str>,
    pub(crate) keywords: Option<&'a str>,
    pub(crate) creator_tool: Option<&'a str>,
    pub(crate) producer: &'a str,
    pub(crate) language: Option<&'a str>,
    /// ISO 8601 creation date, also used as modification and metadata date.
    pub(crate) date: Option<&'a str>,
    /// `uuid:…` identifiers derived from the document content.
    pub(crate) document_id: &'a str,
    pub(crate) instance_id: &'a str,
}

/// Removes characters that XML 1.0 cannot represent, so that the Info dictionary and the
/// XMP packet carry exactly the same strings.
pub(crate) fn sanitize(text: &str) -> String {
    text.chars()
        .filter(|&c| {
            matches!(c, '\t' | '\n' | '\r' | '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}')
                || c >= '\u{10000}'
        })
        .collect()
}

/// Escapes text for use as XML character data.
fn escape(text: &str, out: &mut String) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\r' => out.push_str("&#13;"),
            _ => out.push(c),
        }
    }
}

fn simple(out: &mut String, tag: &str, value: &str) {
    out.push_str("   <");
    out.push_str(tag);
    out.push('>');
    escape(value, out);
    out.push_str("</");
    out.push_str(tag);
    out.push_str(">\n");
}

fn container(out: &mut String, tag: &str, kind: &str, lang_alt: bool, value: &str) {
    out.push_str("   <");
    out.push_str(tag);
    out.push_str("><rdf:");
    out.push_str(kind);
    out.push_str("><rdf:li");
    if lang_alt {
        out.push_str(" xml:lang=\"x-default\"");
    }
    out.push('>');
    escape(value, out);
    out.push_str("</rdf:li></rdf:");
    out.push_str(kind);
    out.push_str("></");
    out.push_str(tag);
    out.push_str(">\n");
}

/// Builds the complete XMP packet (UTF-8, with a writable padding-free trailer).
pub(crate) fn packet(f: &XmpFields<'_>) -> Vec<u8> {
    let mut out = String::with_capacity(2048);
    out.push_str("<?xpacket begin=\"\u{FEFF}\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>\n");
    out.push_str("<x:xmpmeta xmlns:x=\"adobe:ns:meta/\">\n");
    out.push_str(" <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n");
    out.push_str("  <rdf:Description rdf:about=\"\"\n");
    out.push_str("    xmlns:dc=\"http://purl.org/dc/elements/1.1/\"\n");
    out.push_str("    xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\"\n");
    out.push_str("    xmlns:xmpMM=\"http://ns.adobe.com/xap/1.0/mm/\"\n");
    out.push_str("    xmlns:pdf=\"http://ns.adobe.com/pdf/1.3/\"\n");
    out.push_str("    xmlns:pdfx=\"http://ns.adobe.com/pdfx/1.3/\"\n");
    out.push_str("    xmlns:pdfxid=\"http://www.npes.org/pdfx/ns/id/\">\n");

    simple(&mut out, "dc:format", "application/pdf");
    if !f.title.is_empty() {
        container(&mut out, "dc:title", "Alt", true, f.title);
    }
    if let Some(author) = f.author {
        container(&mut out, "dc:creator", "Seq", false, author);
    }
    if let Some(subject) = f.subject {
        container(&mut out, "dc:description", "Alt", true, subject);
    }
    if let Some(language) = f.language {
        container(&mut out, "dc:language", "Bag", false, language);
    }
    if let Some(tool) = f.creator_tool {
        simple(&mut out, "xmp:CreatorTool", tool);
    }
    if let Some(date) = f.date {
        simple(&mut out, "xmp:CreateDate", date);
        simple(&mut out, "xmp:ModifyDate", date);
        simple(&mut out, "xmp:MetadataDate", date);
    }
    simple(&mut out, "xmpMM:DocumentID", f.document_id);
    simple(&mut out, "xmpMM:InstanceID", f.instance_id);
    simple(&mut out, "xmpMM:RenditionClass", "default");
    simple(&mut out, "xmpMM:VersionID", "1");
    simple(&mut out, "pdf:Producer", f.producer);
    if let Some(keywords) = f.keywords {
        simple(&mut out, "pdf:Keywords", keywords);
    }
    simple(&mut out, "pdf:Trapped", "False");
    match f.standard {
        PdfStandard::Plain => {}
        PdfStandard::X1a2001 => {
            simple(&mut out, "pdfx:GTS_PDFXVersion", "PDF/X-1:2001");
            simple(&mut out, "pdfx:GTS_PDFXConformance", "PDF/X-1a:2001");
        }
        PdfStandard::X3_2002 => simple(&mut out, "pdfx:GTS_PDFXVersion", "PDF/X-3:2002"),
        PdfStandard::X4 => simple(&mut out, "pdfxid:GTS_PDFXVersion", "PDF/X-4"),
    }

    out.push_str("  </rdf:Description>\n");
    out.push_str(" </rdf:RDF>\n");
    out.push_str("</x:xmpmeta>\n");
    out.push_str("<?xpacket end=\"r\"?>");
    out.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(standard: PdfStandard) -> XmpFields<'static> {
        XmpFields {
            standard,
            title: "L'Étranger <& \"co\">",
            author: Some("Albert Camus"),
            subject: Some("A card"),
            keywords: Some("gift, ebook"),
            creator_tool: Some("ebc-card"),
            producer: "ebc-pdf",
            language: Some("fr-FR"),
            date: Some("2026-11-12T08:30:00Z"),
            document_id: "uuid:00000000-0000-3000-8000-000000000000",
            instance_id: "uuid:00000000-0000-3000-8000-000000000001",
        }
    }

    #[test]
    fn escapes_markup_and_keeps_unicode() {
        let xml = String::from_utf8(packet(&fields(PdfStandard::X4))).expect("utf-8");
        assert!(xml.contains(
            "<dc:title><rdf:Alt><rdf:li xml:lang=\"x-default\">L'Étranger &lt;&amp; &quot;co&quot;&gt;</rdf:li></rdf:Alt></dc:title>"
        ));
        assert!(xml.contains("<dc:creator><rdf:Seq><rdf:li>Albert Camus</rdf:li></rdf:Seq>"));
        assert!(xml.contains("<dc:language><rdf:Bag><rdf:li>fr-FR</rdf:li></rdf:Bag>"));
        assert!(xml.contains("<xmp:CreateDate>2026-11-12T08:30:00Z</xmp:CreateDate>"));
        assert!(xml.contains("<xmp:ModifyDate>2026-11-12T08:30:00Z</xmp:ModifyDate>"));
        assert!(xml.contains("<xmp:MetadataDate>2026-11-12T08:30:00Z</xmp:MetadataDate>"));
        assert!(xml.contains("<pdf:Trapped>False</pdf:Trapped>"));
        assert!(xml.starts_with("<?xpacket begin=\"\u{FEFF}\""));
        assert!(xml.ends_with("<?xpacket end=\"r\"?>"));
        // No stray markup characters outside the tags we wrote.
        assert_eq!(xml.matches("<dc:title>").count(), 1);
    }

    #[test]
    fn version_markers_per_standard() {
        let x4 = String::from_utf8(packet(&fields(PdfStandard::X4))).expect("utf-8");
        assert!(x4.contains("<pdfxid:GTS_PDFXVersion>PDF/X-4</pdfxid:GTS_PDFXVersion>"));
        assert!(!x4.contains("<pdfx:GTS_PDFXVersion>"));

        let x1 = String::from_utf8(packet(&fields(PdfStandard::X1a2001))).expect("utf-8");
        assert!(x1.contains("<pdfx:GTS_PDFXVersion>PDF/X-1:2001</pdfx:GTS_PDFXVersion>"));
        assert!(x1.contains("<pdfx:GTS_PDFXConformance>PDF/X-1a:2001</pdfx:GTS_PDFXConformance>"));
        assert!(!x1.contains("<pdfxid:GTS_PDFXVersion>"));

        let x3 = String::from_utf8(packet(&fields(PdfStandard::X3_2002))).expect("utf-8");
        assert!(x3.contains("<pdfx:GTS_PDFXVersion>PDF/X-3:2002</pdfx:GTS_PDFXVersion>"));
        assert!(!x3.contains("GTS_PDFXConformance"));

        let plain = String::from_utf8(packet(&fields(PdfStandard::Plain))).expect("utf-8");
        assert!(!plain.contains("GTS_PDFX"));
    }

    #[test]
    fn optional_fields_are_omitted() {
        let mut f = fields(PdfStandard::Plain);
        f.title = "";
        f.author = None;
        f.subject = None;
        f.keywords = None;
        f.creator_tool = None;
        f.language = None;
        f.date = None;
        let xml = String::from_utf8(packet(&f)).expect("utf-8");
        for tag in [
            "dc:title",
            "dc:creator",
            "dc:description",
            "dc:language",
            "xmp:CreatorTool",
            "xmp:CreateDate",
            "pdf:Keywords",
        ] {
            assert!(!xml.contains(tag), "{tag} should be omitted");
        }
        assert!(xml.contains("<pdf:Producer>ebc-pdf</pdf:Producer>"));
    }

    #[test]
    fn sanitize_drops_characters_xml_cannot_hold() {
        assert_eq!(sanitize("a\u{0}b\u{8}c\u{B}d\u{1F}e\u{FFFE}f\u{FFFF}g"), "abcdefg");
        assert_eq!(sanitize("tab\there\nnew\rline"), "tab\there\nnew\rline");
        assert_eq!(sanitize("émoji 🎁 ok"), "émoji 🎁 ok");
    }
}
