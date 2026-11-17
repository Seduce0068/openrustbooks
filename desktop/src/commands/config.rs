//! Configuration commands.

use crate::state::AppState;
use openrustbooks_core::CoreError;
use openrustbooks_core::config::{
    AiConfig, AppConfig, EditorConfig, GeneralConfig, GoogleDriveConfig, LintConfig,
    LocalRootConfig, OllamaConfig, REDACTED_SECRET, ReaderConfig, StorageConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{State, command};

/// Convert CoreError to String for Tauri commands
fn map_err(e: CoreError) -> String {
    e.to_string()
}

/// Allow-list view of [`AppConfig`] returned to the JS side.
///
/// Secret fields are never serialized; only `has_*` presence flags are sent.
#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigView {
    pub general: GeneralConfig,
    pub ai: AiConfigView,
    pub lint: LintConfig,
    pub storage: StorageConfigView,
    pub editor: EditorConfig,
    pub reader: ReaderConfig,
}

impl From<&AppConfig> for ConfigView {
    fn from(config: &AppConfig) -> Self {
        Self {
            general: config.general.clone(),
            ai: AiConfigView::from(&config.ai),
            lint: config.lint.clone(),
            storage: StorageConfigView::from(&config.storage),
            editor: config.editor.clone(),
            reader: config.reader.clone(),
        }
    }
}

/// Allow-list view of [`AiConfig`].
#[derive(Clone, Serialize, Deserialize)]
pub struct AiConfigView {
    pub enabled: bool,
    pub default_provider: String,
    pub openai: OpenAiConfigView,
    pub ollama: OllamaConfig,
    pub max_tokens_per_request: usize,
    pub max_tokens_per_month: Option<u64>,
    pub prefer_local: bool,
    pub timeout_secs: u64,
    pub streaming: bool,
    pub model_overrides: HashMap<String, String>,
}

impl From<&AiConfig> for AiConfigView {
    fn from(ai: &AiConfig) -> Self {
        Self {
            enabled: ai.enabled,
            default_provider: ai.default_provider.clone(),
            openai: OpenAiConfigView::from(&ai.openai),
            ollama: ai.ollama.clone(),
            max_tokens_per_request: ai.max_tokens_per_request,
            max_tokens_per_month: ai.max_tokens_per_month,
            prefer_local: ai.prefer_local,
            timeout_secs: ai.timeout_secs,
            streaming: ai.streaming,
            model_overrides: ai.model_overrides.clone(),
        }
    }
}

/// Allow-list view of [`openrustbooks_core::config::OpenAiConfig`].
#[derive(Clone, Serialize, Deserialize)]
pub struct OpenAiConfigView {
    pub base_url: String,
    /// `true` when an API key is configured; the value itself is never sent.
    pub has_api_key: bool,
    pub model: String,
    pub organization: Option<String>,
    pub extra_headers: HashMap<String, String>,
}

impl From<&openrustbooks_core::config::OpenAiConfig> for OpenAiConfigView {
    fn from(openai: &openrustbooks_core::config::OpenAiConfig) -> Self {
        Self {
            base_url: openai.base_url.clone(),
            has_api_key: openai.api_key.as_deref().is_some_and(|key| !key.is_empty()),
            model: openai.model.clone(),
            organization: openai.organization.clone(),
            extra_headers: openai.extra_headers.clone(),
        }
    }
}

/// Allow-list view of [`StorageConfig`].
#[derive(Clone, Serialize, Deserialize)]
pub struct StorageConfigView {
    pub default_backend: String,
    pub local_roots: Vec<LocalRootConfig>,
    pub google_drive: Option<GoogleDriveConfigView>,
    pub recent_paths: Vec<PathBuf>,
    pub max_recent: usize,
}

impl From<&StorageConfig> for StorageConfigView {
    fn from(storage: &StorageConfig) -> Self {
        Self {
            default_backend: storage.default_backend.clone(),
            local_roots: storage.local_roots.clone(),
            google_drive: storage
                .google_drive
                .as_ref()
                .map(GoogleDriveConfigView::from),
            recent_paths: storage.recent_paths.clone(),
            max_recent: storage.max_recent,
        }
    }
}

