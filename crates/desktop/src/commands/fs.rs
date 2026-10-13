//! File system commands.

use tauri::{command, State, AppHandle};
use openrustbooks_core::{CoreResult, CoreError, io::{StorageManager, local::LocalStorage, ListOptions, ReadOptions, WriteOptions, FileEntry, StorageBackend, StorageCapabilities}};
use crate::state::AppState;
use std::sync::Arc;
use std::path::PathBuf;

/// Read directory contents
#[command]
pub async fn read_directory(
    state: State<'_, Arc<AppState>>,
    path: String,
    options: Option<ListOptions>,
) -> CoreResult<Vec<FileEntry>> {
    let config = state.config();
    let storage = StorageManager::new();

    // Add local roots from config
    for root in &config.storage.local_roots {
        let mut local = LocalStorage::new(root.path.clone());
        local.initialize().await?;
        storage.add_backend(Box::new(local));
    }

    // Use default backend
    let backend = storage.default_backend()
        .ok_or_else(|| CoreError::storage("No storage backend available"))?;

    let path_buf = PathBuf::from(path);
    backend.list(&path_buf, options.unwrap_or_default()).await
}

/// Create a new file
#[command]
pub async fn create_file(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    path: String,
    content: Option<String>,
) -> CoreResult<FileEntry> {
    let path_buf = PathBuf::from(&path);
    let config = state.config();
    let storage = StorageManager::new();

    for root in &config.storage.local_roots {
        let mut local = LocalStorage::new(root.path.clone());
        local.initialize().await?;
        storage.add_backend(Box::new(local));
    }

    let backend = storage.default_backend()
        .ok_or_else(|| CoreError::storage("No storage backend available"))?;

    // Create parent directories
    if let Some(parent) = path_buf.parent() {
        backend.create_dir(parent, true).await?;
    }

    // Write file
    let content = content.unwrap_or_default();
    backend.write_text(&path_buf, &content, WriteOptions::default()).await?;

    let _ = app.emit("file-created", &path);

    backend.metadata(&path_buf).await
}

/// Delete a file or directory
#[command]
pub async fn delete_file(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    path: String,
    recursive: Option<bool>,
) -> CoreResult<()> {
    let path_buf = PathBuf::from(&path);
    let config = state.config();
    let storage = StorageManager::new();

    for root in &config.storage.local_roots {
        let mut local = LocalStorage::new(root.path.clone());
        local.initialize().await?;
        storage.add_backend(Box::new(local));
    }

    let backend = storage.default_backend()
        .ok_or_else(|| CoreError::storage("No storage backend available"))?;

    backend.delete(&path_buf, recursive.unwrap_or(false)).await?;

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
) -> CoreResult<FileEntry> {
    let from_buf = PathBuf::from(&from);
    let to_buf = PathBuf::from(&to);
    let config = state.config();
    let storage = StorageManager::new();

    for root in &config.storage.local_roots {
        let mut local = LocalStorage::new(root.path.clone());
        local.initialize().await?;
        storage.add_backend(Box::new(local));
    }

    let backend = storage.default_backend()
        .ok_or_else(|| CoreError::storage("No storage backend available"))?;

    backend.move_item(&from_buf, &to_buf, overwrite.unwrap_or(false)).await?;

    let _ = app.emit("file-renamed", serde_json::json!({ "from": from, "to": to }));

    backend.metadata(&to_buf).await
}

/// Watch a directory for changes
#[command]
pub async fn watch_directory(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    path: String,
) -> CoreResult<()> {
    let path_buf = PathBuf::from(&path);
    let config = state.config();
    let storage = StorageManager::new();

    for root in &config.storage.local_roots {
        let mut local = LocalStorage::new(root.path.clone());
        local.initialize().await?;
        storage.add_backend(Box::new(local));
    }

    let backend = storage.default_backend()
        .ok_or_else(|| CoreError::storage("No storage backend available"))?;

    let mut stream = backend.watch(&path_buf).await?;

    // Spawn task to forward events
    tauri::async_runtime::spawn(async move {
        while let Some(event) = stream.next().await {
            let _ = app.emit("fs-event", event);
        }
    });

    Ok(())
}