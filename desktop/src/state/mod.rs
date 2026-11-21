//! Application state management.

use openrustbooks_core::{
    AppConfig, CoreError, CoreResult,
    ai_markdown::AiMarkdownService,
    document::{Document, DocumentId},
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 起動時の config.toml 読み込み失敗時に保存を拒否する固定文言。
///
/// 設定値・パス・生エラー文字列は一切含めない（R2-ER01 / fail-closed）。
pub const CONFIG_LOAD_FAILED_MESSAGE: &str = "Configuration load failed; saving is disabled";

/// 設定とその revision を同じロックで保持する（CAS の比較単位）。
struct ConfigState {
    config: AppConfig,
    revision: u64,
    /// 保存中のコミットがあるか。write guard を保存の外で解放するため、
    /// 同一 expected の二重通過を防ぐ予約フラグとして使う。
    pending: bool,
    /// 起動時に config.toml を読み込めなかったか。true の間は保存を拒否する。
    config_load_failed: bool,
}

/// Application state shared across commands
pub struct AppState {
    config: RwLock<ConfigState>,
    documents: RwLock<HashMap<DocumentId, Document>>,
    ai_service: RwLock<Option<Arc<AiMarkdownService>>>,
    unsaved_changes: RwLock<HashMap<DocumentId, bool>>,
    /// `prepare_ai_service` の完了直後に呼ぶテスト専用フック。
    /// await を跨いで `commit_config` が入った状況を外部 AI API なしで再現する。
    /// インスタンスごとに持つため、並列テスト間で干渉しない。
    #[cfg(test)]
    test_after_prepare: std::sync::Mutex<Option<Box<dyn Fn() + Send + Sync>>>,
}

impl AppState {
    /// Create new application state（正常起動用）
    pub fn new(config: AppConfig) -> Self {
        Self::with_config_load_failed(config, false)
    }

    /// 設定読み込み失敗状態で状態を生成する。
    ///
    /// 既定値をメモリ上に保持するが、読み込み失敗状態が続く限り
    /// `commit_config` は fail-closed で保存を拒否する（R2-ER01）。
    pub fn new_with_load_failure(config: AppConfig) -> Self {
        Self::with_config_load_failed(config, true)
    }

    fn with_config_load_failed(config: AppConfig, config_load_failed: bool) -> Self {
        Self {
            config: RwLock::new(ConfigState {
                config,
                revision: 0,
                pending: false,
                config_load_failed,
            }),
            documents: RwLock::new(HashMap::new()),
            ai_service: RwLock::new(None),
            unsaved_changes: RwLock::new(HashMap::new()),
            #[cfg(test)]
            test_after_prepare: std::sync::Mutex::new(None),
        }
    }

    /// 起動時に config.toml の読み込みに失敗したか。
    pub fn config_load_failed(&self) -> bool {
        self.config.read().unwrap().config_load_failed
    }

    /// Get current configuration
    pub fn config(&self) -> AppConfig {
        self.config.read().unwrap().config.clone()
    }

    /// 現在の設定と revision を同一ロック・同一時点で取得する。
    ///
    /// 設定コマンドはここで得た `expected` を後続の `commit_config` へ渡す。
    pub fn config_with_revision(&self) -> (AppConfig, u64) {
        let state = self.config.read().unwrap();
        (state.config.clone(), state.revision)
    }

    /// 低レベル更新（revision を進めない）。既存のテスト用に残す。
    ///
    /// 設定コマンドの保存経路には使わず、CAS を含む `commit_config` を使うこと。
    #[cfg(test)]
    pub fn update_config(&self, config: AppConfig) -> CoreResult<()> {
        self.config.write().unwrap().config = config;
        Ok(())
    }

    /// 候補設定から AI service を準備する（状態を一切変更しない）。
    ///
    /// 無効設定なら `Ok(None)`、有効設定なら生成結果を `Ok(Some(...))` で返す。
    /// 生成失敗時は `Err` を返すだけなので、config / ディスク / service は不変。
    /// ロックは保持せず、生成は呼び出し側のコミット前に完了する。
    pub async fn prepare_ai_service(
        &self,
        config: &AppConfig,
    ) -> CoreResult<Option<Arc<AiMarkdownService>>> {
        if !config.ai.enabled {
            return Ok(None);
        }
        let service = AiMarkdownService::new(config).await?;
        Ok(Some(Arc::new(service)))
    }

    /// 設定と AI service を revision CAS でまとめてコミットする。
    ///
    /// `expected` が現在の revision と不一致なら `CoreError::Conflict` を返し、
    /// config / disk / service を一切変更しない。write guard は比較と予約だけの
    /// 短いクリティカルセクションで解放し、ディスク保存はロックの外で行う
    /// （`save_config()` は read guard を取るため、ここでは候補を直接保存する）。
    pub fn commit_config(
        &self,
        expected: u64,
        config: AppConfig,
        service: Option<Arc<AiMarkdownService>>,
    ) -> CoreResult<()> {
        // --- 短い write クリティカルセクション: 失敗状態確認・比較・予約のみ ---
        let next_revision = {
            let mut state = self.config.write().unwrap();
            // 起動時に config.toml を読み込めていない場合は、既定値での上書きを
            // 防ぐため一切保存しない（固定文言のみ返し、設定値は出さない）。
            if state.config_load_failed {
                return Err(CoreError::storage(CONFIG_LOAD_FAILED_MESSAGE));
            }
            // overflow は保存前に失敗させる（ADR 未決定のため fail-closed）。
            let next = expected
                .checked_add(1)
                .ok_or_else(|| CoreError::internal("configuration revision overflow"))?;
            if state.revision != expected || state.pending {
                return Err(CoreError::Conflict(
                    "Configuration changed since it was read".to_string(),
                ));
            }
            state.pending = true;
            next
        };

        // --- ロックを解放してから保存（await も guard 保持もしない） ---
        if let Err(save_error) = config.save() {
            // メモリはまだ変更していないため、旧 config / revision はそのまま。
            // 予約を保持したまま旧設定を読み、ディスクを再保存する（他コミットの
            // 割込みを防ぐ）。2 次失敗あり得る。
            let previous = self.config.read().unwrap().config.clone();
            if let Err(restore_error) = previous.save() {
                self.config.write().unwrap().pending = false;
                // エラーの表示文字列だけを連結し、設定値（シークレット含む）は出さない。
                return Err(CoreError::internal(format!(
                    "config save failed and rollback re-save also failed: {save_error}; {restore_error}"
                )));
            }
            self.config.write().unwrap().pending = false;
            return Err(save_error);
        }

        // --- 成功時のみ、短い guard で確定（revision 前進・service 置換） ---
        // ロックは config -> ai_service の順で取得し（逆順の箇所は無い）、
        // 設定と service を不可分に反映する。
        let mut state = self.config.write().unwrap();
        state.config = config;
        state.revision = next_revision;
        state.pending = false;
        *self.ai_service.write().unwrap() = service;
        Ok(())
    }

    /// Initialize AI service（起動時）。fail-closed の refresh 処理を共用する。
    pub async fn init_ai_service(&self) -> CoreResult<()> {
        self.refresh_ai_service().await
    }

    /// Get AI service if available
    ///
    /// 設定で無効な場合は service の有無に関わらず None を返す。
    /// ロックは逐次取得して重ねない（デッドロック回避）。
    pub fn ai_service(&self) -> Option<Arc<AiMarkdownService>> {
        if !self.is_ai_enabled() {
            return None;
        }
        self.ai_service.read().unwrap().clone()
    }

    /// Check if AI features are enabled
    pub fn is_ai_enabled(&self) -> bool {
        self.config.read().unwrap().config.ai.enabled
    }

    /// Clear the in-memory AI service (設定無効化時に呼ぶ)
    pub fn disable_ai_service(&self) {
        *self.ai_service.write().unwrap() = None;
    }

    /// 設定変更後に AI service を同期する（fail-closed）。
    ///
    /// 生成中は旧 service を残し、完了時に開始時点の revision を再確認する。
    /// await 中に `commit_config` が新しい config / service を確定していた場合は
    /// 生成結果を捨てて何もしない。生成失敗時も競合が無ければ service をクリアし、
    /// 「古い service は残らない」という fail-closed の契約を維持する。
    pub async fn refresh_ai_service(&self) -> CoreResult<()> {
        // 設定と revision を同じロック・同じ時点で読み、比較基準を固定する。
        let (config, expected) = self.config_with_revision();
        let prepared = self.prepare_ai_service(&config).await;

        #[cfg(test)]
        {
            if let Some(hook) = self.test_after_prepare.lock().unwrap().as_ref() {
                hook();
            }
        }

        match prepared {
            Ok(service) => {
                // read() で確認してから write() を取り直すと、その間に
                // commit_config が入り得る（TOCTOU）。write guard を保持した
                // まま revision を比較し、service 置換まで解放しない。
                // ロック順は commit_config と同じ config -> ai_service。
                // clippy は revision を読むだけと判定するが、書き込まないこと
                // 自体が CAS を壊さないための要件（意図的な排他）。
                #[allow(clippy::readonly_write_lock)]
                let state = self.config.write().unwrap();
                if state.revision != expected {
                    // 古い config から生成した service は捨て、commit 側を守る。
                    return Ok(());
                }
                // state（config の write guard）はこのスコープ末尾まで保持される。
                match service {
                    Some(service) => *self.ai_service.write().unwrap() = Some(service),
                    // 無効設定なら disable と同義。生成前ではなく確認後に呼ぶ。
                    None => self.disable_ai_service(),
                }
                Ok(())
            }
            Err(error) => {
                // 上と同じく、確認から disable までを commit_config と排他する。
                #[allow(clippy::readonly_write_lock)]
                let state = self.config.write().unwrap();
                if state.revision == expected {
                    // 競合が無ければ旧 service を残さない（fail-closed）。
                    // 競合していれば commit_config の service を守る。
                    self.disable_ai_service();
                }
                Err(error)
            }
        }
    }

    /// Add a document
    pub fn add_document(&self, document: Document) {
        let id = document.metadata.id.clone();
        self.documents.write().unwrap().insert(id, document);
    }

    /// Get a document
    pub fn get_document(&self, id: &DocumentId) -> Option<Document> {
        self.documents.read().unwrap().get(id).cloned()
    }

    /// Remove a document
    ///
    /// documents と unsaved_changes のロックは順番に取得・解放する（同時保持しない）。
    /// 文書が存在しない場合でも unsaved キーを削除し、孤立フラグを掃除する。
    pub fn remove_document(&self, id: &DocumentId) -> Option<Document> {
        let removed = self.documents.write().unwrap().remove(id);
        self.unsaved_changes.write().unwrap().remove(id);
        removed
    }

    /// List all documents
    pub fn list_documents(&self) -> Vec<Document> {
        self.documents.read().unwrap().values().cloned().collect()
    }

    /// Mark document as having unsaved changes
    pub fn mark_unsaved(&self, id: &DocumentId) {
        self.unsaved_changes
            .write()
            .unwrap()
            .insert(id.clone(), true);
    }

    /// Mark document as saved
    pub fn mark_saved(&self, id: &DocumentId) {
        self.unsaved_changes
            .write()
            .unwrap()
            .insert(id.clone(), false);
    }

    /// Check if document has unsaved changes
    pub fn is_unsaved(&self, id: &DocumentId) -> bool {
        self.unsaved_changes
            .read()
            .unwrap()
            .get(id)
            .copied()
            .unwrap_or(false)
    }

    /// Check if any document has unsaved changes
    pub fn has_unsaved_changes(&self) -> bool {
        self.unsaved_changes.read().unwrap().values().any(|&v| v)
    }

    /// Save configuration to disk（テスト用。commit は再ロック回避のため直接 save する）
    #[cfg(test)]
    pub fn save_config(&self) -> CoreResult<()> {
        self.config.read().unwrap().config.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openrustbooks_core::document::{DocumentContent, DocumentFormat, DocumentMetadata};
    use std::path::PathBuf;

    /// config.toml を触るテストは同じパスを共有するため直列化する。
    /// async テストは guard を await を跨いで保持するため tokio の Mutex を使う。
    static CONFIG_FILE_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    /// 同期テスト用（tokio runtime 外で呼ぶこと）。
    fn lock_config_file_blocking() -> tokio::sync::MutexGuard<'static, ()> {
        CONFIG_FILE_TEST_LOCK.blocking_lock()
    }

    fn saved_config_toml() -> Option<Vec<u8>> {
        directories::ProjectDirs::from("com", "onoderakeita", "openrustbooks")
            .and_then(|dirs| std::fs::read(dirs.config_dir().join("config.toml")).ok())
    }

    /// commit_config が触れる config.toml をテスト前後で保護する。
    /// 既存ファイルは内容を復元し、無ければ削除して環境を汚さない。
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

    /// AppConfig は PartialEq 未 derive のため JSON 表現で比較する。
    fn assert_config_eq(actual: &AppConfig, expected: &AppConfig) {
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
    }

    fn sample_document(id: &str) -> Document {
        Document {
            metadata: DocumentMetadata::new(
                DocumentId::from_string(id.to_string()),
                DocumentFormat::Txt,
            ),
            content: DocumentContent::Text("sample".to_string()),
        }
    }

    impl AppState {
        /// prepare 完了直後に任意の処理を挟む（テスト専用フック）。
        fn set_after_prepare_hook(&self, hook: impl Fn() + Send + Sync + 'static) {
            *self.test_after_prepare.lock().unwrap() = Some(Box::new(hook));
        }
    }

    /// 有効設定と ollama の service を revision を進めてコミットする。
    ///
    /// ollama は初期化時に HTTP 要求を出さないため、外部 AI API を使わない。
    async fn commit_enabled_service(
        state: &AppState,
        theme: &str,
    ) -> (AppConfig, Arc<AiMarkdownService>) {
        let (mut config, expected) = state.config_with_revision();
        config.ai.enabled = true;
        config.ai.default_provider = "ollama".to_string();
        config.general.theme = theme.to_string();
        let service = state
            .prepare_ai_service(&config)
            .await
            .unwrap()
            .expect("ollama 設定なら service が生成される");
        state
            .commit_config(expected, config.clone(), Some(service.clone()))
            .unwrap();
        (config, service)
    }

    #[test]
    fn unsaved_changes_state_transitions() {
        let state = AppState::new(AppConfig::default());
        let document = sample_document("doc-1");
        let id = document.metadata.id.clone();

        assert!(!state.has_unsaved_changes());

        state.add_document(document);
        assert!(state.get_document(&id).is_some());
        assert!(!state.is_unsaved(&id));

        state.mark_unsaved(&id);
        assert!(state.is_unsaved(&id));
        assert!(state.has_unsaved_changes());

        state.mark_saved(&id);
        assert!(!state.is_unsaved(&id));
        assert!(!state.has_unsaved_changes());

        assert!(state.remove_document(&id).is_some());
        assert!(state.get_document(&id).is_none());
        assert!(!state.has_unsaved_changes());
    }

    #[test]
    fn remove_document_clears_unsaved_flag() {
        let state = AppState::new(AppConfig::default());
        let document = sample_document("doc-remove-unsaved");
        let id = document.metadata.id.clone();

        state.add_document(document);
        state.mark_unsaved(&id);
        assert!(state.is_unsaved(&id));
        assert!(state.has_unsaved_changes());

        // force close 相当: mark_saved を挟まずに削除してもフラグは消える。
        assert!(state.remove_document(&id).is_some());
        assert!(state.get_document(&id).is_none());
        assert!(!state.is_unsaved(&id));
        assert!(!state.has_unsaved_changes());
    }

    #[test]
    fn remove_document_clears_orphaned_unsaved_flag() {
        let state = AppState::new(AppConfig::default());
        let orphan = sample_document("doc-orphan-unsaved");
        let id = orphan.metadata.id.clone();

        // documents に登録せずに unsaved フラグだけ立てる（孤立フラグ）。
        state.mark_unsaved(&id);
        assert!(state.is_unsaved(&id));
        assert!(state.has_unsaved_changes());

        // 文書が無くても remove_document は孤立フラグを掃除する。
        assert!(state.remove_document(&id).is_none());
        assert!(!state.is_unsaved(&id));
        assert!(!state.has_unsaved_changes());
    }

    #[test]
    fn config_round_trip_in_memory() {
        let state = AppState::new(AppConfig::default());
        assert!(!state.is_ai_enabled(), "AI must be off by default");
        let mut config = state.config();
        config.ai.enabled = true;
        state.update_config(config).unwrap();
        assert!(state.is_ai_enabled());
        assert!(state.config().ai.enabled);
    }

    #[test]
    fn save_config_writes_current_config() {
        let _test_lock = lock_config_file_blocking();
        let _guard = ConfigFileGuard::new();
        let state = AppState::new(AppConfig::default());
        let mut config = state.config();
        config.general.theme = "solarized".to_string();
        state.update_config(config).unwrap();

        state.save_config().unwrap();

        let bytes = saved_config_toml().expect("save_config が config.toml を書くこと");
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("solarized"));
    }

    #[test]
    fn ai_service_is_none_before_enabling() {
        let state = AppState::new(AppConfig::default());
        assert!(!state.is_ai_enabled());
        assert!(state.ai_service().is_none());
    }

    #[test]
    fn ai_service_is_none_when_update_config_disables_ai() {
        let state = AppState::new(AppConfig::default());
        let mut config = state.config();
        config.ai.enabled = true;
        state.update_config(config).unwrap();
        assert!(state.is_ai_enabled());

        // service が未生成でも、無効設定ならガードが None を返すことを確認。
        let mut config = state.config();
        config.ai.enabled = false;
        state.update_config(config).unwrap();
        assert!(!state.is_ai_enabled());
        assert!(state.ai_service().is_none());
    }

    #[test]
    fn disable_ai_service_clears_service() {
        let state = AppState::new(AppConfig::default());
        state.disable_ai_service();
        assert!(state.ai_service().is_none());
    }

    #[tokio::test]
    async fn refresh_ai_service_disabled_keeps_service_none() {
        let state = AppState::new(AppConfig::default());
        state.refresh_ai_service().await.unwrap();
        assert!(state.ai_service().is_none());
    }

    #[tokio::test]
    async fn refresh_ai_service_propagates_init_error_and_keeps_service_none() {
        let state = AppState::new(AppConfig::default());
        let mut config = state.config();
        config.ai.enabled = true;
        config.ai.default_provider = "unknown".to_string(); // create_client が即エラー
        state.update_config(config).unwrap();

        // 生成失敗は呼び出し側へ伝播し、service は None のまま（fail-closed）。
        assert!(state.refresh_ai_service().await.is_err());
        assert!(state.ai_service().is_none());
    }

    /// prepare 完了直後に commit_config が入っても、古い service で上書きしない
    /// こと（R2-BV01 の競合修正）。フックにより外部 AI API なしで再現する。
    #[tokio::test]
    async fn refresh_ai_service_discards_stale_service_when_config_is_committed_during_prepare() {
        let _test_lock = CONFIG_FILE_TEST_LOCK.lock().await;
        let _guard = ConfigFileGuard::new();
        let state = Arc::new(AppState::new(AppConfig::default()));

        let (_, old_service) = commit_enabled_service(&state, "dark").await;
        assert!(Arc::ptr_eq(&state.ai_service().unwrap(), &old_service));

        // refresh が読むのは revision 1 の config。その prepare 完了直後に
        // revision 2 をコミットするフックを仕込む（await 中の割込み相当）。
        let (mut replacement_config, commit_expected) = state.config_with_revision();
        replacement_config.general.theme = "light".to_string();
        let replacement_service = state
            .prepare_ai_service(&replacement_config)
            .await
            .unwrap()
            .expect("置換用 service が生成される");
        let state_for_hook = state.clone();
        let config_for_hook = replacement_config.clone();
        let service_for_hook = replacement_service.clone();
        state.set_after_prepare_hook(move || {
            state_for_hook
                .commit_config(
                    commit_expected,
                    config_for_hook.clone(),
                    Some(service_for_hook.clone()),
                )
                .unwrap();
        });

        // 古い config から生成した service を持っていても、競合を検出して
        // commit 側の config / service を守る。
        state.refresh_ai_service().await.unwrap();

        assert_eq!(state.config_with_revision().1, 2);
        assert_config_eq(&state.config(), &replacement_config);
        let current = state
            .ai_service()
            .expect("競合後も有効設定の service が残る");
        assert!(Arc::ptr_eq(&current, &replacement_service));
        assert!(!Arc::ptr_eq(&current, &old_service));
    }

    /// 生成失敗時も、await 中に commit_config が成功していればその service を
    /// 守ること（fail-closed 契約と競合安全性の両立）。
    #[tokio::test]
    async fn refresh_ai_service_keeps_committed_service_when_prepare_fails_after_commit() {
        let _test_lock = CONFIG_FILE_TEST_LOCK.lock().await;
        let _guard = ConfigFileGuard::new();
        let state = Arc::new(AppState::new(AppConfig::default()));

        commit_enabled_service(&state, "dark").await;

        // revision 2: 生成失敗する provider をコミット（service は None）。
        let (mut failing, expected) = state.config_with_revision();
        failing.ai.default_provider = "unknown".to_string();
        state.commit_config(expected, failing, None).unwrap();
        assert!(state.is_ai_enabled());
        assert!(state.ai_service().is_none());

        // revision 3: prepare 失敗後に別の commit_config が成功した状況を作る。
        let (mut replacement_config, commit_expected) = state.config_with_revision();
        replacement_config.ai.default_provider = "ollama".to_string();
        let replacement_service = state
            .prepare_ai_service(&replacement_config)
            .await
            .unwrap()
            .expect("置換用 service が生成される");
        let state_for_hook = state.clone();
        let config_for_hook = replacement_config.clone();
        let service_for_hook = replacement_service.clone();
        state.set_after_prepare_hook(move || {
            state_for_hook
                .commit_config(
                    commit_expected,
                    config_for_hook.clone(),
                    Some(service_for_hook.clone()),
                )
                .unwrap();
        });

        // 生成失敗は Err で伝播するが、commit 済み service は破壊しない。
        assert!(state.refresh_ai_service().await.is_err());
        assert_eq!(state.config_with_revision().1, 3);
        assert_config_eq(&state.config(), &replacement_config);
        let current = state
            .ai_service()
            .expect("競合後も commit 済み service が残る");
        assert!(Arc::ptr_eq(&current, &replacement_service));
    }

    /// revision が変わらなければ生成した service で置換される（通常系の維持）。
    #[tokio::test]
    async fn refresh_ai_service_replaces_service_when_revision_is_unchanged() {
        let _test_lock = CONFIG_FILE_TEST_LOCK.lock().await;
        let _guard = ConfigFileGuard::new();
        let state = AppState::new(AppConfig::default());

        let (_, old_service) = commit_enabled_service(&state, "dark").await;

        state.refresh_ai_service().await.unwrap();

        let current = state.ai_service().expect("有効設定なら service がある");
        assert!(!Arc::ptr_eq(&current, &old_service));
        assert_eq!(state.config_with_revision().1, 1);
    }

    #[tokio::test]
    async fn prepare_ai_service_failure_keeps_config_and_service_unchanged() {
        let state = AppState::new(AppConfig::default());
        let mut valid = AppConfig::default();
        valid.ai.enabled = true;
        valid.ai.default_provider = "ollama".to_string(); // 初期化時に HTTP 要求なし
        state.update_config(valid.clone()).unwrap();
        state.init_ai_service().await.unwrap();
        let before = state.ai_service();
        assert!(before.is_some());

        let mut invalid = valid.clone();
        invalid.ai.default_provider = "unknown".to_string();

        // 生成失敗は Err のみ。コミット（＝保存）に進まないため disk も不変。
        assert!(state.prepare_ai_service(&invalid).await.is_err());

        assert_config_eq(&state.config(), &valid);
        let after = state.ai_service();
        assert!(after.is_some());
        // 古い service は廃棄も復活もされず、同じ Arc が残る。
        assert!(Arc::ptr_eq(
            before.as_ref().unwrap(),
            after.as_ref().unwrap()
        ));
    }

    #[tokio::test]
    async fn prepare_ai_service_disabled_returns_none() {
        let state = AppState::new(AppConfig::default());
        assert!(
            state
                .prepare_ai_service(&AppConfig::default())
                .await
                .unwrap()
                .is_none()
        );
        assert!(state.ai_service().is_none());
    }

    #[test]
    fn normal_state_is_not_in_load_failure_mode() {
        let state = AppState::new(AppConfig::default());
        assert!(!state.config_load_failed());
    }

    /// 読み込み失敗状態では `commit_config` が保存前に拒否され、
    /// メモリ・ディスク・revision のいずれも変化しないこと（R2-ER01）。
    #[tokio::test]
    async fn commit_config_rejected_when_config_load_failed() {
        let _test_lock = CONFIG_FILE_TEST_LOCK.lock().await;
        let _guard = ConfigFileGuard::new();
        let state = AppState::new_with_load_failure(AppConfig::default());
        assert!(state.config_load_failed());

        let (_, expected) = state.config_with_revision();
        let disk_before = saved_config_toml();
        let mut candidate = AppConfig::default();
        candidate.general.theme = "dark".to_string();

        let error = state
            .commit_config(expected, candidate, None)
            .expect_err("読み込み失敗状態では保存が拒否されること");
        match &error {
            CoreError::Storage(message) => {
                assert_eq!(message, CONFIG_LOAD_FAILED_MESSAGE);
            }
            other => panic!("expected Storage, got {other:?}"),
        }
        // 固定文言のみで、設定値やパスを含めないこと。
        assert!(!error.to_string().contains("config.toml"));
        assert!(!error.to_string().contains("dark"));

        assert_config_eq(&state.config(), &AppConfig::default());
        assert_eq!(state.config_with_revision().1, 0);
        assert_eq!(saved_config_toml(), disk_before, "ディスクは不変");
        assert!(state.ai_service().is_none());
    }

    #[tokio::test]
    async fn commit_config_success_replaces_config_and_service_and_advances_revision() {
        let _test_lock = CONFIG_FILE_TEST_LOCK.lock().await;
        let _guard = ConfigFileGuard::new();
        let state = AppState::new(AppConfig::default());
        assert!(state.ai_service().is_none());
        let (_, expected) = state.config_with_revision();
        assert_eq!(expected, 0);

        let mut candidate = AppConfig::default();
        candidate.ai.enabled = true;
        candidate.ai.default_provider = "ollama".to_string();
        candidate.general.theme = "dark".to_string();
        let prepared = state.prepare_ai_service(&candidate).await.unwrap();
        assert!(prepared.is_some());

        state
            .commit_config(expected, candidate.clone(), prepared.clone())
            .unwrap();

        assert_config_eq(&state.config(), &candidate);
        assert_eq!(
            state.config_with_revision().1,
            1,
            "成功時のみ revision が進む"
        );
        let committed = state.ai_service().unwrap();
        assert!(Arc::ptr_eq(prepared.as_ref().unwrap(), &committed));

        // 無効設定をコミットすると service は None に置換される。
        let (_, expected) = state.config_with_revision();
        let disabled = AppConfig::default();
        let none_service = state.prepare_ai_service(&disabled).await.unwrap();
        assert!(none_service.is_none());
        state
            .commit_config(expected, disabled.clone(), none_service)
            .unwrap();
        assert_config_eq(&state.config(), &disabled);
        assert_eq!(state.config_with_revision().1, 2);
        assert!(state.ai_service().is_none());
    }

    #[tokio::test]
    async fn commit_config_stale_expected_conflicts_and_keeps_everything() {
        let _test_lock = CONFIG_FILE_TEST_LOCK.lock().await;
        let _guard = ConfigFileGuard::new();
        let state = AppState::new(AppConfig::default());
        let (_, expected) = state.config_with_revision();

        let mut accepted = AppConfig::default();
        accepted.ai.enabled = true;
        accepted.ai.default_provider = "ollama".to_string();
        accepted.general.theme = "dark".to_string();
        let prepared = state.prepare_ai_service(&accepted).await.unwrap();
        state
            .commit_config(expected, accepted.clone(), prepared.clone())
            .unwrap();
        let service_before = state.ai_service().unwrap();
        let disk_before = saved_config_toml();
        assert!(disk_before.is_some());

        // 読み取ったままの古い expected で別設定をコミットしようとする。
        let mut stale = accepted.clone();
        stale.general.theme = "light".to_string();
        let err = state.commit_config(expected, stale, None).unwrap_err();
        match &err {
            CoreError::Conflict(message) => {
                assert_eq!(message, "Configuration changed since it was read");
            }
            other => panic!("expected Conflict, got {other:?}"),
        }
        assert!(
            err.to_string().starts_with("Configuration conflict:"),
            "Conflict の表示は固定文言のみ（設定値を含まない）"
        );

        assert_config_eq(&state.config(), &accepted);
        assert_eq!(
            state.config_with_revision().1,
            1,
            "Conflict で revision は不変"
        );
        assert_eq!(
            saved_config_toml(),
            disk_before,
            "Conflict でディスクは不変"
        );
        let service_after = state.ai_service().unwrap();
        assert!(
            Arc::ptr_eq(&service_before, &service_after),
            "Conflict で service は不変"
        );
    }

    #[tokio::test]
    async fn two_commits_with_same_expected_only_one_succeeds() {
        let _test_lock = CONFIG_FILE_TEST_LOCK.lock().await;
        let _guard = ConfigFileGuard::new();
        let state = Arc::new(AppState::new(AppConfig::default()));
        let (_, expected) = state.config_with_revision();

        async fn commit(
            state: Arc<AppState>,
            expected: u64,
            theme: &'static str,
        ) -> CoreResult<()> {
            let mut candidate = AppConfig::default();
            candidate.general.theme = theme.to_string();
            // 両者が同じ expected を保持したまま await を跨ぐ状況を明示する。
            tokio::task::yield_now().await;
            state.commit_config(expected, candidate, None)
        }

        let (first, second) = tokio::join!(
            commit(state.clone(), expected, "dark"),
            commit(state.clone(), expected, "light"),
        );

        let conflicts = [&first, &second]
            .iter()
            .filter(|result| matches!(result, Err(CoreError::Conflict(_))))
            .count();
        assert_eq!(first.is_ok() as usize + second.is_ok() as usize, 1);
        assert_eq!(conflicts, 1, "古い expected の片方は必ず Conflict");
        assert_eq!(state.config_with_revision().1, 1);
        let winner = if first.is_ok() { "dark" } else { "light" };
        assert_eq!(state.config().general.theme, winner);
    }
}
