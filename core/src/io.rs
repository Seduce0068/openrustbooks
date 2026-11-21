//! Storage abstraction layer for multiple backends (local, Google Drive, etc.)

use crate::{CoreError, CoreResult, document::DocumentFormat};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 既定の読み取り上限（256 MiB）。
///
/// `ReadOptions.limit` と `StorageCapabilities.max_file_size` がどちらも
/// `None` のときに適用する。EPUB 側の 256 MiB 上限と揃える（R6 PR-M04）。
pub const DEFAULT_MAX_READ_BYTES: u64 = 256 * 1024 * 1024;

/// read-only root への書き込みを拒否する際の固定文言。
///
/// `desktop/src/commands/document.rs` の `save_document` と同じ文言を使い、
/// fs コマンド経由でも意味論を揃える（R3-AU01）。設定値・パスは含めない。
pub const READ_ONLY_ROOT_MESSAGE: &str = "Storage root is read-only";

/// storage root 自身を delete / move_item / create_dir の対象にしたときの固定文言。
///
/// `resolve_path(".")` は canonical root を返すため、root 配下全体が消える経路を
/// 塞ぐ必要がある（R6 レビュー / R2-ER01）。設定値やパスは含めない。
pub const STORAGE_ROOT_OPERATION_MESSAGE: &str = "Refusing to modify the storage root itself";

/// `move_item(overwrite=false)` で既存宛先を見つけたときの固定文言。
///
/// POSIX `rename` は既存宛先を暗黙上書きするため、チェック後も rename 直前に
/// 再確認して同じ文言で拒否する。呼び出し元の生パスは含めない。
pub const MOVE_DESTINATION_EXISTS_MESSAGE: &str = "Destination already exists";