/// Allow-list view of [`GoogleDriveConfig`].
#[derive(Clone, Serialize, Deserialize)]
pub struct GoogleDriveConfigView {
    pub client_id: String,
    /// `true` when a client secret is configured; the value is never sent.
    pub has_client_secret: bool,
    /// `true` when a refresh token is configured; the value is never sent.
    pub has_refresh_token: bool,
    pub root_folder_id: Option<String>,
    pub use_app_data_folder: bool,
    pub scopes: Vec<String>,
}

impl From<&GoogleDriveConfig> for GoogleDriveConfigView {
    fn from(drive: &GoogleDriveConfig) -> Self {
        Self {
            client_id: drive.client_id.clone(),
            has_client_secret: !drive.client_secret.is_empty(),
            has_refresh_token: drive
                .refresh_token
                .as_deref()
                .is_some_and(|token| !token.is_empty()),
            root_folder_id: drive.root_folder_id.clone(),
            use_app_data_folder: drive.use_app_data_folder,
            scopes: drive.scopes.clone(),
        }
    }
}

/// A secret coming back from JS is a placeholder when it is missing, empty or
/// the same mask sentinel used by [`ConfigView`] / `Debug` output.
fn is_placeholder(secret: Option<&str>) -> bool {
    secret.is_none_or(|value| value.is_empty() || value == REDACTED_SECRET)
}

/// Merge currently held secrets into an incoming config.
///
/// The DTO sent to JS contains presence flags instead of secret values, so an
/// update round-trip would otherwise erase every stored credential. Keeping
/// this a pure function makes the rule testable without an [`AppState`].
pub(crate) fn merge_secrets(current: &AppConfig, incoming: &AppConfig) -> AppConfig {
    let mut merged = incoming.clone();

    if is_placeholder(merged.ai.openai.api_key.as_deref()) {
        merged.ai.openai.api_key = current.ai.openai.api_key.clone();
    }

    if let (Some(current_drive), Some(incoming_drive)) = (
        current.storage.google_drive.as_ref(),
        merged.storage.google_drive.as_mut(),
    ) {
        if is_placeholder(Some(incoming_drive.client_secret.as_str())) {
            incoming_drive.client_secret = current_drive.client_secret.clone();
        }
        if is_placeholder(incoming_drive.refresh_token.as_deref()) {
            incoming_drive.refresh_token = current_drive.refresh_token.clone();
        }
    }

    merged
}

/// Get current configuration (secrets redacted)
#[command]
pub fn get_config(state: State<'_, Arc<AppState>>) -> ConfigView {
    ConfigView::from(&state.config())
}

/// Update configuration
#[command]
pub fn update_config(state: State<'_, Arc<AppState>>, config: AppConfig) -> Result<(), String> {
    let merged = merge_secrets(&state.config(), &config);
    let ai_enabled = merged.ai.enabled;
    state.update_config(merged).map_err(map_err)?;
    state.save_config().map_err(map_err)?;
    if !ai_enabled {
        // 無効化してもメモリ上の service が残らないようにする。
        state.disable_ai_service();
    }
    Ok(())
}

