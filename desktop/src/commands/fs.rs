//! File system commands.

use crate::state::AppState;
use openrustbooks_core::{
    CoreError, CoreResult,
    io::{
        FileEntry, ListOptions, READ_ONLY_ROOT_MESSAGE, STORAGE_ROOT_OPERATION_MESSAGE,
        StorageBackend, WriteOptions, local::LocalStorage,
    },
};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State, command};

/// Convert CoreError to String for Tauri commands
fn map_err(e: CoreError) -> String {
    e.to_string()
}

/// 設定された local root ごとに storage を生成し、`path` と `read_only` を対応付けて保持する。
///
/// 各 `LocalStorage` が `read_only` を保持するため、以降のコマンドは対象パスから
/// root を特定して read_only を強制できる（R3-AU01）。
async fn create_storage(config: &AppState) -> Result<Vec<LocalStorage>, String> {
    let app_config = config.config();
    let mut storages = Vec::with_capacity(app_config.storage.local_roots.len());

    for root in &app_config.storage.local_roots {
        let mut local = LocalStorage::new_with_read_only(root.path.clone(), root.read_only);
        local.initialize().await.map_err(map_err)?;
        storages.push(local);
    }

    Ok(storages)
}

/// 既存の最も深い祖先を正規化し、残りを lexical に連結する。
///
/// `desktop/src/commands/document.rs::canonicalize_existing_prefix` と同じ意味論。
/// ファイル本体が未作成でも拒否しないため、新規作成にも使える。
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

/// 解決済みの対象パスと、それを所有する root の storage。
#[derive(Debug)]
struct ResolvedTarget<'a> {
    storage: &'a LocalStorage,
    /// 親 root を経由せずそのまま IO に渡す絶対パス。
    path: PathBuf,
}

/// 対象パスが属する root を解決し、read_only を検査する。
///
/// - root は絶対・存在・ディレクトリのみ受理し、相対／存在しない root は拒否する。
/// - `..` は機械的に拒否する。
/// - 親を canonicalize して root 配下を確認するため、未作成のファイルも許可する。
/// - 候補 root が一つでも read_only なら fail-closed で拒否する。
///   親 root が writable でも子の read-only root 配下へ書ける経路を残さない（R4 指摘）。
/// - canonical target（symlink の解決先）が別の設定 root に属する場合はその root も候補に含め、
///   read_only なら拒否する。`ResolvedTarget.path` には canonical target を使う（R4 再レビュー）。
/// - 残った候補から最長（最深）一致 root を選び、その解決済みパスを IO に使う。
/// - read_only なら `save_document` と同じ固定文言で拒否する（R3-AU01）。
fn resolve_target_storage<'a>(
    storages: &'a [LocalStorage],
    path: &Path,
) -> CoreResult<ResolvedTarget<'a>> {
    if storages.is_empty() {
        return Err(CoreError::storage("No storage backend available"));
    }

    let mut canonical_roots = Vec::with_capacity(storages.len());
    for storage in storages {
        let root = storage.root();
        if !root.is_absolute() {
            return Err(CoreError::storage("Storage root must be an absolute path"));
        }
        let canonical = root
            .canonicalize()
            .map_err(|_| CoreError::storage("Storage root does not exist"))?;
        if !canonical.is_dir() {
            return Err(CoreError::storage("Storage root is not a directory"));
        }
        canonical_roots.push((canonical, storage));
    }

    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(CoreError::validation(
            "Parent directory traversal is not allowed",
        ));
    }

    // canonical target が storage root 自身なら専用の固定文言で拒否する（R2-ER01）。
    // root 自身は親ディレクトリが root 外になるため、下の候補探索では
    // "Path escapes storage root" に陥る。候補探索の前に判定する。
    for (root, _storage) in &canonical_roots {
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        if candidate
            .canonicalize()
            .is_ok_and(|canonical| canonical == *root)
        {
            return Err(CoreError::validation(STORAGE_ROOT_OPERATION_MESSAGE));
        }
    }

    let mut candidates: Vec<ResolvedTarget<'a>> = Vec::new();
    for (root, _storage) in &canonical_roots {
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        let Some(name) = candidate.file_name() else {
            continue;
        };
        let Some(parent) = candidate.parent() else {
            continue;
        };

        let Ok(canonical_parent) = canonicalize_existing_prefix(parent) else {
            continue;
        };
        if !canonical_parent.starts_with(root) {
            continue;
        }
        let resolved = canonical_parent.join(name);

        // canonical target（既存ファイル・symlink の解決先）を採用する。
        // 未作成ファイルは resolved をそのまま使う。
        let effective = match resolved.canonicalize() {
            Ok(target) => target,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => resolved.clone(),
            Err(_) => continue,
        };

        // canonical target が別の設定 root に属する場合も候補に含める。
        // writable 親 root 内の symlink 経由で read-only 子 root を書き換える
        // 経路を塞ぐため、その root の read_only も必ず検査する（R4 再レビュー high）。
        for (other_root, other_storage) in &canonical_roots {
            if effective.starts_with(other_root) {
                candidates.push(ResolvedTarget {
                    storage: other_storage,
                    path: effective.clone(),
                });
            }
        }
    }

    if candidates.is_empty() {
        return Err(CoreError::validation("Path escapes storage root"));
    }

    // 候補に read_only root が一つでもあれば拒否する（fail-closed）。
    if candidates
        .iter()
        .any(|candidate| candidate.storage.is_read_only())
    {
        return Err(CoreError::validation(READ_ONLY_ROOT_MESSAGE));
    }

    // 深さは root の component 数で比較する。同深度なら設定順の先頭を保持する。
    candidates
        .into_iter()
        .reduce(|best, candidate| {
            let best_depth = best.storage.root().components().count();
            let candidate_depth = candidate.storage.root().components().count();
            if candidate_depth > best_depth {
                candidate
            } else {
                best
            }
        })
        .ok_or_else(|| CoreError::validation("Path escapes storage root"))
}