/// 上書き移動の復元にも失敗したときの固定文言。
///
/// 元と宛先の所在が不確実になったことを fail-closed で伝える。
pub const MOVE_ROLLBACK_FAILED_MESSAGE: &str = "Move failed and destination could not be restored";

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
    /// Whether to read as text (bytes are decoded as UTF-8; invalid sequences become U+FFFD)
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

    /// Read a file as UTF-8 text, replacing invalid sequences with U+FFFD
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
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::fs as tokio_fs;

    /// atomic write の一時ファイル名に使うプロセス内カウンタ。
    ///
    /// 固定の `<file>.tmp` では並行書き込みが同じ一時ファイルを同時に開いて
    /// 上書きし合い、片方の内容が失われる／途中状態が rename される恐れがある
    /// （R6 PR-M06）。PID と組み合わせることでプロセス間でも名前が衝突しない。
    static ATOMIC_WRITE_ID: AtomicUsize = AtomicUsize::new(0);

    /// `<file>.<pid>.<counter>.tmp` 形式の一時パスを生成する。
    fn atomic_temp_path(full_path: &Path) -> PathBuf {
        let counter = ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let mut file_name = full_path.file_name().unwrap_or_default().to_os_string();
        file_name.push(format!(".{pid}.{counter}.tmp"));
        full_path.with_file_name(file_name)
    }

    /// 既存の一時ファイルを決して上書きせずに開く。
    /// 同名が残っていた場合はカウンタを進めて再試行する。
    async fn create_atomic_temp(full_path: &Path) -> CoreResult<(PathBuf, tokio_fs::File)> {
        const MAX_TEMP_ATTEMPTS: usize = 128;

        for _ in 0..MAX_TEMP_ATTEMPTS {
            let candidate = atomic_temp_path(full_path);
            match tokio_fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&candidate)
                .await
            {
                Ok(file) => return Ok((candidate, file)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }

        Err(CoreError::storage(format!(
            "Unable to create a unique temporary file after {MAX_TEMP_ATTEMPTS} attempts"
        )))
    }

    /// 一時ファイルへ書き込んでから rename する。失敗時は一時ファイルを削除する。
    async fn write_atomic(full_path: &Path, data: &[u8]) -> CoreResult<()> {
        let (temp_path, mut file) = create_atomic_temp(full_path).await?;

        let write_result = async {
            use tokio::io::AsyncWriteExt;
            file.write_all(data).await?;
            file.flush().await?;
            Ok::<(), std::io::Error>(())
        }
        .await;

        // Windows ではハンドルを開いたまま rename できないため明示的に閉じる。
        drop(file);

        if write_result.is_err() {
            let _ = tokio_fs::remove_file(&temp_path).await;
        }
        write_result?;

        if let Err(error) = tokio_fs::rename(&temp_path, full_path).await {
            let _ = tokio_fs::remove_file(&temp_path).await;
            return Err(error.into());
        }

        Ok(())
    }

    /// 上書き移動の退避先を既存と衝突しない名前で予約する。
    ///
    /// 元がディレクトリならディレクトリとして、ファイル / symlink ならファイルとして
    /// 予約する。種別を揃えないと Unix で `rename` が ENOTDIR / EISDIR になる。
    async fn reserve_move_backup(full_path: &Path) -> CoreResult<PathBuf> {
        const MAX_BACKUP_ATTEMPTS: usize = 128;
        // symlink はリンク先ではなくリンク自体が退避対象なので、リンク先が
        // ディレクトリでもファイルとして予約する（dir 予約へ rename すると失敗する）。
        let is_dir = tokio_fs::symlink_metadata(full_path)
            .await?
            .file_type()
            .is_dir();

        for _ in 0..MAX_BACKUP_ATTEMPTS {
            let candidate = atomic_temp_path(full_path);
            let reserved = if is_dir {
                tokio_fs::create_dir(&candidate).await
            } else {
                tokio_fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&candidate)
                    .await
                    .map(|_| ())
            };
            match reserved {
                Ok(()) => return Ok(candidate),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }

        Err(CoreError::storage(format!(
            "Unable to create a unique backup path after {MAX_BACKUP_ATTEMPTS} attempts"
        )))
    }

    /// ファイル・ディレクトリ・symlink を種別に応じて削除する。
    ///
    /// 退避先の後始末に使う。`symlink_metadata` でリンク先を辿らず、
    /// symlink 自体を確実に削除する。
    async fn remove_path(path: &Path) -> CoreResult<()> {
        let metadata = tokio_fs::symlink_metadata(path).await?;
        if metadata.is_dir() {
            tokio_fs::remove_dir_all(path).await?;
        } else {
            tokio_fs::remove_file(path).await?;
        }
        Ok(())
    }

    #[derive(Debug)]
    pub struct LocalStorage {
        root: PathBuf,
        capabilities: StorageCapabilities,
        /// `storage.local_roots[].read_only` に対応するフラグ。
        /// true のときは write / create_dir / delete / move_item を拒否する。
        read_only: bool,
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
            Self::new_with_read_only(root, false)
        }

        /// read_only フラグを明示して生成する（desktop の local_roots 設定から使う）。
        pub fn new_with_read_only(root: PathBuf, read_only: bool) -> Self {
            let mut capabilities = StorageCapabilities {
                is_local: true,
                // `watch()` は未実装のため、利用可能と報告しない（R6 TD-H04）。
                can_watch: false,
                ..Default::default()
            };
            if read_only {
                capabilities.can_write = false;
                capabilities.can_create_dir = false;
                capabilities.can_delete = false;
                capabilities.can_move = false;
            }
            Self {
                root,
                capabilities,
                read_only,
            }
        }

        /// 設定された root パス（desktop 側の root 判定で使う）。
        pub fn root(&self) -> &Path {
            &self.root
        }

        /// この root が read-only か。
        pub fn is_read_only(&self) -> bool {
            self.read_only
        }

        /// 書き込み系操作の前に呼ぶ fail-closed ガード。
        fn ensure_writable(&self) -> CoreResult<()> {
            if self.read_only {
                Err(CoreError::validation(READ_ONLY_ROOT_MESSAGE))
            } else {
                Ok(())
            }
        }

        /// 解決済みの対象が storage root 自身なら破壊的操作を拒否する（R2-ER01）。
        ///
        /// `resolve_path(".")` や root の絶対パスは canonical root を返すため、
        /// 比較は canonicalize 済みの値同士で行う。root を canonicalize できない
        /// 場合は containment を保証できないので storage エラーで拒否する。
        fn ensure_root_not_target(&self, full_path: &Path) -> CoreResult<()> {
            let root = self.root.canonicalize().map_err(|error| {
                CoreError::storage(format!(
                    "Unable to canonicalize storage root {}: {}",
                    self.root.display(),
                    error
                ))
            })?;
            if full_path == root {
                Err(CoreError::validation(STORAGE_ROOT_OPERATION_MESSAGE))
            } else {
                Ok(())
            }
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

    /// 実際に読み取るバイト数上限を解決する（R6 PR-M04）。
    ///
    /// 1. `effective` = capabilities の `max_file_size`（未指定なら既定の 256 MiB）
    /// 2. `requested` = `options.limit`（未指定なら `effective`）
    /// 3. `requested > effective` なら黙って clamp せず `CoreError::Validation` を返す
    /// 4. それ以外は `requested.min(effective)` を `take()` に渡す
    ///
    /// capabilities 側の上限が既定値より大きい場合もその値を尊重する。
    /// エラー文言にはファイル内容も呼び出し元の生パスも含めない。
    fn resolve_read_limit(
        requested_limit: Option<u64>,
        max_file_size: Option<u64>,
    ) -> CoreResult<u64> {
        let effective = max_file_size.unwrap_or(DEFAULT_MAX_READ_BYTES);
        let requested = requested_limit.unwrap_or(effective);
        if requested > effective {
            return Err(CoreError::validation(format!(
                "Requested read limit {requested} exceeds the storage limit of {effective} bytes"
            )));
        }
        Ok(requested.min(effective))
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
            // read-only root は存在しないディレクトリを「作って書き込み可能にする」
            // ことがないよう、初期化でも生成しない（R4 指摘）。
            if self.read_only {
                return Ok(());
            }
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

            // offset と limit は加算せず独立に扱う（R6 PR-M04）。
            // 要求上限がストレージ上限を超える場合は部分読みを黙って返さずエラーにする。
            let real_limit = resolve_read_limit(options.limit, self.capabilities.max_file_size)?;
            let (data, reached_limit) = {
                use tokio::io::AsyncReadExt;
                let mut limited = (&mut file).take(real_limit);
                let mut data = Vec::new();
                limited.read_to_end(&mut data).await?;
                (data, limited.limit() == 0)
            };

            // limit 未指定は「ファイル全体を読む」要求なので、上限で切り詰めた
            // 場合は呼び出し元が検知できない。残りバイトがあれば明示エラーにする。
            if reached_limit && options.limit.is_none() {
                use tokio::io::AsyncReadExt;
                let mut probe = [0u8; 1];
                if file.read(&mut probe).await? != 0 {
                    return Err(CoreError::validation(format!(
                        "File exceeds the maximum allowed read size of {real_limit} bytes"
                    )));
                }
            }

            Ok(data)
        }

        async fn read_text(&self, path: &Path, options: ReadOptions) -> CoreResult<String> {
            let bytes = self.read(path, options).await?;
            // 有効 UTF-8 はバッファを再利用してゼロコピーで返す。
            // 不正時のみ lossy フォールバックで U+FFFD へ置換する。
            // 元バイト列をエラー文言へ埋め込むと Tauri IPC 経由で漏れるため、
            // いずれの経路でもエラーは生成しない。
            match String::from_utf8(bytes) {
                Ok(text) => Ok(text),
                Err(error) => Ok(String::from_utf8_lossy(error.as_bytes()).into_owned()),
            }
        }

        async fn write(&self, path: &Path, data: &[u8], options: WriteOptions) -> CoreResult<()> {
            self.ensure_writable()?;
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
                write_atomic(&full_path, data).await?;
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
            self.ensure_writable()?;
            let full_path = self.resolve_path(path)?;
            // root 自身は拒否しない。create_file が親ディレクトリ作成で
            // `create_dir(root, recursive=true)` を呼ぶため、ここで拒否すると
            // root 直下への新規作成が壊れる（R4 指摘）。既存 root に対する
            // create_dir_all は idempotent なのでそのまま通す。
            if recursive {
                tokio_fs::create_dir_all(&full_path).await?;
            } else {
                tokio_fs::create_dir(&full_path).await?;
            }
            Ok(())
        }

        async fn delete(&self, path: &Path, recursive: bool) -> CoreResult<()> {
            self.ensure_writable()?;
            let full_path = self.resolve_path(path)?;
            self.ensure_root_not_target(&full_path)?;
            if recursive {
                tokio_fs::remove_dir_all(&full_path).await?;
            } else {
                tokio_fs::remove_file(&full_path).await?;
            }
            Ok(())
        }

        async fn move_item(&self, from: &Path, to: &Path, overwrite: bool) -> CoreResult<()> {
            self.ensure_writable()?;
            let full_from = self.resolve_path(from)?;
            let full_to = self.resolve_path(to)?;
            // 元・宛先のどちらが root 自身でも破壊的な移動を許さない。
            self.ensure_root_not_target(&full_from)?;
            self.ensure_root_not_target(&full_to)?;

            // symlink 等で同じ実体を指す場合は移動の必要がない。
            if full_from == full_to {
                return Ok(());
            }

            if let Some(parent) = full_to.parent() {
                tokio_fs::create_dir_all(parent).await?;
            }

            if !overwrite {
                // 第1回チェック：既存宛先は早期に拒否する。
                if full_to.exists() {
                    return Err(CoreError::validation(MOVE_DESTINATION_EXISTS_MESSAGE));
                }
                // 第2回チェック：POSIX rename は既存宛先を暗黙上書きするため、
                // rename 直前にもう一度確認して TOCTOU の窓を狭める。
                // std には RENAME_NOREPLACE 相当の原子的 API が無く、unsafe FFI は
                // プロジェクト規約で禁止されているため、この再確認が上限となる。
                if full_to.exists() {
                    return Err(CoreError::validation(MOVE_DESTINATION_EXISTS_MESSAGE));
                }
                tokio_fs::rename(&full_from, &full_to).await?;
                return Ok(());
            }

            // 宛先が無い場合は退避不要で、単純な rename で元を失わない。
            if !full_to.exists() {
                tokio_fs::rename(&full_from, &full_to).await?;
                return Ok(());
            }

            // overwrite=true: 旧実装の「宛先削除 → rename」は途中失敗で
            // 元と宛先の両方を失うため、先に宛先を一意な退避先へ退避する。
            // create_new / create_dir で元と同じ種別の名前を予約してから rename で
            // 上書きすることで、退避先の名前衝突を避ける。
            let backup_path = reserve_move_backup(&full_to).await?;

            if let Err(error) = tokio_fs::rename(&full_to, &backup_path).await {
                let _ = tokio_fs::remove_file(&backup_path).await;
                return Err(error.into());
            }

            if let Err(rename_error) = tokio_fs::rename(&full_from, &full_to).await {
                // rename が失敗しても退避先に元があるので復元を試みる。
                if let Err(_restore_error) = tokio_fs::rename(&backup_path, &full_to).await {
                    // 復元も失敗した場合は所在が不確実なため固定文言で fail-closed に伝える。
                    return Err(CoreError::storage(MOVE_ROLLBACK_FAILED_MESSAGE));
                }
                return Err(rename_error.into());
            }

            // 成功時は退避先を削除する（後始末の失敗で操作自体は取り消さない）。
            let _ = remove_path(&backup_path).await;
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

        /// 未実装の `watch()` を「利用可能」と報告しないこと（R6 TD-H04）。
        #[test]
        fn local_storage_reports_watch_as_unsupported() {
            let temp = TempRoot::new();
            let storage = temp.storage();

            // 公開 API（StorageBackend::capabilities）経由でも false が見える。
            let capabilities = StorageBackend::capabilities(&storage);
            assert!(
                !capabilities.can_watch,
                "watch() は未実装のため can_watch は false であるべき"
            );
            assert!(!storage.capabilities.can_watch);
        }

        /// capability と実際の挙動が矛盾しないこと（can_watch=false なら Err）。
        #[tokio::test]
        async fn watch_returns_error_because_it_is_not_implemented() {
            let temp = TempRoot::new();
            let storage = temp.storage();

            let result = storage.watch(Path::new(".")).await;
            assert!(result.is_err(), "未実装の watch() は成功を返してはならない");
            match result.err().expect("watch() は Err を返す") {
                CoreError::Storage(message) => {
                    assert_eq!(message, "Watch not implemented yet");
                }
                other => panic!("expected Storage error, got {other:?}"),
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

        /// 一時ファイル名が呼び出しごとに異なり、`<file>.<pid>.<counter>.tmp`
        /// の形式になること（R6 PR-M06）。
        #[test]
        fn atomic_temp_paths_are_unique() {
            let target = Path::new("/tmp/openrustbooks-test/book.txt");
            let first = atomic_temp_path(target);
            let second = atomic_temp_path(target);

            assert_ne!(first, second, "並行書き込みで同じ一時名を使わない");
            assert_eq!(first.parent(), target.parent());
            let first_name = first.file_name().unwrap().to_string_lossy();
            let second_name = second.file_name().unwrap().to_string_lossy();
            assert!(first_name.starts_with("book.txt."));
            assert!(first_name.ends_with(".tmp"));
            assert!(second_name.starts_with("book.txt."));
            assert!(second_name.ends_with(".tmp"));
        }

        /// 同じ対象への並行な一時ファイル作成が衝突しないこと。
        #[tokio::test]
        async fn concurrent_atomic_temp_creation_uses_distinct_files() {
            let temp = TempRoot::new();
            let target = temp.path.join("book.txt");

            let (first_path, first_file) = create_atomic_temp(&target).await.unwrap();
            let (second_path, second_file) = create_atomic_temp(&target).await.unwrap();

            assert_ne!(first_path, second_path);
            assert!(first_path.exists());
            assert!(second_path.exists());

            drop(first_file);
            drop(second_file);
            fs::remove_file(&first_path).unwrap();
            fs::remove_file(&second_path).unwrap();
        }

        /// 固定名の旧一時ファイルが残っていても上書きせず、別名を使うこと。
        #[tokio::test]
        async fn atomic_write_does_not_overwrite_preexisting_tmp() {
            let temp = TempRoot::new();
            let storage = temp.storage();
            // 旧実装は `note.txt` の tmp を `note.tmp` に固定していた。
            let stale_tmp = temp.path.join("note.tmp");
            fs::write(&stale_tmp, b"stale").unwrap();

            let options = WriteOptions {
                atomic: true,
                ..Default::default()
            };
            storage
                .write(Path::new("note.txt"), b"fresh", options)
                .await
                .unwrap();

            assert_eq!(
                fs::read(temp.path.join("note.txt")).unwrap(),
                b"fresh".to_vec()
            );
            assert_eq!(fs::read(&stale_tmp).unwrap(), b"stale".to_vec());
        }

        /// 2 つの並行 atomic write が両方成功し、内容が混ざらず、
        /// 成功後に一時ファイルが残らないこと。
        #[tokio::test]
        async fn concurrent_atomic_writes_keep_both_contents_and_leave_no_tmp() {
            let temp = TempRoot::new();
            let storage = temp.storage();
            let options_a = WriteOptions {
                atomic: true,
                ..Default::default()
            };
            let options_b = WriteOptions {
                atomic: true,
                ..Default::default()
            };

            let (result_a, result_b) = tokio::join!(
                storage.write(Path::new("a.txt"), b"alpha payload", options_a),
                storage.write(Path::new("b.txt"), b"beta payload", options_b),
            );
            result_a.expect("a.txt の atomic write が成功する");
            result_b.expect("b.txt の atomic write が成功する");

            assert_eq!(
                fs::read(temp.path.join("a.txt")).unwrap(),
                b"alpha payload".to_vec()
            );
            assert_eq!(
                fs::read(temp.path.join("b.txt")).unwrap(),
                b"beta payload".to_vec()
            );

            let leftovers: Vec<PathBuf> = fs::read_dir(&temp.path)
                .unwrap()
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.ends_with(".tmp"))
                })
                .collect();
            assert!(
                leftovers.is_empty(),
                "atomic write 成功後に一時ファイルが残っていないこと: {leftovers:?}"
            );
        }

        #[tokio::test]
        async fn read_text_returns_valid_utf8_unchanged() {
            let temp = TempRoot::new();
            let storage = temp.storage();
            fs::write(temp.path.join("valid.txt"), "こんにちは").unwrap();

            let text = storage
                .read_text(Path::new("valid.txt"), ReadOptions::default())
                .await
                .unwrap();

            assert_eq!(text, "こんにちは");
        }

        #[tokio::test]
        async fn read_text_replaces_invalid_utf8_without_error() {
            let temp = TempRoot::new();
            let storage = temp.storage();
            // 'A' + 不正な UTF-8 バイト 0xFF + 'B'
            fs::write(temp.path.join("invalid.txt"), [b'A', 0xff, b'B']).unwrap();

            let text = storage
                .read_text(Path::new("invalid.txt"), ReadOptions::default())
                .await
                .expect("不正 UTF-8 でもエラーにせず lossy 変換する");

            assert_eq!(text, "A\u{FFFD}B");
            assert!(text.contains('A'), "有効な文字を保持する: {text:?}");
            assert!(
                text.contains('\u{FFFD}'),
                "不正バイトは U+FFFD に置換される: {text:?}"
            );
            assert!(
                !text.contains("Invalid UTF-8"),
                "エラー文言へ元内容を埋め込まない: {text:?}"
            );
        }

        /// 上限超過エラーが `CoreError::Validation` であること（archive.rs と同種）。
        fn assert_limit_validation_error(error: &CoreError) {
            match error {
                CoreError::Validation(message) => {
                    assert!(
                        message.contains("exceeds"),
                        "上限超過を示す文言であること: {message}"
                    );
                }
                other => panic!("expected Validation error, got {other:?}"),
            }
        }

        /// `limit=None` かつ capabilities 未指定なら 256 MiB の既定上限になること（R6 PR-M04）。
        /// 巨大ファイルを作らず pure helper で上限解決ロジックを検証する。
        #[test]
        fn read_limit_defaults_to_256_mib() {
            assert_eq!(DEFAULT_MAX_READ_BYTES, 256 * 1024 * 1024);
            assert_eq!(
                resolve_read_limit(None, None).expect("既定上限を返す"),
                DEFAULT_MAX_READ_BYTES
            );

            // 要求 limit が既定上限より小さければ要求値を保持する。
            assert_eq!(
                resolve_read_limit(Some(1024), None).expect("要求値を返す"),
                1024
            );
            // 既定上限超えは clamp せず明示エラーにする。
            let error = resolve_read_limit(Some(DEFAULT_MAX_READ_BYTES + 1), None)
                .expect_err("既定上限超えはエラーになる");
            assert_limit_validation_error(&error);
        }

        /// capabilities の `max_file_size` を尊重し、上限超えはエラーにすること（R6 PR-M04）。
        #[test]
        fn read_limit_respects_capabilities_max_file_size() {
            // cap Some(小): 要求 <= cap は要求値／既定、要求 > cap はエラー。
            assert_eq!(
                resolve_read_limit(Some(50), Some(100)).expect("要求値を返す"),
                50
            );
            assert_eq!(
                resolve_read_limit(None, Some(100)).expect("cap を返す"),
                100
            );
            let error =
                resolve_read_limit(Some(1_000), Some(100)).expect_err("cap 超えはエラーになる");
            assert_limit_validation_error(&error);

            // cap Some(>DEFAULT): 既定上限へ下げず cap の値を尊重する。
            let large_cap = DEFAULT_MAX_READ_BYTES * 2;
            assert_eq!(
                resolve_read_limit(None, Some(large_cap)).expect("cap を尊重する"),
                large_cap
            );
            assert_eq!(
                resolve_read_limit(Some(DEFAULT_MAX_READ_BYTES), Some(large_cap))
                    .expect("要求値を返す"),
                DEFAULT_MAX_READ_BYTES
            );
            let error = resolve_read_limit(Some(large_cap + 1), Some(large_cap))
                .expect_err("cap 超えはエラーになる");
            assert_limit_validation_error(&error);

            // limit u64::MAX も黙って clamp せず明示エラーにする。
            for cap in [None, Some(100)] {
                let error = resolve_read_limit(Some(u64::MAX), cap)
                    .expect_err("u64::MAX の要求はエラーになる");
                assert_limit_validation_error(&error);
            }
        }

        /// `read()` が上限超過を部分読みではなく Validation エラーとして返すこと（R6 PR-M04）。
        #[tokio::test]
        async fn read_rejects_limit_above_capabilities_max_file_size() {
            let temp = TempRoot::new();
            fs::write(temp.path.join("data.bin"), b"0123456789").unwrap();
            let storage = LocalStorage {
                root: temp.path.clone(),
                capabilities: StorageCapabilities {
                    max_file_size: Some(4),
                    ..Default::default()
                },
                read_only: false,
            };

            // 要求 limit > cap → 部分読みせず Validation エラー。
            let error = storage
                .read(
                    Path::new("data.bin"),
                    ReadOptions {
                        limit: Some(6),
                        ..Default::default()
                    },
                )
                .await
                .expect_err("cap 超えの要求はエラー");
            assert_limit_validation_error(&error);

            // 要求 limit <= cap → 要求値どおりに読める。
            let bytes = storage
                .read(
                    Path::new("data.bin"),
                    ReadOptions {
                        limit: Some(2),
                        ..Default::default()
                    },
                )
                .await
                .expect("cap 以下の要求は成功する");
            assert_eq!(bytes, b"01".to_vec());

            // read_text() も read() を経由するため同じエラーが伝播する。
            let error = storage
                .read_text(
                    Path::new("data.bin"),
                    ReadOptions {
                        limit: Some(u64::MAX),
                        ..Default::default()
                    },
                )
                .await
                .expect_err("read_text も上限超過をエラーにする");
            assert_limit_validation_error(&error);
        }

        /// limit 未指定でファイルが上限より大きい場合も黙って切り詰めないこと（R6 PR-M04）。
        #[tokio::test]
        async fn read_rejects_file_larger_than_effective_limit() {
            let temp = TempRoot::new();
            fs::write(temp.path.join("oversized.bin"), b"0123456789").unwrap();
            let storage = LocalStorage {
                root: temp.path.clone(),
                capabilities: StorageCapabilities {
                    max_file_size: Some(4),
                    ..Default::default()
                },
                read_only: false,
            };

            // limit=None は「ファイル全体を読む」要求なので、切り詰めるならエラーにする。
            let error = storage
                .read(Path::new("oversized.bin"), ReadOptions::default())
                .await
                .expect_err("上限を超えるファイルは黙って部分読みしない");
            assert_limit_validation_error(&error);

            // 上限ちょうどのファイルは正常に読める。
            fs::write(temp.path.join("exact.bin"), b"0123").unwrap();
            let bytes = storage
                .read(Path::new("exact.bin"), ReadOptions::default())
                .await
                .expect("上限ちょうどは成功する");
            assert_eq!(bytes, b"0123".to_vec());
        }

        /// offset がファイル末尾を超えてもエラーではなく空 Vec を返すこと（R6 PR-M04）。
        #[tokio::test]
        async fn read_with_offset_beyond_eof_returns_empty_vec() {
            let temp = TempRoot::new();
            fs::write(temp.path.join("small.txt"), b"abc").unwrap();
            let storage = temp.storage();

            let bytes = storage
                .read(
                    Path::new("small.txt"),
                    ReadOptions {
                        offset: Some(1_000),
                        ..Default::default()
                    },
                )
                .await
                .unwrap();

            assert!(bytes.is_empty(), "末尾超えの offset では空 Vec: {bytes:?}");
        }

        /// read-only root の initialize() はディレクトリを作成しないこと（R4 指摘）。
        #[tokio::test]
        async fn read_only_initialize_does_not_create_root_directory() {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let missing = std::env::temp_dir().join(format!(
                "openrustbooks-io-readonly-missing-{}-{}",
                std::process::id(),
                id
            ));
            let _ = fs::remove_dir_all(&missing);
            let mut storage = LocalStorage::new_with_read_only(missing.clone(), true);

            storage.initialize().await.unwrap();

            assert!(!missing.exists(), "read-only root は初期化で作らない");
        }

        /// read-only root では全ての書き込み系操作が固定文言で拒否されること（R3-AU01）。
        #[tokio::test]
        async fn read_only_storage_rejects_mutating_operations() {
            let temp = TempRoot::new();
            let storage = LocalStorage::new_with_read_only(temp.path.clone(), true);
            let existing = seed_existing_file(&temp);

            assert!(storage.is_read_only());
            assert_eq!(storage.root(), temp.path.as_path());
            let capabilities = StorageBackend::capabilities(&storage);
            assert!(!capabilities.can_write);
            assert!(!capabilities.can_create_dir);
            assert!(!capabilities.can_delete);
            assert!(!capabilities.can_move);

            let write_error = storage
                .write(Path::new("new.txt"), b"data", WriteOptions::default())
                .await
                .expect_err("read-only root への write は拒否される");
            assert_eq!(
                write_error.to_string(),
                format!("Validation error: {READ_ONLY_ROOT_MESSAGE}")
            );

            let create_dir_error = storage
                .create_dir(Path::new("nested"), true)
                .await
                .expect_err("read-only root への create_dir は拒否される");
            assert_eq!(
                create_dir_error.to_string(),
                format!("Validation error: {READ_ONLY_ROOT_MESSAGE}")
            );

            let delete_error = storage
                .delete(Path::new("existing.txt"), false)
                .await
                .expect_err("read-only root の delete は拒否される");
            assert_eq!(
                delete_error.to_string(),
                format!("Validation error: {READ_ONLY_ROOT_MESSAGE}")
            );

            let move_error = storage
                .move_item(Path::new("existing.txt"), Path::new("moved.txt"), false)
                .await
                .expect_err("read-only root の move_item は拒否される");
            assert_eq!(
                move_error.to_string(),
                format!("Validation error: {READ_ONLY_ROOT_MESSAGE}")
            );

            // 拒否されただけでなく、ディスク上の内容も変化していないこと。
            assert_eq!(fs::read(&existing).unwrap(), b"original".to_vec());
            assert!(!temp.path.join("new.txt").exists());
            assert!(!temp.path.join("nested").exists());
            assert!(!temp.path.join("moved.txt").exists());
        }

        /// read_only=false では既存どおり書き込めること（既存動作の回帰確認）。
        #[tokio::test]
        async fn writable_storage_allows_mutating_operations() {
            let temp = TempRoot::new();
            let storage = LocalStorage::new_with_read_only(temp.path.clone(), false);

            assert!(!storage.is_read_only());
            storage
                .write(Path::new("new.txt"), b"data", WriteOptions::default())
                .await
                .unwrap();
            storage.create_dir(Path::new("nested"), true).await.unwrap();
            storage
                .move_item(Path::new("new.txt"), Path::new("moved.txt"), false)
                .await
                .unwrap();
            storage.delete(Path::new("moved.txt"), false).await.unwrap();

            assert!(!temp.path.join("new.txt").exists());
            assert!(temp.path.join("nested").is_dir());
        }

        /// 一時ファイル・退避ファイルが残っていないことを検証する。
        fn assert_no_temp_leftovers(temp: &TempRoot) {
            let leftovers: Vec<PathBuf> = fs::read_dir(&temp.path)
                .unwrap()
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.ends_with(".tmp"))
                })
                .collect();
            assert!(
                leftovers.is_empty(),
                "一時ファイルが残っていないこと: {leftovers:?}"
            );
        }

        /// エラーが root 拒否の固定文言（Validation）であることを検証する。
        fn assert_root_operation_error(error: CoreError) {
            match error {
                CoreError::Validation(message) => {
                    assert_eq!(message, STORAGE_ROOT_OPERATION_MESSAGE);
                    assert!(!message.contains('/'), "エラー文言にパスを含めない");
                }
                other => panic!("expected Validation error, got {other:?}"),
            }
        }

        /// root 自身に対する delete / move_item を固定文言で拒否すること（R2-ER01）。
        /// 相対 "." と root の絶対パスの両方で、canonical 比較により拒否される。
        /// create_dir は root 直下への新規作成を壊さないため対象外（R4 指摘）。
        #[tokio::test]
        async fn mutating_operations_reject_storage_root_itself() {
            let temp = TempRoot::new();
            let storage = temp.storage();
            fs::write(temp.path.join("keep.txt"), b"keep").unwrap();
            let root_abs = temp.path.canonicalize().unwrap();

            for target in [Path::new("."), root_abs.as_path()] {
                assert_root_operation_error(
                    storage
                        .delete(target, true)
                        .await
                        .expect_err("root 自身の delete は拒否される"),
                );
                assert_root_operation_error(
                    storage
                        .move_item(target, Path::new("moved"), false)
                        .await
                        .expect_err("root 自身を移動元にする move_item は拒否される"),
                );
                assert_root_operation_error(
                    storage
                        .move_item(Path::new("keep.txt"), target, true)
                        .await
                        .expect_err("root 自身を移動先にする move_item は拒否される"),
                );
            }

            assert_eq!(
                fs::read(temp.path.join("keep.txt")).unwrap(),
                b"keep".to_vec()
            );
            assert!(!temp.path.join("moved").exists());
        }

        /// root 自身への create_dir は idempotent に成功し、root 直下の新規作成が壊れないこと（R4 指摘）。
        #[tokio::test]
        async fn create_dir_on_root_keeps_direct_child_creation_working() {
            let temp = TempRoot::new();
            let storage = temp.storage();

            storage
                .create_dir(Path::new("."), true)
                .await
                .expect("root 自身の create_dir_all は既存 root に対して成功する");
            storage
                .write(Path::new("new.txt"), b"data", WriteOptions::default())
                .await
                .expect("root 直下への新規作成は成功する");

            assert_eq!(
                fs::read(temp.path.join("new.txt")).unwrap(),
                b"data".to_vec()
            );
        }

        /// overwrite=false が rename 直前に既存宛先を再確認し、両方を保持すること。
        #[tokio::test]
        async fn move_without_overwrite_preserves_existing_destination() {
            let temp = TempRoot::new();
            let storage = temp.storage();
            let from = temp.path.join("source.txt");
            let to = temp.path.join("existing.txt");
            fs::write(&from, b"source").unwrap();
            fs::write(&to, b"original").unwrap();

            let error = storage
                .move_item(Path::new("source.txt"), Path::new("existing.txt"), false)
                .await
                .expect_err("overwrite=false は既存宛先を拒否する");
            match error {
                CoreError::Validation(message) => {
                    assert_eq!(message, MOVE_DESTINATION_EXISTS_MESSAGE);
                    assert!(!message.contains("existing.txt"), "パスを含めない");
                }
                other => panic!("expected Validation error, got {other:?}"),
            }

            assert_eq!(fs::read(&from).unwrap(), b"source".to_vec());
            assert_eq!(fs::read(&to).unwrap(), b"original".to_vec());
            assert_no_temp_leftovers(&temp);
        }

        /// overwrite=true が退避経由で既存宛先を置き換えること。
        #[tokio::test]
        async fn move_with_overwrite_replaces_existing_destination() {
            let temp = TempRoot::new();
            let storage = temp.storage();
            let from = temp.path.join("source.txt");
            let to = temp.path.join("existing.txt");
            fs::write(&from, b"source").unwrap();
            fs::write(&to, b"original").unwrap();

            storage
                .move_item(Path::new("source.txt"), Path::new("existing.txt"), true)
                .await
                .expect("overwrite=true の上書き移動は成功する");

            assert_eq!(fs::read(&to).unwrap(), b"source".to_vec());
            assert!(!from.exists(), "移動元は消える");
            assert_no_temp_leftovers(&temp);
        }

        /// ディレクトリの上書き移動でも退避先が残らないこと。
        #[tokio::test]
        async fn move_with_overwrite_replaces_existing_directory() {
            let temp = TempRoot::new();
            let storage = temp.storage();
            fs::create_dir(temp.path.join("source_dir")).unwrap();
            fs::write(temp.path.join("source_dir/new.txt"), b"new").unwrap();
            fs::create_dir(temp.path.join("target_dir")).unwrap();
            fs::write(temp.path.join("target_dir/old.txt"), b"old").unwrap();

            storage
                .move_item(Path::new("source_dir"), Path::new("target_dir"), true)
                .await
                .expect("ディレクトリの上書き移動も成功する");

            assert_eq!(
                fs::read(temp.path.join("target_dir/new.txt")).unwrap(),
                b"new".to_vec()
            );
            assert!(!temp.path.join("target_dir/old.txt").exists());
            assert!(!temp.path.join("source_dir").exists());
            assert_no_temp_leftovers(&temp);
        }

        /// 退避後に rename が失敗しても宛先を復元すること（元データを失わない）。
        #[tokio::test]
        async fn move_with_overwrite_restores_destination_when_rename_fails() {
            let temp = TempRoot::new();
            let storage = temp.storage();
            let to = temp.path.join("existing.txt");
            fs::write(&to, b"original").unwrap();

            // 移動元が存在しないため、宛先の退避後 rename が失敗する。
            let error = storage
                .move_item(Path::new("missing.txt"), Path::new("existing.txt"), true)
                .await
                .expect_err("存在しない元からの移動は失敗する");
            assert!(
                matches!(error, CoreError::Io(_)),
                "rename 失敗は Io エラーとして伝播する: {error:?}"
            );

            assert_eq!(
                fs::read(&to).unwrap(),
                b"original".to_vec(),
                "失敗時に宛先が復元される"
            );
            assert_no_temp_leftovers(&temp);
        }
    }
}

