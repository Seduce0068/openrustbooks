//! Document models and metadata.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

/// Unique identifier for a document
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(pub String);

impl DocumentId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Supported document formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentFormat {
    /// Plain text
    Txt,
    /// Markdown
    Markdown,
    /// EPUB e-book
    Epub,
    /// PDF document
    Pdf,
    /// ZIP archive
    Zip,
}

impl DocumentFormat {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "txt" => Some(Self::Txt),
            "md" | "markdown" | "mkd" => Some(Self::Markdown),
            "epub" => Some(Self::Epub),
            "pdf" => Some(Self::Pdf),
            "zip" => Some(Self::Zip),
            _ => None,
        }
    }

    /// Get typical file extensions
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Txt => &["txt"],
            Self::Markdown => &["md", "markdown", "mkd"],
            Self::Epub => &["epub"],
            Self::Pdf => &["pdf"],
            Self::Zip => &["zip"],
        }
    }

    /// Check if format is editable as text
    pub fn is_text_editable(&self) -> bool {
        matches!(self, Self::Txt | Self::Markdown)
    }

    /// Check if format is an archive
    pub fn is_archive(&self) -> bool {
        matches!(self, Self::Epub | Self::Zip)
    }
}

impl std::fmt::Display for DocumentFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Txt => write!(f, "txt"),
            Self::Markdown => write!(f, "markdown"),
            Self::Epub => write!(f, "epub"),
            Self::Pdf => write!(f, "pdf"),
            Self::Zip => write!(f, "zip"),
        }
    }
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub id: DocumentId,
    pub format: DocumentFormat,
    pub title: Option<String>,
    pub author: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub size_bytes: u64,
    pub path: Option<PathBuf>,
    pub url: Option<Url>,
    pub checksum: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl DocumentMetadata {
    pub fn new(id: DocumentId, format: DocumentFormat) -> Self {
        Self {
            id,
            format,
            title: None,
            author: None,
            created_at: None,
            modified_at: None,
            size_bytes: 0,
            path: None,
            url: None,
            checksum: None,
            extra: Default::default(),
        }
    }
}

/// Main document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub metadata: DocumentMetadata,
    pub content: DocumentContent,
}

/// Document content variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DocumentContent {
    /// Plain text content
    Text(String),
    /// Markdown content (with optional front matter)
    Markdown {
        content: String,
        front_matter: Option<serde_json::Map<String, serde_json::Value>>,
    },
    /// EPUB structure (simplified)
    Epub {
        spine: Vec<EpubSpineItem>,
        manifest: Vec<EpubManifestItem>,
        toc: Vec<EpubTocItem>,
    },
    /// PDF content (page-based)
    Pdf {
        page_count: usize,
        text_content: Option<String>, // Extracted text if available
    },
    /// ZIP archive listing
    Zip { entries: Vec<ZipEntry> },
    /// Binary content (for formats we don't parse)
    Binary(Vec<u8>),
}

/// EPUB spine item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubSpineItem {
    pub id: String,
    pub href: String,
    pub media_type: String,
    pub linear: bool,
}

/// EPUB manifest item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubManifestItem {
    pub id: String,
    pub href: String,
    pub media_type: String,
    pub properties: Option<String>,
}

/// EPUB table of contents item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubTocItem {
    pub label: String,
    pub href: String,
    pub children: Vec<EpubTocItem>,
}

/// ZIP archive entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipEntry {
    pub name: String,
    pub size: u64,
    pub compressed_size: u64,
    pub is_dir: bool,
    pub format: Option<DocumentFormat>,
}

impl Document {
    /// Create a new text document
    pub fn new_text(id: DocumentId, content: String) -> Self {
        let mut metadata = DocumentMetadata::new(id, DocumentFormat::Txt);
        metadata.size_bytes = content.len() as u64;
        Self {
            metadata,
            content: DocumentContent::Text(content),
        }
    }

    /// Create a new markdown document
    pub fn new_markdown(id: DocumentId, content: String) -> Self {
        let size = content.len() as u64;
        let (front_matter, body) = extract_front_matter(&content);
        let mut metadata = DocumentMetadata::new(id, DocumentFormat::Markdown);
        metadata.size_bytes = size;
        Self {
            metadata,
            content: DocumentContent::Markdown {
                content: body,
                front_matter,
            },
        }
    }

    /// Get the document as plain text (for editing)
    pub fn as_text(&self) -> Option<&str> {
        match &self.content {
            DocumentContent::Text(t) => Some(t),
            DocumentContent::Markdown { content, .. } => Some(content),
            _ => None,
        }
    }

    /// Get mutable text content (for editing)
    pub fn as_text_mut(&mut self) -> Option<&mut String> {
        match &mut self.content {
            DocumentContent::Text(t) => Some(t),
            DocumentContent::Markdown { content, .. } => Some(content),
            _ => None,
        }
    }

    /// Check if document is text-editable
    pub fn is_editable(&self) -> bool {
        self.metadata.format.is_text_editable()
    }
}

/// Extract YAML front matter from markdown content
pub fn extract_front_matter(
    content: &str,
) -> (Option<serde_json::Map<String, serde_json::Value>>, String) {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 2 || lines[0] != "---" {
        return (None, content.to_string());
    }

    for (i, line) in lines.iter().enumerate().skip(1) {
        if *line == "---" || *line == "..." {
            let front_matter_str = lines[1..i].join("\n");
            if let Ok(value) = serde_yaml::from_str(&front_matter_str) {
                let body = lines[i + 1..].join("\n");
                return (Some(value), body);
            }
            break;
        }
    }
    (None, content.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_id() {
        let id = DocumentId::new();
        assert!(!id.0.is_empty());
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(
            DocumentFormat::from_extension("md"),
            Some(DocumentFormat::Markdown)
        );
        assert_eq!(
            DocumentFormat::from_extension("TXT"),
            Some(DocumentFormat::Txt)
        );
        assert_eq!(DocumentFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_front_matter_extraction() {
        let content = "---\ntitle: Test\ntags: [a, b]\n---\n\n# Hello";
        let (fm, body) = extract_front_matter(content);
        assert!(fm.is_some());
        assert!(body.contains("# Hello"));
    }
}
