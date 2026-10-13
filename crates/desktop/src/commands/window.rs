//! Window management commands.

use tauri::{command, Manager, WebviewWindow};

/// Minimize window
#[command]
pub fn minimize_window(window: WebviewWindow) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

/// Maximize window
#[command]
pub fn maximize_window(window: WebviewWindow) -> Result<(), String> {
    if window.is_maximized().unwrap_or(false) {
        window.unmaximize().map_err(|e| e.to_string())
    } else {
        window.maximize().map_err(|e| e.to_string())
    }
}

/// Close window
#[command]
pub fn close_window(window: WebviewWindow) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

/// Set window title
#[command]
pub fn set_title(window: WebviewWindow, title: String) -> Result<(), String> {
    window.set_title(&title).map_err(|e| e.to_string())
}

/// Toggle fullscreen
#[command]
pub fn toggle_fullscreen(window: WebviewWindow) -> Result<(), String> {
    let is_fullscreen = window.is_fullscreen().unwrap_or(false);
    window
        .set_fullscreen(!is_fullscreen)
        .map_err(|e| e.to_string())
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
