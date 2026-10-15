//! Storage abstraction layer for multiple backends (local, Google Drive, etc.)

use crate::{CoreError, CoreResult, document::DocumentFormat};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Capabilities of a storage backend
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageCapabilities {
    /// Can list directory contents
    pub can_list: bool,
    /// Can read files
    pub can_read: bool,
    /// Can write files
    pub can_write: bool,
    /// Can create directories
    pub can_create_dir: bool,
    /// Can delete files/directories
    pub can_delete: bool,
    /// Can move/rename files
    pub can_move: bool,
    /// Supports watching for changes
    pub can_watch: bool,
    /// Provides file metadata (size, modified time, etc.)
    pub has_metadata: bool,
    /// Supports partial reads (range requests)
    pub supports_range: bool,
    /// Is the storage local (fast, no network)
    pub is_local: bool,
    /// Requires authentication
    pub requires_auth: bool,
    /// Maximum file size (None = unlimited)
    pub max_file_size: Option<u64>,
    /// Maximum total storage (None = unlimited)
    pub max_total_size: Option<u64>,
}

impl Default for StorageCapabilities {
    fn default() -> Self {
        Self {
            can_list: true,
            can_read: true,
            can_write: true,
            can_create_dir: true,
            can_delete: true,
            can_move: true,
            can_watch: false,
            has_metadata: true,
            supports_range: false,
            is_local: true,
            requires_auth: false,
            max_file_size: None,
            max_total_size: None,
        }
    }
}

/// File entry for directory listings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
    pub format: Option<DocumentFormat>,
    pub mime_type: Option<String>,
}

/// Options for reading files
#[derive(Debug, Clone, Default)]
pub struct ReadOptions {
    /// Byte offset to start reading from
    pub offset: Option<u64>,
    /// Maximum bytes to read
    pub limit: Option<u64>,
    /// Whether to read as text (with encoding detection)
    pub as_text: bool,
}

/// Options for writing files
#[derive(Debug, Clone, Default)]
pub struct WriteOptions {
    /// Create parent directories if they don't exist
    pub create_parents: bool,
    /// Overwrite existing file
    pub overwrite: bool,
    /// Atomic write (write to temp then rename)
    pub atomic: bool,
}

/// Options for listing directories
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ListOptions {
    /// Include hidden files
    pub include_hidden: bool,
    /// Recursive listing
    pub recursive: bool,
    /// Maximum entries to return
    pub max_entries: Option<usize>,
    /// Filter by format
    pub format_filter: Option<DocumentFormat>,
}

/// Main storage backend trait
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Unique identifier for this backend
    fn id(&self) -> &str;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Get capabilities
    fn capabilities(&self) -> StorageCapabilities;

    /// Initialize the backend (auth, connections, etc.)
    async fn initialize(&mut self) -> CoreResult<()>;

    /// Check if backend is healthy/available
    async fn health_check(&self) -> CoreResult<bool>;

    /// List directory contents
    async fn list(&self, path: &PathBuf, options: ListOptions) -> CoreResult<Vec<FileEntry>>;

    /// Read a file
    async fn read(&self, path: &PathBuf, options: ReadOptions) -> CoreResult<Vec<u8>>;

    /// Read a file as text with encoding detection
    async fn read_text(&self, path: &PathBuf, options: ReadOptions) -> CoreResult<String>;

    /// Write a file
    async fn write(&self, path: &PathBuf, data: &[u8], options: WriteOptions) -> CoreResult<()>;

    /// Write text with encoding
    async fn write_text(&self, path: &PathBuf, text: &str, options: WriteOptions)
    -> CoreResult<()>;

    /// Create a directory
    async fn create_dir(&self, path: &PathBuf, recursive: bool) -> CoreResult<()>;

    /// Delete a file or directory
    async fn delete(&self, path: &PathBuf, recursive: bool) -> CoreResult<()>;

    /// Move/rename a file or directory
    async fn move_item(&self, from: &PathBuf, to: &PathBuf, overwrite: bool) -> CoreResult<()>;

    /// Get file metadata
    async fn metadata(&self, path: &PathBuf) -> CoreResult<FileEntry>;

    /// Check if path exists
    async fn exists(&self, path: &PathBuf) -> CoreResult<bool>;

    /// Get available space (if applicable)
    async fn available_space(&self) -> CoreResult<Option<u64>>;

    /// Open the path in system file manager (if local)
    async fn open_in_system(&self, path: &PathBuf) -> CoreResult<()>;

    /// Watch for changes (returns a stream of events)
    async fn watch(&self, path: &PathBuf) -> CoreResult<Box<dyn StorageWatchStream>>;
}

