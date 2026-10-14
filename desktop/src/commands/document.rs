//! Document management commands.

use tauri::{command, State, AppHandle};
use openrustbooks_core::{CoreResult, CoreError, document::{Document, DocumentId, DocumentFormat, DocumentMetadata, DocumentContent}};
use crate::state::AppState;
use std::sync::Arc;
use std::path::PathBuf;

/// Open a document from path
#[command]
pub async fn open_document(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    path: String,
) -> CoreResult<Document> {
    let path_buf = PathBuf::from(&path);

    // Read file content
    let content = tokio::fs::read_to_string(&path_buf).await
        .map_err(|e| CoreError::Io(e))?;

    // Detect format
    let format = DocumentFormat::from_extension(
        path_buf.extension().and_then(|s| s.to_str()).unwrap_or("")
    ).unwrap_or(DocumentFormat::Txt);

    // Create document
    let id = DocumentId::from_string(path.clone());
    let metadata = DocumentMetadata {
        id: id.clone(),
        format,
        title: path_buf.file_stem().and_then(|s| s.to_str()).map(String::from),
        author: None,
        created_at: None,
        modified_at: None,
        size_bytes: content.len() as u64,
        path: Some(path_buf.clone()),
        url: None,
        checksum: None,
        extra: Default::default(),
    };

    let document = Document {
        metadata,
        content: match format {
            DocumentFormat::Markdown => {
                let (fm, body) = crate::document::extract_front_matter(&content);
                DocumentContent::Markdown { content: body, front_matter: fm }
            }
            _ => DocumentContent::Text(content),
        },
    };

    state.add_document(document.clone());
    state.mark_saved(&id);

    // Emit event to frontend
    let _ = app.emit("document-opened", &document);

    Ok(document)
}

/// Save a document
#[command]
pub async fn save_document(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    id: String,
    content: Option<String>,
) -> CoreResult<()> {
    let doc_id = DocumentId::from_string(id);
    let mut document = state.get_document(&doc_id)
        .ok_or_else(|| CoreError::document_not_found(&doc_id.0))?;

    // Update content if provided
    if let Some(new_content) = content {
        if let Some(text) = document.as_text_mut() {
            *text = new_content;
        }
    }

    // Write to file if path exists
    if let Some(path) = &document.metadata.path {
        let text = document.as_text()
            .ok_or_else(|| CoreError::validation("Document is not text-editable"))?;

        tokio::fs::write(path, text).await
            .map_err(|e| CoreError::Io(e))?;

        // Update metadata
        document.metadata.size_bytes = text.len() as u64;
        document.metadata.modified_at = Some(chrono::Utc::now());
    }

    state.add_document(document.clone());
    state.mark_saved(&doc_id);

    let _ = app.emit("document-saved", &document);

    Ok(())
}

/// Close a document
#[command]
pub async fn close_document(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    id: String,
    force: Option<bool>,
) -> CoreResult<()> {
    let doc_id = DocumentId::from_string(id);

    // Check for unsaved changes
    if !force.unwrap_or(false) && state.is_unsaved(&doc_id) {
        return Err(CoreError::validation("Document has unsaved changes"));
    }

    let document = state.remove_document(&doc_id);
    if document.is_some() {
        let _ = app.emit("document-closed", doc_id.0);
    }

    Ok(())
}

/// List all open documents
#[command]
pub fn list_documents(state: State<'_, Arc<AppState>>) -> Vec<Document> {
    state.list_documents()
}

/// Get document content
#[command]
pub fn get_document_content(state: State<'_, Arc<AppState>>, id: String) -> CoreResult<String> {
    let doc_id = DocumentId::from_string(id);
    let document = state.get_document(&doc_id)
        .ok_or_else(|| CoreError::document_not_found(&doc_id.0))?;

    document.as_text()
        .map(String::from)
        .ok_or_else(|| CoreError::validation("Document is not text-editable"))
}

/// Set document content
#[command]
pub fn set_document_content(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    id: String,
    content: String,
) -> CoreResult<()> {
    let doc_id = DocumentId::from_string(id);
    let mut document = state.get_document(&doc_id)
        .ok_or_else(|| CoreError::document_not_found(&doc_id.0))?;

    if let Some(text) = document.as_text_mut() {
        *text = content;
        state.add_document(document.clone());
        state.mark_unsaved(&doc_id);
        let _ = app.emit("document-changed", &document);
        Ok(())
    } else {
        Err(CoreError::validation("Document is not text-editable"))
    }
}