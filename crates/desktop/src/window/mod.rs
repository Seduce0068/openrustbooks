//! Window setup and management.

use tauri::{Manager, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

/// Setup main window
pub fn setup_window(window: &WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    // Set window title
    window.set_title("openrustbooks")?;

    // Register global shortcuts
    let app = window.app_handle();
    app.global_shortcut()
        .on_shortcut("CmdOrCtrl+N", |app, _shortcut, _event| {
            let _ = app.emit("global-shortcut", "new-file");
        })?;

    app.global_shortcut()
        .on_shortcut("CmdOrCtrl+O", |app, _shortcut, _event| {
            let _ = app.emit("global-shortcut", "open-file");
        })?;

    app.global_shortcut()
        .on_shortcut("CmdOrCtrl+S", |app, _shortcut, _event| {
            let _ = app.emit("global-shortcut", "save-file");
        })?;

    app.global_shortcut()
        .on_shortcut("CmdOrCtrl+Shift+P", |app, _shortcut, _event| {
            let _ = app.emit("global-shortcut", "command-palette");
        })?;

    Ok(())
}

/// Create a new document window
pub fn create_document_window(
    app: &tauri::AppHandle,
    id: &str,
) -> Result<WebviewWindow, Box<dyn std::error::Error>> {
    let window = WebviewWindowBuilder::new(
        app,
        format!("document-{}", id),
        tauri::WebviewUrl::App(format!("/document/{}", id).into()),
    )
    .title("openrustbooks - Document")
    .inner_size(1000.0, 700.0)
    .min_inner_size(600.0, 400.0)
    .center()
    .build()?;

    Ok(window)
}

/// Create settings window
pub fn create_settings_window(
    app: &tauri::AppHandle,
) -> Result<WebviewWindow, Box<dyn std::error::Error>> {
    let window =
        WebviewWindowBuilder::new(app, "settings", tauri::WebviewUrl::App("/settings".into()))
            .title("openrustbooks - Settings")
            .inner_size(600.0, 500.0)
            .min_inner_size(500.0, 400.0)
            .center()
            .resizable(false)
            .build()?;

    Ok(window)
}

/// Create about window
pub fn create_about_window(
    app: &tauri::AppHandle,
) -> Result<WebviewWindow, Box<dyn std::error::Error>> {
    let window = WebviewWindowBuilder::new(app, "about", tauri::WebviewUrl::App("/about".into()))
        .title("About openrustbooks")
        .inner_size(400.0, 300.0)
        .resizable(false)
        .maximizable(false)
        .center()
        .build()?;

    Ok(window)
}