/// Stream of storage watch events
#[async_trait]
pub trait StorageWatchStream: Send + Sync {
    async fn next(&mut self) -> Option<StorageEvent>;
}

/// Storage change events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
    Moved { from: PathBuf, to: PathBuf },
    Error(String),
}

/// Multi-backend storage manager
pub struct StorageManager {
    backends: Vec<Box<dyn StorageBackend>>,
    default_backend: Option<String>,
}

impl StorageManager {
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
            default_backend: None,
        }
    }

    pub fn add_backend(&mut self, backend: Box<dyn StorageBackend>) {
        let id = backend.id().to_string();
        if self.default_backend.is_none() {
            self.default_backend = Some(id.clone());
        }
        self.backends.push(backend);
    }

    pub fn get_backend(&self, id: &str) -> Option<&dyn StorageBackend> {
        self.backends
            .iter()
            .find(|b| b.id() == id)
            .map(|b| b.as_ref())
    }

    pub fn get_backend_mut(&mut self, id: &str) -> Option<&mut (dyn StorageBackend + '_)> {
        for b in self.backends.iter_mut() {
            if b.id() == id {
                return Some(b.as_mut());
            }
        }
        None
    }

    pub fn default_backend(&self) -> Option<&dyn StorageBackend> {
        self.default_backend
            .as_ref()
            .and_then(|id| self.get_backend(id))
    }

    pub fn set_default_backend(&mut self, id: &str) -> CoreResult<()> {
        if self.get_backend(id).is_some() {
            self.default_backend = Some(id.to_string());
            Ok(())
        } else {
            Err(CoreError::storage(format!("Backend not found: {}", id)))
        }
    }

    pub fn list_backends(&self) -> Vec<&dyn StorageBackend> {
        self.backends.iter().map(|b| b.as_ref()).collect()
    }
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Local filesystem storage backend.
///
/// Native only: relies on `tokio::fs` and OS process spawning, neither of
/// which exists on `wasm32-unknown-unknown`.
#[cfg(not(target_arch = "wasm32"))]
pub mod local {
    use super::*;
    use tokio::fs as tokio_fs;

    pub struct LocalStorage {
        root: PathBuf,
        capabilities: StorageCapabilities,
    }

    impl LocalStorage {
        pub fn new(root: PathBuf) -> Self {
            let mut capabilities = StorageCapabilities::default();
            capabilities.is_local = true;
            capabilities.can_watch = true;
            Self { root, capabilities }
        }

        fn resolve_path(&self, path: &PathBuf) -> PathBuf {
            if path.is_absolute() {
                path.clone()
            } else {
                self.root.join(path)
            }
        }
    }

    #[async_trait]
    impl StorageBackend for LocalStorage {
        fn id(&self) -> &str {
            "local"
        }
        fn name(&self) -> &str {
            "Local Filesystem"
        }
        fn capabilities(&self) -> StorageCapabilities {
            self.capabilities.clone()
        }

        async fn initialize(&mut self) -> CoreResult<()> {
            tokio_fs::create_dir_all(&self.root).await?;
            Ok(())
        }

        async fn health_check(&self) -> CoreResult<bool> {
            Ok(self.root.exists())
        }

        async fn list(&self, path: &PathBuf, options: ListOptions) -> CoreResult<Vec<FileEntry>> {
            let full_path = self.resolve_path(path);
            let mut entries = Vec::new();

            let mut read_dir = tokio_fs::read_dir(&full_path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                let name = entry.file_name().to_string_lossy().to_string();
                if !options.include_hidden && name.starts_with('.') {
                    continue;
                }

                let path = entry.path();
                let metadata = entry.metadata().await?;
                let is_dir = metadata.is_dir();
                let format = if !is_dir {
                    DocumentFormat::from_extension(
                        path.extension().and_then(|s| s.to_str()).unwrap_or(""),
                    )
                } else {
                    None
                };

                entries.push(FileEntry {
                    name,
                    path: path.strip_prefix(&self.root).unwrap_or(&path).to_path_buf(),
                    is_dir,
                    size: if is_dir { None } else { Some(metadata.len()) },
                    modified: metadata.modified().ok().map(|t| t.into()),
                    format,
                    mime_type: None,
                });
            }

            if let Some(max) = options.max_entries {
                entries.truncate(max);
            }

            Ok(entries)
        }

        async fn read(&self, path: &PathBuf, options: ReadOptions) -> CoreResult<Vec<u8>> {
            let full_path = self.resolve_path(path);
            let mut file = tokio_fs::File::open(&full_path).await?;

            if let Some(offset) = options.offset {
                use tokio::io::AsyncSeekExt;
                file.seek(std::io::SeekFrom::Start(offset)).await?;
            }

            let mut data = Vec::new();
            if let Some(limit) = options.limit {
                use tokio::io::AsyncReadExt;
                let mut limited = file.take(limit);
                limited.read_to_end(&mut data).await?;
            } else {
                use tokio::io::AsyncReadExt;
                file.read_to_end(&mut data).await?;
            }

            Ok(data)
        }

