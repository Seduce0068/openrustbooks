//! Pure helpers for the GPUI viewer: book opening, chapter navigation and
//! display strings. Kept free of GPUI types so the behaviour is unit-testable
//! without a graphics window.

use openrustbooks_core::{EpubBook, EpubMetadata};

/// Same upper bound as the core reader input gate.
pub const MAX_INPUT_BYTES: u64 = 256 * 1024 * 1024;

pub const UNTITLED: &str = "（無題）";
pub const UNKNOWN_AUTHOR: &str = "（著者不明）";
pub const NO_CHAPTERS: &str = "（章なし）";
pub const EMPTY_BOOK: &str = "表示できる章がありません。";
pub const OPEN_ERROR: &str = "EPUB を開けませんでした";
pub const CHAPTER_ERROR: &str = "章を読み込めませんでした";

/// Build a non-empty title for the header.
pub fn display_title(metadata: &EpubMetadata) -> String {
    metadata
        .title
        .as_deref()
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .unwrap_or(UNTITLED)
        .to_string()
}

/// Build a non-empty author line for the header.
pub fn display_authors(metadata: &EpubMetadata) -> String {
    let names: Vec<&str> = metadata
        .creators
        .iter()
        .map(|name| name.trim())
        .filter(|name| !name.is_empty())
        .collect();
    if names.is_empty() {
        UNKNOWN_AUTHOR.to_string()
    } else {
        names.join("、")
    }
}

/// Human-readable chapter position, e.g. `3 / 12`.
pub fn chapter_label(index: usize, total: usize) -> String {
    format!("{} / {}", if total == 0 { 0 } else { index + 1 }, total)
}

/// Clamp a chapter index to the available range.
pub fn next_index(index: usize, total: usize) -> usize {
    if total == 0 {
        0
    } else {
        index.saturating_add(1).min(total - 1)
    }
}

/// Clamp a chapter index to the available range.
pub fn previous_index(index: usize) -> usize {
    index.saturating_sub(1)
}

/// An opened EPUB plus the currently displayed chapter.
pub struct OpenedBook {
    book: EpubBook,
    index: usize,
    title: String,
    authors: String,
    chapter_title: String,
    text: String,
}

impl OpenedBook {
    /// Open EPUB bytes with the core fail-closed reader and select the first chapter.
    pub fn open(bytes: Vec<u8>) -> Result<Self, &'static str> {
        if bytes.len() as u64 > MAX_INPUT_BYTES {
            return Err(OPEN_ERROR);
        }
        let book = EpubBook::open_owned(bytes).map_err(|_| OPEN_ERROR)?;
        let title = display_title(book.metadata());
        let authors = display_authors(book.metadata());
        let mut opened = Self {
            book,
            index: 0,
            title,
            authors,
            chapter_title: NO_CHAPTERS.to_string(),
            text: EMPTY_BOOK.to_string(),
        };
        if !opened.book.chapters().is_empty() {
            opened.load(0)?;
        }
        Ok(opened)
    }

    /// Load a chapter by spine index. Out-of-range indices leave the view unchanged.
    pub fn goto(&mut self, index: usize) -> Result<(), &'static str> {
        if index >= self.book.chapters().len() {
            return Ok(());
        }
        self.load(index)
    }

    fn load(&mut self, index: usize) -> Result<(), &'static str> {
        let title = self
            .book
            .chapter_title(index)
            .map_err(|_| CHAPTER_ERROR)?
            .filter(|title| !title.trim().is_empty())
            .unwrap_or_else(|| format!("第 {} 章", index + 1));
        let text = self.book.chapter_text(index).map_err(|_| CHAPTER_ERROR)?;
        self.index = index;
        self.chapter_title = title;
        self.text = text;
        Ok(())
    }

    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn authors(&self) -> &str {
        &self.authors
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn total(&self) -> usize {
        self.book.chapters().len()
    }
    pub fn chapter_title(&self) -> &str {
        &self.chapter_title
    }
    pub fn text(&self) -> &str {
        &self.text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata(title: Option<&str>, creators: &[&str]) -> EpubMetadata {
        EpubMetadata {
            title: title.map(str::to_string),
            creators: creators.iter().map(|name| name.to_string()).collect(),
            language: None,
            identifier: None,
            publisher: None,
            date: None,
            modified: None,
            version: None,
        }
    }

    #[test]
    fn title_and_authors_have_fixed_fallbacks() {
        assert_eq!(display_title(&metadata(None, &[])), UNTITLED);
        assert_eq!(display_title(&metadata(Some("  "), &[])), UNTITLED);
        assert_eq!(display_title(&metadata(Some(" 本 "), &[])), "本");
        assert_eq!(display_authors(&metadata(None, &[])), UNKNOWN_AUTHOR);
        assert_eq!(display_authors(&metadata(None, &["  A ", "", "B"])), "A、B");
    }

    #[test]
    fn chapter_position_is_clamped() {
        assert_eq!(chapter_label(0, 0), "0 / 0");
        assert_eq!(chapter_label(2, 12), "3 / 12");
        assert_eq!(next_index(0, 0), 0);
        assert_eq!(next_index(0, 3), 1);
        assert_eq!(next_index(2, 3), 2);
        assert_eq!(previous_index(0), 0);
        assert_eq!(previous_index(2), 1);
    }

    #[test]
    fn invalid_bytes_fail_closed_without_echoing_input() {
        let error = match OpenedBook::open(b"not a zip".to_vec()) {
            Ok(_) => panic!("不正なバイト列が受け入れられました"),
            Err(error) => error,
        };
        assert_eq!(error, OPEN_ERROR);
    }
}
