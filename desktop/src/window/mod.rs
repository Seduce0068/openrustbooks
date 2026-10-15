//! Window setup and management.

use tauri::WebviewWindow;

/// Setup main window
pub fn setup_window(window: &WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    // Set window title
    window.set_title("openrustbooks")?;

    // Note: Global shortcuts are registered via tauri_plugin_global_shortcut
    // The plugin handles shortcut registration automatically based on tauri.conf.json
    // or can be configured programmatically via the plugin API

    Ok(())
}
