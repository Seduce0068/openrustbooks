//! Configuration types for openrustbooks.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Sentinel used whenever a secret is redacted in `Debug` output or IPC views.
///
/// The value is intentionally not a valid credential so that echoing a masked
/// view back into an update command can be detected and replaced with the
/// secret currently held by the application.
pub const REDACTED_SECRET: &str = "***";

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// General application settings
    pub general: GeneralConfig,
    /// AI/LLM settings
    pub ai: AiConfig,
    /// Markdown lint settings
    pub lint: LintConfig,
    /// Storage settings
    pub storage: StorageConfig,
    /// Editor settings
    pub editor: EditorConfig,
    /// Reader settings
    pub reader: ReaderConfig,
}

/// General application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Application language (ISO 639-1)
    pub language: String,
    /// Theme (light, dark, system)
    pub theme: String,
    /// Enable telemetry
    pub telemetry: bool,
    /// Check for updates on startup
    pub check_updates: bool,
    /// Log level
    pub log_level: String,
    /// Data directory
    pub data_dir: Option<PathBuf>,
    /// Cache directory
    pub cache_dir: Option<PathBuf>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            language: "ja".to_string(),
            theme: "system".to_string(),
            telemetry: false,
            check_updates: true,
            log_level: "info".to_string(),
            data_dir: None,
            cache_dir: None,
        }
    }
}

/// AI/LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// Enable AI features.
    ///
    /// Disabled by default: no content is sent to any provider until the user
    /// explicitly opts in.
    pub enabled: bool,
    /// Default provider (openai, ollama, etc.)
    pub default_provider: String,
    /// OpenAI-compatible API settings
    pub openai: OpenAiConfig,
    /// Ollama local LLM settings
    pub ollama: OllamaConfig,
    /// Token limits
    pub max_tokens_per_request: usize,
    pub max_tokens_per_month: Option<u64>,
    /// 再生成の最大リトライ回数（キー欠落時は既定 3）
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
    /// Prefer local models
    pub prefer_local: bool,
    /// Timeout for API requests (seconds)
    pub timeout_secs: u64,
    /// Enable streaming responses
    pub streaming: bool,
    /// Custom model overrides
    pub model_overrides: std::collections::HashMap<String, String>,
}

/// TOML / IPC に `max_retries` が無い古い設定でも既定値を使う（後方互換）。
fn default_max_retries() -> usize {
    3
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            // Off by default; requires explicit user opt-in (see `AiConfig::enabled`).
            enabled: false,
            default_provider: "openai".to_string(),
            openai: OpenAiConfig::default(),
            ollama: OllamaConfig::default(),
            max_tokens_per_request: 8192,
            max_tokens_per_month: None,
            max_retries: 3,
            prefer_local: false,
            timeout_secs: 60,
            streaming: true,
            model_overrides: Default::default(),
        }
    }
}

/// OpenAI-compatible API configuration
#[derive(Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    /// API base URL (e.g., https://api.openai.com/v1)
    pub base_url: String,
    /// API key (from environment if not set)
    pub api_key: Option<String>,
    /// Default model
    pub model: String,
    /// Organization ID
    pub organization: Option<String>,
    /// Additional headers
    pub extra_headers: std::collections::HashMap<String, String>,
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: None,
            model: "gpt-4o-mini".to_string(),
            organization: None,
            extra_headers: Default::default(),
        }
    }
}

impl std::fmt::Debug for OpenAiConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAiConfig")
            .field("base_url", &self.base_url)
            .field("api_key", &self.api_key.as_ref().map(|_| REDACTED_SECRET))
            .field("model", &self.model)
            .field("organization", &self.organization)
            // キー名は残し、認証情報を含み得る値だけをマスクする。
            .field(
                "extra_headers",
                &self
                    .extra_headers
                    .keys()
                    .map(|name| (name.as_str(), REDACTED_SECRET))
                    .collect::<std::collections::HashMap<_, _>>(),
            )
            .finish()
    }
}

/// Ollama local LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Base URL (e.g., http://localhost:11434)
    pub base_url: String,
    /// Default model
    pub model: String,
    /// Keep model loaded (minutes)
    pub keep_alive_minutes: u32,
    /// Additional options
    pub options: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            model: "llama3.1:8b".to_string(),
            keep_alive_minutes: 5,
            options: Default::default(),
        }
    }
}

