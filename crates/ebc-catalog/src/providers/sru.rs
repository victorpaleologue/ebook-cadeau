//! National and union catalogues that speak SRU and answer with MARC: BnF (UNIMARC in
//! MarcXchange, or Dublin Core), DNB, K10plus and the Library of Congress (MARC 21).

use ebc_isbn::Isbn13;
use ebc_model::{HttpRequest, HttpResponse};

use crate::error::CatalogError;
use crate::marc::{self, Extracted};
use crate::provider::{Parsed, Permission, Provider, ProviderInfo, SourceClass, finalize};
use crate::providers::{get_xml, unknown_request};
use crate::util::{Found, check_http, encode_component, url_with_query, without_fragment};
use crate::xml::{MarcRecord, marc_records, parse_xml, sru_records};

static BNF: ProviderInfo = ProviderInfo {
    id: "bnf",
    name: "Bibliothèque nationale de France — catalogue général (SRU)",
    homepage: "https://catalogue.bnf.fr/",
    data_licence: "Licence Ouverte / Open Licence (Etalab)",
    terms_url: "https://www.bnf.fr/fr/conditions-de-reutilisation-des-donnees-de-la-bnf",
    requires_key: false,
    browser_cors: false,
    commercial_use: Permission::Allowed,
    attribution: Some(
        "Données bibliographiques : Bibliothèque nationale de France (catalogue général)",
    ),
    min_interval_ms: 1000,
    licence_id: Some("etalab-2.0"),
    permanent_storage: Permission::Allowed,
    hosts: &["catalogue.bnf.fr"],
    notes: "Licence statement read in a 2016 BnF page; the licence version applied today and \
            the terms URL are from memory. Attribution must name the source and a date: show \
            the retrieval date next to it. No published rate limit is known; 1 request/s is \
            our own default. CORS unverified. The record schema name `unimarcXchange` is \
            from memory. Cover images are not covered by the metadata licence.",
    class: SourceClass::Open,
    default_enabled: true,
};

static DNB: ProviderInfo = ProviderInfo {
    id: "dnb",
    name: "Deutsche Nationalbibliothek (SRU)",
    homepage: "https://www.dnb.de/",
    data_licence: "CC0 1.0",
    terms_url: "https://www.dnb.de/EN/Professionell/Metadatendienste/Datenbezug/SRU/sru_node.html",
    requires_key: false,
    browser_cors: false,
    commercial_use: Permission::Allowed,
    attribution: None,
    min_interval_ms: 1000,
    licence_id: Some("CC0-1.0"),
    permanent_storage: Permission::Allowed,
    hosts: &["services.dnb.de"],
    notes: "CC0 for the data is from memory (the DNBLab README only says the data is freely \
            accessible); terms URL from memory. No access token appears in current client \
            code. CORS unverified.",
    class: SourceClass::Open,
    default_enabled: true,
};

static K10PLUS: ProviderInfo = ProviderInfo {
    id: "k10plus",
    name: "K10plus — GBV and BSZ union catalogue (SRU)",
    homepage: "https://www.k10plus.de/",
    data_licence: "CC0 1.0",
    terms_url: "https://wiki.k10plus.de/display/K10PLUS/Open+Data",
    requires_key: false,
    browser_cors: false,
    commercial_use: Permission::Allowed,
    attribution: None,
    min_interval_ms: 1000,
    licence_id: Some("CC0-1.0"),
    permanent_storage: Permission::Allowed,
    hosts: &["sru.k10plus.de"],
    notes: "CC0 and the terms URL are from memory. Endpoint read in the Zotero translator \
            (2024-10). The `recordSchema=marcxml` parameter is our addition. CORS unverified.",
    class: SourceClass::Open,
    default_enabled: true,
};

