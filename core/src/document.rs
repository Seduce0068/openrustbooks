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
        /// 開始・終了区切り線と本文直前の改行を含む front matter の元テキスト。
        /// lint 等が構造化値を使う一方、保存時はこの原文を再利用して
        /// コメント・クォート・キー順・改行スタイルの変化を防ぐ。
        #[serde(default)]
        front_matter_raw: Option<String>,
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
        let (front_matter, body, front_matter_raw) = extract_front_matter_preserving(&content);
        let mut metadata = DocumentMetadata::new(id, DocumentFormat::Markdown);
        metadata.size_bytes = size;
        Self {
            metadata,
            content: DocumentContent::Markdown {
                content: body,
                front_matter,
                front_matter_raw,
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

    /// 保存用の完全テキストを返す。Markdown は front matter を先頭に結合する。
    pub fn full_text(&self) -> Option<String> {
        match &self.content {
            DocumentContent::Text(t) => Some(t.clone()),
            DocumentContent::Markdown {
                content,
                front_matter,
                front_matter_raw,
            } => {
                // 元の front matter があれば再シリアライズせずそのまま連結する。
                // body が編集されていても区切り線・改行・コメント等は維持される。
                if let Some(raw) = front_matter_raw {
                    return Some(format!("{}{}", raw, content));
                }
                match front_matter {
                    Some(fm) => {
                        // 旧データや外部から構築された値向けの互換パス。
                        let yaml = serde_norway::to_string(fm).ok()?;
                        Some(format!("---\n{}\n---\n{}", yaml.trim_end(), content))
                    }
                    None => Some(content.clone()),
                }
            }
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

/// 指定位置から始まる行の「内容末尾」と「次行の開始位置」を返す。
///
/// LF / CRLF のどちらでも改行として扱い、スライス位置は常に UTF-8 境界になる
/// （`\n` とその直後でしか切断しないため）。
fn line_span(bytes: &[u8], start: usize) -> (usize, usize) {
    let mut end = start;
    while end < bytes.len() && bytes[end] != b'\n' {
        end += 1;
    }
    let next = if end < bytes.len() { end + 1 } else { end };
    (end, next)
}

/// CRLF の `\r` を除いて区切り線かどうか判定する。
fn is_front_matter_delimiter(line: &str, delimiter: &str) -> bool {
    line.strip_suffix('\r').unwrap_or(line) == delimiter
}

/// front matter を元のバイト列・改行スタイルを保ったまま抽出する。
///
/// 戻り値は `(構造化 front matter, body, front matter の元テキスト)`。
/// `元テキスト` は開始区切り線から終了区切り線とその直後の改行までを含み、
/// 例: `"---\ntitle: x\n---\n"`。構造化値は lint 等の既存利用のために維持する。
///
/// 既存の [`extract_front_matter`] は `lines()` / `join("\n")` による正規化を
/// 行うため、互換性のため挙動を変更せずに本関数を追加した。
pub fn extract_front_matter_preserving(
    content: &str,
) -> (
    Option<serde_json::Map<String, serde_json::Value>>,
    String,
    Option<String>,
) {
    let bytes = content.as_bytes();
    if bytes.is_empty() {
        return (None, content.to_string(), None);
    }

    let (open_end, yaml_start) = line_span(bytes, 0);
    if !is_front_matter_delimiter(&content[..open_end], "---") {
        return (None, content.to_string(), None);
    }

    let mut cursor = yaml_start;
    let mut close = None;
    while cursor < bytes.len() {
        let (line_end, next) = line_span(bytes, cursor);
        let line = &content[cursor..line_end];
        if is_front_matter_delimiter(line, "---") || is_front_matter_delimiter(line, "...") {
            close = Some((cursor, next));
            break;
        }
        cursor = next;
    }
    let Some((yaml_end, body_start)) = close else {
        return (None, content.to_string(), None);
    };

    let Ok(front_matter) = serde_norway::from_str::<serde_json::Map<String, serde_json::Value>>(
        &content[yaml_start..yaml_end],
    ) else {
        return (None, content.to_string(), None);
    };

    let raw = content[..body_start].to_string();
    let body = content[body_start..].to_string();
    (Some(front_matter), body, Some(raw))
}

/// Extract YAML front matter from markdown content
///
/// 注意: 本文は `lines()` / `join("\n")` で LF に正規化され末尾改行が除かれる。
/// 往復で元バイト列を保全したい場合は [`extract_front_matter_preserving`] を使う。
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
            if let Ok(value) = serde_norway::from_str(&front_matter_str) {
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

    #[test]
    fn full_text_restores_front_matter() {
        // front_matter_raw が無い旧データ向けの再シリアライズ経路。
        let content = "---\ntitle: Test\n---\n\n# Hello";
        let (fm, body) = extract_front_matter(content);
        let document = Document {
            metadata: DocumentMetadata::new(DocumentId::new(), DocumentFormat::Markdown),
            content: DocumentContent::Markdown {
                content: body,
                front_matter: fm,
                front_matter_raw: None,
            },
        };
        let full = document.full_text().unwrap();
        assert!(full.starts_with("---\n"));
        assert!(full.contains("title: Test"));
        assert!(full.contains("# Hello"));
        // 再抽出でも同じ情報が得られる。
        let (fm2, body2) = extract_front_matter(&full);
        assert!(fm2.is_some());
        assert!(body2.contains("# Hello"));
    }

    #[test]
    fn extract_preserving_keeps_crlf_and_trailing_newline() {
        let content = "---\r\ntitle: Test\r\ntags: [a, b]\r\n---\r\n\r\n# Hello\r\n";
        let (fm, body, raw) = extract_front_matter_preserving(content);
        assert!(fm.is_some());
        assert_eq!(
            raw.as_deref(),
            Some("---\r\ntitle: Test\r\ntags: [a, b]\r\n---\r\n")
        );
        assert_eq!(body, "\r\n# Hello\r\n");
        assert_eq!(format!("{}{}", raw.unwrap(), body), content);
    }

    #[test]
    fn legacy_extract_still_normalizes_crlf() {
        // 既存 API の互換性: 旧関数は LF 化し末尾改行を除く挙動を維持する。
        let content = "---\r\ntitle: Test\r\n---\r\n# Hello\r\n";
        let (fm, body) = extract_front_matter(content);
        assert!(fm.is_some());
        assert_eq!(body, "# Hello");
    }

    #[test]
    fn round_trip_preserves_crlf_and_trailing_newline() {
        let content = "---\r\ntitle: Test\r\n---\r\n\r\n# Hello\r\n";
        let document = Document::new_markdown(DocumentId::new(), content.to_string());
        assert_eq!(document.full_text().unwrap(), content);
    }

    #[test]
    fn round_trip_preserves_missing_trailing_newline() {
        let content = "---\ntitle: Test\n---\n# Hello";
        let document = Document::new_markdown(DocumentId::new(), content.to_string());
        assert_eq!(document.full_text().unwrap(), content);
    }

    #[test]
    fn round_trip_preserves_comments_key_order_and_quotes() {
        let content =
            "---\n# この行は保持される\nzzz: \"double\"\naaa: 'single'\nn: 001\n---\nbody\n";
        let document = Document::new_markdown(DocumentId::new(), content.to_string());
        assert_eq!(document.full_text().unwrap(), content);
    }

    #[test]
    fn round_trip_preserves_dot_closing_delimiter() {
        let content = "---\ntitle: x\n...\nbody\n";
        let document = Document::new_markdown(DocumentId::new(), content.to_string());
        assert_eq!(document.full_text().unwrap(), content);
    }

    #[test]
    fn round_trip_without_front_matter_is_unchanged() {
        let content = "# 単なる本文\r\n";
        let document = Document::new_markdown(DocumentId::new(), content.to_string());
        assert_eq!(document.full_text().unwrap(), content);
    }

    #[test]
    fn round_trip_keeps_bytes_for_degenerate_front_matters() {
        // 抽出できず構造化値を諦める場合も、保存テキストは入力とバイト一致する。
        let cases = [
            "---\ntitle: [unclosed\n---\nbody\n", // 不正 YAML
            "---",                                // '---' のみ
            "---\n---\n---\nbody\n",              // 連続区切り
            "---\ntitle: x\n---\n",               // 空 body
            "\u{feff}---\ntitle: x\n---\nbody\n", // BOM 付き
        ];
        for content in cases {
            let document = Document::new_markdown(DocumentId::new(), content.to_string());
            assert_eq!(document.full_text().unwrap(), content);
        }
    }

    #[test]
    fn body_edit_keeps_original_front_matter() {
        let content = "---\n# 元のコメント\ntitle: \"Original\"\nzzz: first\n---\n# Old\n";
        let mut document = Document::new_markdown(DocumentId::new(), content.to_string());
        document.as_text_mut().unwrap().push_str("Edited\n");

        let full = document.full_text().unwrap();
        let expected = "---\n# 元のコメント\ntitle: \"Original\"\nzzz: first\n---\n# Old\nEdited\n";
        assert_eq!(full, expected);

        // lint 等が使う構造化値も従来どおり保持される。
        match &document.content {
            DocumentContent::Markdown { front_matter, .. } => {
                let fm = front_matter.as_ref().unwrap();
                assert_eq!(fm.get("title"), Some(&serde_json::json!("Original")));
                assert_eq!(fm.get("zzz"), Some(&serde_json::json!("first")));
            }
            _ => panic!("markdown document expected"),
        }
    }
}
