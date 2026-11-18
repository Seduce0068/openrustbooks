//! Application state management.

use openrustbooks_core::{
    AppConfig, CoreError, CoreResult,
    ai_markdown::AiMarkdownService,
    document::{Document, DocumentId},
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Application state shared across commands
pub struct AppState {
    config: RwLock<AppConfig>,
    documents: RwLock<HashMap<DocumentId, Document>>,
    ai_service: RwLock<Option<Arc<AiMarkdownService>>>,
    unsaved_changes: RwLock<HashMap<DocumentId, bool>>,
}

impl AppState {
    /// Create new application state
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: RwLock::new(config),
            documents: RwLock::new(HashMap::new()),
            ai_service: RwLock::new(None),
            unsaved_changes: RwLock::new(HashMap::new()),
        }
    }

    /// Get current configuration
    pub fn config(&self) -> AppConfig {
        self.config.read().unwrap().clone()
    }

    /// Update configuration
    pub fn update_config(&self, config: AppConfig) -> CoreResult<()> {
        *self.config.write().unwrap() = config;
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

    /// 設定と AI service をまとめてコミットする。
    ///
    /// 保存成功時のみ新しい設定と service を反映する。保存失敗時はメモリ上の
    /// 設定を旧値へ戻し、旧設定での再保存を試みる。再保存も失敗した場合は
    /// ディスク状態が不定であることを明示するエラーを返す（設定値は出さない）。
    pub fn commit_config(
        &self,
        config: AppConfig,
        service: Option<Arc<AiMarkdownService>>,
    ) -> CoreResult<()> {
        let previous = self.config();
        self.update_config(config)?;

        if let Err(save_error) = self.save_config() {
            // update_config は失敗しないため、旧 config の復元は確実に行える。
            let _ = self.update_config(previous);
            // ディスクも旧値へ戻す試み（2 次失敗あり得る）。
            if let Err(restore_error) = self.save_config() {
                // エラーの表示文字列だけを連結し、設定値（シークレット含む）は出さない。
                return Err(CoreError::internal(format!(
                    "config save failed and rollback re-save also failed: {save_error}; {restore_error}"
                )));
            }
            return Err(save_error);
        }

        // 保存成功後のみ service を置換する。生成失敗時はここに到達しないため、
        // 古い service が復活することも、意図せず消えることもない。
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
        self.config.read().unwrap().ai.enabled
    }

    /// Clear the in-memory AI service (設定無効化時に呼ぶ)
    pub fn disable_ai_service(&self) {
        *self.ai_service.write().unwrap() = None;
    }

    /// 設定変更後に AI service を同期する（fail-closed）。
    ///
    /// 先に旧 service を破棄してから、有効なら再生成する。
    /// 生成に失敗しても None のままなので、古い service は残らない。
    pub async fn refresh_ai_service(&self) -> CoreResult<()> {
        self.disable_ai_service();
        let config = self.config();
        if let Some(service) = self.prepare_ai_service(&config).await? {
            *self.ai_service.write().unwrap() = Some(service);
        }
        Ok(())
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
    pub fn remove_document(&self, id: &DocumentId) -> Option<Document> {
        self.documents.write().unwrap().remove(id)
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

    /// Save configuration to disk
    pub fn save_config(&self) -> CoreResult<()> {
        self.config.read().unwrap().save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openrustbooks_core::document::{DocumentContent, DocumentFormat, DocumentMetadata};
    use std::path::PathBuf;

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

    #[tokio::test]
    async fn commit_config_success_replaces_config_and_service() {
        let _guard = ConfigFileGuard::new();
        let state = AppState::new(AppConfig::default());
        assert!(state.ai_service().is_none());

        let mut candidate = AppConfig::default();
        candidate.ai.enabled = true;
        candidate.ai.default_provider = "ollama".to_string();
        candidate.general.theme = "dark".to_string();
        let prepared = state.prepare_ai_service(&candidate).await.unwrap();
        assert!(prepared.is_some());

        state
            .commit_config(candidate.clone(), prepared.clone())
            .unwrap();

        assert_config_eq(&state.config(), &candidate);
        let committed = state.ai_service().unwrap();
        assert!(Arc::ptr_eq(prepared.as_ref().unwrap(), &committed));

        // 無効設定をコミットすると service は None に置換される。
        let disabled = AppConfig::default();
        let none_service = state.prepare_ai_service(&disabled).await.unwrap();
        assert!(none_service.is_none());
        state.commit_config(disabled.clone(), none_service).unwrap();
        assert_config_eq(&state.config(), &disabled);
        assert!(state.ai_service().is_none());
    }
}
