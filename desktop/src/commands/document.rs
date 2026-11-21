//! Document management commands.

use crate::commands::ipc_error;
use crate::state::AppState;
use openrustbooks_core::{
    CoreError, CoreResult,
    config::LocalRootConfig,
    document::{
        Document, DocumentContent, DocumentFormat, DocumentId, DocumentMetadata,
        extract_front_matter_preserving,
    },
};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::{AppHandle, Emitter, State, command};

/// 既存の最も深い祖先を正規化し、残りを lexical に連結する（ADR-0097 と同じ意味論）。
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
                let name = ancestor
                    .file_name()
                    .ok_or_else(|| CoreError::validation("Unable to resolve path"))?;
                tail.push(name.to_os_string());
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| CoreError::validation("Unable to resolve path"))?;
            }
            // エラー文言にはパスを含めない（ADR-0103）
            Err(_) => return Err(CoreError::validation("Unable to resolve path")),
        }
    }
}

/// 解決済みパスと、一致した root の `read_only` フラグ。
#[derive(Debug)]
struct ResolvedPath {
    path: PathBuf,
    read_only: bool,
}

/// `local_roots[].path` を root として入力パスを検証する。
///
/// - root は絶対・存在・ディレクトリのみ受理し、相対／存在しない root は拒否する。
/// - `local_roots` が空なら fail-closed で拒否する（fs コマンドと一貫）。
/// - `..` は機械的に拒否する。
/// - 親ディレクトリを canonicalize → root 配下確認 → ファイル名を連結する。
///   ファイル本体が未作成でも拒否しない（ADR-0097 と同じ意味論）。
/// - 絶対パスも root 配下なら許可する。
/// - エラー文言にはパスを含めない（ADR-0103）。
fn resolve_within_roots(roots: &[LocalRootConfig], path: &Path) -> CoreResult<ResolvedPath> {
    if roots.is_empty() {
        return Err(CoreError::storage("No storage root configured"));
    }

    let mut canonical_roots = Vec::with_capacity(roots.len());
    for root in roots {
        if !root.path.is_absolute() {
            return Err(CoreError::storage("Storage root must be an absolute path"));
        }
        let canonical = root
            .path
            .canonicalize()
            .map_err(|_| CoreError::storage("Storage root does not exist"))?;
        if !canonical.is_dir() {
            return Err(CoreError::storage("Storage root is not a directory"));
        }
        canonical_roots.push((canonical, root.read_only));
    }

    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(CoreError::validation(
            "Parent directory traversal is not allowed",
        ));
    }

    for (root, read_only) in &canonical_roots {
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        let name = match candidate.file_name() {
            Some(name) => name,
            None => continue,
        };
        let parent = match candidate.parent() {
            Some(parent) => parent,
            None => continue,
        };

        // 親の既存部分を正規化して root 配下を確認し、ファイル名を連結
        let canonical_parent = match canonicalize_existing_prefix(parent) {
            Ok(parent) => parent,
            Err(_) => continue,
        };
        if !canonical_parent.starts_with(root) {
            continue;
        }
        let resolved = canonical_parent.join(name);

        // 既存ファイルが root 外への symlink なら拒否（新規ファイルは対象外）
        match resolved.canonicalize() {
            Ok(target) if target.starts_with(root) => {}
            Ok(_) => continue,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => continue,
        }

        return Ok(ResolvedPath {
            path: resolved,
            read_only: *read_only,
        });
    }

    Err(CoreError::validation("Path escapes storage root"))
}

