//! Configuration commands.

use crate::state::AppState;
use openrustbooks_core::CoreError;
use openrustbooks_core::ai_markdown::client::{ProviderKind, validate_base_url};
use openrustbooks_core::config::{
    AiConfig, AppConfig, EditorConfig, GeneralConfig, GoogleDriveConfig, LintConfig,
    LocalRootConfig, OllamaConfig, REDACTED_SECRET, ReaderConfig, StorageConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{State, command};

/// Convert CoreError to String for Tauri commands
fn map_err(e: CoreError) -> String {
    e.to_string()
}

/// Allow-list view of [`AppConfig`] returned to the JS side.
///
/// Secret fields are never serialized; only `has_*` presence flags are sent.
/// パスを含むフィールド（data_dir / cache_dir / config_file / rule_config /
/// local_roots[].path / recent_paths）も DTO から除外して JS への開示を防ぐ。
#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigView {
    pub general: GeneralConfigView,
    pub ai: AiConfigView,
    pub lint: LintConfigView,
    pub storage: StorageConfigView,
    pub editor: EditorConfig,
    pub reader: ReaderConfig,
}

impl From<&AppConfig> for ConfigView {
    fn from(config: &AppConfig) -> Self {
        Self {
            general: GeneralConfigView::from(&config.general),
            ai: AiConfigView::from(&config.ai),
            lint: LintConfigView::from(&config.lint),
            storage: StorageConfigView::from(&config.storage),
            editor: config.editor.clone(),
            reader: config.reader.clone(),
        }
    }
}

/// Allow-list view of [`GeneralConfig`]. `data_dir` / `cache_dir` は除外する。
#[derive(Clone, Serialize, Deserialize)]
pub struct GeneralConfigView {
    pub language: String,
    pub theme: String,
    pub telemetry: bool,
    pub check_updates: bool,
    pub log_level: String,
}

impl From<&GeneralConfig> for GeneralConfigView {
    fn from(general: &GeneralConfig) -> Self {
        Self {
            language: general.language.clone(),
            theme: general.theme.clone(),
            telemetry: general.telemetry,
            check_updates: general.check_updates,
            log_level: general.log_level.clone(),
        }
    }
}

/// Allow-list view of [`AiConfig`]. `model_overrides` は除外する。
#[derive(Clone, Serialize, Deserialize)]
pub struct AiConfigView {
    pub enabled: bool,
    pub default_provider: String,
    pub openai: OpenAiConfigView,
    pub ollama: OllamaConfigView,
    pub max_tokens_per_request: usize,
    pub max_tokens_per_month: Option<u64>,
    pub prefer_local: bool,
    pub timeout_secs: u64,
    pub streaming: bool,
}

impl From<&AiConfig> for AiConfigView {
    fn from(ai: &AiConfig) -> Self {
        Self {
            enabled: ai.enabled,
            default_provider: ai.default_provider.clone(),
            openai: OpenAiConfigView::from(&ai.openai),
            ollama: OllamaConfigView::from(&ai.ollama),
            max_tokens_per_request: ai.max_tokens_per_request,
            max_tokens_per_month: ai.max_tokens_per_month,
            prefer_local: ai.prefer_local,
            timeout_secs: ai.timeout_secs,
            streaming: ai.streaming,
        }
    }
}

/// Allow-list view of [`OllamaConfig`]. `options` は除外する。
#[derive(Clone, Serialize, Deserialize)]
pub struct OllamaConfigView {
    pub base_url: String,
    pub model: String,
    pub keep_alive_minutes: u32,
}

impl From<&OllamaConfig> for OllamaConfigView {
    fn from(ollama: &OllamaConfig) -> Self {
        Self {
            base_url: ollama.base_url.clone(),
            model: ollama.model.clone(),
            keep_alive_minutes: ollama.keep_alive_minutes,
        }
    }
}

/// Allow-list view of [`openrustbooks_core::config::OpenAiConfig`].
///
/// `extra_headers` は認証情報を含み得るため DTO に含めない（編集は config.toml 直接のみ）。
#[derive(Clone, Serialize, Deserialize)]
pub struct OpenAiConfigView {
    pub base_url: String,
    /// `true` when an API key is configured; the value itself is never sent.
    pub has_api_key: bool,
    pub model: String,
    pub organization: Option<String>,
}

impl From<&openrustbooks_core::config::OpenAiConfig> for OpenAiConfigView {
    fn from(openai: &openrustbooks_core::config::OpenAiConfig) -> Self {
        Self {
            base_url: openai.base_url.clone(),
            has_api_key: openai.api_key.as_deref().is_some_and(|key| !key.is_empty()),
            model: openai.model.clone(),
            organization: openai.organization.clone(),
        }
    }
}

