//! Configuration commands.

use crate::commands::{ipc_error, ipc_message};
use crate::state::{AppState, CONFIG_LOAD_FAILED_MESSAGE};
use openrustbooks_core::ai_markdown::client::{ProviderKind, validate_base_url};
use openrustbooks_core::config::{
    AiConfig, AppConfig, EditorConfig, GeneralConfig, GoogleDriveConfig, LintConfig,
    LocalRootConfig, OllamaConfig, OpenAiConfig, REDACTED_SECRET, ReaderConfig, StorageConfig,
};
use openrustbooks_core::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{State, command};

/// 設定保存コマンド専用の構造化エラー。
///
/// `code` は [`CoreError::code()`] 由来の固定値、`message` は code ごとの固定文言のみ。
/// 生エラー文字列・設定値・パス・秘密情報は一切含めない。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommandError {
    pub code: &'static str,
    pub message: String,
}

impl CommandError {
    fn from_code(code: &'static str) -> Self {
        // 文言は共通マッパーに一本化する。code / message の JSON 契約は維持。
        Self {
            code,
            message: ipc_message(code).to_string(),
        }
    }
}

impl From<CoreError> for CommandError {
    fn from(err: CoreError) -> Self {
        Self::from_code(err.code())
    }
}

/// 起動時の読み込み失敗状態では、保存コマンドを固定文言で拒否する。
///
/// 値・パス・生エラーは返さない。`String` を返す既存コマンド向けの
/// [`load_failure_message`] と同じ文言を使う。
fn load_failed_command_error(state: &AppState) -> Result<(), CommandError> {
    if state.config_load_failed() {
        Err(CommandError::from_code("CONFIG_LOAD_FAILED"))
    } else {
        Ok(())
    }
}

/// `Result<_, String>` を返す保存コマンド向けの読み込み失敗チェック。
fn load_failure_message(state: &AppState) -> Option<String> {
    state
        .config_load_failed()
        .then(|| CONFIG_LOAD_FAILED_MESSAGE.to_string())
}

/// Allow-list view of [`AppConfig`] returned to the JS side.
///
/// Secret fields are never serialized; only `has_*` presence flags are sent.
/// パスを含むフィールド（data_dir / cache_dir / config_file / rule_config /
/// local_roots[].path / recent_paths）も DTO から除外して JS への開示を防ぐ。
/// さらに未配線の死設定（telemetry / max_tokens_per_month / model_overrides /
/// keep_alive_minutes / default_backend / max_recent / auto_save_delay /
/// local_roots[].watch）も DTO から除外する（ADR-0304）。config.toml の値は
/// `merge_secrets()` が現在値から復元するため、保存で失われない。
#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigView {
    /// 現在の revision。コミット衝突後に `get_config` で再取得し、
    /// 次のコミットの `expected` として使う（UI の再試行は ADR 未決定）。
    pub revision: u64,
    pub general: GeneralConfigView,
    pub ai: AiConfigView,
    pub lint: LintConfigView,
    pub storage: StorageConfigView,
    pub editor: EditorConfigView,
    pub reader: ReaderConfig,
}

impl ConfigView {
    /// IPC 返却用: 設定と同じ時点の revision を添えて DTO を作る。
    pub fn from_state(config: &AppConfig, revision: u64) -> Self {
        Self {
            revision,
            general: GeneralConfigView::from(&config.general),
            ai: AiConfigView::from(&config.ai),
            lint: LintConfigView::from(&config.lint),
            storage: StorageConfigView::from(&config.storage),
            editor: EditorConfigView::from(&config.editor),
            reader: config.reader.clone(),
        }
    }
}

impl From<&AppConfig> for ConfigView {
    /// テスト／内部変換用。実 IPC の返却には `from_state` を使うこと。
    fn from(config: &AppConfig) -> Self {
        Self::from_state(config, 0)
    }
}

/// Allow-list view of [`GeneralConfig`].
/// `data_dir` / `cache_dir` と死設定の `telemetry` は除外する（ADR-0304）。
#[derive(Clone, Serialize, Deserialize)]
pub struct GeneralConfigView {
    pub language: String,
    pub theme: String,
    pub check_updates: bool,
    pub log_level: String,
}

impl From<&GeneralConfig> for GeneralConfigView {
    fn from(general: &GeneralConfig) -> Self {
        Self {
            language: general.language.clone(),
            theme: general.theme.clone(),
            check_updates: general.check_updates,
            log_level: general.log_level.clone(),
        }
    }
}

/// Allow-list view of [`AiConfig`].
/// 死設定の `model_overrides` / `max_tokens_per_month` と `max_retries` は除外する
/// （ADR-0228 / 0304）。値は `merge_secrets()` が現在値から復元する。
#[derive(Clone, Serialize, Deserialize)]
pub struct AiConfigView {
    pub enabled: bool,
    pub default_provider: String,
    pub openai: OpenAiConfigView,
    pub ollama: OllamaConfigView,
    pub max_tokens_per_request: usize,
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
            prefer_local: ai.prefer_local,
            timeout_secs: ai.timeout_secs,
            streaming: ai.streaming,
        }
    }
}

/// Allow-list view of [`OllamaConfig`].
/// `options` と死設定の `keep_alive_minutes` は除外する（ADR-0304）。
#[derive(Clone, Serialize, Deserialize)]
pub struct OllamaConfigView {
    pub base_url: String,
    pub model: String,
}