/// 読み込んだテキストから Document を構築する。
///
/// `open_document` と往復テストで同じ経路を使うため純粋関数に切り出している。
/// Markdown は front matter の元テキストも保持し、保存時の再シリアライズを避ける。
fn document_from_loaded_text(
    id: DocumentId,
    format: DocumentFormat,
    content: String,
    path: Option<PathBuf>,
    title: Option<String>,
) -> Document {
    let size_bytes = content.len() as u64;
    let content = match format {
        DocumentFormat::Markdown => {
            let (front_matter, body, front_matter_raw) = extract_front_matter_preserving(&content);
            DocumentContent::Markdown {
                content: body,
                front_matter,
                front_matter_raw,
            }
        }
        _ => DocumentContent::Text(content),
    };
    Document {
        metadata: DocumentMetadata {
            id,
            format,
            title,
            author: None,
            created_at: None,
            modified_at: None,
            size_bytes,
            path,
            url: None,
            checksum: None,
            extra: Default::default(),
        },
        content,
    }
}

/// atomic write の一時ファイル名に使うプロセス内カウンタ。
///
/// 固定の `<file>.tmp` では並行書き込みが同じ一時ファイルを同時に開き、
/// 片方の内容が失われる／途中状態が rename される恐れがある。
/// PID と組み合わせることでプロセス間でも名前が衝突しない（core::io::local と同種）。
static ATOMIC_WRITE_ID: AtomicUsize = AtomicUsize::new(0);

/// テスト時のみ一時ファイルへの書き込み失敗を注入する。
///
/// 本番ビルドには存在しない。スレッドローカルにすることで並列テスト間の
/// フラグ消費を防ぐ（`#[tokio::test]` は current-thread runtime で poll される）。
#[cfg(test)]
mod test_hooks {
    use std::cell::Cell;

    thread_local! {
        pub static FAIL_NEXT_ATOMIC_WRITE: Cell<bool> = const { Cell::new(false) };
    }
}

/// `<file>.<pid>.<counter>.tmp` 形式の一時パスを生成する（元ファイルと同一ディレクトリ）。
fn atomic_temp_path(full_path: &Path) -> PathBuf {
    let counter = ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let mut file_name = full_path.file_name().unwrap_or_default().to_os_string();
    file_name.push(format!(".{pid}.{counter}.tmp"));
    full_path.with_file_name(file_name)
}