/// `StorageManager::set_default_backend()` の既定切替テスト（R6 R4-unit#3）。
///
/// `local` モジュール内の `LocalStorage` は id が `"local"` でファイルシステムも
/// 必要なため、`"known"` を返す独立した最小 stub を使う（本番コードは変更しない）。
#[cfg(test)]
mod storage_manager_tests {
    use super::*;

    struct KnownBackend;

    #[async_trait]
    impl StorageBackend for KnownBackend {
        fn id(&self) -> &str {
            "known"
        }

        fn name(&self) -> &str {
            "Known Test Backend"
        }

        fn capabilities(&self) -> StorageCapabilities {
            StorageCapabilities::default()
        }

        async fn initialize(&mut self) -> CoreResult<()> {
            Ok(())
        }

        async fn health_check(&self) -> CoreResult<bool> {
            Ok(true)
        }

        async fn list(&self, _path: &Path, _options: ListOptions) -> CoreResult<Vec<FileEntry>> {
            Err(CoreError::storage("not used in this test"))
        }

        async fn read(&self, _path: &Path, _options: ReadOptions) -> CoreResult<Vec<u8>> {
            Err(CoreError::storage("not used in this test"))
        }

        async fn read_text(&self, _path: &Path, _options: ReadOptions) -> CoreResult<String> {
            Err(CoreError::storage("not used in this test"))
        }

