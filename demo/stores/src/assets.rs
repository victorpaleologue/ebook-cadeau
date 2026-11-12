//! Process-wide cache of the generated static assets: cover SVGs, cover PNGs and the
//! unstamped EPUB of every book.
//!
//! The assets depend only on the catalogue, so both stores (and every test in a process)
//! share one copy. EPUBs and small covers are built on first use ("at startup": the binary
//! and [`crate::app`] touch the cache before serving); the large print renditions are
//! rasterised on first request and then kept.

use std::sync::OnceLock;

use crate::catalogue::{self, Book};
use crate::covers;
use crate::epub;
use crate::error::StoreError;

/// The generated files of one book.
#[derive(Debug)]
pub struct BookAssets {
    /// The catalogue entry.
    pub book: &'static Book,
    /// The typographic cover as SVG.
    pub cover_svg: String,
    /// Small PNG rendition of the cover (embedded in the EPUBs).
    pub cover_thumb_png: Vec<u8>,
    /// The unstamped demo edition.
    pub epub: Vec<u8>,
    print_png: OnceLock<Result<Vec<u8>, StoreError>>,
}

impl BookAssets {
    /// The print rendition of the cover (opaque RGB PNG, at least 1300 × 2100 px),
    /// rasterised on first use.
    ///
    /// # Errors
    /// [`StoreError::Cover`] when rasterisation fails.
    pub fn print_png(&self) -> Result<&[u8], StoreError> {
        self.print_png
            .get_or_init(|| covers::render_png(&self.cover_svg, covers::PRINT_WIDTH_PX))
            .as_ref()
            .map(Vec::as_slice)
            .map_err(Clone::clone)
    }

    /// A stamped copy of the EPUB.
    ///
    /// # Errors
    /// [`StoreError::Epub`] when the container cannot be written.
    pub fn stamped_epub(&self, watermark: &epub::Watermark) -> Result<Vec<u8>, StoreError> {
        epub::build(self.book, &self.cover_thumb_png, Some(watermark))
    }
}

/// All generated assets.
#[derive(Debug)]
pub struct Assets {
    books: Vec<BookAssets>,
}

impl Assets {
    /// The process-wide instance, generated on first call.
    ///
    /// # Errors
    /// The error of the first failed generation, repeated on every call.
    pub fn get() -> Result<&'static Assets, StoreError> {
        static ASSETS: OnceLock<Result<Assets, StoreError>> = OnceLock::new();
        ASSETS.get_or_init(Assets::generate).as_ref().map_err(Clone::clone)
    }

    fn generate() -> Result<Assets, StoreError> {
        let mut books = Vec::with_capacity(catalogue::books().len());
        for book in catalogue::books() {
            let cover_svg = covers::svg(book);
            let cover_thumb_png = covers::render_png(&cover_svg, covers::THUMB_WIDTH_PX)?;
            let epub = epub::build(book, &cover_thumb_png, None)?;
            books.push(BookAssets {
                book,
                cover_svg,
                cover_thumb_png,
                epub,
                print_png: OnceLock::new(),
            });
        }
        Ok(Assets { books })
    }

    /// The assets of the book `id`.
    #[must_use]
    pub fn book(&self, id: &str) -> Option<&BookAssets> {
        self.books.iter().find(|b| b.book.id == id)
    }

    /// Rasterises every print cover now (the binary calls this at startup so that the
    /// first checkout is not slowed down).
    ///
    /// # Errors
    /// [`StoreError::Cover`] when a rasterisation fails.
    pub fn warm_up(&self) -> Result<(), StoreError> {
        self.books.iter().try_for_each(|b| b.print_png().map(|_| ()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assets_exist_for_every_book_and_are_shared() {
        let a = Assets::get().unwrap();
        let b = Assets::get().unwrap();
        assert!(std::ptr::eq(a, b));
        for book in catalogue::books() {
            let assets = a.book(book.id).unwrap();
            assert!(assets.cover_svg.starts_with("<svg"));
            assert!(assets.cover_thumb_png.starts_with(b"\x89PNG\r\n\x1a\n"));
            assert!(assets.epub.starts_with(b"PK\x03\x04"));
            assert!(assets.epub.len() < 200_000, "{} bytes", assets.epub.len());
        }
        assert!(a.book("unknown").is_none());
    }
}
