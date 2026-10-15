//! Configuration commands.

use crate::state::AppState;
use openrustbooks_core::{AiConfig, AppConfig, CoreError, LintConfig};
use std::sync::Arc;
use tauri::{State, command};

/// Convert CoreError to String for Tauri commands
fn map_err(e: CoreError) -> String {
    e.to_string()
}

/// Get current configuration
#[command]
pub fn get_config(state: State<'_, Arc<AppState>>) -> AppConfig {
    state.config()
}

/// Update configuration
#[command]
pub fn update_config(state: State<'_, Arc<AppState>>, config: AppConfig) -> Result<(), String> {
    state.update_config(config).map_err(map_err)?;
    state.save_config().map_err(map_err)?;
    Ok(())
}

/// Update AI configuration
#[command]
pub fn update_ai_config(
    state: State<'_, Arc<AppState>>,
    ai_config: AiConfig,
) -> Result<(), String> {
    let mut config = state.config();
    let ai_enabled = ai_config.enabled; // Check before moving
    config.ai = ai_config;
    state.update_config(config).map_err(map_err)?;
    state.save_config().map_err(map_err)?;

    // Reinitialize AI service if enabled
    if ai_enabled {
        let state_clone = state.inner().clone();
        tauri::async_runtime::spawn(async move {
            let _ = state_clone.init_ai_service().await;
        });
    }

    Ok(())
}

/// Update lint configuration
#[command]
pub fn update_lint_config(
    state: State<'_, Arc<AppState>>,
    lint_config: LintConfig,
) -> Result<(), String> {
    let mut config = state.config();
    config.lint = lint_config;
    state.update_config(config).map_err(map_err)?;
    state.save_config().map_err(map_err)?;
    Ok(())
}

/// Reset configuration to defaults
#[command]
pub fn reset_config(state: State<'_, Arc<AppState>>) -> Result<AppConfig, String> {
    let config = AppConfig::default();
    state.update_config(config.clone()).map_err(map_err)?;
    state.save_config().map_err(map_err)?;
    Ok(config)
}