/// Markdown lint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintConfig {
    /// Enable linting
    pub enabled: bool,
    /// Configuration file path
    pub config_file: Option<PathBuf>,
    /// Rules to enable (empty = all)
    pub enable_rules: Vec<String>,
    /// Rules to disable
    pub disable_rules: Vec<String>,
    /// Auto-fix on save
    pub auto_fix: bool,
    /// Show warnings as errors
    pub warnings_as_errors: bool,
    /// Custom rule configurations
    pub rule_config: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            config_file: None,
            enable_rules: vec![],
            disable_rules: vec!["MD013".to_string()], // Line length - often too strict
            auto_fix: true,
            warnings_as_errors: false,
            rule_config: Default::default(),
        }
    }
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Default storage backend
    pub default_backend: String,
    /// Local filesystem roots
    pub local_roots: Vec<LocalRootConfig>,
    /// Google Drive settings
    pub google_drive: Option<GoogleDriveConfig>,
    /// Recently opened paths
    pub recent_paths: Vec<PathBuf>,
    /// Max recent paths to remember
    pub max_recent: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            default_backend: "local".to_string(),
            local_roots: vec![],
            google_drive: None,
            recent_paths: vec![],
            max_recent: 10,
        }
    }
}

/// Local filesystem root configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalRootConfig {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub read_only: bool,
    pub watch: bool,
}

/// Google Drive configuration
#[derive(Clone, Serialize, Deserialize)]
pub struct GoogleDriveConfig {
    /// OAuth client ID
    pub client_id: String,
    /// OAuth client secret (encrypted)
    pub client_secret: String,
    /// Refresh token (encrypted)
    pub refresh_token: Option<String>,
    /// Root folder ID
    pub root_folder_id: Option<String>,
    /// Use appDataFolder (hidden from user)
    pub use_app_data_folder: bool,
    /// Scopes
    pub scopes: Vec<String>,
}

impl std::fmt::Debug for GoogleDriveConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoogleDriveConfig")
            .field("client_id", &self.client_id)
            .field("client_secret", &REDACTED_SECRET)
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| REDACTED_SECRET),
            )
            .field("root_folder_id", &self.root_folder_id)
            .field("use_app_data_folder", &self.use_app_data_folder)
            .field("scopes", &self.scopes)
            .finish()
    }
}

/// Editor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Font family
    pub font_family: String,
    /// Font size (px)
    pub font_size: f32,
    /// Line height
    pub line_height: f32,
    /// Tab size
    pub tab_size: usize,
    /// Insert spaces instead of tabs
    pub insert_spaces: bool,
    /// Show line numbers
    pub line_numbers: bool,
    /// Word wrap
    pub word_wrap: bool,
    /// Minimap
    pub minimap: bool,
    /// Auto-save delay (ms)
    pub auto_save_delay: u32,
    /// Enable vim keybindings
    pub vim_mode: bool,
    /// Bracket matching
    pub match_brackets: bool,
    /// Auto-close brackets
    pub auto_close_brackets: bool,
    /// Indent with tabs
    pub indent_with_tabs: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            font_family: "'JetBrains Mono', 'Fira Code', monospace".to_string(),
            font_size: 14.0,
            line_height: 1.6,
            tab_size: 2,
            insert_spaces: true,
            line_numbers: true,
            word_wrap: true,
            minimap: false,
            auto_save_delay: 2000,
            vim_mode: false,
            match_brackets: true,
            auto_close_brackets: true,
            indent_with_tabs: false,
        }
    }
}

/// Reader configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaderConfig {
    /// Font family
    pub font_family: String,
    /// Font size (px)
    pub font_size: f32,
    /// Line height
    pub line_height: f32,
    /// Theme (light, dark, sepia, auto)
    pub theme: String,
    /// Column width (characters)
    pub column_width: Option<u32>,
    /// Scroll mode (vertical, horizontal, paginated)
    pub scroll_mode: String,
    /// Enable hyphenation
    pub hyphenation: bool,
    /// Vertical writing mode (for Japanese)
    pub vertical_writing: bool,
    /// Show page numbers
    pub show_page_numbers: bool,
    /// Keep position on reload
    pub remember_position: bool,
}

impl Default for ReaderConfig {
    fn default() -> Self {
        Self {
            font_family: "'Noto Serif JP', serif".to_string(),
            font_size: 16.0,
            line_height: 1.8,
            theme: "auto".to_string(),
            column_width: None,
            scroll_mode: "vertical".to_string(),
            hyphenation: false,
            vertical_writing: false,
            show_page_numbers: true,
            remember_position: true,
        }
    }
}