/// Update AI configuration
#[command]
pub fn update_ai_config(
    state: State<'_, Arc<AppState>>,
    ai_config: AiConfig,
) -> Result<(), String> {
    let current = state.config();
    let ai_enabled = ai_config.enabled;
    let mut incoming = current.clone();
    incoming.ai = ai_config;
    let config = merge_secrets(&current, &incoming);

    state.update_config(config).map_err(map_err)?;
    state.save_config().map_err(map_err)?;

    // 有効なら再初期化、無効なら既存 service を破棄する。
    if ai_enabled {
        let state_clone = state.inner().clone();
        tauri::async_runtime::spawn(async move {
            let _ = state_clone.init_ai_service().await;
        });
    } else {
        state.disable_ai_service();
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

/// Reset configuration to defaults (secrets redacted)
#[command]
pub fn reset_config(state: State<'_, Arc<AppState>>) -> Result<ConfigView, String> {
    let config = AppConfig::default();
    state.update_config(config.clone()).map_err(map_err)?;
    state.save_config().map_err(map_err)?;
    // 既定は AI 無効なので service も破棄する。
    state.disable_ai_service();
    Ok(ConfigView::from(&config))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with_secrets() -> AppConfig {
        let mut config = AppConfig::default();
        config.ai.openai.api_key = Some("sk-dummy-do-not-leak".to_string());
        config.storage.google_drive = Some(GoogleDriveConfig {
            client_id: "dummy-client-id".to_string(),
            client_secret: "dummy-client-secret".to_string(),
            refresh_token: Some("dummy-refresh-token".to_string()),
            root_folder_id: None,
            use_app_data_folder: true,
            scopes: vec!["scope".to_string()],
        });
        config
    }

    #[test]
    fn config_view_json_hides_secret_fields_and_values() {
        let view = ConfigView::from(&config_with_secrets());
        let json = serde_json::to_string(&view).unwrap();

        // Match JSON object keys, not substrings of the `has_*` flags.
        for forbidden in ["\"api_key\"", "\"client_secret\"", "\"refresh_token\""] {
            assert!(!json.contains(forbidden), "leaked field name: {forbidden}");
        }
        for secret in [
            "sk-dummy-do-not-leak",
            "dummy-client-secret",
            "dummy-refresh-token",
        ] {
            assert!(!json.contains(secret), "leaked secret value: {secret}");
        }

        assert!(json.contains("\"has_api_key\":true"));
        assert!(json.contains("\"has_client_secret\":true"));
        assert!(json.contains("\"has_refresh_token\":true"));
    }

    #[test]
    fn config_view_reports_unset_secrets_as_false() {
        let view = ConfigView::from(&AppConfig::default());
        let json = serde_json::to_string(&view).unwrap();

        assert!(json.contains("\"has_api_key\":false"));
        assert!(json.contains("\"google_drive\":null"));
    }

    #[test]
    fn merge_secrets_restores_placeholder_values() {
        let current = config_with_secrets();
        let mut incoming = current.clone();
        incoming.ai.openai.api_key = Some(REDACTED_SECRET.to_string());
        let drive = incoming.storage.google_drive.as_mut().unwrap();
        drive.client_secret.clear();
        drive.refresh_token = Some(String::new());

        let merged = merge_secrets(&current, &incoming);

        assert_eq!(
            merged.ai.openai.api_key.as_deref(),
            Some("sk-dummy-do-not-leak")
        );
        let drive = merged.storage.google_drive.unwrap();
        assert_eq!(drive.client_secret, "dummy-client-secret");
        assert_eq!(drive.refresh_token.as_deref(), Some("dummy-refresh-token"));
    }

    #[test]
    fn merge_secrets_keeps_explicitly_replaced_values() {
        let current = config_with_secrets();
        let mut incoming = current.clone();
        incoming.ai.openai.api_key = Some("sk-new-key".to_string());
        let drive = incoming.storage.google_drive.as_mut().unwrap();
        drive.client_secret = "new-client-secret".to_string();
        drive.refresh_token = Some("new-refresh-token".to_string());

        let merged = merge_secrets(&current, &incoming);

        assert_eq!(merged.ai.openai.api_key.as_deref(), Some("sk-new-key"));
        let drive = merged.storage.google_drive.unwrap();
        assert_eq!(drive.client_secret, "new-client-secret");
        assert_eq!(drive.refresh_token.as_deref(), Some("new-refresh-token"));
    }

    #[test]
    fn merge_secrets_does_not_revive_removed_drive_config() {
        let current = config_with_secrets();
        let mut incoming = current.clone();
        incoming.storage.google_drive = None;

        let merged = merge_secrets(&current, &incoming);

        assert!(merged.storage.google_drive.is_none());
    }
}