static LOC: ProviderInfo = ProviderInfo {
    id: "loc",
    name: "Library of Congress catalog (SRU)",
    homepage: "https://catalog.loc.gov/",
    data_licence: "US government work, no known copyright restriction",
    terms_url: "https://www.loc.gov/legal/",
    requires_key: false,
    browser_cors: false,
    commercial_use: Permission::Allowed,
    attribution: None,
    min_interval_ms: 3000,
    licence_id: None,
    permanent_storage: Permission::Allowed,
    hosts: &["lx2.loc.gov"],
    notes: "Off by default: the research ranks it \"skip\" and the fate of the lx2.loc.gov \
            SRU endpoint after the 2025 platform change is unknown. Licence and limits are \
            from memory.",
    class: SourceClass::Open,
    default_enabled: false,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Catalogue {
    Bnf,
    Dnb,
    K10plus,
    Loc,
}

/// An SRU catalogue queried by ISBN.
///
/// | Constructor | Endpoint | Records |
/// |---|---|---|
/// | [`SruProvider::bnf`] | `https://catalogue.bnf.fr/api/SRU` (SRU 1.2) | UNIMARC in MarcXchange; Dublin Core is understood too |
/// | [`SruProvider::dnb`] | `https://services.dnb.de/sru/dnb` (SRU 1.1) | MARC 21 |
/// | [`SruProvider::k10plus`] | `https://sru.k10plus.de/opac-de-627` (SRU 1.1) | MARC 21 |
/// | [`SruProvider::loc`] | `https://lx2.loc.gov/sru/lcdb` (SRU 1.1) | MARC 21 |
#[derive(Debug, Clone)]
pub struct SruProvider {
    catalogue: Catalogue,
    endpoint: String,
    record_schema: String,
}

impl SruProvider {
    fn new(catalogue: Catalogue, endpoint: &str, record_schema: &str) -> Self {
        Self { catalogue, endpoint: endpoint.to_owned(), record_schema: record_schema.to_owned() }
    }

    /// BnF catalogue général. Sends the ISBN-10 and the ISBN-13 in one query for 978
    /// ISBNs, because old records carry only the ISBN-10.
    #[must_use]
    pub fn bnf() -> Self {
        Self::new(Catalogue::Bnf, "https://catalogue.bnf.fr/api/SRU", "unimarcXchange")
    }

    /// Deutsche Nationalbibliothek.
    #[must_use]
    pub fn dnb() -> Self {
        Self::new(Catalogue::Dnb, "https://services.dnb.de/sru/dnb", "MARC21-xml")
    }

    /// K10plus union catalogue (GBV and BSZ).
    #[must_use]
    pub fn k10plus() -> Self {
        Self::new(Catalogue::K10plus, "https://sru.k10plus.de/opac-de-627", "marcxml")
    }

    /// Library of Congress. Off by default; see the provider notes.
    #[must_use]
    pub fn loc() -> Self {
        Self::new(Catalogue::Loc, "https://lx2.loc.gov/sru/lcdb", "marcxml")
    }

    /// Replaces the SRU endpoint URL (tests and the CLI point it at a fixture server).
    #[must_use]
    pub fn with_base_url(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into().trim_end_matches(['?', '&']).to_owned();
        self
    }

    /// Replaces the `recordSchema` parameter, e.g. `"dublincore"` for the BnF. The parser
    /// recognises MARC and Dublin Core records whatever was asked for.
    #[must_use]
    pub fn with_record_schema(mut self, schema: impl Into<String>) -> Self {
        self.record_schema = schema.into();
        self
    }

    fn query(&self, isbn: &Isbn13) -> String {
        let plain = isbn.to_plain();
        match self.catalogue {
            Catalogue::Bnf => match isbn.to_isbn10() {
                Ok(ten) => {
                    format!("(bib.isbn all \"{}\" or bib.isbn all \"{plain}\")", ten.to_plain())
                }
                Err(_) => format!("bib.isbn all \"{plain}\""),
            },
            Catalogue::Dnb => format!("num={plain}"),
            Catalogue::K10plus => format!("pica.isb={plain}"),
            Catalogue::Loc => format!("bath.isbn={plain}"),
        }
    }

    fn record_url(&self, record: &MarcRecord) -> Option<String> {
        let control = record.control("001").map(str::trim).filter(|id| {
            !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        });
        match self.catalogue {
            Catalogue::Bnf => record
                .id
                .as_deref()
                .filter(|id| id.starts_with("ark:/") && !id.contains(char::is_whitespace))
                .map(|ark| format!("https://catalogue.bnf.fr/{ark}")),
            Catalogue::Dnb => control.map(|id| format!("https://d-nb.info/{id}")),
            Catalogue::K10plus => {
                control.map(|ppn| format!("https://opac.k10plus.de/DB=2.299/PPNSET?PPN={ppn}"))
            }
            Catalogue::Loc => record
                .first("010", 'a')
                .map(|lccn| lccn.split_whitespace().collect::<String>())
                .filter(|lccn| !lccn.is_empty() && lccn.chars().all(|c| c.is_ascii_alphanumeric()))
                .map(|lccn| format!("https://lccn.loc.gov/{}", encode_component(&lccn))),
        }
    }
}

impl Provider for SruProvider {
    fn info(&self) -> &ProviderInfo {
        match self.catalogue {
            Catalogue::Bnf => &BNF,
            Catalogue::Dnb => &DNB,
            Catalogue::K10plus => &K10PLUS,
            Catalogue::Loc => &LOC,
        }
    }

    fn lookup(&self, isbn: &Isbn13) -> Vec<HttpRequest> {
        let version = if self.catalogue == Catalogue::Bnf { "1.2" } else { "1.1" };
        let query = self.query(isbn);
        let url = url_with_query(
            &self.endpoint,
            &[
                ("version", version),
                ("operation", "searchRetrieve"),
                ("query", &query),
                ("recordSchema", &self.record_schema),
                ("maximumRecords", "5"),
            ],
        );
        vec![get_xml(url)]
    }

    fn parse(
        &self,
        isbn: &Isbn13,
        request: &HttpRequest,
        response: &HttpResponse,
    ) -> Result<Parsed, CatalogError> {
        let info = self.info();
        if !without_fragment(&request.url).starts_with(&self.endpoint) {
            return Err(unknown_request(info, request));
        }
        // An SRU endpoint answers 200 with zero records for an unknown ISBN; a 404 means
        // the endpoint itself is gone.
        if check_http(info.id, response)? == Found::No {
            return Err(CatalogError::Status { provider: info.id, status: response.status });
        }
        let root = parse_xml(info.id, response)?;
        let mut notes = Vec::new();
        let mut candidates: Vec<Extracted> = Vec::new();
        for data in sru_records(info.id, &root)? {
            let records = marc_records(data);
            if records.is_empty() {
                let mut dublin = Vec::new();
                data.find_all("dc", &mut dublin);
                candidates.extend(dublin.into_iter().map(marc::dublin_core));
            }
            for record in &records {
                let mut extracted = match self.catalogue {
                    Catalogue::Bnf => marc::unimarc(record),
                    _ => marc::marc21(record),
                };
                extracted.url = self.record_url(record).or(extracted.url);
                candidates.push(extracted);
            }
        }
        let records = marc::select(isbn, candidates, &mut notes)
            .map(|chosen| finalize(info, isbn, chosen))
            .into_iter()
            .collect();
        Ok(Parsed { records, follow_up: Vec::new(), notes })
    }
}
