//! openrustbooks-core: Shared core library for openrustbooks.
//!
//! Provides document models, I/O abstractions, and the AI Markdown pipeline
//! for both desktop (native) and web (Wasm) targets.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod ai_markdown;
pub mod archive;
pub mod config;
pub mod document;
pub mod epub;
pub mod error;
pub mod image_dimensions;
pub mod io;
pub mod sanitize;
pub mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

pub use archive::{ArchiveEntry, ArchiveLimits, ArchiveReader};
pub use config::{AiConfig, AppConfig, LintConfig};
pub use document::{Document, DocumentFormat, DocumentId, DocumentMetadata};
pub use epub::{ChapterLink, EpubBook, EpubChapter, EpubCoverImage, EpubMetadata, EpubTocEntry};
/// Re-export commonly used types
pub use error::{CoreError, CoreResult};
pub use image_dimensions::{MAX_DIMENSION, image_dimensions};
pub use io::{FileEntry, StorageBackend, StorageCapabilities};
pub use sanitize::sanitize_epub_html;

/// Initialize the core library (logging, panic hook for Wasm, etc.)
pub fn init() {
    #[cfg(target_arch = "wasm32")]
    {
        wasm::init_core();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }
}