/// Allow-list view of [`LintConfig`]. `config_file` / `rule_config` は除外する。
#[derive(Clone, Serialize, Deserialize)]
pub struct LintConfigView {
    pub enabled: bool,
    pub enable_rules: Vec<String>,
    pub disable_rules: Vec<String>,
    pub auto_fix: bool,
    pub warnings_as_errors: bool,
}

impl From<&LintConfig> for LintConfigView {
    fn from(lint: &LintConfig) -> Self {
        Self {
            enabled: lint.enabled,
            enable_rules: lint.enable_rules.clone(),
            disable_rules: lint.disable_rules.clone(),
            auto_fix: lint.auto_fix,
            warnings_as_errors: lint.warnings_as_errors,
        }
    }
}

/// Allow-list view of [`StorageConfig`].
/// `local_roots[].path` と `recent_paths` は JS に開示しない。
#[derive(Clone, Serialize, Deserialize)]
pub struct StorageConfigView {
    pub default_backend: String,
    pub local_roots: Vec<LocalRootView>,
    pub google_drive: Option<GoogleDriveConfigView>,
    pub max_recent: usize,
}

impl From<&StorageConfig> for StorageConfigView {
    fn from(storage: &StorageConfig) -> Self {
        Self {
            default_backend: storage.default_backend.clone(),
            local_roots: storage
                .local_roots
                .iter()
                .map(LocalRootView::from)
                .collect(),
            google_drive: storage
                .google_drive
                .as_ref()
                .map(GoogleDriveConfigView::from),
            max_recent: storage.max_recent,
        }
    }
}

/// Allow-list view of [`LocalRootConfig`]. `path` は除外する。
#[derive(Clone, Serialize, Deserialize)]
pub struct LocalRootView {
    pub id: String,
    pub name: String,
    pub read_only: bool,
    pub watch: bool,
}

