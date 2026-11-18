//! Storage abstraction layer for multiple backends (local, Google Drive, etc.)

use crate::{CoreError, CoreResult, document::DocumentFormat};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
    async fn list(&self, path: &Path, options: ListOptions) -> CoreResult<Vec<FileEntry>>;

    /// Read a file
    async fn read(&self, path: &Path, options: ReadOptions) -> CoreResult<Vec<u8>>;

    /// Read a file as text with encoding detection
    async fn read_text(&self, path: &Path, options: ReadOptions) -> CoreResult<String>;

    /// Write a file
    async fn write(&self, path: &Path, data: &[u8], options: WriteOptions) -> CoreResult<()>;

    /// Write text with encoding
    async fn write_text(&self, path: &Path, text: &str, options: WriteOptions) -> CoreResult<()>;

    /// Create a directory
    async fn create_dir(&self, path: &Path, recursive: bool) -> CoreResult<()>;

    /// Delete a file or directory
    async fn delete(&self, path: &Path, recursive: bool) -> CoreResult<()>;

    /// Move/rename a file or directory
    async fn move_item(&self, from: &Path, to: &Path, overwrite: bool) -> CoreResult<()>;

    /// Get file metadata
    async fn metadata(&self, path: &Path) -> CoreResult<FileEntry>;

    /// Check if path exists
    async fn exists(&self, path: &Path) -> CoreResult<bool>;

    /// Get available space (if applicable)
    async fn available_space(&self) -> CoreResult<Option<u64>>;

    /// Open the path in system file manager (if local)
    async fn open_in_system(&self, path: &Path) -> CoreResult<()>;

    /// Watch for changes (returns a stream of events)
    async fn watch(&self, path: &Path) -> CoreResult<Box<dyn StorageWatchStream>>;
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
    use std::path::Component;
    use tokio::fs as tokio_fs;

    pub struct LocalStorage {
        root: PathBuf,
        capabilities: StorageCapabilities,
    }

    /// 既存の最も深い祖先を正規化し、残りの要素を連結して返す。
    /// 書き込み先など存在しない末尾にも対応するため、末尾は Lexical に連結する。
    fn canonicalize_existing_prefix(path: &Path) -> CoreResult<PathBuf> {
        let mut ancestor = path;
        let mut tail = Vec::new();
        loop {
            match ancestor.canonicalize() {
                Ok(canonical) => {
                    let mut resolved = canonical;
                    for name in tail.into_iter().rev() {
                        resolved.push(name);
                    }
                    return Ok(resolved);
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    let name = ancestor.file_name().ok_or_else(|| {
                        CoreError::storage(format!("Unable to resolve path: {}", path.display()))
                    })?;
                    tail.push(name.to_os_string());
                    ancestor = ancestor.parent().ok_or_else(|| {
                        CoreError::storage(format!("Unable to resolve path: {}", path.display()))
                    })?;
                }
                Err(error) => return Err(error.into()),
            }
        }
    }

    impl LocalStorage {
        pub fn new(root: PathBuf) -> Self {
            let capabilities = StorageCapabilities {
                is_local: true,
                can_watch: true,
                ..Default::default()
            };
            Self { root, capabilities }
        }

        /// root 外への脱出を拒否する fail-closed なパス解決。
        /// 相対パスは root 配下に解決し、絶対パスも root 配下のみ許可する。
        fn resolve_path(&self, path: &Path) -> CoreResult<PathBuf> {
            // root が正規化できない場合は containment を保証できないため拒否
            let root = self.root.canonicalize().map_err(|error| {
                CoreError::storage(format!(
                    "Unable to canonicalize storage root {}: {}",
                    self.root.display(),
                    error
                ))
            })?;

            // `..` は canonicalize 前に機械的に拒否（脱出だけでなく防御的に弾く）
            if path
                .components()
                .any(|component| matches!(component, Component::ParentDir))
            {
                return Err(CoreError::validation(format!(
                    "Parent directory traversal is not allowed: {}",
                    path.display()
                )));
            }

            let joined = if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            };

            // 既存部分は canonicalize してシンボリックリンクを解決し、
            // 存在しない末尾のみ Lexical に連結する
            let resolved = canonicalize_existing_prefix(&joined)?;

            if !resolved.starts_with(&root) {
                return Err(CoreError::validation(format!(
                    "Path escapes storage root: {}",
                    path.display()
                )));
            }

            Ok(resolved)
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

        async fn list(&self, path: &Path, options: ListOptions) -> CoreResult<Vec<FileEntry>> {
            let full_path = self.resolve_path(path)?;
            // エントリのパスは正規化済みなので、比較する root も正規化しておく。
            let root = self.root.canonicalize().map_err(|error| {
                CoreError::storage(format!(
                    "Unable to canonicalize storage root {}: {}",
                    self.root.display(),
                    error
                ))
            })?;
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
                    path: path.strip_prefix(&root).unwrap_or(&path).to_path_buf(),
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

        async fn read(&self, path: &Path, options: ReadOptions) -> CoreResult<Vec<u8>> {
            let full_path = self.resolve_path(path)?;
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

        async fn read_text(&self, path: &Path, options: ReadOptions) -> CoreResult<String> {
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

        async fn write(&self, path: &Path, data: &[u8], options: WriteOptions) -> CoreResult<()> {
            let full_path = self.resolve_path(path)?;

            // overwrite=false のときは既存ファイルへの書き込みを拒否する。
            // atomic 分岐の前で判定するため、atomic=true でも同じガードが効く。
            // エラー文言には内容も呼び出し元の生パスも出さず、
            // 正規化済み root からの相対パスのみを載せる（正規化不能なら固定文言）。
            if !options.overwrite && full_path.exists() {
                let message = self
                    .root
                    .canonicalize()
                    .ok()
                    .and_then(|root| full_path.strip_prefix(&root).ok().map(Path::to_path_buf))
                    .map(|relative| format!("File already exists: {}", relative.display()))
                    .unwrap_or_else(|| "File already exists".to_string());
                return Err(CoreError::storage(message));
            }

            if options.create_parents
                && let Some(parent) = full_path.parent()
            {
                tokio_fs::create_dir_all(parent).await?;
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
            path: &Path,
            text: &str,
            options: WriteOptions,
        ) -> CoreResult<()> {
            self.write(path, text.as_bytes(), options).await
        }

        async fn create_dir(&self, path: &Path, recursive: bool) -> CoreResult<()> {
            let full_path = self.resolve_path(path)?;
            if recursive {
                tokio_fs::create_dir_all(&full_path).await?;
            } else {
                tokio_fs::create_dir(&full_path).await?;
            }
            Ok(())
        }

        async fn delete(&self, path: &Path, recursive: bool) -> CoreResult<()> {
            let full_path = self.resolve_path(path)?;
            if recursive {
                tokio_fs::remove_dir_all(&full_path).await?;
            } else {
                tokio_fs::remove_file(&full_path).await?;
            }
            Ok(())
        }

        async fn move_item(&self, from: &Path, to: &Path, overwrite: bool) -> CoreResult<()> {
            let full_from = self.resolve_path(from)?;
            let full_to = self.resolve_path(to)?;

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

        async fn metadata(&self, path: &Path) -> CoreResult<FileEntry> {
            let full_path = self.resolve_path(path)?;
            let metadata = tokio_fs::metadata(&full_path).await?;
            let is_dir = metadata.is_dir();

            Ok(FileEntry {
                name: full_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                path: path.to_path_buf(),
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

        async fn exists(&self, path: &Path) -> CoreResult<bool> {
            Ok(self.resolve_path(path)?.exists())
        }

        async fn available_space(&self) -> CoreResult<Option<u64>> {
            // Simplified - would need platform-specific implementation
            Ok(None)
        }

        async fn open_in_system(&self, path: &Path) -> CoreResult<()> {
            let full_path = self.resolve_path(path)?;

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

        async fn watch(&self, _path: &Path) -> CoreResult<Box<dyn StorageWatchStream>> {
            // Would use notify crate for real implementation
            Err(CoreError::storage("Watch not implemented yet"))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::fs;
        use std::sync::atomic::{AtomicUsize, Ordering};

        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

        struct TempRoot {
            path: PathBuf,
        }

        impl TempRoot {
            fn new() -> Self {
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir().join(format!(
                    "openrustbooks-io-test-{}-{}",
                    std::process::id(),
                    id
                ));
                fs::create_dir_all(&path).unwrap();
                Self { path }
            }

            fn storage(&self) -> LocalStorage {
                LocalStorage::new(self.path.clone())
            }
        }

        impl Drop for TempRoot {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.path);
            }
        }

        #[test]
        fn relative_path_under_root_resolves() {
            let temp = TempRoot::new();
            let storage = temp.storage();
            let resolved = storage
                .resolve_path(Path::new("sub/nested/file.txt"))
                .unwrap();
            assert_eq!(
                resolved,
                temp.path
                    .canonicalize()
                    .unwrap()
                    .join("sub/nested/file.txt")
            );
        }

        #[test]
        fn parent_dir_escape_is_rejected() {
            let temp = TempRoot::new();
            let storage = temp.storage();
            assert!(storage.resolve_path(Path::new("../escape")).is_err());
            assert!(storage.resolve_path(Path::new("sub/../../escape")).is_err());
        }

        #[test]
        fn absolute_path_outside_root_is_rejected() {
            let temp = TempRoot::new();
            let outside = TempRoot::new();
            let storage = temp.storage();
            let abs = outside.path.canonicalize().unwrap().join("secret.txt");
            assert!(storage.resolve_path(&abs).is_err());
        }

        #[test]
        fn absolute_path_under_root_is_allowed() {
            let temp = TempRoot::new();
            let storage = temp.storage();
            let abs = temp.path.canonicalize().unwrap().join("inside.txt");
            fs::write(&abs, b"ok").unwrap();
            let resolved = storage.resolve_path(&abs).unwrap();
            assert_eq!(resolved, abs);
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn symlink_escape_is_rejected() {
            use std::os::unix::fs::symlink;

            let temp = TempRoot::new();
            let outside = TempRoot::new();
            let storage = temp.storage();
            let link = temp.path.join("link");
            if symlink(outside.path.canonicalize().unwrap(), &link).is_err() {
                // シンボリックリンクを作成できない環境ではスキップ
                return;
            }
            let result = storage.resolve_path(Path::new("link/secret.txt"));
            assert!(result.is_err());
        }

        /// 既存ファイルを用意して、そのフルパスを返すテストヘルパー。
        fn seed_existing_file(temp: &TempRoot) -> PathBuf {
            let file = temp.path.join("existing.txt");
            fs::write(&file, b"original").unwrap();
            file
        }

        /// Storage エラーの文言を検証する（root 相対パスのみ／元内容を含まない）。
        fn assert_existing_file_error(error: CoreError) {
            match error {
                CoreError::Storage(message) => {
                    assert_eq!(message, "File already exists: existing.txt");
                    assert!(
                        !message.contains("original"),
                        "エラー文言にファイル内容を含めない"
                    );
                }
                other => panic!("expected Storage error, got {other:?}"),
            }
        }

        #[tokio::test]
        async fn write_without_overwrite_rejects_existing_file() {
            let temp = TempRoot::new();
            let storage = temp.storage();
            let existing = seed_existing_file(&temp);

            let result = storage
                .write(
                    Path::new("existing.txt"),
                    b"replacement",
                    WriteOptions::default(),
                )
                .await;

            assert_existing_file_error(
                result.expect_err("overwrite=false は既存ファイルを拒否する"),
            );
            assert_eq!(fs::read(&existing).unwrap(), b"original".to_vec());
        }

        #[tokio::test]
        async fn write_without_overwrite_allows_new_file() {
            let temp = TempRoot::new();
            let storage = temp.storage();

            // overwrite=false でも新規ファイルは作成できる（GOAL の合格条件）。
            storage
                .write(Path::new("new.txt"), b"created", WriteOptions::default())
                .await
                .unwrap();
            assert_eq!(
                fs::read(temp.path.join("new.txt")).unwrap(),
                b"created".to_vec()
            );

            // create_parents と組み合わせても新規作成は成功する。
            let options = WriteOptions {
                create_parents: true,
                ..Default::default()
            };
            storage
                .write(Path::new("sub/nested.txt"), b"nested", options)
                .await
                .unwrap();
            assert_eq!(
                fs::read(temp.path.join("sub/nested.txt")).unwrap(),
                b"nested".to_vec()
            );
        }

        #[tokio::test]
        async fn write_error_exposes_only_root_relative_path() {
            let temp = TempRoot::new();
            let storage = temp.storage();
            let existing = seed_existing_file(&temp);

            // 呼び出し元が絶対パスを渡しても、エラー文言は root 相対パスだけに正規化される。
            let result = storage
                .write(&existing, b"replacement", WriteOptions::default())
                .await;

            match result.expect_err("既存ファイルへの書き込みは拒否される") {
                CoreError::Storage(message) => {
                    assert_eq!(message, "File already exists: existing.txt");
                    assert!(!message.contains("original"));
                    assert!(!message.contains(existing.to_string_lossy().as_ref()));
                }
                other => panic!("expected Storage error, got {other:?}"),
            }
        }

        #[tokio::test]
        async fn write_with_overwrite_replaces_existing_file() {
            let temp = TempRoot::new();
            let storage = temp.storage();
            let existing = seed_existing_file(&temp);

            let options = WriteOptions {
                overwrite: true,
                ..Default::default()
            };
            storage
                .write(Path::new("existing.txt"), b"replacement", options)
                .await
                .unwrap();

            assert_eq!(fs::read(&existing).unwrap(), b"replacement".to_vec());
        }

        #[tokio::test]
        async fn atomic_write_without_overwrite_rejects_existing_file() {
            let temp = TempRoot::new();
            let storage = temp.storage();
            let existing = seed_existing_file(&temp);

            let options = WriteOptions {
                atomic: true,
                ..Default::default()
            };
            let result = storage
                .write(Path::new("existing.txt"), b"replacement", options)
                .await;

            assert_existing_file_error(
                result.expect_err("atomic=true でも overwrite=false は既存ファイルを拒否する"),
            );
            assert_eq!(fs::read(&existing).unwrap(), b"original".to_vec());
            assert!(!temp.path.join("existing.tmp").exists());
        }

        #[tokio::test]
        async fn atomic_write_with_overwrite_replaces_existing_file() {
            let temp = TempRoot::new();
            let storage = temp.storage();
            let existing = seed_existing_file(&temp);

            let options = WriteOptions {
                atomic: true,
                overwrite: true,
                ..Default::default()
            };
            storage
                .write(Path::new("existing.txt"), b"replacement", options)
                .await
                .unwrap();

            assert_eq!(fs::read(&existing).unwrap(), b"replacement".to_vec());
            assert!(!temp.path.join("existing.tmp").exists());
        }
    }
}