/// Read directory contents（読み取りは read-only root からも許可する）
#[command]
pub async fn read_directory(
    state: State<'_, Arc<AppState>>,
    path: String,
    options: Option<ListOptions>,
) -> Result<Vec<FileEntry>, String> {
    let storages = create_storage(&state).await?;
    let storage = storages
        .first()
        .ok_or_else(|| CoreError::storage("No storage backend available").to_string())?;

    let path_buf = PathBuf::from(path);
    storage
        .list(&path_buf, options.unwrap_or_default())
        .await
        .map_err(map_err)
}

/// command 本体。generic の `AppHandle<R>` を使い、test からも同じ経路で呼べるようにする。
async fn create_file_impl<R: tauri::Runtime>(
    state: &AppState,
    app: &AppHandle<R>,
    path: String,
    content: Option<String>,
) -> Result<FileEntry, String> {
    let path_buf = PathBuf::from(&path);
    let storages = create_storage(state).await?;
    // 対象 root が read-only なら、親ディレクトリ作成の前に拒否する。
    let resolved = resolve_target_storage(&storages, &path_buf).map_err(map_err)?;

    // Create parent directories（解決済みの絶対パスを使う）
    if let Some(parent) = resolved.path.parent() {
        resolved
            .storage
            .create_dir(parent, true)
            .await
            .map_err(map_err)?;
    }

    // Write file
    let content = content.unwrap_or_default();
    resolved
        .storage
        .write_text(&resolved.path, &content, WriteOptions::default())
        .await
        .map_err(map_err)?;

    let _ = app.emit("file-created", &path);

    resolved
        .storage
        .metadata(&resolved.path)
        .await
        .map_err(map_err)
}

/// Create a new file
#[command]
pub async fn create_file(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    path: String,
    content: Option<String>,
) -> Result<FileEntry, String> {
    create_file_impl(state.inner().as_ref(), &app, path, content).await
}

async fn delete_file_impl<R: tauri::Runtime>(
    state: &AppState,
    app: &AppHandle<R>,
    path: String,
    recursive: Option<bool>,
) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    let storages = create_storage(state).await?;
    let resolved = resolve_target_storage(&storages, &path_buf).map_err(map_err)?;

    resolved
        .storage
        .delete(&resolved.path, recursive.unwrap_or(false))
        .await
        .map_err(map_err)?;

    let _ = app.emit("file-deleted", &path);

    Ok(())
}

