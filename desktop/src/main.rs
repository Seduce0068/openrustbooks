//! openrustbooks-desktop: Tauri desktop application entry point.

use openrustbooks_core::{AppConfig, init};
use std::sync::Arc;
use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_log::{Target, TargetKind};

mod commands;
mod state;
mod window;

use state::AppState;
use window::setup_window;

/// Apply the plugins shared by desktop and mobile builds.
///
/// `platform_plugins` is applied at the boundary between `store` and
/// `notification`, preserving the original registration position of any
/// platform-specific plugin. Tauri keeps plugins in a `Vec` and initializes
/// them in registration order, so this position is intentional.
fn with_common_plugins(
    builder: tauri::Builder<tauri::Wry>,
    platform_plugins: impl FnOnce(tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry>,
) -> tauri::Builder<tauri::Wry> {
    platform_plugins(
        builder
            .plugin(
                tauri_plugin_log::Builder::new()
                    .targets([
                        Target::new(TargetKind::Stdout),
                        Target::new(TargetKind::LogDir {
                            file_name: Some("openrustbooks".to_string()),
                        }),
                        Target::new(TargetKind::Webview),
                    ])
                    .level(log::LevelFilter::Info)
                    .build(),
            )
            .plugin(tauri_plugin_opener::init())
            .plugin(tauri_plugin_fs::init())
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_shell::init())
            .plugin(tauri_plugin_store::Builder::new().build()),
    )
    .plugin(tauri_plugin_notification::init())
    .plugin(tauri_plugin_updater::Builder::new().build())
}

/// Build the Tauri application with platform-specific plugins
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn build_app() -> tauri::Builder<tauri::Wry> {
    with_common_plugins(tauri::Builder::default(), |builder| {
        builder.plugin(tauri_plugin_global_shortcut::Builder::new().build())
    })
}

/// Build the Tauri application for mobile platforms (without global shortcut)
#[cfg(any(target_os = "android", target_os = "ios"))]
fn build_app() -> tauri::Builder<tauri::Wry> {
    with_common_plugins(tauri::Builder::default(), |builder| builder)
}

/// Application entry point
pub fn run() -> tauri::Result<()> {
    init();

    build_app()
        .setup(|app| {
            // Load configuration
            let config = AppConfig::load().unwrap_or_default();

            // Initialize application state
            let state = AppState::new(config);
            app.manage(Arc::new(state));

            // Setup main window
            let window = app.get_webview_window("main").ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "main window not found")
            })?;
            setup_window(&window)?;

            // Initialize AI service if enabled
            let state = app.state::<Arc<AppState>>();
            if state.is_ai_enabled() {
                let state_clone = state.inner().clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = state_clone.init_ai_service().await {
                        log::error!("Failed to initialize AI service: {}", e);
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Document commands
            commands::document::open_document,
            commands::document::save_document,
            commands::document::close_document,
            commands::document::list_documents,
            commands::document::get_document_content,
            commands::document::set_document_content,
            // File system commands
            commands::fs::read_directory,
            commands::fs::create_file,
            commands::fs::delete_file,
            commands::fs::rename_file,
            // `watch_directory` は watch 未実装のため登録しない（UI から到達不能）。
            // AI Markdown commands
            commands::ai::generate_markdown,
            commands::ai::refine_markdown,
            commands::ai::convert_to_table,
            commands::ai::generate_front_matter,
            commands::ai::lint_markdown,
            commands::ai::lint_and_fix_markdown,
            // Configuration commands
            commands::config::get_config,
            commands::config::update_config,
            commands::config::update_config_patch,
            commands::config::update_ai_config,
            commands::config::update_lint_config,
            commands::config::reset_config,
            commands::config::clear_secrets,
            // Window commands
            commands::window::close_window,
            commands::window::set_title,
            commands::window::get_window_state,
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<Arc<AppState>>();
                if state.has_unsaved_changes() {
                    api.prevent_close();
                    let _ = window.emit("unsaved-changes", ());
                }
            }
        })
        .run(tauri::generate_context!())
}

/// Tauri mobile entry point (for iOS/Android)
#[cfg(mobile)]
#[tauri::mobile_entry_point]
fn main() {
    if let Err(error) = run() {
        log::error!("error while running tauri application: {error}");
    }
}

#[cfg(not(mobile))]
fn main() {
    if let Err(error) = run() {
        eprintln!("error while running tauri application: {error}");
        std::process::exit(101);
    }
}
