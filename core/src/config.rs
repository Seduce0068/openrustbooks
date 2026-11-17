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
    /// Prefer local models
    pub prefer_local: bool,
    /// Timeout for API requests (seconds)
    pub timeout_secs: u64,
    /// Enable streaming responses
    pub streaming: bool,
    /// Custom model overrides
    pub model_overrides: std::collections::HashMap<String, String>,
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
            providers::{Env, Format, Serialized, Toml},
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

        // Load from environment variables (OPENRUSTBOOKS_*)
        figment = figment.merge(Env::prefixed("OPENRUSTBOOKS_").split("_"));

        Ok(figment.extract()?)
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