        async fn write(
            &self,
            _path: &Path,
            _data: &[u8],
            _options: WriteOptions,
        ) -> CoreResult<()> {
            Err(CoreError::storage("not used in this test"))
        }

        async fn write_text(
            &self,
            _path: &Path,
            _text: &str,
            _options: WriteOptions,
        ) -> CoreResult<()> {
            Err(CoreError::storage("not used in this test"))
        }

        async fn create_dir(&self, _path: &Path, _recursive: bool) -> CoreResult<()> {
            Err(CoreError::storage("not used in this test"))
        }

        async fn delete(&self, _path: &Path, _recursive: bool) -> CoreResult<()> {
            Err(CoreError::storage("not used in this test"))
        }

        async fn move_item(&self, _from: &Path, _to: &Path, _overwrite: bool) -> CoreResult<()> {
            Err(CoreError::storage("not used in this test"))
        }

        async fn metadata(&self, _path: &Path) -> CoreResult<FileEntry> {
            Err(CoreError::storage("not used in this test"))
        }

        async fn exists(&self, _path: &Path) -> CoreResult<bool> {
            Err(CoreError::storage("not used in this test"))
        }

        async fn open_in_system(&self, _path: &Path) -> CoreResult<()> {
            Err(CoreError::storage("not used in this test"))
        }

