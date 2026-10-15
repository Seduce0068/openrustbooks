//! AI Markdown pipeline commands.

use crate::state::AppState;
use openrustbooks_core::{
    CoreError,
    ai_markdown::{
        FrontMatterRequest, FrontMatterResult, GenerateRequest, GenerateResult, RefineRequest,
        RefineResult, TableAlignment, TableFormat, TableRequest, TableResult,
    },
};
use std::sync::Arc;
use tauri::{State, command};

/// Convert CoreError to String for Tauri commands
fn map_err(e: CoreError) -> String {
    e.to_string()
}

/// Generate Markdown from natural language prompt
#[command]
pub async fn generate_markdown(
    state: State<'_, Arc<AppState>>,
    prompt: String,
    context: Option<String>,
    template: Option<String>,
) -> Result<GenerateResult, String> {
    let service = state
        .ai_service()
        .ok_or_else(|| CoreError::ai_pipeline("AI service not initialized").to_string())?;

    service
        .pipeline()
        .generate(GenerateRequest {
            prompt,
            context,
            template,
        })
        .await
        .map_err(map_err)
}

/// Refine existing Markdown with instructions
#[command]
pub async fn refine_markdown(
    state: State<'_, Arc<AppState>>,
    content: String,
    instruction: String,
    context: Option<String>,
) -> Result<RefineResult, String> {
    let service = state
        .ai_service()
        .ok_or_else(|| CoreError::ai_pipeline("AI service not initialized").to_string())?;

    service
        .pipeline()
        .refine(RefineRequest {
            content,
            instruction,
            context,
        })
        .await
        .map_err(map_err)
}

/// Convert structured data to Markdown table
#[command]
pub async fn convert_to_table(
    state: State<'_, Arc<AppState>>,
    data: String,
    format: String,
    headers: Option<Vec<String>>,
    alignment: Option<Vec<String>>,
) -> Result<TableResult, String> {
    let service = state
        .ai_service()
        .ok_or_else(|| CoreError::ai_pipeline("AI service not initialized").to_string())?;

    let table_format = match format.to_lowercase().as_str() {
        "csv" => TableFormat::Csv,
        "json" => TableFormat::Json,
        "tsv" => TableFormat::Tsv,
        _ => {
            return Err(CoreError::validation("Invalid format. Use csv, json, or tsv").to_string());
        }
    };

    let table_alignment = alignment.map(|a| {
        a.into_iter()
            .map(|s| match s.to_lowercase().as_str() {
                "left" => TableAlignment::Left,
                "center" => TableAlignment::Center,
                "right" => TableAlignment::Right,
                _ => TableAlignment::Left,
            })
            .collect()
    });

    service
        .pipeline()
        .convert_to_table(TableRequest {
            data,
            format: table_format,
            headers,
            alignment: table_alignment,
        })
        .await
        .map_err(map_err)
}

/// Generate front matter for a document
#[command]
pub async fn generate_front_matter(
    state: State<'_, Arc<AppState>>,
    content: String,
    schema: Option<serde_json::Value>,
) -> Result<FrontMatterResult, String> {
    let service = state
        .ai_service()
        .ok_or_else(|| CoreError::ai_pipeline("AI service not initialized").to_string())?;

    service
        .pipeline()
        .generate_front_matter(FrontMatterRequest { content, schema })
        .await
        .map_err(map_err)
}

/// Lint Markdown content
#[command]
pub async fn lint_markdown(
    state: State<'_, Arc<AppState>>,
    content: String,
) -> Result<Vec<openrustbooks_core::ai_markdown::lint::LintIssue>, String> {
    let service = state
        .ai_service()
        .ok_or_else(|| CoreError::ai_pipeline("AI service not initialized").to_string())?;

    service.pipeline().linter().lint(&content).map_err(map_err)
}

/// Lint and auto-fix Markdown content
#[command]
pub async fn lint_and_fix_markdown(
    state: State<'_, Arc<AppState>>,
    content: String,
) -> Result<openrustbooks_core::ai_markdown::lint::LintResultWrapper, String> {
    let service = state
        .ai_service()
        .ok_or_else(|| CoreError::ai_pipeline("AI service not initialized").to_string())?;

    service
        .pipeline()
        .linter()
        .lint_and_fix(&content)
        .await
        .map_err(map_err)
}