/// Delete a file or directory
#[command]
pub async fn delete_file(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    path: String,
    recursive: Option<bool>,
) -> Result<(), String> {
    delete_file_impl(state.inner().as_ref(), &app, path, recursive).await
}

async fn rename_file_impl<R: tauri::Runtime>(
    state: &AppState,
    app: &AppHandle<R>,
    from: String,
    to: String,
    overwrite: Option<bool>,
) -> Result<FileEntry, String> {
    let from_buf = PathBuf::from(&from);
    let to_buf = PathBuf::from(&to);
    let storages = create_storage(state).await?;
    // 移動元・移動先の両方が writable root に属することを確認する。
    let from_resolved = resolve_target_storage(&storages, &from_buf).map_err(map_err)?;
    let to_resolved = resolve_target_storage(&storages, &to_buf).map_err(map_err)?;
    if !std::ptr::eq(from_resolved.storage, to_resolved.storage) {
        return Err(CoreError::storage("Cross-root moves are not supported").to_string());
    }

    from_resolved
        .storage
        .move_item(
            &from_resolved.path,
            &to_resolved.path,
            overwrite.unwrap_or(false),
        )
        .await
        .map_err(map_err)?;

    let _ = app.emit(
        "file-renamed",
        serde_json::json!({ "from": from, "to": to }),
    );

    from_resolved
        .storage
        .metadata(&to_resolved.path)
        .await
        .map_err(map_err)
}

