//! Window setup and management.

use tauri::WebviewWindow;

/// Setup main window
pub fn setup_window(window: &WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    // Set window title
    window.set_title("openrustbooks")?;

    // Note: global keyboard shortcuts are not implemented. No shortcut plugin is
    // registered in the application builder, and this setup only sets the title.

    Ok(())
}
