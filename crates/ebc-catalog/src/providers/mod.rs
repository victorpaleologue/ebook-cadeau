//! The providers, one module per service (the SRU catalogues share one).

use ebc_isbn::Isbn13;
use ebc_model::{BookFormat, BookMeta, HttpRequest};

use crate::error::CatalogError;
use crate::normalize::{fold_key, normalize_record};
use crate::provider::ProviderInfo;
use crate::util::redact;

pub mod crossref;
pub mod finna;
pub mod googlebooks;
pub mod inventaire;
pub mod libris;
pub mod lobid;
pub mod openlibrary;
pub mod sru;
pub mod sudoc;
pub mod wikidata;

/// A `GET` asking for JSON.
pub(crate) fn get_json(url: String) -> HttpRequest {
    HttpRequest::get(url).with_header("Accept", "application/json")
}

/// A `GET` asking for XML.
pub(crate) fn get_xml(url: String) -> HttpRequest {
    HttpRequest::get(url).with_header("Accept", "application/xml, text/xml;q=0.9")
}

/// The error for a request `parse` does not recognise as its own.
pub(crate) fn unknown_request(info: &ProviderInfo, request: &HttpRequest) -> CatalogError {
    CatalogError::UnknownRequest { provider: info.id, url: redact(&request.url) }
}

/// A partial record (the answer to a follow-up request): carries the ISBN, is normalised,
/// credits no source — the main record of the same provider does.
pub(crate) fn fragment(isbn: &Isbn13, mut book: BookMeta) -> BookMeta {
    book.isbn13 = Some(*isbn);
    normalize_record(&mut book);
    book
}

/// Reads a product form out of words such as `"Paperback"`, `"Poche"`, `"E-Book"`.
pub(crate) fn format_from_text(text: &str) -> Option<BookFormat> {
    let key = fold_key(text);
    let has = |needles: &[&str]| needles.iter().any(|n| key.contains(n));
    if has(&["ebook", "e book", "kindle", "epub", "electronic", "online", "numerique", "digital"]) {
        Some(BookFormat::Ebook)
    } else if has(&["audio", "horbuch"]) {
        Some(BookFormat::Audio)
    } else if has(&["paperback", "poche", "broche", "softcover", "taschenbuch", "pocket"]) {
        Some(BookFormat::Paperback)
    } else if has(&["hardcover", "hardback", "relie", "gebunden", "inbunden"]) {
        Some(BookFormat::Hardcover)
    } else {
        None
    }
}

/// `true` for a Wikidata-style identifier (`Q42`), which is what label services return
/// when an entity has no label.
pub(crate) fn is_qid(text: &str) -> bool {
    text.strip_prefix('Q').is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_and_qids() {
        assert_eq!(format_from_text("Mass Market Paperback"), Some(BookFormat::Paperback));
        assert_eq!(format_from_text("E-Book"), Some(BookFormat::Ebook));
        assert_eq!(format_from_text("Relié"), Some(BookFormat::Hardcover));
        assert_eq!(format_from_text("Book"), None);
        assert!(is_qid("Q150"));
        assert!(!is_qid("Quentin"));
        assert!(!is_qid("Q"));
    }
}
