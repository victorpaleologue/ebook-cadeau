//! Bibliographic metadata for one edition of a book.

use ebc_isbn::Isbn13;
use serde::{Deserialize, Serialize};

/// What a contributor did.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContributorRole {
    /// Author of the text.
    #[default]
    Author,
    /// Translator.
    Translator,
    /// Illustrator or cover artist.
    Illustrator,
    /// Editor / volume editor.
    Editor,
    /// Author of a preface, foreword or afterword.
    Preface,
    /// Narrator (audio).
    Narrator,
    /// Any other role, described in free text.
    Other(String),
}

/// A person or organisation that contributed to the book.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Contributor {
    /// Display name in natural order, e.g. `"Albert Camus"`.
    pub name: String,
    /// Role.
    #[serde(default)]
    pub role: ContributorRole,
}

impl Contributor {
    /// An author with the given display name.
    #[must_use]
    pub fn author(name: impl Into<String>) -> Self {
        Self { name: name.into(), role: ContributorRole::Author }
    }
}

/// Product form of an edition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BookFormat {
    /// Digital text (EPUB, Kindle, PDF…).
    Ebook,
    /// Paperback / softcover.
    Paperback,
    /// Hardcover.
    Hardcover,
    /// Audiobook.
    Audio,
    /// Anything else.
    Other,
}

/// Which part of the physical book an image shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CoverKind {
    /// Front cover.
    #[default]
    Front,
    /// Back cover.
    Back,
    /// Spine only.
    Spine,
    /// Full wraparound cover (back + spine + front in one image).
    FullWrap,
}

/// A reference to a cover image somewhere on the web.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CoverRef {
    /// Image URL.
    pub url: String,
    /// Which part of the book the image shows.
    #[serde(default)]
    pub kind: CoverKind,
    /// Width in pixels, if known without downloading.
    #[serde(default)]
    pub width_px: Option<u32>,
    /// Height in pixels, if known without downloading.
    #[serde(default)]
    pub height_px: Option<u32>,
    /// Identifier of the provider that supplied the reference, e.g. `"openlibrary"`.
    pub source: String,
}

/// Where a piece of metadata came from, for attribution and licence compliance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceRef {
    /// Provider identifier, e.g. `"bnf"`.
    pub provider: String,
    /// URL of the human-readable record, if any.
    #[serde(default)]
    pub record_url: Option<String>,
    /// Licence of the data, e.g. `"CC0-1.0"`, `"Etalab-2.0"`.
    #[serde(default)]
    pub licence: Option<String>,
    /// Attribution text required by the licence or terms, if any.
    #[serde(default)]
    pub attribution: Option<String>,
}

/// Metadata for one edition of a book. Every field except `title` may be unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BookMeta {
    /// ISBN-13 of this edition.
    #[serde(default)]
    pub isbn13: Option<Isbn13>,
    /// ISBNs of other editions/formats of the same work (e.g. the print edition of an eBook).
    #[serde(default)]
    pub related_isbns: Vec<Isbn13>,
    /// Main title.
    pub title: String,
    /// Subtitle.
    #[serde(default)]
    pub subtitle: Option<String>,
    /// Contributors, most important first.
    #[serde(default)]
    pub contributors: Vec<Contributor>,
    /// Publisher name.
    #[serde(default)]
    pub publisher: Option<String>,
    /// Imprint, when distinct from the publisher.
    #[serde(default)]
    pub imprint: Option<String>,
    /// Series or collection name (e.g. `"Folio"`).
    #[serde(default)]
    pub series: Option<String>,
    /// Publication date as a (possibly partial) ISO 8601 date: `"1942"`, `"1942-05"`, `"1942-05-19"`.
    #[serde(default)]
    pub published: Option<String>,
    /// Language of the text as a BCP 47 / ISO 639 code, e.g. `"fr"`.
    #[serde(default)]
    pub language: Option<String>,
    /// Page count of the (print-equivalent) edition.
    #[serde(default)]
    pub page_count: Option<u32>,
    /// Publisher's description / blurb, plain text.
    #[serde(default)]
    pub description: Option<String>,
    /// Subject headings or categories.
    #[serde(default)]
    pub subjects: Vec<String>,
    /// Product form.
    #[serde(default)]
    pub format: Option<BookFormat>,
    /// Known cover images, best first.
    #[serde(default)]
    pub covers: Vec<CoverRef>,
    /// Where the metadata came from.
    #[serde(default)]
    pub sources: Vec<SourceRef>,
}

impl BookMeta {
    /// Creates metadata with just a title.
    #[must_use]
    pub fn titled(title: impl Into<String>) -> Self {
        Self { title: title.into(), ..Self::default() }
    }

    /// Names of the contributors whose role is [`ContributorRole::Author`], in order.
    pub fn authors(&self) -> impl Iterator<Item = &str> {
        self.contributors
            .iter()
            .filter(|c| c.role == ContributorRole::Author)
            .map(|c| c.name.as_str())
    }

    /// Authors joined for display: `"A"`, `"A & B"`, `"A, B & C"`; `None` if there are none.
    #[must_use]
    pub fn author_line(&self) -> Option<String> {
        let names: Vec<&str> = self.authors().collect();
        match names.as_slice() {
            [] => None,
            [one] => Some((*one).to_owned()),
            [init @ .., last] => Some(format!("{} & {last}", init.join(", "))),
        }
    }

    /// The first front cover reference, if any.
    #[must_use]
    pub fn front_cover(&self) -> Option<&CoverRef> {
        self.covers.iter().find(|c| c.kind == CoverKind::Front)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn author_line_formats() {
        let mut b = BookMeta::titled("T");
        assert_eq!(b.author_line(), None);
        b.contributors.push(Contributor::author("A"));
        assert_eq!(b.author_line().as_deref(), Some("A"));
        b.contributors.push(Contributor { name: "Tr".into(), role: ContributorRole::Translator });
        b.contributors.push(Contributor::author("B"));
        assert_eq!(b.author_line().as_deref(), Some("A & B"));
        b.contributors.push(Contributor::author("C"));
        assert_eq!(b.author_line().as_deref(), Some("A, B & C"));
    }

    #[test]
    fn json_round_trip_with_defaults() {
        let json = r#"{"title":"L'Étranger","isbn13":"978-2-07-036002-4","contributors":[{"name":"Albert Camus"}]}"#;
        let b: BookMeta = serde_json::from_str(json).expect("parses");
        assert_eq!(b.isbn13.expect("isbn").to_string(), "9782070360024");
        assert_eq!(b.contributors[0].role, ContributorRole::Author);
        let again: BookMeta =
            serde_json::from_str(&serde_json::to_string(&b).expect("serialises")).expect("parses");
        assert_eq!(b, again);
    }

    #[test]
    fn invalid_isbn_is_rejected() {
        assert!(serde_json::from_str::<BookMeta>(r#"{"title":"x","isbn13":"123"}"#).is_err());
    }
}