impl From<&OllamaConfig> for OllamaConfigView {
    fn from(ollama: &OllamaConfig) -> Self {
        Self {
            base_url: ollama.base_url.clone(),
            model: ollama.model.clone(),
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
/// `local_roots[].path` / `recent_paths` と死設定の `default_backend` /
/// `max_recent` は JS に開示しない（ADR-0304）。
#[derive(Clone, Serialize, Deserialize)]
pub struct StorageConfigView {
    pub local_roots: Vec<LocalRootView>,
    pub google_drive: Option<GoogleDriveConfigView>,
}

impl From<&StorageConfig> for StorageConfigView {
    fn from(storage: &StorageConfig) -> Self {
        Self {
            local_roots: storage
                .local_roots
                .iter()
                .map(LocalRootView::from)
                .collect(),
            google_drive: storage
                .google_drive
                .as_ref()
                .map(GoogleDriveConfigView::from),
        }
    }
}

/// Allow-list view of [`LocalRootConfig`].
/// `path` と死設定の `watch` は除外する（ADR-0178 / 0304）。
#[derive(Clone, Serialize, Deserialize)]
pub struct LocalRootView {
    pub id: String,
    pub name: String,
    pub read_only: bool,
}

impl From<&LocalRootConfig> for LocalRootView {
    fn from(root: &LocalRootConfig) -> Self {
        Self {
            id: root.id.clone(),
            name: root.name.clone(),
            read_only: root.read_only,
        }
    }
}

/// Allow-list view of [`EditorConfig`]. 死設定の `auto_save_delay` は除外する
/// （ADR-0179 / 0217 / 0304）。config.toml の値は `merge_secrets()` が復元する。
#[derive(Clone, Serialize, Deserialize)]
pub struct EditorConfigView {
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub tab_size: usize,
    pub insert_spaces: bool,
    pub line_numbers: bool,
    pub word_wrap: bool,
    pub minimap: bool,
    pub vim_mode: bool,
    pub match_brackets: bool,
    pub auto_close_brackets: bool,
    pub indent_with_tabs: bool,
}

impl From<&EditorConfig> for EditorConfigView {
    fn from(editor: &EditorConfig) -> Self {
        Self {
            font_family: editor.font_family.clone(),
            font_size: editor.font_size,
            line_height: editor.line_height,
            tab_size: editor.tab_size,
            insert_spaces: editor.insert_spaces,
            line_numbers: editor.line_numbers,
            word_wrap: editor.word_wrap,
            minimap: editor.minimap,
            vim_mode: editor.vim_mode,
            match_brackets: editor.match_brackets,
            auto_close_brackets: editor.auto_close_brackets,
            indent_with_tabs: editor.indent_with_tabs,
        }
    }
}

impl EditorConfigView {
    fn into_config(self) -> EditorConfig {
        EditorConfig {
            font_family: self.font_family,
            font_size: self.font_size,
            line_height: self.line_height,
            tab_size: self.tab_size,
            insert_spaces: self.insert_spaces,
            line_numbers: self.line_numbers,
            word_wrap: self.word_wrap,
            minimap: self.minimap,
            // DTO 外: プレースホルダを入れ、現在値は merge_secrets() が復元する。
            auto_save_delay: 0,
            vim_mode: self.vim_mode,
            match_brackets: self.match_brackets,
            auto_close_brackets: self.auto_close_brackets,
            indent_with_tabs: self.indent_with_tabs,
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

/// Web から送られる部分更新 DTO。[`ConfigView`] と同じ公開フィールドだけを持つ。
/// `has_*` フラグはデシリアライズのために受け取るが、更新時は無視して現在値を保持する。
#[derive(Clone, Deserialize)]
pub struct ConfigPatch {
    /// UI が `get_config` で読んだ revision。CAS の `expected` として使う（必須）。
    /// `ConfigView.revision` とは名前が異なるが、同じ値を入れる契約（ADR-0253）。
    pub expected: u64,
    pub general: GeneralConfigView,
    pub ai: AiConfigView,
    pub lint: LintConfigView,
    pub storage: StorageConfigView,
    pub editor: EditorConfigView,
    pub reader: ReaderConfig,
}

impl From<(&AppConfig, u64)> for ConfigPatch {
    fn from((config, expected): (&AppConfig, u64)) -> Self {
        let view = ConfigView::from(config);
        Self {
            expected,
            general: view.general,
            ai: view.ai,
            lint: view.lint,
            storage: view.storage,
            editor: view.editor,
            reader: view.reader,
        }
    }
}

impl ConfigPatch {
    /// 公開フィールドだけを反映する。DTO 外のフィールドはプレースホルダ／既定値にする。
    ///
    /// 不変条件: `apply_to()` の結果は必ず `merge_secrets()` を通すこと。
    /// 現在値の復元は `merge_secrets()` のみが行い、ここでは `current` を使わない。
    pub fn apply_to(self, _current: &AppConfig) -> AppConfig {
        AppConfig {
            general: self.general.into_config(),
            ai: self.ai.into_config(),
            lint: self.lint.into_config(),
            storage: self.storage.into_config(),
            editor: self.editor.into_config(),
            reader: self.reader,
        }
    }
}

// View -> core の逆変換。DTO に無いフィールドは既定値／空値を入れ、
// 現在値の復元は merge_secrets() に任せる。
impl GeneralConfigView {
    fn into_config(self) -> GeneralConfig {
        GeneralConfig {
            language: self.language,
            theme: self.theme,
            // DTO 外: merge_secrets() が現在値を復元する。
            telemetry: false,
            check_updates: self.check_updates,
            log_level: self.log_level,
            // DTO 外: merge_secrets() が現在値を復元する。
            data_dir: None,
            cache_dir: None,
        }
    }
}

impl AiConfigView {
    fn into_config(self) -> AiConfig {
        AiConfig {
            enabled: self.enabled,
            default_provider: self.default_provider,
            openai: self.openai.into_config(),
            ollama: self.ollama.into_config(),
            max_tokens_per_request: self.max_tokens_per_request,
            // DTO 外: merge_secrets() が現在値を復元する。
            max_tokens_per_month: None,
            prefer_local: self.prefer_local,
            timeout_secs: self.timeout_secs,
            streaming: self.streaming,
            // DTO 外: プレースホルダを入れ、現在値は merge_secrets() が復元する。
            max_retries: 0,
            model_overrides: HashMap::new(),
        }
    }
}

impl OpenAiConfigView {
    fn into_config(self) -> OpenAiConfig {
        OpenAiConfig {
            base_url: self.base_url,
            // has_api_key は無視。空値を入れ、実値は merge_secrets() が復元する。
            api_key: None,
            model: self.model,
            organization: self.organization,
            // DTO 外: 空のまま merge_secrets() に任せる。
            extra_headers: HashMap::new(),
        }
    }
}

impl OllamaConfigView {
    fn into_config(self) -> OllamaConfig {
        OllamaConfig {
            base_url: self.base_url,
            model: self.model,
            // DTO 外: merge_secrets() が現在値を復元する。
            keep_alive_minutes: 0,
            options: HashMap::new(),
        }
    }
}

impl LintConfigView {
    fn into_config(self) -> LintConfig {
        LintConfig {
            enabled: self.enabled,
            // DTO 外: merge_secrets() が現在値を復元する。
            config_file: None,
            enable_rules: self.enable_rules,
            disable_rules: self.disable_rules,
            auto_fix: self.auto_fix,
            warnings_as_errors: self.warnings_as_errors,
            rule_config: HashMap::new(),
        }
    }
}

impl StorageConfigView {
    fn into_config(self) -> StorageConfig {
        StorageConfig {
            // DTO 外: merge_secrets() が現在値を復元する。
            default_backend: String::new(),
            local_roots: self
                .local_roots
                .into_iter()
                .map(LocalRootView::into_config)
                .collect(),
            google_drive: self.google_drive.map(GoogleDriveConfigView::into_config),
            // DTO 外: merge_secrets() が現在値を復元する。
            recent_paths: Vec::new(),
            max_recent: 0,
        }
    }
}

impl LocalRootView {
    fn into_config(self) -> LocalRootConfig {
        // path は DTO 外なので常に空にする。merge_secrets() が id 一意一致時のみ
        // 現在値を復元し、新規／不一致 id は incoming（空）のまま残す。
        LocalRootConfig {
            id: self.id,
            name: self.name,
            path: PathBuf::new(),
            read_only: self.read_only,
            // DTO 外: merge_secrets() が現在値を復元する。
            watch: false,
        }
    }
}

impl GoogleDriveConfigView {
    fn into_config(self) -> GoogleDriveConfig {
        // has_* は無視。空値／None を入れ、実値は merge_secrets() が復元する。
        GoogleDriveConfig {
            client_id: self.client_id,
            client_secret: String::new(),
            refresh_token: None,
            root_folder_id: self.root_folder_id,
            use_app_data_folder: self.use_app_data_folder,
            scopes: self.scopes,
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
/// DTO 外フィールド（パス・シークレット・マップ等）の復元はここが唯一の箇所。
pub(crate) fn merge_secrets(current: &AppConfig, incoming: &AppConfig) -> AppConfig {
    let mut merged = incoming.clone();

    if is_placeholder(merged.ai.openai.api_key.as_deref()) {
        merged.ai.openai.api_key = current.ai.openai.api_key.clone();
    }

    // extra_headers は DTO から除外している。空 map は「未変更」として全件を現在値で
    // 維持する（ConfigPatch 往復 / 既存契約）。非空ならキー単位でマージし、
    // マスク値（***）のキーだけ現在値を復元し、それ以外のキーは incoming を採用する
    // （PR-M05: 一部マスクでヘッダ全体が捨てられる問題の修正）。
    // 現在側に存在しないマスクキーは sentinel を実値として保存しないため破棄する。
    if merged.ai.openai.extra_headers.is_empty() {
        merged.ai.openai.extra_headers = current.ai.openai.extra_headers.clone();
    } else {
        let current_headers = &current.ai.openai.extra_headers;
        let incoming_headers = std::mem::take(&mut merged.ai.openai.extra_headers);
        merged.ai.openai.extra_headers = incoming_headers
            .into_iter()
            .filter_map(|(key, value)| {
                if value == REDACTED_SECRET {
                    current_headers
                        .get(&key)
                        .map(|current| (key, current.clone()))
                } else {
                    Some((key, value))
                }
            })
            .collect();
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
    // max_retries は DTO から除外したため、JS 往復で現在値を保持する。
    merged.ai.max_retries = current.ai.max_retries;
    // ADR-0222 / ADR-0277: check_updates は updater 未実装の死設定。UI 側は固定表示へ
    // 変更したが、DTO 往復のためフィールド自体は残る。JS が値を書き換えても現在値を
    // 復元し、config.toml へ誤った値が保存されないようにする（fail-closed）。
    merged.general.check_updates = current.general.check_updates;

    // ADR-0304: 未配線の死設定は DTO から除外したため、JS 往復で消えないよう
    // 現在値（config.toml から読み込んだ値）を復元する。値は JS に開示されない。
    // local_roots[].watch は直下の id 一致ロジックで復元する。
    merged.general.telemetry = current.general.telemetry;
    merged.ai.max_tokens_per_month = current.ai.max_tokens_per_month;
    merged.ai.ollama.keep_alive_minutes = current.ai.ollama.keep_alive_minutes;
    merged.storage.default_backend = current.storage.default_backend.clone();
    merged.storage.max_recent = current.storage.max_recent;
    merged.editor.auto_save_delay = current.editor.auto_save_delay;

    // local_roots[].path / read_only / watch は id が一意に一致した場合のみ現在値を復元する。
    // id が空／不一致／現在側で重複している場合は空 PathBuf に統一し、flags も現在値を
    // 採用せず incoming の値をそのまま残す（新規 root として扱う）。incoming 側で id が
    // 重複した group も fail-closed で同様に扱う（ADR-0129 / ADR-0281 の fail-closed）。
    // DTO は flags をクライアント writable で公開するため、一意一致以外で現在値を
    // 採用すると read-only root が writable へ昇格し得る（R3-AU01）。
    let mut incoming_id_counts: HashMap<String, usize> = HashMap::new();
    for root in &merged.storage.local_roots {
        if !root.id.is_empty() {
            *incoming_id_counts.entry(root.id.clone()).or_insert(0) += 1;
        }
    }

    for root in &mut merged.storage.local_roots {
        // 空 id は復元対象外（path を空にし、flags は incoming のまま）。
        if root.id.is_empty() {
            root.path = PathBuf::new();
            continue;
        }
        // incoming 側で id が重複した group は誤マージを避け、path を空にして
        // flags も現在値へ復元しない。
        if incoming_id_counts
            .get(&root.id)
            .is_some_and(|&count| count > 1)
        {
            root.path = PathBuf::new();
            continue;
        }
        let mut matches = current
            .storage
            .local_roots
            .iter()
            .filter(|current_root| current_root.id == root.id);
        let Some(current_root) = matches.next() else {
            root.path = PathBuf::new(); // 不一致: 復元せず空にする
            continue;
        };
        if matches.next().is_some() {
            root.path = PathBuf::new(); // 現在側で id 重複: 誤マージを避け空にする
            continue;
        }
        // id 一意一致: path に加えて read_only / watch も現在値を採用する。
        // incoming の read_only=false / watch=true を無視し、昇格を防ぐ。
        root.path = current_root.path.clone();
        root.read_only = current_root.read_only;
        root.watch = current_root.watch;
    }

    merged
}

/// JS から届いた生の [`AppConfig`] を [`ConfigView`] / [`ConfigPatch`] と同じ DTO 境界へ正規化する。
///
/// `update_config` は `AppConfig` 全体を受け取るため、JS が DTO 外の
/// `extra_headers` / ローカルパス / 秘密値を直接注入できる。ここで
/// `ConfigPatch::apply_to()` と同じプレースホルダ化を通し、現在値の復元を
/// [`merge_secrets`] に一本化する（ADR-0132 の不変条件を維持）。
/// 公開フィールドは `ConfigView` の allow-list を通るためそのまま保持される。
///
/// `apply_to()` は現在 `current` を参照しないが、将来復元処理が移っても挙動が
/// 変わらないよう、呼び出し側の現在値をそのまま渡す。
fn normalize_to_dto_boundary(raw: &AppConfig, current: &AppConfig) -> AppConfig {
    // expected は CAS の入力だが、この正規化は apply_to() だけを呼ぶため値は無関係。
    ConfigPatch::from((raw, 0)).apply_to(current)
}

/// JS から届いた生の [`AiConfig`] のうち、DTO 外の `extra_headers` だけを落とす。
///
/// `api_key` は ADR-0098 の契約で明示的新値を受け入れるため維持する。
/// 空にした `extra_headers` は [`merge_secrets`] が現在値を復元する（既存契約）。
fn normalize_ai_from_js(mut ai: AiConfig) -> AiConfig {
    ai.openai.extra_headers.clear();
    ai
}

/// API キー更新専用の純粋ヘルパー。
///
/// 現在設定を DTO 境界へ落としてから [`merge_secrets`] で復元し、
/// `api_key` だけを差し替える。空文字は未設定（`None`）として扱う。
/// 値はログ・エラー・DTO へ一切出さない。
fn config_with_api_key(current: &AppConfig, api_key: String) -> AppConfig {
    let normalized = normalize_to_dto_boundary(current, current);
    let mut config = merge_secrets(current, &normalized);
    config.ai.openai.api_key = if api_key.is_empty() {
        None
    } else {
        Some(api_key)
    };
    config
}

/// 保存・再初期化前に両プロバイダの base_url を検証する。
/// JS は AppConfig 全体を渡せるため、ホストの許可リストは core 側にハードコードする。
fn validate_ai_base_urls(ai: &AiConfig) -> Result<(), String> {
    validate_base_url(&ai.openai.base_url, ProviderKind::OpenAi).map_err(ipc_error)?;
    validate_base_url(&ai.ollama.base_url, ProviderKind::Ollama).map_err(ipc_error)?;
    Ok(())
}

/// ai / lint 設定が変わったかを JSON 表現の差分で判定する。
/// これらの設定型は PartialEq 未 derive のため serde_json の値を比較する。
pub(crate) fn ai_or_lint_changed(current: &AppConfig, merged: &AppConfig) -> bool {
    fn json_differs<T: Serialize>(before: &T, after: &T) -> bool {
        match (serde_json::to_value(before), serde_json::to_value(after)) {
            (Ok(before), Ok(after)) => before != after,
            // シリアライズ失敗時は安全側に倒して「変更あり」とみなす。
            _ => true,
        }
    }
    json_differs(&current.ai, &merged.ai) || json_differs(&current.lint, &merged.lint)
}

/// 検証済みの候補設定をコミットする共通経路。
///
/// 差分がある時だけ候補設定から service を準備し、成功した場合のみ
/// 設定と service をまとめて保存する。生成失敗時はコミットしないため
/// config / ディスク / service は不変。`merge_secrets` は呼び出し側の責務で、
/// ここでは行わない（clear_secrets の契約を守るため）。
async fn commit_validated_config(
    state: &AppState,
    expected: u64,
    merged: AppConfig,
    changed: bool,
) -> CoreResult<()> {
    let service = if changed {
        state.prepare_ai_service(&merged).await?
    } else {
        // 差分なし: 現在の service を維持する（無効設定なら None）。
        state.ai_service()
    };
    // expected は各コマンドが await 前に取得した値をそのまま使う（再取得しない）。
    state.commit_config(expected, merged, service)
}

/// Get current configuration (secrets redacted)
#[command]
pub fn get_config(state: State<'_, Arc<AppState>>) -> ConfigView {
    let (config, revision) = state.config_with_revision();
    ConfigView::from_state(&config, revision)
}

/// Update configuration
#[command]
pub async fn update_config(
    state: State<'_, Arc<AppState>>,
    config: AppConfig,
) -> Result<(), String> {
    if let Some(message) = load_failure_message(state.inner()) {
        return Err(message);
    }
    let (current, expected) = state.config_with_revision();
    // 生 AppConfig の DTO 外フィールドは JS の注入を許さない。境界まで落としてから
    // merge_secrets() に渡し、現在値の復元は従来どおり merge_secrets() に任せる。
    let normalized = normalize_to_dto_boundary(&config, &current);
    let merged = merge_secrets(&current, &normalized);
    validate_ai_base_urls(&merged.ai)?;
    let changed = ai_or_lint_changed(&current, &merged);
    commit_validated_config(state.inner(), expected, merged, changed)
        .await
        .map_err(ipc_error)?;
    Ok(())
}

/// Update configuration from a DTO patch, returning the resulting view.
#[command]
pub async fn update_config_patch(
    state: State<'_, Arc<AppState>>,
    patch: ConfigPatch,
) -> Result<ConfigView, CommandError> {
    update_config_patch_inner(state.inner(), patch).await
}

/// コマンド本体。テストから `AppState` を直接渡せるよう分離している。
async fn update_config_patch_inner(
    state: &AppState,
    patch: ConfigPatch,
) -> Result<ConfigView, CommandError> {
    // 読み込み失敗状態では既定値の上書きを防ぐため、保存前に拒否する。
    load_failed_command_error(state)?;
    // UI が get_config で読んだ revision をそのまま CAS の expected に使う。
    // 内部で再取得した revision で上書きしてはならない（ADR-0253 実装前提 3）。
    let expected = patch.expected;
    let (current, _internal_revision) = state.config_with_revision();
    // 不変条件: apply_to() の結果は必ず merge_secrets() を通してから保存する。
    let applied = patch.apply_to(&current);
    let merged = merge_secrets(&current, &applied);
    // 検証エラーの生メッセージは返さず、code ごとの固定文言へ変換する。
    validate_ai_base_urls(&merged.ai).map_err(|_| CommandError::from_code("CONFIG_VALIDATION"))?;
    let changed = ai_or_lint_changed(&current, &merged);
    commit_validated_config(state, expected, merged, changed)
        .await
        .map_err(CommandError::from)?;
    let (config, revision) = state.config_with_revision();
    Ok(ConfigView::from_state(&config, revision))
}

/// Update AI configuration
#[command]
pub async fn update_ai_config(
    state: State<'_, Arc<AppState>>,
    ai_config: AiConfig,
) -> Result<(), String> {
    if let Some(message) = load_failure_message(state.inner()) {
        return Err(message);
    }
    let (current, expected) = state.config_with_revision();
    let mut incoming = current.clone();
    // extra_headers は DTO 外。JS の注入を許さず空にし、merge_secrets() の
    // 現在値復元に任せる。api_key は ADR-0098 の明示的新値契約を維持する。
    incoming.ai = normalize_ai_from_js(ai_config);
    let config = merge_secrets(&current, &incoming);
    validate_ai_base_urls(&config.ai)?;
    let changed = ai_or_lint_changed(&current, &config);

    // 差分がある時だけ再準備。失敗時はコミットしないため状態は不変。
    commit_validated_config(state.inner(), expected, config, changed)
        .await
        .map_err(ipc_error)?;
    Ok(())
}

/// Web の API キー保存専用コマンド。
///
/// 秘密値は DTO（`ConfigPatch` / `ConfigView`）を通さず、この引数 `api_key`
/// だけから受け取る。`expected` は UI が `get_config` で読んだ revision を
/// そのまま CAS に使う（ADR-0253）。成功時は更新後の view を返す。
#[command]
pub async fn set_openai_api_key(
    state: State<'_, Arc<AppState>>,
    expected: u64,
    api_key: String,
) -> Result<ConfigView, CommandError> {
    set_openai_api_key_inner(state.inner(), expected, api_key).await
}

/// コマンド本体。テストから `AppState` を直接渡せるよう分離している。
async fn set_openai_api_key_inner(
    state: &AppState,
    expected: u64,
    api_key: String,
) -> Result<ConfigView, CommandError> {
    load_failed_command_error(state)?;
    let (current, _internal_revision) = state.config_with_revision();
    let config = config_with_api_key(&current, api_key);
    // 生メッセージは返さず code ごとの固定文言へ変換する。
    validate_ai_base_urls(&config.ai).map_err(|_| CommandError::from_code("CONFIG_VALIDATION"))?;
    let changed = ai_or_lint_changed(&current, &config);
    commit_validated_config(state, expected, config, changed)
        .await
        .map_err(CommandError::from)?;
    let (config, revision) = state.config_with_revision();
    Ok(ConfigView::from_state(&config, revision))
}

/// Update lint configuration
#[command]
pub async fn update_lint_config(
    state: State<'_, Arc<AppState>>,
    lint_config: LintConfig,
) -> Result<(), String> {
    if let Some(message) = load_failure_message(state.inner()) {
        return Err(message);
    }
    let (current, expected) = state.config_with_revision();
    let mut incoming = current.clone();
    incoming.lint = lint_config;
    // DTO から除外した config_file / rule_config を消さないようマージを通す。
    let config = merge_secrets(&current, &incoming);
    let changed = ai_or_lint_changed(&current, &config);
    commit_validated_config(state.inner(), expected, config, changed)
        .await
        .map_err(ipc_error)?;
    Ok(())
}

/// Reset configuration to defaults (secrets redacted)
#[command]
pub async fn reset_config(state: State<'_, Arc<AppState>>) -> Result<ConfigView, String> {
    if let Some(message) = load_failure_message(state.inner()) {
        return Err(message);
    }
    // current 自体は不要だが、読み取った revision を expected として渡す。
    let (_, expected) = state.config_with_revision();
    let config = AppConfig::default();
    // 既定は AI 無効。prepare は None を返し、保存後に service もクリアされる。
    commit_validated_config(state.inner(), expected, config, true)
        .await
        .map_err(ipc_error)?;
    let (current, revision) = state.config_with_revision();
    Ok(ConfigView::from_state(&current, revision))
}

/// 個別に消すシークレットの指定。既定はすべて false（消さない fail-closed）。
#[derive(Deserialize, Clone, Copy, Default)]
#[serde(default)]
pub struct SecretTargets {
    api_key: bool,
    client_secret: bool,
    refresh_token: bool,
}

/// 実質空（None / `Some("")` / 空 String）は未設定とみなす。
fn has_secret_value(secret: Option<&str>) -> bool {
    secret.is_some_and(|value| !value.is_empty())
}

/// 指定されたシークレットだけを消す純粋ヘルパー。
/// `merge_secrets` を通すと消した値が復元されるため、ここでは使わない。
/// 実際に消した対象名だけを返す（空なら no-op）。値・パスは含めない。
fn clear_targeted_secrets(config: &mut AppConfig, targets: SecretTargets) -> Vec<&'static str> {
    let mut cleared = Vec::new();
    if targets.api_key && has_secret_value(config.ai.openai.api_key.as_deref()) {
        config.ai.openai.api_key = None;
        cleared.push("api_key");
    }
    if let Some(drive) = config.storage.google_drive.as_mut() {
        if targets.client_secret && has_secret_value(Some(drive.client_secret.as_str())) {
            drive.client_secret.clear();
            cleared.push("client_secret");
        }
        if targets.refresh_token && has_secret_value(drive.refresh_token.as_deref()) {
            drive.refresh_token = None;
            cleared.push("refresh_token");
        }
    }
    cleared
}

/// 指定されたシークレットだけを消す（他設定は維持）。
#[command]
pub async fn clear_secrets(
    state: State<'_, Arc<AppState>>,
    targets: SecretTargets,
) -> Result<ConfigView, String> {
    if let Some(message) = load_failure_message(state.inner()) {
        return Err(message);
    }
    let (mut config, expected) = state.config_with_revision();
    let cleared = clear_targeted_secrets(&mut config, targets);
    if cleared.is_empty() {
        // 消す値がなければログ・ディスク書込み・再初期化は不要（no-op）。
        let (current, revision) = state.config_with_revision();
        return Ok(ConfigView::from_state(&current, revision));
    }
    // merge_secrets は意図的に通さない。通すと消した値が現在値として復元される。
    // 共有コミット経路にも merge は入れていないため、消した値は復元されない。
    commit_validated_config(state.inner(), expected, config, true)
        .await
        .map_err(ipc_error)?;
    // 保存成功後にのみ記録する。対象名だけを出力し、値やパスは出さない。
    log::info!("cleared secrets: {}", cleared.join(", "));
    let (current, revision) = state.config_with_revision();
    Ok(ConfigView::from_state(&current, revision))
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

    /// API キー以外の設定（含む DTO 外フィールド）が不変であること。
    #[test]
    fn config_with_api_key_replaces_only_the_key() {
        let current = config_with_dead_values();
        let updated = config_with_api_key(&current, "sk-replacement-for-tests".to_string());

        let mut expected = serde_json::to_value(&current).unwrap();
        expected["ai"]["openai"]["api_key"] = serde_json::json!("sk-replacement-for-tests");
        assert_eq!(serde_json::to_value(&updated).unwrap(), expected);
    }

    /// 空文字は未設定として扱い、変更検知に反映されること。
    #[test]
    fn config_with_api_key_treats_empty_as_unset() {
        let current = config_with_secrets();
        let updated = config_with_api_key(&current, String::new());

        assert!(updated.ai.openai.api_key.is_none());
        assert!(ai_or_lint_changed(&current, &updated));
    }

    /// 返却 view に秘密値が出ないこと（value は has_api_key フラグのみ）。
    #[test]
    fn set_api_key_result_view_never_exposes_the_value() {
        let current = config_with_secrets();
        let config = config_with_api_key(&current, "sk-do-not-expose-in-view".to_string());
        let json = serde_json::to_string(&ConfigView::from_state(&config, 3)).unwrap();

        assert!(!json.contains("sk-do-not-expose-in-view"));
        assert!(json.contains("\"has_api_key\":true"));
        assert!(json.contains("\"revision\":3"));
    }

    #[test]
    fn config_view_exposes_revision_for_cas_retry() {
        let view = ConfigView::from_state(&AppConfig::default(), 7);
        let json = serde_json::to_value(&view).unwrap();
        assert_eq!(json["revision"], 7);
    }

    /// ConfigView の JSON キー集合が web 側 DTO と同じであることを desktop 側でも検証する。
    /// フィールド名の追加・削除・改名はシェルゲートとこの test の両方で検出される。
    #[test]
    fn config_view_json_key_set_matches_web_dto_contract() {
        use std::collections::BTreeSet;

        fn assert_keys(value: &serde_json::Value, expected: &[&str]) {
            let actual: BTreeSet<&str> = value
                .as_object()
                .expect("DTO は JSON オブジェクトであること")
                .keys()
                .map(String::as_str)
                .collect();
            let expected: BTreeSet<&str> = expected.iter().copied().collect();
            assert_eq!(actual, expected);
        }

        let mut config = AppConfig::default();
        config.storage.local_roots.push(LocalRootConfig {
            id: "root-1".to_string(),
            name: "library".to_string(),
            path: PathBuf::from("/dummy/path"),
            read_only: true,
            watch: false,
        });
        let json = serde_json::to_value(ConfigView::from_state(&config, 1)).unwrap();

        assert_keys(
            &json,
            &[
                "revision", "general", "ai", "lint", "storage", "editor", "reader",
            ],
        );
        assert_keys(
            &json["general"],
            &["language", "theme", "check_updates", "log_level"],
        );
        assert_keys(
            &json["ai"],
            &[
                "enabled",
                "default_provider",
                "openai",
                "ollama",
                "max_tokens_per_request",
                "prefer_local",
                "timeout_secs",
                "streaming",
            ],
        );
        assert_keys(
            &json["ai"]["openai"],
            &["base_url", "has_api_key", "model", "organization"],
        );
        assert_keys(&json["ai"]["ollama"], &["base_url", "model"]);
        assert_keys(
            &json["lint"],
            &[
                "enabled",
                "enable_rules",
                "disable_rules",
                "auto_fix",
                "warnings_as_errors",
            ],
        );
        assert_keys(&json["storage"], &["local_roots", "google_drive"]);
        assert_keys(
            &json["storage"]["local_roots"][0],
            &["id", "name", "read_only"],
        );
        assert_keys(
            &json["editor"],
            &[
                "font_family",
                "font_size",
                "line_height",
                "tab_size",
                "insert_spaces",
                "line_numbers",
                "word_wrap",
                "minimap",
                "vim_mode",
                "match_brackets",
                "auto_close_brackets",
                "indent_with_tabs",
            ],
        );
        assert_keys(
            &json["reader"],
            &[
                "font_family",
                "font_size",
                "line_height",
                "theme",
                "column_width",
                "scroll_mode",
                "hyphenation",
                "vertical_writing",
                "show_page_numbers",
                "remember_position",
            ],
        );
    }

    #[test]
    fn config_patch_requires_expected_and_ignores_revision_from_view_json() {
        let current = AppConfig::default();
        let mut json = serde_json::to_value(ConfigView::from_state(&current, 9)).unwrap();

        // ConfigView.revision は expected ではないため、そのままでは必須フィールド不足。
        let missing = serde_json::from_value::<ConfigPatch>(json.clone());
        assert!(missing.is_err(), "expected が欠けた DTO は拒否されること");

        // expected を明示すれば revision は extra フィールドとして無視される。
        json.as_object_mut()
            .unwrap()
            .insert("expected".to_string(), serde_json::json!(9));
        let patch: ConfigPatch = serde_json::from_value(json).unwrap();
        assert_eq!(patch.expected, 9);
        let merged = merge_secrets(&current, &patch.apply_to(&current));
        assert_eq!(
            serde_json::to_value(&merged).unwrap(),
            serde_json::to_value(&current).unwrap()
        );
    }

    #[test]
    fn ai_or_lint_changed_detects_ai_updates() {
        let current = AppConfig::default();

        let mut enabled = current.clone();
        enabled.ai.enabled = true;
        assert!(ai_or_lint_changed(&current, &enabled));

        let mut provider = current.clone();
        provider.ai.default_provider = "ollama".to_string();
        assert!(ai_or_lint_changed(&current, &provider));
    }

    #[test]
    fn ai_or_lint_changed_detects_lint_updates() {
        let current = AppConfig::default();

        let mut toggled = current.clone();
        toggled.lint.enabled = false;
        assert!(ai_or_lint_changed(&current, &toggled));

        let mut rules = current.clone();
        rules.lint.disable_rules.push("MD999".to_string());
        assert!(ai_or_lint_changed(&current, &rules));
    }

    #[test]
    fn ai_or_lint_changed_ignores_other_sections() {
        let current = AppConfig::default();
        let mut merged = current.clone();
        merged.general.theme = "dark".to_string();
        merged.editor.font_size = 20.0;
        merged.reader.theme = "sepia".to_string();
        merged.storage.max_recent = 42;

        assert!(!ai_or_lint_changed(&current, &merged));
        assert!(!ai_or_lint_changed(&current, &current));
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
    fn ai_config_view_json_excludes_max_retries() {
        let mut config = AppConfig::default();
        config.ai.max_retries = 5;
        let json = serde_json::to_value(AiConfigView::from(&config.ai)).unwrap();

        // max_retries は DTO に出さない（JS 往復で消えるため現在値は merge_secrets で保持）。
        assert!(json.get("max_retries").is_none());
    }

    #[test]
    fn merge_secrets_preserves_max_retries() {
        let mut current = AppConfig::default();
        current.ai.max_retries = 5;
        let mut incoming = current.clone();
        incoming.ai.max_retries = 3;

        let merged = merge_secrets(&current, &incoming);

        // incoming が 3 でも current の 5 を保持すること。
        assert_eq!(merged.ai.max_retries, 5);
    }

    #[test]
    fn merge_secrets_preserves_check_updates_value() {
        // ADR-0222 / ADR-0277: updater 未実装の死設定。UI は固定表示だが DTO 往復で
        // 値が届くため、JS が false へ書き換えても current の true を保持すること。
        let mut current = AppConfig::default();
        current.general.check_updates = true;
        let mut incoming = current.clone();
        incoming.general.check_updates = false;

        let merged = merge_secrets(&current, &incoming);

        assert!(merged.general.check_updates);
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

    /// `update_config` 相当の正規化＋マージで、JS が DTO 外に注入した値が保存されないこと。
    #[test]
    fn update_config_normalization_drops_js_injected_hidden_fields() {
        let mut current = config_with_paths();
        current.ai.openai.extra_headers.insert(
            "X-Real-Header".to_string(),
            "real-value-from-config-toml".to_string(),
        );

        // JS が完全な AppConfig に詰めて送ってきた想定の生入力。
        let mut raw = current.clone();
        raw.ai.openai.extra_headers.insert(
            "X-Js-Injected".to_string(),
            "attacker-controlled".to_string(),
        );
        raw.ai.openai.api_key = Some("attacker-key".to_string());
        raw.general.data_dir = Some(PathBuf::from("/js/injected/data"));
        raw.storage
            .recent_paths
            .push(PathBuf::from("/js/injected/book.epub"));

        // 正規化単体で DTO 外フィールドが落ちること。
        let normalized = normalize_to_dto_boundary(&raw, &current);
        assert!(normalized.ai.openai.extra_headers.is_empty());
        assert!(normalized.ai.openai.api_key.is_none());
        assert!(normalized.general.data_dir.is_none());
        assert!(normalized.storage.recent_paths.is_empty());
        // 公開フィールドは allow-list を通して保持されること。
        assert_eq!(normalized.general.theme, raw.general.theme);
        assert_eq!(normalized.ai.openai.base_url, raw.ai.openai.base_url);

        let merged = merge_secrets(&current, &normalized);

        // 注入ヘッダは保存されず、config.toml 側の実ヘッダだけが残る。
        assert_eq!(
            merged.ai.openai.extra_headers,
            current.ai.openai.extra_headers
        );
        assert!(!merged.ai.openai.extra_headers.contains_key("X-Js-Injected"));
        assert_eq!(merged.ai.openai.api_key, current.ai.openai.api_key);
        assert_eq!(merged.general.data_dir, current.general.data_dir);
        assert_eq!(merged.storage.recent_paths, current.storage.recent_paths);
    }

    /// `update_ai_config` 相当の正規化＋マージで、JS が送った extra_headers が
    /// 保存されず現在値が残ること。api_key の明示的新値は受け入れられること。
    #[test]
    fn update_ai_normalization_drops_js_headers_but_accepts_new_api_key() {
        let mut current = config_with_secrets();
        current.ai.openai.extra_headers.insert(
            "X-Real-Header".to_string(),
            "real-value-from-config-toml".to_string(),
        );

        // JS が生 AiConfig に詰めて送ってきた想定の入力。
        let mut js_ai = current.ai.clone();
        js_ai.openai.extra_headers.insert(
            "X-Js-Injected".to_string(),
            "attacker-controlled".to_string(),
        );
        js_ai.openai.api_key = Some("sk-explicit-new-key".to_string());

        let normalized = normalize_ai_from_js(js_ai);
        assert!(normalized.openai.extra_headers.is_empty());
        assert_eq!(
            normalized.openai.api_key.as_deref(),
            Some("sk-explicit-new-key")
        );

        let mut incoming = current.clone();
        incoming.ai = normalized;
        let merged = merge_secrets(&current, &incoming);

        // 注入ヘッダは保存されず、config.toml 側の実ヘッダだけが残る。
        assert_eq!(
            merged.ai.openai.extra_headers,
            current.ai.openai.extra_headers
        );
        assert!(!merged.ai.openai.extra_headers.contains_key("X-Js-Injected"));
        // api_key は ADR-0098 の契約どおり明示的新値が採用される。
        assert_eq!(
            merged.ai.openai.api_key.as_deref(),
            Some("sk-explicit-new-key")
        );
    }

    /// マスク値キーは現在値を復元し、それ以外のキーは incoming を採用すること。
    #[test]
    fn merge_secrets_restores_masked_header_keys_and_keeps_other_updates() {
        let mut current = config_with_secrets();
        current
            .ai
            .openai
            .extra_headers
            .insert("X-Kept".to_string(), "current-value".to_string());
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
            .insert("X-Kept".to_string(), REDACTED_SECRET.to_string());
        incoming
            .ai
            .openai
            .extra_headers
            .insert("X-New".to_string(), "new-value".to_string());
        // 現在側に存在しないマスクキーは sentinel を保存しないため破棄される。
        incoming
            .ai
            .openai
            .extra_headers
            .insert("X-Unknown".to_string(), REDACTED_SECRET.to_string());

        let merged = merge_secrets(&current, &incoming);

        assert_eq!(merged.ai.openai.extra_headers.len(), 2);
        assert_eq!(
            merged
                .ai
                .openai
                .extra_headers
                .get("X-Kept")
                .map(String::as_str),
            Some("current-value")
        );
        assert_eq!(
            merged
                .ai
                .openai
                .extra_headers
                .get("X-New")
                .map(String::as_str),
            Some("new-value")
        );
        assert!(!merged.ai.openai.extra_headers.contains_key("X-Old"));
        assert!(!merged.ai.openai.extra_headers.contains_key("X-Unknown"));
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
            "\"check_updates\"",
            "\"log_level\"",
            "\"enable_rules\"",
            "\"disable_rules\"",
            "\"auto_fix\"",
            "\"warnings_as_errors\"",
            "\"id\"",
            "\"name\"",
            "\"read_only\"",
            "\"model\"",
        ] {
            assert!(json.contains(kept), "missing field: {kept}");
        }
    }

    /// 未配線の死設定を非既定値で持つ config.toml 相当の設定を作る。
    fn config_with_dead_values() -> AppConfig {
        let mut config = config_with_paths();
        config.general.telemetry = true;
        config.ai.max_tokens_per_month = Some(1234);
        config.ai.ollama.keep_alive_minutes = 17;
        config
            .ai
            .model_overrides
            .insert("custom".to_string(), "alias".to_string());
        config.lint.config_file = Some(PathBuf::from("/opt/lint.toml"));
        config.storage.default_backend = "custom".to_string();
        config.storage.max_recent = 42;
        config.editor.auto_save_delay = 999;
        config
    }

    /// ADR-0304: 死設定 9 件が ConfigView の JSON に一切出ないこと。
    #[test]
    fn config_view_json_excludes_dead_fields() {
        let json = serde_json::to_string(&ConfigView::from(&config_with_dead_values())).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let mut keys = Vec::new();
        collect_json_keys(&value, &mut keys);

        for forbidden in [
            "telemetry",
            "max_tokens_per_month",
            "keep_alive_minutes",
            "model_overrides",
            "config_file",
            "default_backend",
            "max_recent",
            "auto_save_delay",
            "watch",
        ] {
            assert!(
                !keys.iter().any(|key| key == forbidden),
                "leaked dead field in ConfigView: {forbidden}"
            );
        }
    }

    /// ADR-0304: DTO 往復で dead 値がプレースホルダ化されても、
    /// merge_secrets() が config.toml から読んだ現在値を復元し、
    /// 保存される AppConfig の値が変わらないこと。
    #[test]
    fn config_patch_round_trip_preserves_dead_config_values() {
        let current = config_with_dead_values();
        let mut json = serde_json::to_value(ConfigView::from(&current)).unwrap();
        json.as_object_mut()
            .unwrap()
            .insert("expected".to_string(), serde_json::json!(0));
        let patch: ConfigPatch = serde_json::from_value(json).unwrap();

        let applied = patch.apply_to(&current);
        // apply_to 単体では dead 値がプレースホルダ／既定値になる。
        assert!(!applied.general.telemetry);
        assert_eq!(applied.ai.max_tokens_per_month, None);
        assert_eq!(applied.ai.ollama.keep_alive_minutes, 0);
        assert!(applied.ai.model_overrides.is_empty());
        assert!(applied.lint.config_file.is_none());
        assert!(applied.storage.default_backend.is_empty());
        assert_eq!(applied.storage.max_recent, 0);
        assert_eq!(applied.editor.auto_save_delay, 0);
        for root in &applied.storage.local_roots {
            assert!(!root.watch);
        }

        let merged = merge_secrets(&current, &applied);
        // config.toml へ保存される値（AppConfig の serde 表現）が不変であること。
        assert_eq!(
            serde_json::to_value(&merged).unwrap(),
            serde_json::to_value(&current).unwrap()
        );
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
        // 新規 id は現在値がないため空 PathBuf に統一する。
        incoming.storage.local_roots.push(LocalRootConfig {
            id: "root-new".to_string(),
            name: "New".to_string(),
            path: PathBuf::from("/incoming/new"),
            read_only: false,
            watch: false,
        });
        // id が空のルートもマージ対象外（空 PathBuf に統一）。
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
        assert!(merged.storage.local_roots[2].path.as_os_str().is_empty());
        assert!(merged.storage.local_roots[3].path.as_os_str().is_empty());
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

    /// R3-AU01: id 一意一致時は `read_only` / `watch` も現在値へ復元し、
    /// incoming の false / true（昇格・監視変更）を無視すること。
    #[test]
    fn merge_secrets_restores_local_root_flags_by_id() {
        let current = config_with_paths();
        let current_root = &current.storage.local_roots[1];
        // root-b は read_only=true / watch=false。
        assert!(current_root.read_only);
        assert!(!current_root.watch);

        let mut incoming = current.clone();
        incoming.storage.local_roots[1].path.clear();
        // 攻撃者の送入: read_only を false へ、watch を true へ書き換える。
        incoming.storage.local_roots[1].read_only = false;
        incoming.storage.local_roots[1].watch = true;

        let merged = merge_secrets(&current, &incoming);

        let root = &merged.storage.local_roots[1];
        assert_eq!(root.path, current_root.path);
        assert!(root.read_only, "read_only は現在値 true を保持すること");
        assert!(!root.watch, "watch は現在値 false を保持すること");
    }

    /// R3-AU01: incoming 側で同一 id が重複した group は path を空にし、
    /// flags を現在値へ復元しない（incoming の値をそのまま残す）こと。
    #[test]
    fn merge_secrets_ignores_duplicate_incoming_local_root_ids() {
        let current = config_with_paths();
        let current_root = &current.storage.local_roots[1]; // root-b: true / false

        let mut incoming = current.clone();
        // 重複 group の双方で path を空にし、flags を現在値と逆へ書き換える。
        incoming.storage.local_roots[1].path.clear();
        incoming.storage.local_roots[1].read_only = false;
        incoming.storage.local_roots[1].watch = true;
        let duplicate = incoming.storage.local_roots[1].clone();
        incoming.storage.local_roots.push(duplicate);

        let merged = merge_secrets(&current, &incoming);

        // 一意な root-a は従来どおり path / flags が現在値へ復元される。
        let unique = &merged.storage.local_roots[0];
        assert_eq!(unique.path, current.storage.local_roots[0].path);
        assert_eq!(unique.read_only, current.storage.local_roots[0].read_only);
        assert_eq!(unique.watch, current.storage.local_roots[0].watch);

        // 重複 group は path が空で、flags は incoming（現在値と逆）のまま。
        for index in [1, 2] {
            let root = &merged.storage.local_roots[index];
            assert!(
                root.path.as_os_str().is_empty(),
                "重複 id の path は空になること"
            );
            assert_eq!(
                root.read_only,
                incoming.storage.local_roots[index].read_only
            );
            assert_eq!(root.watch, incoming.storage.local_roots[index].watch);
            assert_ne!(root.read_only, current_root.read_only);
            assert_ne!(root.watch, current_root.watch);
        }
    }

    /// R4: 現在側に存在しない新規 id は、現在側の flags を継承しない。
    #[test]
    fn merge_secrets_does_not_inherit_flags_for_new_local_root_ids() {
        let current = config_with_paths();
        let mut incoming = current.clone();
        // 先頭を新規 id に置き換える。現在側 root-a は false / true。
        incoming.storage.local_roots[0].id = "new-root".to_string();
        incoming.storage.local_roots[0].name = "New".to_string();
        incoming.storage.local_roots[0].path = PathBuf::from("/incoming/new");
        incoming.storage.local_roots[0].read_only = false;
        incoming.storage.local_roots[0].watch = true;

        let merged = merge_secrets(&current, &incoming);
        let root = &merged.storage.local_roots[0];

        assert!(root.path.as_os_str().is_empty());
        assert!(!root.read_only, "新規 id は現在値を継承しないこと");
        assert!(root.watch, "新規 id は incoming の値を維持すること");
    }

    /// R4: 順序変更と name 変更は incoming のまま、path / flags は id に追随する。
    #[test]
    fn merge_secrets_preserves_local_root_order_and_name() {
        let current = config_with_paths();
        let mut incoming = current.clone();
        incoming.storage.local_roots.swap(0, 1);
        incoming.storage.local_roots[0].name = "Renamed B".to_string();
        incoming.storage.local_roots[1].name = "Renamed A".to_string();
        for root in &mut incoming.storage.local_roots {
            root.path.clear();
            root.read_only = false;
            root.watch = false;
        }

        let merged = merge_secrets(&current, &incoming);
        let roots = &merged.storage.local_roots;

        assert_eq!(roots[0].id, "root-b");
        assert_eq!(roots[0].name, "Renamed B");
        assert_eq!(roots[0].path, PathBuf::from("/secret/root-b"));
        assert!(roots[0].read_only);
        assert!(!roots[0].watch);
        assert_eq!(roots[1].id, "root-a");
        assert_eq!(roots[1].name, "Renamed A");
        assert_eq!(roots[1].path, PathBuf::from("/secret/root-a"));
        assert!(!roots[1].read_only);
        assert!(roots[1].watch);
    }

    /// 現在側で id が重複した場合は incoming 側が一意でも復元しない（既存ロジックの維持）。
    #[test]
    fn merge_secrets_skips_current_side_duplicate_local_root_flags() {
        let mut current = config_with_paths();
        current.storage.local_roots.push(LocalRootConfig {
            id: "root-a".to_string(),
            name: "Duplicate".to_string(),
            path: PathBuf::from("/secret/duplicate"),
            read_only: true,
            watch: false,
        });
        // incoming は root-a / root-b のみで id は一意。
        let mut incoming = config_with_paths();
        incoming.storage.local_roots[0].path.clear();
        // 現在値は false / true。復元されれば false / true に上書きされる。
        incoming.storage.local_roots[0].read_only = true;
        incoming.storage.local_roots[0].watch = false;

        let merged = merge_secrets(&current, &incoming);

        let root = &merged.storage.local_roots[0];
        assert!(
            root.path.as_os_str().is_empty(),
            "現在側重複では path を復元しないこと"
        );
        assert!(root.read_only, "現在側重複では flags を復元しないこと");
        assert!(!root.watch, "現在側重複では flags を復元しないこと");
    }

    #[test]
    fn config_patch_apply_to_emits_placeholders_for_hidden_fields() {
        let current = config_with_paths();
        let applied = ConfigPatch::from((&current, 0)).apply_to(&current);

        // DTO 外のフィールドはすべてプレースホルダ／既定値になる。
        assert!(!applied.general.telemetry);
        assert_eq!(applied.ai.max_tokens_per_month, None);
        assert_eq!(applied.ai.ollama.keep_alive_minutes, 0);
        assert!(applied.storage.default_backend.is_empty());
        assert_eq!(applied.storage.max_recent, 0);
        assert_eq!(applied.editor.auto_save_delay, 0);
        assert!(applied.general.data_dir.is_none());
        assert!(applied.general.cache_dir.is_none());
        assert_eq!(applied.ai.max_retries, 0);
        assert!(applied.ai.model_overrides.is_empty());
        assert!(applied.ai.openai.api_key.is_none());
        assert!(applied.ai.openai.extra_headers.is_empty());
        assert!(applied.ai.ollama.options.is_empty());
        assert!(applied.lint.config_file.is_none());
        assert!(applied.lint.rule_config.is_empty());
        assert!(applied.storage.recent_paths.is_empty());
        for root in &applied.storage.local_roots {
            assert!(root.path.as_os_str().is_empty());
            assert!(!root.watch);
        }
        let drive = applied.storage.google_drive.as_ref().unwrap();
        assert!(drive.client_secret.is_empty());
        assert!(drive.refresh_token.is_none());
    }

    #[test]
    fn config_patch_preserves_hidden_fields() {
        let current = config_with_paths();
        // apply_to は DTO 外を空にし、merge_secrets() が現在値を復元する。
        let applied = ConfigPatch::from((&current, 0)).apply_to(&current);
        let merged = merge_secrets(&current, &applied);

        // apply_to 単体では隠しフィールドが空／既定値になること。
        assert!(applied.general.data_dir.is_none());
        assert!(applied.ai.openai.api_key.is_none());
        assert!(applied.ai.model_overrides.is_empty());

        // update_config_patch 経路ではパス系 / secret 系 / マップ系が現在値のまま。
        assert_eq!(merged.general.data_dir, current.general.data_dir);
        assert_eq!(merged.general.cache_dir, current.general.cache_dir);
        assert_eq!(merged.lint.config_file, current.lint.config_file);
        assert_eq!(merged.lint.rule_config, current.lint.rule_config);
        assert_eq!(merged.storage.recent_paths, current.storage.recent_paths);
        assert_eq!(merged.ai.ollama.options, current.ai.ollama.options);
        assert_eq!(merged.ai.model_overrides, current.ai.model_overrides);
        assert_eq!(merged.ai.max_retries, current.ai.max_retries);
        assert_eq!(merged.ai.openai.api_key, current.ai.openai.api_key);
        assert_eq!(
            merged.ai.openai.extra_headers,
            current.ai.openai.extra_headers
        );
        let drive = merged.storage.google_drive.as_ref().unwrap();
        let current_drive = current.storage.google_drive.as_ref().unwrap();
        assert_eq!(drive.client_secret, current_drive.client_secret);
        assert_eq!(drive.refresh_token, current_drive.refresh_token);
        for (actual, expected) in merged
            .storage
            .local_roots
            .iter()
            .zip(&current.storage.local_roots)
        {
            assert_eq!(actual.path, expected.path);
        }
    }

    #[test]
    fn config_patch_updates_public_fields() {
        let current = config_with_paths();
        let mut patch = ConfigPatch::from((&current, 0));
        patch.ai.enabled = true;
        patch.lint.enabled = false;
        patch.general.theme = "dark".to_string();
        patch.editor.font_size = 20.0;
        patch.reader.theme = "sepia".to_string();

        let merged = merge_secrets(&current, &patch.apply_to(&current));

        assert!(merged.ai.enabled);
        assert!(!merged.lint.enabled);
        assert_eq!(merged.general.theme, "dark");
        assert_eq!(merged.editor.font_size, 20.0);
        assert_eq!(merged.reader.theme, "sepia");
        // 公開フィールドを変えても merge_secrets() で隠しフィールドは維持される。
        assert_eq!(merged.ai.openai.api_key, current.ai.openai.api_key);
        assert_eq!(merged.general.data_dir, current.general.data_dir);
        assert_eq!(
            merged.storage.local_roots[0].path,
            current.storage.local_roots[0].path
        );
    }

    #[test]
    fn config_patch_from_view_json_restores_paths_by_id() {
        let current = config_with_paths();
        // Web が get_config の DTO に expected を添えて送り返すケースを模擬する。
        let mut json = serde_json::to_value(ConfigView::from(&current)).unwrap();
        json.as_object_mut()
            .unwrap()
            .insert("expected".to_string(), serde_json::json!(0));
        let mut patch: ConfigPatch = serde_json::from_value(json).unwrap();
        patch.storage.local_roots.push(LocalRootView {
            id: "root-new".to_string(),
            name: "New".to_string(),
            read_only: false,
        });
        patch.storage.local_roots.push(LocalRootView {
            id: String::new(),
            name: "NoId".to_string(),
            read_only: false,
        });

        let merged = merge_secrets(&current, &patch.apply_to(&current));

        assert_eq!(merged.ai.openai.api_key, current.ai.openai.api_key);
        assert_eq!(merged.general.data_dir, current.general.data_dir);
        assert_eq!(merged.storage.recent_paths, current.storage.recent_paths);
        assert_eq!(
            merged.storage.local_roots[0].path,
            PathBuf::from("/secret/root-a")
        );
        assert_eq!(
            merged.storage.local_roots[1].path,
            PathBuf::from("/secret/root-b")
        );
        // 不一致 / 空 id の新規ルートは incoming（空）のまま残る。
        assert!(merged.storage.local_roots[2].path.as_os_str().is_empty());
        assert!(merged.storage.local_roots[3].path.as_os_str().is_empty());
    }

    #[test]
    fn config_patch_apply_to_leaves_local_root_paths_empty() {
        let current = config_with_paths();
        let patch = ConfigPatch::from((&current, 0));

        let applied = patch.apply_to(&current);

        // 既存 id でも apply_to は空 path を返す。復元は merge_secrets() の責務。
        for root in &applied.storage.local_roots {
            assert!(root.path.as_os_str().is_empty());
        }
        let merged = merge_secrets(&current, &applied);
        assert_eq!(
            merged.storage.local_roots[0].path,
            PathBuf::from("/secret/root-a")
        );
    }

    #[test]
    fn clear_targeted_secrets_only_removes_api_key() {
        let original = config_with_secrets();
        let mut config = original.clone();

        let cleared = clear_targeted_secrets(
            &mut config,
            SecretTargets {
                api_key: true,
                ..Default::default()
            },
        );

        assert_eq!(cleared, vec!["api_key"]);
        assert!(config.ai.openai.api_key.is_none());
        let original_drive = original.storage.google_drive.as_ref().unwrap();
        let drive = config.storage.google_drive.as_ref().unwrap();
        assert_eq!(drive.client_secret, original_drive.client_secret);
        assert_eq!(drive.refresh_token, original_drive.refresh_token);

        // 他設定が不変であることを JSON 全体で確認する。
        let mut expected = original;
        expected.ai.openai.api_key = None;
        assert_eq!(
            serde_json::to_value(&config).unwrap(),
            serde_json::to_value(&expected).unwrap()
        );
        assert!(!ConfigView::from(&config).ai.openai.has_api_key);
    }

    #[test]
    fn clear_targeted_secrets_all_false_is_noop() {
        let original = config_with_secrets();
        let mut config = original.clone();

        let cleared = clear_targeted_secrets(&mut config, SecretTargets::default());

        assert!(cleared.is_empty());
        assert_eq!(
            serde_json::to_value(&config).unwrap(),
            serde_json::to_value(&original).unwrap()
        );
    }

    #[test]
    fn clear_targeted_secrets_without_google_drive_does_not_panic() {
        let mut config = config_with_secrets();
        config.storage.google_drive = None;

        // drive だけ指定して drive が None なら no-op（panic せず、更新なし）。
        let cleared = clear_targeted_secrets(
            &mut config,
            SecretTargets {
                client_secret: true,
                refresh_token: true,
                ..Default::default()
            },
        );
        assert!(cleared.is_empty());
        assert!(config.storage.google_drive.is_none());
        assert!(config.ai.openai.api_key.is_some());

        // api_key も指定されていれば、そちらだけ消える。
        let cleared = clear_targeted_secrets(
            &mut config,
            SecretTargets {
                api_key: true,
                client_secret: true,
                refresh_token: true,
            },
        );
        assert_eq!(cleared, vec!["api_key"]);
        assert!(config.ai.openai.api_key.is_none());
        assert!(config.storage.google_drive.is_none());
    }

    #[test]
    fn clear_targeted_secrets_removes_drive_secrets_only() {
        let original = config_with_secrets();
        let mut config = original.clone();

        let cleared = clear_targeted_secrets(
            &mut config,
            SecretTargets {
                client_secret: true,
                refresh_token: true,
                ..Default::default()
            },
        );

        assert_eq!(cleared, vec!["client_secret", "refresh_token"]);
        let original_drive = original.storage.google_drive.as_ref().unwrap();
        let drive = config.storage.google_drive.as_ref().unwrap();
        assert!(drive.client_secret.is_empty());
        assert!(drive.refresh_token.is_none());
        assert_eq!(config.ai.openai.api_key, original.ai.openai.api_key);

        // client_id / root_folder_id / use_app_data_folder / scopes は不変。
        assert_eq!(drive.client_id, original_drive.client_id);
        assert_eq!(drive.root_folder_id, original_drive.root_folder_id);
        assert_eq!(
            drive.use_app_data_folder,
            original_drive.use_app_data_folder
        );
        assert_eq!(drive.scopes, original_drive.scopes);
        // JSON 全体でもシークレット以外の変更がないことを確認する。
        let mut expected = original.clone();
        let expected_drive = expected.storage.google_drive.as_mut().unwrap();
        expected_drive.client_secret.clear();
        expected_drive.refresh_token = None;
        assert_eq!(
            serde_json::to_value(&config).unwrap(),
            serde_json::to_value(&expected).unwrap()
        );

        let view = ConfigView::from(&config);
        let drive_view = view.storage.google_drive.as_ref().unwrap();
        assert!(!drive_view.has_client_secret);
        assert!(!drive_view.has_refresh_token);
        assert!(view.ai.openai.has_api_key);
    }

    #[test]
    fn clear_targeted_secrets_already_empty_is_noop() {
        let mut config = config_with_secrets();
        // Some("") も実質空として扱う。
        config.ai.openai.api_key = Some(String::new());
        let drive = config.storage.google_drive.as_mut().unwrap();
        drive.client_secret.clear();
        drive.refresh_token = Some(String::new());
        let before = serde_json::to_value(&config).unwrap();

        // 指定されていても消す値がなければ空（保存・再初期化をしない）。
        let cleared = clear_targeted_secrets(
            &mut config,
            SecretTargets {
                api_key: true,
                client_secret: true,
                refresh_token: true,
            },
        );

        assert!(cleared.is_empty());
        assert_eq!(serde_json::to_value(&config).unwrap(), before);
    }

    #[test]
    fn clear_targeted_secrets_returns_api_key_name_only() {
        let mut config = config_with_secrets();

        let cleared = clear_targeted_secrets(
            &mut config,
            SecretTargets {
                api_key: true,
                ..Default::default()
            },
        );

        // 対象名だけを返し、値は含めない。
        assert_eq!(cleared, vec!["api_key"]);
        assert!(
            cleared
                .iter()
                .all(|name| !name.contains("sk-dummy-do-not-leak"))
        );
    }

    #[test]
    fn clear_targeted_secrets_returns_empty_when_no_targets() {
        let mut config = config_with_secrets();

        let cleared = clear_targeted_secrets(&mut config, SecretTargets::default());

        assert!(cleared.is_empty());
    }

    #[test]
    fn clear_targeted_secrets_returns_empty_for_drive_targets_without_drive() {
        let mut config = config_with_secrets();
        config.storage.google_drive = None;

        let drive_only = clear_targeted_secrets(
            &mut config,
            SecretTargets {
                client_secret: true,
                refresh_token: true,
                ..Default::default()
            },
        );
        assert!(drive_only.is_empty());

        let with_api_key = clear_targeted_secrets(
            &mut config,
            SecretTargets {
                api_key: true,
                client_secret: true,
                refresh_token: true,
            },
        );
        assert_eq!(with_api_key, vec!["api_key"]);
    }

    #[test]
    fn clear_targeted_secrets_returns_empty_for_already_empty_values() {
        let mut config = config_with_secrets();
        config.ai.openai.api_key = Some(String::new());
        let drive = config.storage.google_drive.as_mut().unwrap();
        drive.client_secret.clear();
        drive.refresh_token = Some(String::new());

        let cleared = clear_targeted_secrets(
            &mut config,
            SecretTargets {
                api_key: true,
                client_secret: true,
                refresh_token: true,
            },
        );

        assert!(cleared.is_empty());
    }

    #[test]
    fn clear_targeted_secrets_skips_empty_values_and_reports_real_ones() {
        let mut config = config_with_secrets();
        config.ai.openai.api_key = Some(String::new());
        let drive = config.storage.google_drive.as_mut().unwrap();
        drive.refresh_token = Some(String::new());

        let cleared = clear_targeted_secrets(
            &mut config,
            SecretTargets {
                api_key: true,
                client_secret: true,
                refresh_token: true,
            },
        );

        // 実質空は除外され、実値のある client_secret だけが報告される。
        assert_eq!(cleared, vec!["client_secret"]);
        assert_eq!(config.ai.openai.api_key.as_deref(), Some(""));
        assert_eq!(
            config
                .storage
                .google_drive
                .as_ref()
                .and_then(|drive| drive.refresh_token.as_deref()),
            Some("")
        );
    }

    #[test]
    fn clear_targeted_secrets_returns_names_in_stable_order() {
        let mut config = config_with_secrets();

        let cleared = clear_targeted_secrets(
            &mut config,
            SecretTargets {
                api_key: true,
                client_secret: true,
                refresh_token: true,
            },
        );

        assert_eq!(cleared, vec!["api_key", "client_secret", "refresh_token"]);
    }

    #[test]
    fn secret_targets_missing_fields_default_to_false() {
        // JSON で項目が欠けても消さない方向に倒れること（fail-closed）。
        let empty: SecretTargets = serde_json::from_str("{}").unwrap();
        assert!(!empty.api_key);
        assert!(!empty.client_secret);
        assert!(!empty.refresh_token);

        let partial: SecretTargets = serde_json::from_str(r#"{"api_key":true}"#).unwrap();
        assert!(partial.api_key);
        assert!(!partial.client_secret);
        assert!(!partial.refresh_token);
    }

    #[test]
    fn command_error_uses_fixed_code_messages_without_raw_values() {
        let cases = [
            (
                CoreError::Conflict("stale revision at /secret/path".into()),
                "CONFIG_CONFLICT",
                "Configuration conflict",
            ),
            (
                CoreError::validation("bad value sk-secret"),
                "CONFIG_VALIDATION",
                "Configuration validation failed",
            ),
            (
                CoreError::storage("/secret/path"),
                "CONFIG_STORAGE",
                "Configuration storage failed",
            ),
            (
                CoreError::internal("bug at /secret/path"),
                "CONFIG_INTERNAL",
                "Internal configuration error",
            ),
        ];

        for (core_error, code, message) in cases {
            let error = CommandError::from(core_error);
            assert_eq!(error.code, code);
            assert_eq!(error.message, message);
            // 生エラー・パス・設定値が fixed message に漏れていないこと。
            for leaked in ["/secret", "sk-secret", "stale revision"] {
                assert!(!error.message.contains(leaked), "leaked: {leaked}");
            }
        }

        let json =
            serde_json::to_value(CommandError::from(CoreError::Conflict("stale".into()))).unwrap();
        assert_eq!(json["code"], "CONFIG_CONFLICT");
        assert_eq!(json["message"], "Configuration conflict");
    }

    #[tokio::test]
    async fn update_config_patch_stale_expected_returns_conflict() {
        let state = AppState::new(AppConfig::default());
        // state の revision は 0 だが、UI が古い 1 を読んだ想定の patch。
        let patch = ConfigPatch::from((&AppConfig::default(), 1));

        let error = match update_config_patch_inner(&state, patch).await {
            Ok(_) => panic!("stale expected は Conflict になること"),
            Err(error) => error,
        };

        assert_eq!(error.code, "CONFIG_CONFLICT");
        assert_eq!(error.message, "Configuration conflict");
        assert_eq!(
            state.config_with_revision().1,
            0,
            "Conflict で revision は不変"
        );
    }

    /// 読み込み失敗状態では `update_config_patch` が固定文言の CommandError で
    /// 拒否され、revision も設定も進まないこと（R2-ER01）。
    #[tokio::test]
    async fn update_config_patch_load_failed_rejects_with_fixed_message() {
        let state = AppState::new_with_load_failure(AppConfig::default());
        let (current, expected) = state.config_with_revision();
        let mut patch = ConfigPatch::from((&current, expected));
        patch.general.theme = "dark".to_string();

        let error = match update_config_patch_inner(&state, patch).await {
            Ok(_) => panic!("読み込み失敗状態では保存が拒否されること"),
            Err(error) => error,
        };

        assert_eq!(error.code, "CONFIG_LOAD_FAILED");
        assert_eq!(error.message, CONFIG_LOAD_FAILED_MESSAGE);
        // 固定文言のみで、設定値やパスが漏れないこと。
        assert!(!error.message.contains("config.toml"));
        assert!(!error.message.contains("dark"));
        assert_eq!(state.config_with_revision().1, 0);
        assert_config_eq_for_commands(&state.config(), &AppConfig::default());
    }

    /// `String` を返す保存コマンドが使う拒否文言も、値・パスを含まない固定文言であること。
    #[test]
    fn load_failure_message_returns_fixed_text_without_values() {
        let state = AppState::new_with_load_failure(AppConfig::default());
        let message = load_failure_message(&state).expect("拒否文言が返ること");
        assert_eq!(message, CONFIG_LOAD_FAILED_MESSAGE);
        assert!(!message.contains("config.toml"));

        let healthy = AppState::new(AppConfig::default());
        assert!(load_failure_message(&healthy).is_none());
    }

    fn assert_config_eq_for_commands(actual: &AppConfig, expected: &AppConfig) {
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
    }

    /// 成功経路が触れる config.toml をテスト前後で保護する。
    /// 既存ファイルは復元し、無ければ削除して環境を汚さない。
    struct ConfigFileGuard {
        path: Option<PathBuf>,
        original: Option<Vec<u8>>,
    }

    impl ConfigFileGuard {
        fn new() -> Self {
            let path = directories::ProjectDirs::from("com", "onoderakeita", "openrustbooks")
                .map(|dirs| dirs.config_dir().join("config.toml"));
            let original = path.as_ref().and_then(|p| std::fs::read(p).ok());
            Self { path, original }
        }
    }

    impl Drop for ConfigFileGuard {
        fn drop(&mut self) {
            let Some(path) = &self.path else {
                return;
            };
            match &self.original {
                Some(bytes) => {
                    let _ = std::fs::write(path, bytes);
                }
                None => {
                    let _ = std::fs::remove_file(path);
                }
            }
        }
    }

    #[tokio::test]
    async fn update_config_patch_fresh_expected_succeeds_and_advances_revision() {
        let _guard = ConfigFileGuard::new();
        let state = AppState::new(AppConfig::default());
        let (current, expected) = state.config_with_revision();
        let mut patch = ConfigPatch::from((&current, expected));
        patch.general.theme = "dark".to_string();

        let view = update_config_patch_inner(&state, patch).await.unwrap();

        assert_eq!(view.revision, expected + 1);
        assert_eq!(state.config_with_revision().1, expected + 1);
        assert_eq!(state.config().general.theme, "dark");
    }
}
