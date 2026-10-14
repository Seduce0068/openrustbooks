//! Configuration commands.

use tauri::{command, State};
use openrustbooks_core::{CoreResult, AppConfig, AiConfig, LintConfig};
use crate::state::AppState;
use std::sync::Arc;

/// Get current configuration
#[command]
pub fn get_config(state: State<'_, Arc<AppState>>) -> AppConfig {
    state.config()
}

/// Update configuration
#[command]
pub fn update_config(
    state: State<'_, Arc<AppState>>,
    config: AppConfig,
) -> CoreResult<()> {
    state.update_config(config)?;
    state.save_config()?;
    Ok(())
}

/// Update AI configuration
#[command]
pub fn update_ai_config(
    state: State<'_, Arc<AppState>>,
    ai_config: AiConfig,
) -> CoreResult<()> {
    let mut config = state.config();
    config.ai = ai_config;
    state.update_config(config)?;
    state.save_config()?;

    // Reinitialize AI service if enabled
    if config.ai.enabled {
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
) -> CoreResult<()> {
    let mut config = state.config();
    config.lint = lint_config;
    state.update_config(config)?;
    state.save_config()?;
    Ok(())
}

/// Reset configuration to defaults
#[command]
pub fn reset_config(state: State<'_, Arc<AppState>>) -> CoreResult<AppConfig> {
    let config = AppConfig::default();
    state.update_config(config.clone())?;
    state.save_config()?;
    Ok(config)
}