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
    pub fn ai_service(&self) -> Option<Arc<AiMarkdownService>> {
        self.ai_service.read().unwrap().clone()
    }

    /// Check if AI features are enabled
    pub fn is_ai_enabled(&self) -> bool {
        self.config.read().unwrap().ai.enabled
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