impl From<&LocalRootConfig> for LocalRootView {
    fn from(root: &LocalRootConfig) -> Self {
        Self {
            id: root.id.clone(),
            name: root.name.clone(),
            read_only: root.read_only,
            watch: root.watch,
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

/// 追加ヘッダが空またはマスク値なら「未変更」とみなす。
fn are_headers_placeholder(headers: &HashMap<String, String>) -> bool {
    headers.is_empty() || headers.values().any(|value| value == REDACTED_SECRET)
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

    // headers は DTO から除外したため、GUI の往復で空／マスク値が来ても現在値を保持する。
    // 削除する手段は config.toml の直接編集に限定する。
    if are_headers_placeholder(&merged.ai.openai.extra_headers) {
        merged.ai.openai.extra_headers = current.ai.openai.extra_headers.clone();
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

    // 以下は DTO から除外したフィールド。JS 往復で消えないよう現在値を保持する。
    // 値は JS に開示されないため、incoming の値に関わらず current を採用する。
    merged.general.data_dir = current.general.data_dir.clone();
    merged.general.cache_dir = current.general.cache_dir.clone();
    merged.lint.config_file = current.lint.config_file.clone();
    merged.lint.rule_config = current.lint.rule_config.clone();
    merged.storage.recent_paths = current.storage.recent_paths.clone();
    merged.ai.ollama.options = current.ai.ollama.options.clone();
    merged.ai.model_overrides = current.ai.model_overrides.clone();

    // local_roots[].path は id が一意に一致した場合のみ復元する。
    // id が空／不一致／現在側で重複している場合はマージせず安全にスキップする。
    for root in &mut merged.storage.local_roots {
        if root.id.is_empty() {
            continue;
        }
        let mut matches = current
            .storage
            .local_roots
            .iter()
            .filter(|current_root| current_root.id == root.id);
        let Some(current_root) = matches.next() else {
            continue; // 不一致の場合はマージしない
        };
        if matches.next().is_some() {
            continue; // 現在側で id が重複している場合も安全のためスキップ
        }
        root.path = current_root.path.clone();
    }

    merged
}

/// 保存・再初期化前に両プロバイダの base_url を検証する。
/// JS は AppConfig 全体を渡せるため、ホストの許可リストは core 側にハードコードする。
fn validate_ai_base_urls(ai: &AiConfig) -> Result<(), String> {
    validate_base_url(&ai.openai.base_url, ProviderKind::OpenAi).map_err(map_err)?;
    validate_base_url(&ai.ollama.base_url, ProviderKind::Ollama).map_err(map_err)?;
    Ok(())
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
    validate_ai_base_urls(&merged.ai)?;
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
    validate_ai_base_urls(&config.ai)?;

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
    let current = state.config();
    let mut incoming = current.clone();
    incoming.lint = lint_config;
    // DTO から除外した config_file / rule_config を消さないようマージを通す。
    let config = merge_secrets(&current, &incoming);
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
    use std::path::PathBuf;

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
    fn config_view_json_hides_extra_headers() {
        let mut config = config_with_secrets();
        config.ai.openai.extra_headers.insert(
            "X-Dummy-Header".to_string(),
            "dummy-header-value-do-not-leak".to_string(),
        );
        let json = serde_json::to_string(&ConfigView::from(&config)).unwrap();

        // DTO にフィールドも値も出ないこと。
        assert!(!json.contains("\"extra_headers\""), "leaked field name");
        assert!(
            !json.contains("dummy-header-value-do-not-leak"),
            "leaked header value"
        );
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
    fn merge_secrets_preserves_extra_headers() {
        let mut current = config_with_secrets();
        current.ai.openai.extra_headers.insert(
            "X-Dummy-Header".to_string(),
            "dummy-header-value-do-not-leak".to_string(),
        );

        // GUI の往復では headers が空になるため、現在値が消えてはいけない。
        let empty_incoming = AppConfig::default();
        let merged = merge_secrets(&current, &empty_incoming);
        assert_eq!(
            merged.ai.openai.extra_headers,
            current.ai.openai.extra_headers
        );

        // マスク sentinel が返ってきた場合も実値を維持する。
        let mut masked = current.clone();
        masked
            .ai
            .openai
            .extra_headers
            .insert("X-Dummy-Header".to_string(), REDACTED_SECRET.to_string());
        let merged = merge_secrets(&current, &masked);
        assert_eq!(
            merged.ai.openai.extra_headers,
            current.ai.openai.extra_headers
        );
    }

    #[test]
    fn merge_secrets_replaces_unmasked_extra_headers() {
        let mut current = config_with_secrets();
        current
            .ai
            .openai
            .extra_headers
            .insert("X-Old".to_string(), "old-value".to_string());
        let mut incoming = current.clone();
        incoming.ai.openai.extra_headers.clear();
        incoming
            .ai
            .openai
            .extra_headers
            .insert("X-New".to_string(), "new-value".to_string());

        let merged = merge_secrets(&current, &incoming);

        assert_eq!(
            merged.ai.openai.extra_headers,
            incoming.ai.openai.extra_headers
        );
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

    /// JSON オブジェクトのキーを再帰的に集める（値の部分文字列との誤検知を避ける）。
    fn collect_json_keys(value: &serde_json::Value, out: &mut Vec<String>) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, child) in map {
                    out.push(key.clone());
                    collect_json_keys(child, out);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    collect_json_keys(item, out);
                }
            }
            _ => {}
        }
    }

    fn config_with_paths() -> AppConfig {
        let mut config = config_with_secrets();
        config.general.data_dir = Some(PathBuf::from("/secret/data-dir"));
        config.general.cache_dir = Some(PathBuf::from("/secret/cache-dir"));
        config.lint.config_file = Some(PathBuf::from("/secret/lint.toml"));
        config.lint.rule_config.insert(
            "secret-rule".to_string(),
            serde_json::json!({"token": "rule-secret-value"}),
        );
        config.storage.recent_paths = vec![PathBuf::from("/secret/recent/book.epub")];
        config.storage.local_roots = vec![
            LocalRootConfig {
                id: "root-a".to_string(),
                name: "A".to_string(),
                path: PathBuf::from("/secret/root-a"),
                read_only: false,
                watch: true,
            },
            LocalRootConfig {
                id: "root-b".to_string(),
                name: "B".to_string(),
                path: PathBuf::from("/secret/root-b"),
                read_only: true,
                watch: false,
            },
        ];
        config.ai.ollama.options.insert(
            "secret-opt".to_string(),
            serde_json::json!("ollama-secret-value"),
        );
        config.ai.model_overrides.insert(
            "secret-model".to_string(),
            "override-secret-value".to_string(),
        );
        config
    }

    #[test]
    fn config_view_json_hides_path_bearing_fields() {
        let json = serde_json::to_string(&ConfigView::from(&config_with_paths())).unwrap();

        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let mut keys = Vec::new();
        collect_json_keys(&value, &mut keys);
        for forbidden in [
            "data_dir",
            "cache_dir",
            "config_file",
            "rule_config",
            "recent_paths",
            "path",
            "options",
            "model_overrides",
        ] {
            assert!(
                !keys.iter().any(|key| key == forbidden),
                "leaked field name as key: {forbidden}"
            );
            assert!(
                !json.contains(&format!("\"{forbidden}\"")),
                "leaked field name in JSON: {forbidden}"
            );
        }

        for secret in [
            "/secret/data-dir",
            "/secret/cache-dir",
            "/secret/lint.toml",
            "/secret/recent/book.epub",
            "/secret/root-a",
            "/secret/root-b",
            "rule-secret-value",
            "ollama-secret-value",
            "override-secret-value",
        ] {
            assert!(!json.contains(secret), "leaked value: {secret}");
        }
    }

    #[test]
    fn config_view_keeps_non_path_fields() {
        let json = serde_json::to_string(&ConfigView::from(&config_with_paths())).unwrap();

        for kept in [
            "\"language\"",
            "\"theme\"",
            "\"telemetry\"",
            "\"check_updates\"",
            "\"log_level\"",
            "\"enable_rules\"",
            "\"disable_rules\"",
            "\"auto_fix\"",
            "\"warnings_as_errors\"",
            "\"default_backend\"",
            "\"max_recent\"",
            "\"keep_alive_minutes\"",
            "\"id\"",
            "\"name\"",
            "\"read_only\"",
            "\"watch\"",
            "\"model\"",
        ] {
            assert!(json.contains(kept), "missing field: {kept}");
        }
    }

    #[test]
    fn merge_secrets_preserves_hidden_fields() {
        let current = config_with_paths();
        // DTO 往復では除外フィールドが既定値／マスク値で返ってくる想定。
        let mut incoming = AppConfig::default();
        incoming.storage.google_drive = Some(GoogleDriveConfig {
            client_id: "dummy-client-id".to_string(),
            client_secret: String::new(),
            refresh_token: Some(REDACTED_SECRET.to_string()),
            root_folder_id: None,
            use_app_data_folder: true,
            scopes: vec![],
        });
        let merged = merge_secrets(&current, &incoming);

        assert_eq!(merged.general.data_dir, current.general.data_dir);
        assert_eq!(merged.general.cache_dir, current.general.cache_dir);
        assert_eq!(merged.lint.config_file, current.lint.config_file);
        assert_eq!(merged.lint.rule_config, current.lint.rule_config);
        assert_eq!(merged.storage.recent_paths, current.storage.recent_paths);
        assert_eq!(merged.ai.ollama.options, current.ai.ollama.options);
        assert_eq!(merged.ai.model_overrides, current.ai.model_overrides);

        // 既存のシークレット保持も壊れていないこと。
        assert_eq!(
            merged.ai.openai.api_key.as_deref(),
            current.ai.openai.api_key.as_deref()
        );
        let drive = merged.storage.google_drive.as_ref().unwrap();
        let current_drive = current.storage.google_drive.as_ref().unwrap();
        assert_eq!(drive.client_secret, current_drive.client_secret);
        assert_eq!(drive.refresh_token, current_drive.refresh_token);
    }

    #[test]
    fn merge_secrets_restores_local_root_paths_by_id() {
        let current = config_with_paths();
        let mut incoming = current.clone();

        // id が一致すれば path が空でも現在値が復元される。
        incoming.storage.local_roots[0].path.clear();
        // 新規 id は現在値がないためマージせず incoming の値を維持する。
        incoming.storage.local_roots.push(LocalRootConfig {
            id: "root-new".to_string(),
            name: "New".to_string(),
            path: PathBuf::from("/incoming/new"),
            read_only: false,
            watch: false,
        });
        // id が空のルートもマージ対象外。
        incoming.storage.local_roots.push(LocalRootConfig {
            id: String::new(),
            name: "NoId".to_string(),
            path: PathBuf::from("/incoming/no-id"),
            read_only: false,
            watch: false,
        });

        let merged = merge_secrets(&current, &incoming);

        assert_eq!(
            merged.storage.local_roots[0].path,
            PathBuf::from("/secret/root-a")
        );
        assert_eq!(
            merged.storage.local_roots[1].path,
            PathBuf::from("/secret/root-b")
        );
        assert_eq!(
            merged.storage.local_roots[2].path,
            PathBuf::from("/incoming/new")
        );
        assert_eq!(
            merged.storage.local_roots[3].path,
            PathBuf::from("/incoming/no-id")
        );
    }

    #[test]
    fn merge_secrets_skips_ambiguous_local_root_ids() {
        let mut current = config_with_paths();
        // 現在側で id が重複した場合は誤マージを避ける。
        current.storage.local_roots.push(LocalRootConfig {
            id: "root-a".to_string(),
            name: "Duplicate".to_string(),
            path: PathBuf::from("/secret/duplicate"),
            read_only: false,
            watch: false,
        });
        let mut incoming = current.clone();
        incoming.storage.local_roots[0].path.clear();

        let merged = merge_secrets(&current, &incoming);

        assert!(merged.storage.local_roots[0].path.as_os_str().is_empty());
    }
}