/// 既存の一時ファイルを決して上書きせずに開く。
async fn create_atomic_temp(full_path: &Path) -> CoreResult<(PathBuf, tokio::fs::File)> {
    const MAX_TEMP_ATTEMPTS: usize = 128;

    for _ in 0..MAX_TEMP_ATTEMPTS {
        let candidate = atomic_temp_path(full_path);
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
            .await
        {
            Ok(file) => return Ok((candidate, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(CoreError::Io(error)),
        }
    }

    Err(CoreError::storage(format!(
        "Unable to create a unique temporary file after {MAX_TEMP_ATTEMPTS} attempts"
    )))
}

/// 一時ファイルへ書き込んでから rename する。失敗時は一時ファイルを削除し、元ファイルを保持する。
///
/// Windows 等で rename が既存ファイルを置換できない場合も、旧来の直接書き込みへは
/// フォールバックせず fail-closed とする（元ファイルを破壊しない）。
async fn atomic_write(full_path: &Path, data: &[u8]) -> CoreResult<()> {
    let (temp_path, mut file) = create_atomic_temp(full_path).await?;

    let write_result = async {
        use tokio::io::AsyncWriteExt;
        #[cfg(test)]
        if test_hooks::FAIL_NEXT_ATOMIC_WRITE.replace(false) {
            return Err(std::io::Error::other("injected temporary write failure"));
        }
        file.write_all(data).await?;
        file.flush().await?;
        // 可能ならディスクへ同期する。失敗したら保存失敗として扱う（fail-closed）。
        file.sync_all().await?;
        Ok::<(), std::io::Error>(())
    }
    .await;

    // Windows ではハンドルを開いたまま rename できないため明示的に閉じる。
    drop(file);

    if write_result.is_err() {
        let _ = tokio::fs::remove_file(&temp_path).await;
    }
    write_result?;

    if let Err(error) = tokio::fs::rename(&temp_path, full_path).await {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(CoreError::Io(error));
    }

    Ok(())
}

/// Open a document from path
#[command]
pub async fn open_document(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    path: String,
) -> Result<Document, String> {
    let config = state.config();
    // 設定済み root 配下のみ読み取りを許可する
    let path_buf = resolve_within_roots(&config.storage.local_roots, &PathBuf::from(&path))
        .map_err(ipc_error)?
        .path;

    // Read file content
    let content = tokio::fs::read_to_string(&path_buf)
        .await
        .map_err(|e| ipc_error(CoreError::Io(e)))?;

    // Detect format
    let format =
        DocumentFormat::from_extension(path_buf.extension().and_then(|s| s.to_str()).unwrap_or(""))
            .unwrap_or(DocumentFormat::Txt);

    // Create document
    let id = DocumentId::from_string(path.clone());
    let document = document_from_loaded_text(
        id.clone(),
        format,
        content,
        Some(path_buf.clone()),
        path_buf
            .file_stem()
            .and_then(|s| s.to_str())
            .map(String::from),
    );

    state.add_document(document.clone());
    state.mark_saved(&id);

    // Emit event to frontend
    let _ = app.emit("document-opened", &document);

    Ok(document)
}

/// `save_document` の本体。テストから AppHandle / MockRuntime で直接呼べるように切り出している。
async fn save_document_impl<R: tauri::Runtime>(
    state: &AppState,
    app: &AppHandle<R>,
    id: &str,
    content: Option<String>,
) -> Result<(), String> {
    let doc_id = DocumentId::from_string(id.to_string());
    let mut document = state
        .get_document(&doc_id)
        .ok_or_else(|| ipc_error(CoreError::DocumentNotFound(doc_id.0.to_string())))?;

    // Update content if provided
    if let Some(new_content) = content
        && let Some(text) = document.as_text_mut()
    {
        *text = new_content;
    }

    // ディスク書き込みが成功した場合にのみメタデータを更新する（fail-closed）。
    if let Some(path) = &document.metadata.path {
        let config = state.config();
        // 設定済み root 配下のみ書き込みを許可する
        let resolved =
            resolve_within_roots(&config.storage.local_roots, path).map_err(ipc_error)?;
        if resolved.read_only {
            return Err(ipc_error(CoreError::validation(
                "Storage root is read-only",
            )));
        }
        let text = document
            .full_text()
            .ok_or_else(|| ipc_error(CoreError::validation("Document is not text-editable")))?;

        // atomic temp + rename で保存する。失敗時は元ファイルを保持し、
        // ここを抜けるため metadata / mark_saved / emit は行われない。
        atomic_write(&resolved.path, text.as_bytes())
            .await
            .map_err(ipc_error)?;

        // Update metadata（成功後のみ）
        document.metadata.size_bytes = text.len() as u64;
        document.metadata.modified_at = Some(chrono::Utc::now());
    }

    state.add_document(document.clone());
    state.mark_saved(&doc_id);

    let _ = app.emit("document-saved", &document);

    Ok(())
}

/// Save a document
#[command]
pub async fn save_document(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    id: String,
    content: Option<String>,
) -> Result<(), String> {
    save_document_impl(state.inner().as_ref(), &app, &id, content).await
}

/// Close a document
#[command]
pub async fn close_document(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    id: String,
    force: Option<bool>,
) -> Result<(), String> {
    let doc_id = DocumentId::from_string(id);

    // Check for unsaved changes
    if !force.unwrap_or(false) && state.is_unsaved(&doc_id) {
        return Err(ipc_error(CoreError::validation(
            "Document has unsaved changes",
        )));
    }

    let document = state.remove_document(&doc_id);
    if document.is_some() {
        let _ = app.emit("document-closed", doc_id.0);
    }

    Ok(())
}

/// List all open documents
#[command]
pub fn list_documents(state: State<'_, Arc<AppState>>) -> Vec<Document> {
    state.list_documents()
}

/// Get document content
#[command]
pub fn get_document_content(state: State<'_, Arc<AppState>>, id: String) -> Result<String, String> {
    let doc_id = DocumentId::from_string(id);
    let document = state
        .get_document(&doc_id)
        .ok_or_else(|| ipc_error(CoreError::DocumentNotFound(doc_id.0.to_string())))?;

    document
        .as_text()
        .map(String::from)
        .ok_or_else(|| ipc_error(CoreError::validation("Document is not text-editable")))
}

/// Set document content
#[command]
pub fn set_document_content(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    id: String,
    content: String,
) -> Result<(), String> {
    let doc_id = DocumentId::from_string(id);
    let mut document = state
        .get_document(&doc_id)
        .ok_or_else(|| ipc_error(CoreError::DocumentNotFound(doc_id.0.to_string())))?;

    if let Some(text) = document.as_text_mut() {
        *text = content;
        state.add_document(document.clone());
        state.mark_unsaved(&doc_id);
        let _ = app.emit("document-changed", &document);
        Ok(())
    } else {
        Err(ipc_error(CoreError::validation(
            "Document is not text-editable",
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openrustbooks_core::AppConfig;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tauri::{Listener, test::MockRuntime, test::mock_app};

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    struct TempRoot {
        path: PathBuf,
    }

    impl TempRoot {
        fn new() -> Self {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "openrustbooks-document-test-{}-{}",
                std::process::id(),
                id
            ));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn roots(&self) -> Vec<LocalRootConfig> {
            vec![LocalRootConfig {
                id: "test".to_string(),
                name: "Test root".to_string(),
                path: self.path.clone(),
                read_only: false,
                watch: false,
            }]
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn relative_path_under_root_is_allowed() {
        let temp = TempRoot::new();
        let resolved =
            resolve_within_roots(&temp.roots(), Path::new("sub/nested/file.txt")).unwrap();
        assert_eq!(
            resolved.path,
            temp.path
                .canonicalize()
                .unwrap()
                .join("sub/nested/file.txt")
        );
        assert!(!resolved.read_only);
    }

    #[test]
    fn parent_dir_escape_is_rejected() {
        let temp = TempRoot::new();
        assert!(resolve_within_roots(&temp.roots(), Path::new("../escape")).is_err());
        assert!(resolve_within_roots(&temp.roots(), Path::new("sub/../../escape")).is_err());
    }

    #[test]
    fn absolute_path_outside_root_is_rejected() {
        let temp = TempRoot::new();
        let outside = TempRoot::new();
        let abs = outside.path.canonicalize().unwrap().join("secret.txt");
        assert!(resolve_within_roots(&temp.roots(), &abs).is_err());
    }

    #[test]
    fn absolute_path_under_root_is_allowed() {
        let temp = TempRoot::new();
        let abs = temp.path.canonicalize().unwrap().join("inside.txt");
        fs::write(&abs, b"ok").unwrap();
        let resolved = resolve_within_roots(&temp.roots(), &abs).unwrap();
        assert_eq!(resolved.path, abs);
    }

    #[test]
    fn empty_local_roots_is_rejected() {
        let error = resolve_within_roots(&[], Path::new("file.txt")).unwrap_err();
        assert!(error.to_string().contains("No storage root configured"));
    }

    #[test]
    fn nonexistent_new_file_is_allowed() {
        let temp = TempRoot::new();
        // 親ディレクトリも未作成でも、root 配下なら許可する
        let new_file = temp
            .path
            .canonicalize()
            .unwrap()
            .join("new_dir/new_file.txt");
        let resolved = resolve_within_roots(&temp.roots(), &new_file).unwrap();
        assert_eq!(resolved.path, new_file);
    }

    #[test]
    fn relative_root_is_rejected() {
        let temp = TempRoot::new();
        let mut roots = temp.roots();
        roots[0].path = PathBuf::from("relative/root");
        let error = resolve_within_roots(&roots, Path::new("file.txt")).unwrap_err();
        assert!(error.to_string().contains("absolute"));
    }

    #[test]
    fn nonexistent_root_is_rejected() {
        let temp = TempRoot::new();
        let mut roots = temp.roots();
        roots[0].path = temp.path.join("no-such-root");
        let error = resolve_within_roots(&roots, Path::new("file.txt")).unwrap_err();
        assert!(error.to_string().contains("does not exist"));
    }

    #[test]
    fn file_as_root_is_rejected() {
        let temp = TempRoot::new();
        let file = temp.path.join("not-a-dir.txt");
        fs::write(&file, b"x").unwrap();
        let mut roots = temp.roots();
        roots[0].path = file;
        let error = resolve_within_roots(&roots, Path::new("file.txt")).unwrap_err();
        assert!(error.to_string().contains("not a directory"));
    }

    #[test]
    fn read_only_flag_is_propagated() {
        let temp = TempRoot::new();
        let abs = temp.path.canonicalize().unwrap().join("inside.txt");
        fs::write(&abs, b"ok").unwrap();
        let mut roots = temp.roots();
        roots[0].read_only = true;
        let resolved = resolve_within_roots(&roots, &abs).unwrap();
        assert!(resolved.read_only);
    }

    /// DocumentNotFound の id（path 由来）が IPC 文字列へ漏れないこと。
    #[test]
    fn document_ipc_error_string_never_contains_id_or_path() {
        let secret_id = "/home/koppe/secret/private.epub";
        let rendered = ipc_error(CoreError::DocumentNotFound(secret_id.to_string()));

        assert_eq!(rendered, "Internal configuration error");
        assert!(!rendered.contains(secret_id));
        assert!(!rendered.contains("private.epub"));
    }

    #[test]
    fn error_messages_do_not_include_paths() {
        let temp = TempRoot::new();
        let outside = TempRoot::new();
        let root = temp
            .path
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let outside_path = outside
            .path
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let outside_abs = PathBuf::from(&outside_path).join("secret.txt");

        let errors = [
            resolve_within_roots(&temp.roots(), Path::new("../escape")).unwrap_err(),
            resolve_within_roots(&temp.roots(), Path::new("sub/../../escape")).unwrap_err(),
            resolve_within_roots(&temp.roots(), &outside_abs).unwrap_err(),
        ];
        for error in errors {
            let message = error.to_string();
            assert!(!message.contains(&root));
            assert!(!message.contains(&outside_path));
        }
    }

    /// `open_document` と同じ構築経路、`save_document` と同じ `full_text` で
    /// 編集なしの open→save が元バイト列を保持することを固定する。
    fn assert_open_save_round_trip(original: &str) {
        let temp = TempRoot::new();
        let path = temp.path.join("note.md");
        fs::write(&path, original.as_bytes()).unwrap();

        // open: ファイルを読み、本番と同じヘルパーで Document を構築する。
        let loaded = fs::read_to_string(&path).unwrap();
        let document = document_from_loaded_text(
            DocumentId::from_string("note.md".to_string()),
            DocumentFormat::Markdown,
            loaded,
            Some(path.clone()),
            Some("note".to_string()),
        );

        // save: 編集せず full_text を書き出し、バイト単位で一致を確認する。
        let saved = document.full_text().unwrap();
        fs::write(&path, saved.as_bytes()).unwrap();
        assert_eq!(fs::read(&path).unwrap(), original.as_bytes().to_vec());
    }

    #[test]
    fn open_save_round_trip_preserves_crlf_and_trailing_newline() {
        assert_open_save_round_trip("---\r\ntitle: Test\r\n---\r\n\r\n# Hello\r\n");
    }

    #[test]
    fn open_save_round_trip_preserves_yaml_comments_and_key_order() {
        assert_open_save_round_trip(
            "---\n# 元のコメント\nzzz: \"double\"\naaa: 'single'\nn: 001\n---\nbody\n",
        );
    }

    #[test]
    fn open_save_round_trip_preserves_missing_trailing_newline() {
        assert_open_save_round_trip("---\ntitle: Test\n---\n# Hello");
    }

    // --- atomic save のテスト ---

    fn state_with_roots(roots: Vec<(PathBuf, bool)>) -> AppState {
        let mut config = AppConfig::default();
        config.storage.local_roots = roots
            .into_iter()
            .enumerate()
            .map(|(index, (path, read_only))| LocalRootConfig {
                id: format!("root-{index}"),
                name: format!("Root {index}"),
                path,
                read_only,
                watch: false,
            })
            .collect();
        AppState::new(config)
    }

    /// command 本体を実際の AppHandle / MockRuntime で呼び、emit 数を記録する。
    struct SaveHarness {
        app: tauri::App<MockRuntime>,
        state: Arc<AppState>,
        saved: Arc<AtomicUsize>,
    }

    impl SaveHarness {
        fn new(state: AppState) -> Self {
            let app = mock_app();
            let saved = Arc::new(AtomicUsize::new(0));
            app.listen("document-saved", {
                let counter = saved.clone();
                move |_| {
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            });
            Self {
                app,
                state: Arc::new(state),
                saved,
            }
        }

        fn state(&self) -> &AppState {
            self.state.as_ref()
        }

        fn handle(&self) -> tauri::AppHandle<MockRuntime> {
            self.app.handle().clone()
        }
    }

    /// 既存ファイルを open_document と同じ経路で state に登録する。
    fn register_document(state: &AppState, path: &Path, original: &str) -> DocumentId {
        let id = DocumentId::from_string("note.md".to_string());
        let document = document_from_loaded_text(
            id.clone(),
            DocumentFormat::Markdown,
            original.to_string(),
            Some(path.to_path_buf()),
            Some("note".to_string()),
        );
        state.add_document(document);
        id
    }

    /// 一時ファイルが残っていないこと（対象ディレクトリに元ファイル 1 つのみ）。
    fn assert_no_temp_files(dir: &Path) {
        let entries = fs::read_dir(dir).unwrap().collect::<Vec<_>>();
        assert_eq!(entries.len(), 1, "一時ファイルが残っていないこと");
    }

    #[tokio::test]
    async fn atomic_write_replaces_existing_file_and_leaves_no_temp() {
        let temp = TempRoot::new();
        let path = temp.path.join("note.md");
        fs::write(&path, b"original").unwrap();

        atomic_write(&path, b"updated").await.unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"updated".to_vec());
        assert_no_temp_files(&temp.path);
    }

    /// 一時ファイル作成後・rename 前に失敗した場合、元内容が残り temp も消えること。
    #[tokio::test]
    async fn atomic_write_failure_preserves_original_and_removes_temp() {
        let temp = TempRoot::new();
        let path = temp.path.join("note.md");
        fs::write(&path, b"original").unwrap();

        test_hooks::FAIL_NEXT_ATOMIC_WRITE.set(true);
        assert!(atomic_write(&path, b"updated").await.is_err());

        assert_eq!(fs::read(&path).unwrap(), b"original".to_vec());
        assert_no_temp_files(&temp.path);
    }

    #[tokio::test]
    async fn command_save_document_success_updates_metadata_and_emits() {
        let writable = TempRoot::new();
        let path = writable.path.join("note.md");
        let original = "---\ntitle: Test\n---\n# Old\n";
        fs::write(&path, original.as_bytes()).unwrap();
        let harness = SaveHarness::new(state_with_roots(vec![(writable.path.clone(), false)]));
        let id = register_document(harness.state(), &path, original);
        harness.state().mark_unsaved(&id);

        save_document_impl(
            harness.state(),
            &harness.handle(),
            &id.0,
            Some("# New\n".to_string()),
        )
        .await
        .expect("writable root への保存は成功する");

        let expected = b"---\ntitle: Test\n---\n# New\n".to_vec();
        assert_eq!(fs::read(&path).unwrap(), expected);
        assert_no_temp_files(&writable.path);
        let saved = harness.state().get_document(&id).unwrap();
        assert_eq!(saved.metadata.size_bytes, expected.len() as u64);
        assert!(saved.metadata.modified_at.is_some());
        assert!(!harness.state().is_unsaved(&id));
        assert_eq!(harness.saved.load(Ordering::SeqCst), 1);
    }

    /// 保存失敗時は元ファイル・metadata・unsaved・emit のすべてが不変であること。
    #[tokio::test]
    async fn command_save_document_failure_preserves_original_and_skips_state_updates() {
        let writable = TempRoot::new();
        let path = writable.path.join("note.md");
        let original = "---\ntitle: Test\n---\n# Old\n";
        fs::write(&path, original.as_bytes()).unwrap();
        let harness = SaveHarness::new(state_with_roots(vec![(writable.path.clone(), false)]));
        let id = register_document(harness.state(), &path, original);
        harness.state().mark_unsaved(&id);

        test_hooks::FAIL_NEXT_ATOMIC_WRITE.set(true);
        assert!(
            save_document_impl(
                harness.state(),
                &harness.handle(),
                &id.0,
                Some("# New\n".to_string()),
            )
            .await
            .is_err()
        );

        assert_eq!(fs::read(&path).unwrap(), original.as_bytes().to_vec());
        assert_no_temp_files(&writable.path);
        let unchanged = harness.state().get_document(&id).unwrap();
        assert_eq!(unchanged.metadata.size_bytes, original.len() as u64);
        assert!(unchanged.metadata.modified_at.is_none());
        assert!(harness.state().is_unsaved(&id));
        assert_eq!(harness.saved.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn command_save_document_rejects_read_only_root_without_side_effects() {
        let read_only = TempRoot::new();
        let path = read_only.path.join("note.md");
        let original = "original";
        fs::write(&path, original.as_bytes()).unwrap();
        let harness = SaveHarness::new(state_with_roots(vec![(read_only.path.clone(), true)]));
        let id = register_document(harness.state(), &path, original);
        harness.state().mark_unsaved(&id);

        assert!(
            save_document_impl(
                harness.state(),
                &harness.handle(),
                &id.0,
                Some("updated".to_string()),
            )
            .await
            .is_err()
        );

        assert_eq!(fs::read(&path).unwrap(), original.as_bytes().to_vec());
        assert_no_temp_files(&read_only.path);
        let unchanged = harness.state().get_document(&id).unwrap();
        assert_eq!(unchanged.metadata.size_bytes, original.len() as u64);
        assert!(unchanged.metadata.modified_at.is_none());
        assert!(harness.state().is_unsaved(&id));
        assert_eq!(harness.saved.load(Ordering::SeqCst), 0);
    }

    /// CRLF・コメント・キー順を含む front matter 原文が保存後も保持されること。
    #[tokio::test]
    async fn command_save_document_preserves_front_matter_raw() {
        let writable = TempRoot::new();
        let path = writable.path.join("note.md");
        let original =
            "---\r\n# 元のコメント\r\nzzz: \"double\"\r\naaa: 'single'\r\n---\r\n# Old\r\n";
        fs::write(&path, original.as_bytes()).unwrap();
        let harness = SaveHarness::new(state_with_roots(vec![(writable.path.clone(), false)]));
        let id = register_document(harness.state(), &path, original);

        save_document_impl(
            harness.state(),
            &harness.handle(),
            &id.0,
            Some("# Changed\r\n".to_string()),
        )
        .await
        .unwrap();

        let expected =
            "---\r\n# 元のコメント\r\nzzz: \"double\"\r\naaa: 'single'\r\n---\r\n# Changed\r\n";
        assert_eq!(fs::read(&path).unwrap(), expected.as_bytes().to_vec());
    }
}
