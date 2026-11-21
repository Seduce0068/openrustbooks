//! Window management commands.

use crate::commands::ipc_message;
use tauri::{WebviewWindow, command};

/// Close window
#[command]
pub fn close_window(window: WebviewWindow) -> Result<(), String> {
    // tauri::Error の Display は内部情報を含み得るため IPC へ出さない。
    window
        .close()
        .map_err(|_| ipc_message("WINDOW_INTERNAL").to_string())
}

/// Set window title
#[command]
pub fn set_title(window: WebviewWindow, title: String) -> Result<(), String> {
    window
        .set_title(&title)
        .map_err(|_| ipc_message("WINDOW_INTERNAL").to_string())
}

/// Get window state
#[command]
pub fn get_window_state(window: WebviewWindow) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "maximized": window.is_maximized().unwrap_or(false),
        "minimized": window.is_minimized().unwrap_or(false),
        "fullscreen": window.is_fullscreen().unwrap_or(false),
        "visible": window.is_visible().unwrap_or(false),
        "width": window.inner_size().map(|s| s.width).unwrap_or(0),
        "height": window.inner_size().map(|s| s.height).unwrap_or(0),
    }))
}