/// `OPENRUSTBOOKS_` を除いた環境変数名を figment のキーパスへ明示変換する。
///
/// 従来の `split("_")` は全ての `_` を `.` に置換するため、複数語フィールド
/// （`AI_MAX_RETRIES` -> `ai.max.retries`）が `ai.max_retries` に届かなかった。
/// ここでは既存の単一語 env を維持しつつ、対象フィールドだけを正しい
/// キーパスへ接続する（ADR-0122 / R6 TD-M08b）。
///
/// 未知のキーは小文字化するだけにし、serde の既定動作（未知フィールド無視）
/// に任せる。秘密フィールド（`api_key` / `client_secret` / `refresh_token`）は
/// ADR-0098 の前提を守るため意図的にマップしない。
#[cfg(not(target_arch = "wasm32"))]
fn map_env_key(key: &str) -> String {
    match key {
        "AI_MAX_RETRIES" => "ai.max_retries".to_string(),

        // 従来 split("_") で届いていた単一語フィールド（環境変数名の互換維持）。
        "GENERAL_LANGUAGE" => "general.language".to_string(),
        "GENERAL_THEME" => "general.theme".to_string(),
        "GENERAL_TELEMETRY" => "general.telemetry".to_string(),
        "AI_ENABLED" => "ai.enabled".to_string(),
        "AI_OPENAI_MODEL" => "ai.openai.model".to_string(),
        "AI_OPENAI_ORGANIZATION" => "ai.openai.organization".to_string(),
        "AI_OLLAMA_MODEL" => "ai.ollama.model".to_string(),
        "AI_STREAMING" => "ai.streaming".to_string(),
        "LINT_ENABLED" => "lint.enabled".to_string(),
        "EDITOR_MINIMAP" => "editor.minimap".to_string(),
        "READER_THEME" => "reader.theme".to_string(),
        "READER_HYPHENATION" => "reader.hyphenation".to_string(),

        other => other.to_ascii_lowercase(),
    }
}

/// env プロバイダへ載せるキーを選別する。
///
/// `Env::map()` はキーしか変換できず、値が "abc" の場合 serde が `usize` へ
/// 変換できず `extract()` が Err になり起動失敗する（figment 0.10.19 で実測）。
/// 生値を merge 前に検証し、不正値はキーごと除外して toml / 既定値を維持する
/// （ADR-0122 の「パース失敗時は維持」契約）。
#[cfg(not(target_arch = "wasm32"))]
fn include_env_key(mapped_key: &str, raw_retries: Option<&str>) -> bool {
    mapped_key != "ai.max_retries"
        || raw_retries.is_some_and(|value| value.parse::<usize>().is_ok())
}

/// 明示マップ付きの環境変数プロバイダを組み立てる。
///
/// `raw_retries` は `OPENRUSTBOOKS_AI_MAX_RETRIES` の生値。不正値を
/// `include_env_key` でふるいにかけ、figment に渡さない。
#[cfg(not(target_arch = "wasm32"))]
fn env_provider(raw_retries: Option<String>) -> figment::providers::Env {
    use figment::providers::Env;

    Env::prefixed("OPENRUSTBOOKS_")
        .map(|key| map_env_key(key.as_str()).into())
        .filter(move |key| include_env_key(key.as_str(), raw_retries.as_deref()))
}

/// 環境変数の生値をパースして `max_retries` を上書きする（失敗時は維持）。
///
/// 通常は [`env_provider`] が有効値だけを figment に載せるため既に反映済みだが、
/// `extract()` 後の防御的フォールバックとして残す。不正値／未設定は no-op。
#[cfg(not(target_arch = "wasm32"))]
fn apply_env_max_retries(config: &mut AppConfig, raw: Option<String>) {
    if let Some(retries) = raw.and_then(|value| value.parse::<usize>().ok()) {
        config.ai.max_retries = retries;
    }
}

impl AppConfig {
    /// Load configuration from default locations.
    ///
    /// Native only: the Wasm build has no filesystem or environment access.
    /// Browser persistence is planned to go through `localStorage` instead.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load() -> crate::CoreResult<Self> {
        use directories::ProjectDirs;
        use figment::{
            Figment,
            providers::{Format, Serialized, Toml},
        };

        let mut figment = Figment::new().merge(Serialized::defaults(Self::default()));

