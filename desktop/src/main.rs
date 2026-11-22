//! openrustbooks-desktop: Tauri desktop application entry point.

use openrustbooks_core::AppConfig;
use openrustbooks_core::config::{backup_failed_config, config_file_path};
use std::sync::Arc;
use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_log::{Target, TargetKind};

mod commands;
mod state;
mod window;

use state::AppState;
use window::setup_window;

/// デスクトップ / モバイル共通の plugin を登録する。
///
/// Tauri は登録順に plugin を初期化するため、`tauri_plugin_log` を最初に
/// 登録する（wdio plugin の setup は log の後に行う必要がある）。
fn with_common_plugins(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
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
        .plugin(tauri_plugin_dialog::init())
}

/// WebdriverIO 用 plugin は debug ビルドにのみ登録する。
#[cfg(all(debug_assertions, feature = "wdio"))]
fn with_wdio_plugins(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    // tauri_plugin_log の後に登録すること。逆順にすると log の setup が失敗する。
    builder
        .plugin(tauri_plugin_wdio::init())
        .plugin(tauri_plugin_wdio_webdriver::init())
}

#[cfg(not(all(debug_assertions, feature = "wdio")))]
fn with_wdio_plugins(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder
}

/// Build the Tauri application
fn build_app() -> tauri::Builder<tauri::Wry> {
    with_common_plugins(tauri::Builder::default())
}

/// 起動時の設定読み込み。
///
/// 失敗時は生エラー文字列と絶対パスを出さない（figment のエラーにはパース対象
/// TOML の内容＝シークレットが含まれ得る）。エラー種別（固定コード）と固定文言
/// のみを報告し、元 config.toml を退避して既定値を返す。第 2 戻り値は保存拒否に
/// 使う読み込み失敗状態（R2-ER01 / R4 high）。
fn load_config_on_startup() -> (AppConfig, bool) {
    match AppConfig::load() {
        Ok(config) => (config, false),
        Err(error) => {
            let report = format!(
                "設定ファイルを読み込めませんでした (error kind: {}); 既定値を使用し、保存は無効です",
                error.code()
            );
            log::error!("{report}");
            eprintln!("{report}");

            // パスは退避処理にだけ使い、ログには出さない。
            if let Some(path) = config_file_path() {
                match backup_failed_config(&path) {
                    Ok(Some(_)) => {
                        log::error!("退避ファイルを設定ディレクトリに作成しました");
                    }
                    Ok(None) => {
                        log::error!("退避する既存の設定ファイルがありませんでした");
                    }
                    Err(backup_error) => log::error!(
                        "設定ファイルの退避に失敗しました (error kind: {})",
                        backup_error.code()
                    ),
                }
            }
            (AppConfig::default(), true)
        }
    }
}

/// Application entry point
pub fn run() -> tauri::Result<()> {
    // `openrustbooks_core::init()` is intentionally not called here: on native
    // targets it installs a global `tracing_subscriber`, which conflicts with
    // the logger installed by `tauri_plugin_log` and causes
    // "attempted to set a logger after the logging system was already initialized".
    // The core `init()` remains in use on the wasm side; `tauri_plugin_log` is
    // the only logger for this desktop application.
    with_wdio_plugins(build_app())
        .setup(|app| {
            // Load configuration（読み込み失敗は報告・退避し、保存を拒否する）
            let (config, config_load_failed) = load_config_on_startup();

            // Initialize application state
            let state = if config_load_failed {
                AppState::new_with_load_failure(config)
            } else {
                AppState::new(config)
            };
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
            commands::config::set_openai_api_key,
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
                    // emit 失敗は黙って捨てず、イベント名だけの固定文言で警告する。
                    if window.emit("unsaved-changes", ()).is_err() {
                        log::warn!("unsaved-changes event failed");
                    }
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