/// Rename/move a file
#[command]
pub async fn rename_file(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    from: String,
    to: String,
    overwrite: Option<bool>,
) -> Result<FileEntry, String> {
    rename_file_impl(state.inner().as_ref(), &app, from, to, overwrite).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use openrustbooks_core::{
        AppConfig, config::LocalRootConfig, io::MOVE_DESTINATION_EXISTS_MESSAGE,
    };
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
                "openrustbooks-fs-test-{}-{}",
                std::process::id(),
                id
            ));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

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

    #[tokio::test]
    async fn create_storage_keeps_read_only_flag_per_root() {
        let read_only_root = TempRoot::new();
        let writable_root = TempRoot::new();
        let state = state_with_roots(vec![
            (read_only_root.path.clone(), true),
            (writable_root.path.clone(), false),
        ]);

        let storages = create_storage(&state).await.unwrap();

        assert_eq!(storages.len(), 2);
        assert_eq!(storages[0].root(), read_only_root.path.as_path());
        assert!(storages[0].is_read_only());
        assert_eq!(storages[1].root(), writable_root.path.as_path());
        assert!(!storages[1].is_read_only());
    }

    /// read_only root は固定文言で拒否し、writable root は許可すること（R3-AU01）。
    #[tokio::test]
    async fn read_only_root_is_rejected_and_writable_root_is_allowed() {
        let read_only_root = TempRoot::new();
        let writable_root = TempRoot::new();
        fs::write(read_only_root.path.join("note.txt"), b"locked").unwrap();
        let state = state_with_roots(vec![
            (read_only_root.path.clone(), true),
            (writable_root.path.clone(), false),
        ]);
        let storages = create_storage(&state).await.unwrap();

        let rejected = resolve_target_storage(&storages, Path::new("note.txt"))
            .expect_err("read-only root への操作は拒否されること");
        assert_eq!(
            rejected.to_string(),
            format!("Validation error: {READ_ONLY_ROOT_MESSAGE}")
        );
        assert!(!rejected.to_string().contains("note.txt"));

        // 未作成の相対パスは既定 root（設定順の先頭）に解決されるため、
        // read-only が先頭にある設定では拒否される。
        let rejected_new = resolve_target_storage(&storages, Path::new("new.txt"))
            .expect_err("既定 root が read-only なら未作成パスも拒否されること");
        assert_eq!(
            rejected_new.to_string(),
            format!("Validation error: {READ_ONLY_ROOT_MESSAGE}")
        );

        // writable root 配下の絶対パス（未作成でも）は許可する（document.rs と同じ意味論）。
        let absolute = writable_root.path.join("absolute.txt");
        let accepted_absolute = resolve_target_storage(&storages, &absolute)
            .expect("root 配下の絶対パスも許可されること");
        assert_eq!(
            accepted_absolute.storage.root(),
            writable_root.path.as_path()
        );
        assert!(!accepted_absolute.storage.is_read_only());

        // 内容は変化していないこと。
        assert_eq!(
            fs::read(read_only_root.path.join("note.txt")).unwrap(),
            b"locked".to_vec()
        );
    }

    /// 親 root と子 root が重なる場合は最長（最深）の root を選ぶこと。
    #[tokio::test]
    async fn deepest_matching_root_is_selected() {
        let parent = TempRoot::new();
        let child = parent.path.join("child");
        fs::create_dir_all(&child).unwrap();
        // 両方の root から見て配下にあるファイルを用意する。
        let shared = parent.path.join("child/note.txt");
        fs::write(&shared, b"shared").unwrap();
        let state = state_with_roots(vec![(parent.path.clone(), false), (child.clone(), false)]);
        let storages = create_storage(&state).await.unwrap();

        let resolved = resolve_target_storage(&storages, &shared)
            .expect("両方 writable なら最深 root が選ばれること");
        assert_eq!(resolved.storage.root(), child.as_path());
        assert_eq!(resolved.path, shared.canonicalize().unwrap());
    }

    /// 候補に read-only root が一つでもあれば固定文言で拒否すること（R4 指摘）。
    #[tokio::test]
    async fn read_only_candidate_is_rejected_fail_closed() {
        let parent = TempRoot::new();
        let child = parent.path.join("child");
        fs::create_dir_all(&child).unwrap();
        let state = state_with_roots(vec![(parent.path.clone(), false), (child.clone(), true)]);
        let storages = create_storage(&state).await.unwrap();

        // 相対パスは親 root にも子 root にも解決され得る。
        let relative = resolve_target_storage(&storages, Path::new("child/new.txt"))
            .expect_err("子 root が read-only なら拒否されること");
        assert_eq!(
            relative.to_string(),
            format!("Validation error: {READ_ONLY_ROOT_MESSAGE}")
        );

        // 絶対パスでも親 root 経由の経路を塞ぐ。
        let absolute = child.join("absolute.txt");
        let absolute_error = resolve_target_storage(&storages, &absolute)
            .expect_err("絶対パスでも子 root が read-only なら拒否されること");
        assert_eq!(
            absolute_error.to_string(),
            format!("Validation error: {READ_ONLY_ROOT_MESSAGE}")
        );
    }

    #[tokio::test]
    async fn path_outside_all_roots_is_rejected() {
        let root = TempRoot::new();
        let outside = TempRoot::new();
        let state = state_with_roots(vec![(root.path.clone(), false)]);
        let storages = create_storage(&state).await.unwrap();

        assert!(
            resolve_target_storage(&storages, Path::new("../escape")).is_err(),
            "親ディレクトリ traversal は拒否されること"
        );
        let absolute_outside = outside.path.join("secret.txt");
        assert!(
            resolve_target_storage(&storages, &absolute_outside).is_err(),
            "root 外の絶対パスは拒否されること"
        );
    }

    /// root 自身は汎用の "Path escapes storage root" ではなく専用固定文言で拒否されること（R4 指摘）。
    #[tokio::test]
    async fn resolve_target_storage_rejects_root_with_dedicated_message() {
        let root = TempRoot::new();
        let state = state_with_roots(vec![(root.path.clone(), false)]);
        let storages = create_storage(&state).await.unwrap();
        let canonical = root.path.canonicalize().unwrap();

        for target in [PathBuf::from("."), canonical] {
            let error =
                resolve_target_storage(&storages, &target).expect_err("root 自身は拒否される");
            let rendered = error.to_string();
            assert_eq!(
                rendered,
                format!("Validation error: {STORAGE_ROOT_OPERATION_MESSAGE}")
            );
            assert!(
                !rendered.contains("escapes"),
                "汎用の脱出文言を使わないこと"
            );
        }
    }

    /// command 本体（`*_impl`）を実際の AppHandle / MockRuntime で呼ぶ harness。
    /// emit されたイベント数も記録する。
    struct CommandHarness {
        app: tauri::App<MockRuntime>,
        state: Arc<AppState>,
        created: Arc<AtomicUsize>,
        deleted: Arc<AtomicUsize>,
        renamed: Arc<AtomicUsize>,
    }

    impl CommandHarness {
        fn new(state: AppState) -> Self {
            let app = mock_app();
            let created = Arc::new(AtomicUsize::new(0));
            let deleted = Arc::new(AtomicUsize::new(0));
            let renamed = Arc::new(AtomicUsize::new(0));

            app.listen("file-created", {
                let counter = created.clone();
                move |_| {
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            });
            app.listen("file-deleted", {
                let counter = deleted.clone();
                move |_| {
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            });
            app.listen("file-renamed", {
                let counter = renamed.clone();
                move |_| {
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            });

            Self {
                app,
                state: Arc::new(state),
                created,
                deleted,
                renamed,
            }
        }

        fn state(&self) -> &AppState {
            self.state.as_ref()
        }

        fn handle(&self) -> tauri::AppHandle<MockRuntime> {
            self.app.handle().clone()
        }
    }

    /// ディレクトリ構造とファイル内容のスナップショット（副作用検証用）。
    fn snapshot(root: &Path) -> Vec<String> {
        fn visit(dir: &Path, prefix: &str, out: &mut Vec<String>) {
            let mut entries: Vec<_> = fs::read_dir(dir)
                .unwrap()
                .filter_map(Result::ok)
                .map(|entry| {
                    (
                        entry.file_name().to_string_lossy().into_owned(),
                        entry.path(),
                    )
                })
                .collect();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            for (name, path) in entries {
                let relative = format!("{prefix}{name}");
                if path.is_dir() {
                    out.push(format!("d:{relative}"));
                    visit(&path, &format!("{relative}/"), out);
                } else {
                    out.push(format!("f:{relative}:{:?}", fs::read(&path).unwrap()));
                }
            }
        }

        let mut entries = Vec::new();
        visit(root, "", &mut entries);
        entries
    }

    fn assert_read_only_error(result: Result<impl std::fmt::Debug, String>) {
        let error = result.expect_err("read-only root への操作は拒否されること");
        assert_eq!(error, format!("Validation error: {READ_ONLY_ROOT_MESSAGE}"));
    }

    /// writable root では command が成功し、イベントが発火すること（emit 検証の基準）。
    #[tokio::test]
    async fn command_create_file_on_writable_root_succeeds_and_emits() {
        let writable = TempRoot::new();
        let state = state_with_roots(vec![(writable.path.clone(), false)]);
        let harness = CommandHarness::new(state);

        create_file_impl(
            harness.state(),
            &harness.handle(),
            "new.txt".to_string(),
            Some("hello".to_string()),
        )
        .await
        .expect("writable root への作成は成功する");

        assert!(writable.path.join("new.txt").exists());
        assert_eq!(harness.created.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn command_create_file_rejects_read_only_root_without_side_effects() {
        let read_only = TempRoot::new();
        fs::create_dir_all(read_only.path.join("protected")).unwrap();
        fs::write(read_only.path.join("locked.txt"), b"original").unwrap();
        let writable = TempRoot::new();
        let state = state_with_roots(vec![
            (read_only.path.clone(), true),
            (writable.path.clone(), false),
        ]);
        let harness = CommandHarness::new(state);
        let before = snapshot(&read_only.path);

        assert_read_only_error(
            create_file_impl(
                harness.state(),
                &harness.handle(),
                "protected/injected.txt".to_string(),
                Some("bad".to_string()),
            )
            .await,
        );

        assert_eq!(
            snapshot(&read_only.path),
            before,
            "ファイル・ディレクトリが変化しないこと"
        );
        assert_eq!(
            fs::read(read_only.path.join("locked.txt")).unwrap(),
            b"original".to_vec()
        );
        assert_eq!(
            harness.created.load(Ordering::SeqCst),
            0,
            "emit されないこと"
        );
    }

    #[tokio::test]
    async fn command_delete_file_rejects_read_only_root_without_side_effects() {
        let read_only = TempRoot::new();
        fs::write(read_only.path.join("locked.txt"), b"original").unwrap();
        let state = state_with_roots(vec![(read_only.path.clone(), true)]);
        let harness = CommandHarness::new(state);
        let before = snapshot(&read_only.path);

        assert_read_only_error(
            delete_file_impl(
                harness.state(),
                &harness.handle(),
                "locked.txt".to_string(),
                None,
            )
            .await,
        );

        assert_eq!(snapshot(&read_only.path), before);
        assert_eq!(harness.deleted.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn command_rename_file_rejects_read_only_root_without_side_effects() {
        let read_only = TempRoot::new();
        fs::write(read_only.path.join("locked.txt"), b"original").unwrap();
        let state = state_with_roots(vec![(read_only.path.clone(), true)]);
        let harness = CommandHarness::new(state);
        let before = snapshot(&read_only.path);

        assert_read_only_error(
            rename_file_impl(
                harness.state(),
                &harness.handle(),
                "locked.txt".to_string(),
                "renamed.txt".to_string(),
                None,
            )
            .await,
        );

        assert_eq!(snapshot(&read_only.path), before);
        assert_eq!(harness.renamed.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn command_nested_read_only_child_root_rejects_create_and_rename() {
        let parent = TempRoot::new();
        let child = parent.path.join("child");
        fs::create_dir_all(&child).unwrap();
        fs::write(parent.path.join("movable.txt"), b"keep").unwrap();
        let state = state_with_roots(vec![(parent.path.clone(), false), (child.clone(), true)]);
        let harness = CommandHarness::new(state);
        let before = snapshot(&parent.path);

        // 親 root が writable でも、相対パスから子 read-only root への作成は拒否。
        assert_read_only_error(
            create_file_impl(
                harness.state(),
                &harness.handle(),
                "child/injected.txt".to_string(),
                Some("bad".to_string()),
            )
            .await,
        );
        // 絶対パスでも同じく拒否。
        let absolute = child.join("absolute.txt");
        assert_read_only_error(
            create_file_impl(
                harness.state(),
                &harness.handle(),
                absolute.to_string_lossy().into_owned(),
                Some("bad".to_string()),
            )
            .await,
        );

        // 移動元は親 root の絶対パスにして、移動先の子 root が拒否理由であることを確認する。
        let from = parent.path.join("movable.txt");
        assert_read_only_error(
            rename_file_impl(
                harness.state(),
                &harness.handle(),
                from.to_string_lossy().into_owned(),
                "child/moved.txt".to_string(),
                None,
            )
            .await,
        );

        assert_eq!(snapshot(&parent.path), before);
        assert_eq!(harness.created.load(Ordering::SeqCst), 0);
        assert_eq!(harness.renamed.load(Ordering::SeqCst), 0);
    }

    /// writable 親 root 内の symlink 経由で read-only 子 root を変更できないこと（R4 再レビュー high）。
    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn command_symlink_into_read_only_child_root_is_rejected() {
        use std::os::unix::fs::symlink;

        let parent = TempRoot::new();
        let child = parent.path.join("child");
        fs::create_dir_all(&child).unwrap();
        let target = child.join("target.txt");
        fs::write(&target, b"protected").unwrap();

        // 親 root 内に子 root（およびその実ファイル）を指す symlink を置く。
        let dir_link = parent.path.join("link");
        if symlink(&child, &dir_link).is_err() {
            return; // symlink を作成できない環境ではスキップ
        }
        let file_link = parent.path.join("link.txt");
        symlink(&target, &file_link).unwrap();

        let state = state_with_roots(vec![(parent.path.clone(), false), (child.clone(), true)]);
        let harness = CommandHarness::new(state);

        // symlink ディレクトリ経由の新規作成（絶対パス）を拒否。
        let injected = dir_link.join("injected.txt");
        assert_read_only_error(
            create_file_impl(
                harness.state(),
                &harness.handle(),
                injected.to_string_lossy().into_owned(),
                Some("bad".to_string()),
            )
            .await,
        );
        assert!(!target.parent().unwrap().join("injected.txt").exists());

        // symlink ファイル経由の削除で read-only の実ファイルが消えないこと。
        assert_read_only_error(
            delete_file_impl(
                harness.state(),
                &harness.handle(),
                file_link.to_string_lossy().into_owned(),
                None,
            )
            .await,
        );
        assert_eq!(fs::read(&target).unwrap(), b"protected".to_vec());

        // symlink ディレクトリ経由の移動も拒否。
        let from = parent.path.join("movable.txt");
        fs::write(&from, b"keep").unwrap();
        let before_rename = snapshot(&parent.path);
        let to = dir_link.join("renamed.txt");
        assert_read_only_error(
            rename_file_impl(
                harness.state(),
                &harness.handle(),
                from.to_string_lossy().into_owned(),
                to.to_string_lossy().into_owned(),
                None,
            )
            .await,
        );

        assert_eq!(snapshot(&parent.path), before_rename);
        assert_eq!(harness.created.load(Ordering::SeqCst), 0);
        assert_eq!(harness.deleted.load(Ordering::SeqCst), 0);
        assert_eq!(harness.renamed.load(Ordering::SeqCst), 0);
    }

    /// root 拒否が Core の固定文言をそのまま Tauri 文字列へ変換することを検証する。
    fn assert_root_operation_error(result: Result<impl std::fmt::Debug, String>) {
        let error = result.expect_err("root 自身への操作は拒否されること");
        assert_eq!(
            error,
            format!("Validation error: {STORAGE_ROOT_OPERATION_MESSAGE}")
        );
    }

    /// root 自身の削除が固定文言で拒否され、emit も副作用も起きないこと（R2-ER01）。
    #[tokio::test]
    async fn command_delete_file_rejects_storage_root_without_emit() {
        let root = TempRoot::new();
        fs::write(root.path.join("keep.txt"), b"keep").unwrap();
        let state = state_with_roots(vec![(root.path.clone(), false)]);
        let harness = CommandHarness::new(state);
        let before = snapshot(&root.path);

        let canonical = root.path.canonicalize().unwrap();
        for target in [PathBuf::from("."), canonical] {
            assert_root_operation_error(
                delete_file_impl(
                    harness.state(),
                    &harness.handle(),
                    target.to_string_lossy().into_owned(),
                    Some(true),
                )
                .await,
            );
        }

        assert!(root.path.exists(), "root 自身が消えていないこと");
        assert_eq!(snapshot(&root.path), before);
        assert_eq!(harness.deleted.load(Ordering::SeqCst), 0);
    }

    /// root 自身を移動元・移動先にした rename が拒否され、emit も副作用も起きないこと。
    #[tokio::test]
    async fn command_rename_file_rejects_storage_root_without_emit() {
        let root = TempRoot::new();
        fs::write(root.path.join("keep.txt"), b"keep").unwrap();
        let state = state_with_roots(vec![(root.path.clone(), false)]);
        let harness = CommandHarness::new(state);
        let before = snapshot(&root.path);
        let root_string = root
            .path
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .into_owned();

        assert_root_operation_error(
            rename_file_impl(
                harness.state(),
                &harness.handle(),
                root_string.clone(),
                "moved.txt".to_string(),
                None,
            )
            .await,
        );
        assert_root_operation_error(
            rename_file_impl(
                harness.state(),
                &harness.handle(),
                "keep.txt".to_string(),
                root_string,
                None,
            )
            .await,
        );
        assert_root_operation_error(
            rename_file_impl(
                harness.state(),
                &harness.handle(),
                ".".to_string(),
                "moved.txt".to_string(),
                None,
            )
            .await,
        );

        assert!(root.path.exists(), "root 自身が消えていないこと");
        assert_eq!(snapshot(&root.path), before);
        assert_eq!(harness.renamed.load(Ordering::SeqCst), 0);
    }

    /// overwrite=false の rename が既存宛先を保持し、emit しないこと。
    #[tokio::test]
    async fn command_rename_without_overwrite_preserves_destination_without_emit() {
        let root = TempRoot::new();
        fs::write(root.path.join("source.txt"), b"source").unwrap();
        fs::write(root.path.join("existing.txt"), b"original").unwrap();
        let state = state_with_roots(vec![(root.path.clone(), false)]);
        let harness = CommandHarness::new(state);
        let before = snapshot(&root.path);

        let error = rename_file_impl(
            harness.state(),
            &harness.handle(),
            "source.txt".to_string(),
            "existing.txt".to_string(),
            Some(false),
        )
        .await
        .expect_err("overwrite=false は既存宛先を拒否する");
        assert_eq!(
            error,
            format!("Validation error: {MOVE_DESTINATION_EXISTS_MESSAGE}")
        );

        assert_eq!(snapshot(&root.path), before);
        assert_eq!(harness.renamed.load(Ordering::SeqCst), 0);
    }
}