        // Load from project directory
        if let Some(proj_dirs) = ProjectDirs::from("com", "onoderakeita", "openrustbooks") {
            let config_dir = proj_dirs.config_dir();
            let config_file = config_dir.join("config.toml");
            if config_file.exists() {
                figment = figment.merge(Toml::file(&config_file));
            }
        }

        // Load from environment variables (OPENRUSTBOOKS_*).
        // 明示マップで AI_MAX_RETRIES -> ai.max_retries を接続し、不正値は
        // provider 側で除外する（ADR-0122 の「パース失敗時は維持」契約）。
        let raw_retries = std::env::var("OPENRUSTBOOKS_AI_MAX_RETRIES").ok();
        figment = figment.merge(env_provider(raw_retries.clone()));

        let mut config: Self = figment.extract()?;

        // 防御的フォールバック: 有効値のみ上書き（不正値／未設定は no-op）。
        apply_env_max_retries(&mut config, raw_retries);

        Ok(config)
    }

    /// Save configuration to default location.
    ///
    /// Native only; see [`AppConfig::load`].
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) -> crate::CoreResult<()> {
        use directories::ProjectDirs;
        use std::fs;

        if let Some(proj_dirs) = ProjectDirs::from("com", "onoderakeita", "openrustbooks") {
            let config_dir = proj_dirs.config_dir();
            fs::create_dir_all(config_dir)?;
            let config_file = config_dir.join("config.toml");
            let toml = toml::to_string_pretty(self)?;
            write_private(&config_file, toml.as_bytes())?;
        }
        Ok(())
    }
}

/// Write `contents` to `path` with owner-only permissions.
///
/// `OpenOptions::mode` applies only on creation, so `set_permissions` is also
/// called to tighten files that already existed with a wider mode.
#[cfg(all(not(target_arch = "wasm32"), unix))]
fn write_private(path: &std::path::Path, contents: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    file.write_all(contents)
}

