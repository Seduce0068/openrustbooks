//! File system commands.

use crate::state::AppState;
use openrustbooks_core::{
    CoreError,
    io::{
        FileEntry, ListOptions, StorageBackend, StorageManager, WriteOptions, local::LocalStorage,
    },
};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State, command};

/// Convert CoreError to String for Tauri commands
fn map_err(e: CoreError) -> String {
    e.to_string()
}

/// Helper to create storage manager with configured local roots
async fn create_storage(config: &crate::state::AppState) -> Result<StorageManager, String> {
    let mut storage = StorageManager::new();
    let app_config = config.config();

    for root in &app_config.storage.local_roots {
        let mut local = LocalStorage::new(root.path.clone());
        local.initialize().await.map_err(map_err)?;
        storage.add_backend(Box::new(local));
    }

    Ok(storage)
}

/// Read directory contents
#[command]
pub async fn read_directory(
    state: State<'_, Arc<AppState>>,
    path: String,
    options: Option<ListOptions>,
) -> Result<Vec<FileEntry>, String> {
    let storage = create_storage(&state).await?;
    let backend = storage
        .default_backend()
        .ok_or_else(|| CoreError::storage("No storage backend available").to_string())?;

    let path_buf = PathBuf::from(path);
    backend
        .list(&path_buf, options.unwrap_or_default())
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
    let path_buf = PathBuf::from(&path);
    let storage = create_storage(&state).await?;

    let backend = storage
        .default_backend()
        .ok_or_else(|| CoreError::storage("No storage backend available").to_string())?;

    // Create parent directories
    if let Some(parent) = path_buf.parent() {
        let parent_buf = parent.to_path_buf();
        backend
            .create_dir(&parent_buf, true)
            .await
            .map_err(map_err)?;
    }

    // Write file
    let content = content.unwrap_or_default();
    backend
        .write_text(&path_buf, &content, WriteOptions::default())
        .await
        .map_err(map_err)?;

    let _ = app.emit("file-created", &path);

    backend.metadata(&path_buf).await.map_err(map_err)
}

/// Delete a file or directory
#[command]
pub async fn delete_file(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    path: String,
    recursive: Option<bool>,
) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    let storage = create_storage(&state).await?;

    let backend = storage
        .default_backend()
        .ok_or_else(|| CoreError::storage("No storage backend available").to_string())?;

    backend
        .delete(&path_buf, recursive.unwrap_or(false))
        .await
        .map_err(map_err)?;

    let _ = app.emit("file-deleted", &path);

    Ok(())
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
    let from_buf = PathBuf::from(&from);
    let to_buf = PathBuf::from(&to);
    let storage = create_storage(&state).await?;

    let backend = storage
        .default_backend()
        .ok_or_else(|| CoreError::storage("No storage backend available").to_string())?;

    backend
        .move_item(&from_buf, &to_buf, overwrite.unwrap_or(false))
        .await
        .map_err(map_err)?;

    let _ = app.emit(
        "file-renamed",
        serde_json::json!({ "from": from, "to": to }),
    );

    backend.metadata(&to_buf).await.map_err(map_err)
}

/// Watch a directory for changes
#[command]
pub async fn watch_directory(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    path: String,
) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    let storage = create_storage(&state).await?;

    let backend = storage
        .default_backend()
        .ok_or_else(|| CoreError::storage("No storage backend available").to_string())?;

    let mut stream = backend.watch(&path_buf).await.map_err(map_err)?;

    // Spawn task to forward events
    tauri::async_runtime::spawn(async move {
        while let Some(event) = stream.next().await {
            let _ = app.emit("fs-event", event);
        }
    });

    Ok(())
}