        async fn read_text(&self, path: &PathBuf, options: ReadOptions) -> CoreResult<String> {
            let bytes = self.read(path, options).await?;
            // Try UTF-8 first, then fall back to lossy conversion
            String::from_utf8(bytes).map_err(|e| {
                let lossy = String::from_utf8_lossy(e.as_bytes()).to_string();
                CoreError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid UTF-8, using lossy: {}", lossy),
                ))
            })
        }

        async fn write(
            &self,
            path: &PathBuf,
            data: &[u8],
            options: WriteOptions,
        ) -> CoreResult<()> {
            let full_path = self.resolve_path(path);

            if options.create_parents {
                if let Some(parent) = full_path.parent() {
                    tokio_fs::create_dir_all(parent).await?;
                }
            }

            if options.atomic {
                let temp_path = full_path.with_extension("tmp");
                tokio_fs::write(&temp_path, data).await?;
                tokio_fs::rename(&temp_path, &full_path).await?;
            } else {
                tokio_fs::write(&full_path, data).await?;
            }

            Ok(())
        }

        async fn write_text(
            &self,
            path: &PathBuf,
            text: &str,
            options: WriteOptions,
        ) -> CoreResult<()> {
            self.write(path, text.as_bytes(), options).await
        }

        async fn create_dir(&self, path: &PathBuf, recursive: bool) -> CoreResult<()> {
            let full_path = self.resolve_path(path);
            if recursive {
                tokio_fs::create_dir_all(&full_path).await?;
            } else {
                tokio_fs::create_dir(&full_path).await?;
            }
            Ok(())
        }

        async fn delete(&self, path: &PathBuf, recursive: bool) -> CoreResult<()> {
            let full_path = self.resolve_path(path);
            if recursive {
                tokio_fs::remove_dir_all(&full_path).await?;
            } else {
                tokio_fs::remove_file(&full_path).await?;
            }
            Ok(())
        }

        async fn move_item(&self, from: &PathBuf, to: &PathBuf, overwrite: bool) -> CoreResult<()> {
            let full_from = self.resolve_path(from);
            let full_to = self.resolve_path(to);

            if overwrite && full_to.exists() {
                if full_to.is_dir() {
                    tokio_fs::remove_dir_all(&full_to).await?;
                } else {
                    tokio_fs::remove_file(&full_to).await?;
                }
            }

            if let Some(parent) = full_to.parent() {
                tokio_fs::create_dir_all(parent).await?;
            }

            tokio_fs::rename(&full_from, &full_to).await?;
            Ok(())
        }

        async fn metadata(&self, path: &PathBuf) -> CoreResult<FileEntry> {
            let full_path = self.resolve_path(path);
            let metadata = tokio_fs::metadata(&full_path).await?;
            let is_dir = metadata.is_dir();

            Ok(FileEntry {
                name: full_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                path: path.clone(),
                is_dir,
                size: if is_dir { None } else { Some(metadata.len()) },
                modified: metadata.modified().ok().map(|t| t.into()),
                format: if !is_dir {
                    DocumentFormat::from_extension(
                        full_path.extension().and_then(|s| s.to_str()).unwrap_or(""),
                    )
                } else {
                    None
                },
                mime_type: None,
            })
        }

        async fn exists(&self, path: &PathBuf) -> CoreResult<bool> {
            Ok(self.resolve_path(path).exists())
        }

        async fn available_space(&self) -> CoreResult<Option<u64>> {
            // Simplified - would need platform-specific implementation
            Ok(None)
        }

        async fn open_in_system(&self, path: &PathBuf) -> CoreResult<()> {
            let full_path = self.resolve_path(path);

            // `cfg!` rather than `#[cfg]` so every platform compiles the same
            // code path; unsupported platforms report an explicit error
            // instead of pretending the file was opened.
            let opener = if cfg!(target_os = "macos") {
                "open"
            } else if cfg!(target_os = "windows") {
                "explorer"
            } else if cfg!(target_os = "linux") {
                "xdg-open"
            } else {
                return Err(CoreError::storage(format!(
                    "Opening files in the system handler is not supported on this platform: {}",
                    full_path.display()
                )));
            };

            std::process::Command::new(opener).arg(&full_path).spawn()?;
            Ok(())
        }

        async fn watch(&self, _path: &PathBuf) -> CoreResult<Box<dyn StorageWatchStream>> {
            // Would use notify crate for real implementation
            Err(CoreError::storage("Watch not implemented yet"))
        }
    }
}