/// Non-UNIX fallback: use the platform's default file permissions.
#[cfg(all(not(target_arch = "wasm32"), not(unix)))]
fn write_private(path: &std::path::Path, contents: &[u8]) -> std::io::Result<()> {
    std::fs::write(path, contents)
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
    fn ai_disabled_by_default() {
        assert!(!AiConfig::default().enabled);
    }

    #[test]
    fn toml_can_explicitly_enable_ai() {
        let config = AppConfig::default();
        assert!(!config.ai.enabled, "sanity: default must be disabled");

        let serialized = toml::to_string_pretty(&config).unwrap();
        let mut value: toml::Value = toml::from_str(&serialized).unwrap();
        value
            .get_mut("ai")
            .and_then(|v| v.as_table_mut())
            .expect("ai table")
            .insert("enabled".to_string(), toml::Value::Boolean(true));

        let restored: AppConfig = toml::from_str(&toml::to_string(&value).unwrap()).unwrap();
        assert!(restored.ai.enabled);
    }

    #[test]
    fn ai_config_default_max_retries_is_three() {
        assert_eq!(AiConfig::default().max_retries, 3);
    }

    #[test]
    fn toml_without_max_retries_uses_default() {
        let serialized = toml::to_string_pretty(&AppConfig::default()).unwrap();
        let mut value: toml::Value = toml::from_str(&serialized).unwrap();
        value
            .get_mut("ai")
            .and_then(|v| v.as_table_mut())
            .expect("ai table")
            .remove("max_retries");

        let stripped = toml::to_string(&value).unwrap();
        let restored: AppConfig = toml::from_str(&stripped).unwrap();
        assert_eq!(restored.ai.max_retries, 3);
    }

    #[test]
    fn toml_restores_explicit_max_retries() {
        let serialized = toml::to_string_pretty(&AppConfig::default()).unwrap();
        let mut value: toml::Value = toml::from_str(&serialized).unwrap();
        value
            .get_mut("ai")
            .and_then(|v| v.as_table_mut())
            .expect("ai table")
            .insert("max_retries".to_string(), toml::Value::Integer(5));

        let restored: AppConfig = toml::from_str(&toml::to_string(&value).unwrap()).unwrap();
        assert_eq!(restored.ai.max_retries, 5);
    }

    /// 環境変数の (キー, 値) を `map_env_key` / `include_env_key` に通して
    /// Figment を組み立てる。`AppConfig::load()` と同じ変換経路を
    /// プロセス環境を汚さずに再現する（Rust 2024 では set_var が unsafe）。
    #[cfg(not(target_arch = "wasm32"))]
    fn figment_from_env_entries<I>(entries: I, toml_override: Option<&str>) -> figment::Figment
    where
        I: IntoIterator<Item = (String, String)>,
    {
        use figment::{
            Figment,
            providers::{Format, Serialized, Toml},
        };

        let mut figment = Figment::new().merge(Serialized::defaults(AppConfig::default()));
        if let Some(toml) = toml_override {
            figment = figment.merge(Toml::string(toml));
        }

        let mut overrides = serde_json::Map::new();
        for (key, value) in entries {
            let mapped = map_env_key(&key);
            let raw = if mapped == "ai.max_retries" {
                Some(value.as_str())
            } else {
                None
            };
            if !include_env_key(&mapped, raw) {
                continue;
            }
            insert_env_value(&mut overrides, &mapped, parse_env_value(&value));
        }
        figment.merge(Serialized::defaults(serde_json::Value::Object(overrides)))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn insert_env_value(
        root: &mut serde_json::Map<String, serde_json::Value>,
        path: &str,
        value: serde_json::Value,
    ) {
        let parts: Vec<_> = path.split('.').collect();
        let (leaf, parents) = parts.split_last().expect("non-empty key path");
        let mut cursor = root;
        for part in parents {
            let entry = cursor
                .entry((*part).to_string())
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
            cursor = entry.as_object_mut().expect("dotted path must be object");
        }
        cursor.insert((*leaf).to_string(), value);
    }

    /// figment が env 値に行うパース（数値 / bool / 文字列）の最小再現。
    #[cfg(not(target_arch = "wasm32"))]
    fn parse_env_value(raw: &str) -> serde_json::Value {
        if let Ok(value) = raw.parse::<u64>() {
            serde_json::Value::Number(value.into())
        } else if raw == "true" || raw == "false" {
            serde_json::Value::Bool(raw == "true")
        } else {
            serde_json::Value::String(raw.to_string())
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn env_key_mapping_is_explicit() {
        assert_eq!(map_env_key("AI_MAX_RETRIES"), "ai.max_retries");
        // 既存の single-word env を壊さない（ADR-0122 の互換要件）。
        assert_eq!(map_env_key("GENERAL_LANGUAGE"), "general.language");
        assert_eq!(map_env_key("AI_ENABLED"), "ai.enabled");
        assert_eq!(map_env_key("AI_OPENAI_MODEL"), "ai.openai.model");
        assert_eq!(map_env_key("READER_THEME"), "reader.theme");
        // 未知 env は小文字化のみ（serde 側で無視）。
        assert_eq!(map_env_key("AI_FOO_BAR"), "ai_foo_bar");
        // 秘密フィールドはマップしない（ADR-0098 の前提維持）。
        assert_eq!(map_env_key("AI_OPENAI_API_KEY"), "ai_openai_api_key");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn env_invalid_max_retries_is_filtered_out() {
        assert!(include_env_key("ai.max_retries", Some("7")));
        assert!(!include_env_key("ai.max_retries", Some("abc")));
        // 他キーは生値に関係なく通過（例: boolean の "abc" は serde が扱う）。
        assert!(include_env_key("ai.enabled", Some("abc")));
    }

    /// フィルタを外して不正値を figment に渡すと extract が失敗することを固定。
    /// これが `include_env_key` による事前検証の根拠。
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn invalid_max_retries_would_fail_extract_without_filter() {
        use figment::{Figment, providers::Serialized};

        let invalid = Figment::new()
            .merge(Serialized::defaults(AppConfig::default()))
            .merge(Serialized::defaults(
                serde_json::json!({"ai": {"max_retries": "abc"}}),
            ))
            .extract::<AppConfig>();
        assert!(invalid.is_err(), "不正文字列は usize に変換できない");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn figment_extract_applies_mapped_max_retries() {
        let entries = [("AI_MAX_RETRIES".to_string(), "7".to_string())];
        let config: AppConfig = figment_from_env_entries(entries, None).extract().unwrap();
        assert_eq!(config.ai.max_retries, 7);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn figment_extract_keeps_toml_value_on_invalid_env() {
        let entries = [("AI_MAX_RETRIES".to_string(), "abc".to_string())];
        let config: AppConfig = figment_from_env_entries(entries, Some("[ai]\nmax_retries = 5\n"))
            .extract()
            .unwrap();
        assert_eq!(config.ai.max_retries, 5);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn figment_extract_keeps_default_on_invalid_env() {
        let entries = [("AI_MAX_RETRIES".to_string(), "abc".to_string())];
        let config: AppConfig = figment_from_env_entries(entries, None).extract().unwrap();
        assert_eq!(config.ai.max_retries, 3);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn figment_extract_preserves_single_word_env() {
        let entries = [
            ("AI_ENABLED".to_string(), "true".to_string()),
            ("GENERAL_LANGUAGE".to_string(), "en".to_string()),
        ];
        let config: AppConfig = figment_from_env_entries(entries, None).extract().unwrap();
        assert!(config.ai.enabled);
        assert_eq!(config.general.language, "en");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn apply_env_max_retries_overwrites_with_valid_value() {
        let mut config = AppConfig::default();
        apply_env_max_retries(&mut config, Some("7".to_string()));
        assert_eq!(config.ai.max_retries, 7);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn apply_env_max_retries_keeps_value_on_parse_failure() {
        let mut config = AppConfig::default();
        config.ai.max_retries = 5;
        apply_env_max_retries(&mut config, Some("abc".to_string()));
        assert_eq!(config.ai.max_retries, 5);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn apply_env_max_retries_keeps_value_when_absent() {
        let mut config = AppConfig::default();
        config.ai.max_retries = 5;
        apply_env_max_retries(&mut config, None);
        assert_eq!(config.ai.max_retries, 5);
    }

    #[test]
    fn debug_output_masks_secret_values() {
        let debug = format!("{:?}", config_with_secrets());

        assert!(!debug.contains("sk-dummy-do-not-leak"));
        assert!(!debug.contains("dummy-client-secret"));
        assert!(!debug.contains("dummy-refresh-token"));
        assert!(debug.contains(REDACTED_SECRET));
    }

    #[test]
    fn debug_output_masks_extra_header_values() {
        let mut config = config_with_secrets();
        config.ai.openai.extra_headers.insert(
            "X-Dummy-Header".to_string(),
            "dummy-header-value-do-not-leak".to_string(),
        );
        let debug = format!("{:?}", config.ai.openai);

        assert!(
            !debug.contains("dummy-header-value-do-not-leak"),
            "leaked header value"
        );
        // キー名は残り、値だけが sentinel になる。
        assert!(debug.contains(&format!("\"X-Dummy-Header\": \"{REDACTED_SECRET}\"")));
    }

    #[test]
    fn toml_round_trip_keeps_extra_headers() {
        let mut config = AppConfig::default();
        config.ai.openai.extra_headers.insert(
            "X-Dummy-Header".to_string(),
            "dummy-header-value".to_string(),
        );

        // Debug マスクは表現だけに影響し、シリアライズ値は変えない。
        let serialized = toml::to_string_pretty(&config).unwrap();
        let restored: AppConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(
            restored.ai.openai.extra_headers,
            config.ai.openai.extra_headers
        );
    }

    #[test]
    fn config_without_optional_secret_fields_still_deserializes() {
        let config = config_with_secrets();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let mut value: toml::Value = toml::from_str(&serialized).unwrap();

        value
            .get_mut("ai")
            .and_then(|v| v.get_mut("openai"))
            .and_then(|v| v.as_table_mut())
            .expect("openai table")
            .remove("api_key");
        value
            .get_mut("storage")
            .and_then(|v| v.get_mut("google_drive"))
            .and_then(|v| v.as_table_mut())
            .expect("google_drive table")
            .remove("refresh_token");

        let stripped = toml::to_string(&value).unwrap();
        let restored: AppConfig = toml::from_str(&stripped).unwrap();

        assert!(restored.ai.openai.api_key.is_none());
        assert_eq!(restored.ai.openai.base_url, config.ai.openai.base_url);
        let drive = restored.storage.google_drive.unwrap();
        assert!(drive.refresh_token.is_none());
        assert_eq!(drive.client_secret, "dummy-client-secret");
    }

    #[cfg(unix)]
    #[test]
    fn write_private_creates_and_tightens_0600() {
        use std::os::unix::fs::PermissionsExt;

        let dir =
            std::env::temp_dir().join(format!("openrustbooks-config-perms-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");

        write_private(&path, b"first").unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);

        // `mode` only applies on creation; an existing 0644 file must also be
        // tightened when it is overwritten.
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        write_private(&path, b"second").unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        assert_eq!(std::fs::read(&path).unwrap(), b"second");

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
