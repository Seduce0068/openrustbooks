//! Window setup and management.

use tauri::{Emitter, Manager, WebviewWindow, WebviewWindowBuilder};

/// Setup main window
pub fn setup_window(window: &WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    // Set window title
    window.set_title("openrustbooks")?;

    // Note: Global shortcuts are registered via tauri_plugin_global_shortcut
    // The plugin handles shortcut registration automatically based on tauri.conf.json
    // or can be configured programmatically via the plugin API

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
    .build()?;

    // Center the window after creation
    if let Some(monitor) = window.current_monitor().ok().flatten() {
        let monitor_size = monitor.size();
        let window_size = window.outer_size().unwrap_or_default();
        let x = ((monitor_size.width as f64 - window_size.width as f64) / 2.0) as i32;
        let y = ((monitor_size.height as f64 - window_size.height as f64) / 2.0) as i32;
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
    }

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
            .resizable(false)
            .build()?;

    // Center the window
    if let Some(monitor) = window.current_monitor().ok().flatten() {
        let monitor_size = monitor.size();
        let window_size = window.outer_size().unwrap_or_default();
        let x = ((monitor_size.width as f64 - window_size.width as f64) / 2.0) as i32;
        let y = ((monitor_size.height as f64 - window_size.height as f64) / 2.0) as i32;
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
    }

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
        .build()?;

    // Center the window
    if let Some(monitor) = window.current_monitor().ok().flatten() {
        let monitor_size = monitor.size();
        let window_size = window.outer_size().unwrap_or_default();
        let x = ((monitor_size.width as f64 - window_size.width as f64) / 2.0) as i32;
        let y = ((monitor_size.height as f64 - window_size.height as f64) / 2.0) as i32;
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
    }

    Ok(window)
}