        async fn watch(&self, _path: &Path) -> CoreResult<Box<dyn StorageWatchStream>> {
            Err(CoreError::storage("not used in this test"))
        }
    }

    /// 登録済み backend を既定に設定できること（成功経路の維持）。
    #[test]
    fn set_default_backend_with_known_backend_succeeds() {
        let mut manager = StorageManager::new();
        manager.add_backend(Box::new(KnownBackend));

        // add_backend() が最初の backend を既定にすることも併せて確認する。
        assert_eq!(manager.default_backend().map(|b| b.id()), Some("known"));

        manager
            .set_default_backend("known")
            .expect("登録済み backend は既定に設定できる");
        assert_eq!(manager.default_backend().map(|b| b.id()), Some("known"));
    }

    /// 未検出 backend が `CoreError::Storage` になり、固定文言のみで
    /// パス・秘密・本文を含まないこと（R6 R4-unit#3 / GOAL.md）。
    #[test]
    fn set_default_backend_with_missing_backend_returns_storage_error() {
        let mut manager = StorageManager::new();
        manager.add_backend(Box::new(KnownBackend));

        let error = manager
            .set_default_backend("missing")
            .expect_err("未登録 backend は既定に設定できない");
        let rendered = error.to_string();

        assert!(
            rendered.contains("Backend not found: missing"),
            "想定外のエラー: {rendered}"
        );
        // エラー文言は id だけから組み立てられ、それ以外の情報は一切載らない。
        assert_eq!(rendered, "Storage error: Backend not found: missing");

        match error {
            CoreError::Storage(message) => assert_eq!(message, "Backend not found: missing"),
            other => panic!("expected Storage error, got {other:?}"),
        }

        // 失敗しても既定 backend は変化しない。
        assert_eq!(manager.default_backend().map(|b| b.id()), Some("known"));
    }
}
