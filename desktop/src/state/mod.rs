//! Application state management.

use openrustbooks_core::{
    AppConfig, CoreResult,
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

    /// Initialize AI service
    pub async fn init_ai_service(&self) -> CoreResult<()> {
        let config = self.config();
        if config.ai.enabled {
            let service = AiMarkdownService::new(&config).await?;
            *self.ai_service.write().unwrap() = Some(Arc::new(service));
        }
        Ok(())
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
        if !self.is_ai_enabled() {
            return Ok(());
        }
        self.init_ai_service().await
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
}
