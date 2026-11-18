//! Document management commands.

use crate::state::AppState;
use openrustbooks_core::{
    CoreError, CoreResult,
    config::LocalRootConfig,
    document::{
        Document, DocumentContent, DocumentFormat, DocumentId, DocumentMetadata,
        extract_front_matter,
    },
};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
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
        .map_err(|e| e.to_string())?
        .path;

    // Read file content
    let content = tokio::fs::read_to_string(&path_buf)
        .await
        .map_err(|e| CoreError::Io(e).to_string())?;

    // Detect format
    let format =
        DocumentFormat::from_extension(path_buf.extension().and_then(|s| s.to_str()).unwrap_or(""))
            .unwrap_or(DocumentFormat::Txt);

    // Create document
    let id = DocumentId::from_string(path.clone());
    let metadata = DocumentMetadata {
        id: id.clone(),
        format,
        title: path_buf
            .file_stem()
            .and_then(|s| s.to_str())
            .map(String::from),
        author: None,
        created_at: None,
        modified_at: None,
        size_bytes: content.len() as u64,
        path: Some(path_buf.clone()),
        url: None,
        checksum: None,
        extra: Default::default(),
    };

    let document = Document {
        metadata,
        content: match format {
            DocumentFormat::Markdown => {
                let (fm, body) = extract_front_matter(&content);
                DocumentContent::Markdown {
                    content: body,
                    front_matter: fm,
                }
            }
            _ => DocumentContent::Text(content),
        },
    };

    state.add_document(document.clone());
    state.mark_saved(&id);

    // Emit event to frontend
    let _ = app.emit("document-opened", &document);

    Ok(document)
}

/// Save a document
#[command]
pub async fn save_document(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    id: String,
    content: Option<String>,
) -> Result<(), String> {
    let doc_id = DocumentId::from_string(id);
    let mut document = state
        .get_document(&doc_id)
        .ok_or_else(|| CoreError::DocumentNotFound(doc_id.0.to_string()).to_string())?;

    // Update content if provided
    if let Some(new_content) = content
        && let Some(text) = document.as_text_mut()
    {
        *text = new_content;
    }

    // Write to file if path exists
    if let Some(path) = &document.metadata.path {
        let config = state.config();
        // 設定済み root 配下のみ書き込みを許可する
        let resolved =
            resolve_within_roots(&config.storage.local_roots, path).map_err(|e| e.to_string())?;
        if resolved.read_only {
            return Err(CoreError::validation("Storage root is read-only").to_string());
        }
        let text = document
            .full_text()
            .ok_or_else(|| CoreError::validation("Document is not text-editable").to_string())?;

        tokio::fs::write(&resolved.path, &text)
            .await
            .map_err(|e| CoreError::Io(e).to_string())?;

        // Update metadata
        document.metadata.size_bytes = text.len() as u64;
        document.metadata.modified_at = Some(chrono::Utc::now());
    }

    state.add_document(document.clone());
    state.mark_saved(&doc_id);

    let _ = app.emit("document-saved", &document);

    Ok(())
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
        return Err(CoreError::validation("Document has unsaved changes").to_string());
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
        .ok_or_else(|| CoreError::DocumentNotFound(doc_id.0.to_string()).to_string())?;

    document
        .as_text()
        .map(String::from)
        .ok_or_else(|| CoreError::validation("Document is not text-editable").to_string())
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
        .ok_or_else(|| CoreError::DocumentNotFound(doc_id.0.to_string()).to_string())?;

    if let Some(text) = document.as_text_mut() {
        *text = content;
        state.add_document(document.clone());
        state.mark_unsaved(&doc_id);
        let _ = app.emit("document-changed", &document);
        Ok(())
    } else {
        Err(CoreError::validation("Document is not text-editable").to_string())
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
}
