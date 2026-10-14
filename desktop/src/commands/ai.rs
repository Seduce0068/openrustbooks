//! AI Markdown pipeline commands.

use tauri::{command, State};
use openrustbooks_core::{CoreResult, CoreError, ai_markdown::{GenerateRequest, RefineRequest, TableRequest, FrontMatterRequest, TableFormat, TableAlignment, AiMarkdownService}};
use crate::state::AppState;
use std::sync::Arc;

/// Generate Markdown from natural language prompt
#[command]
pub async fn generate_markdown(
    state: State<'_, Arc<AppState>>,
    prompt: String,
    context: Option<String>,
    template: Option<String>,
) -> CoreResult<openrustbooks_core::ai_markdown::GenerateResult> {
    let service = state.ai_service()
        .ok_or_else(|| CoreError::ai_pipeline("AI service not initialized"))?;

    service.pipeline.generate(GenerateRequest {
        prompt,
        context,
        template,
    }).await
}

/// Refine existing Markdown with instructions
#[command]
pub async fn refine_markdown(
    state: State<'_, Arc<AppState>>,
    content: String,
    instruction: String,
    context: Option<String>,
) -> CoreResult<openrustbooks_core::ai_markdown::RefineResult> {
    let service = state.ai_service()
        .ok_or_else(|| CoreError::ai_pipeline("AI service not initialized"))?;

    service.pipeline.refine(RefineRequest {
        content,
        instruction,
        context,
    }).await
}

/// Convert structured data to Markdown table
#[command]
pub async fn convert_to_table(
    state: State<'_, Arc<AppState>>,
    data: String,
    format: String,
    headers: Option<Vec<String>>,
    alignment: Option<Vec<String>>,
) -> CoreResult<openrustbooks_core::ai_markdown::TableResult> {
    let service = state.ai_service()
        .ok_or_else(|| CoreError::ai_pipeline("AI service not initialized"))?;

    let table_format = match format.to_lowercase().as_str() {
        "csv" => TableFormat::Csv,
        "json" => TableFormat::Json,
        "tsv" => TableFormat::Tsv,
        _ => return Err(CoreError::validation("Invalid format. Use csv, json, or tsv")),
    };

    let table_alignment = alignment.map(|a| {
        a.into_iter().map(|s| match s.to_lowercase().as_str() {
            "left" => TableAlignment::Left,
            "center" => TableAlignment::Center,
            "right" => TableAlignment::Right,
            _ => TableAlignment::Left,
        }).collect()
    });

    service.pipeline.convert_to_table(TableRequest {
        data,
        format: table_format,
        headers,
        alignment: table_alignment,
    }).await
}

/// Generate front matter for a document
#[command]
pub async fn generate_front_matter(
    state: State<'_, Arc<AppState>>,
    content: String,
    schema: Option<serde_json::Value>,
) -> CoreResult<openrustbooks_core::ai_markdown::FrontMatterResult> {
    let service = state.ai_service()
        .ok_or_else(|| CoreError::ai_pipeline("AI service not initialized"))?;

    service.pipeline.generate_front_matter(FrontMatterRequest {
        content,
        schema,
    }).await
}

/// Lint Markdown content
#[command]
pub async fn lint_markdown(
    state: State<'_, Arc<AppState>>,
    content: String,
) -> CoreResult<Vec<openrustbooks_core::ai_markdown::lint::LintIssue>> {
    let service = state.ai_service()
        .ok_or_else(|| CoreError::ai_pipeline("AI service not initialized"))?;

    service.pipeline.linter.lint(&content).await
}

/// Lint and auto-fix Markdown content
#[command]
pub async fn lint_and_fix_markdown(
    state: State<'_, Arc<AppState>>,
    content: String,
) -> CoreResult<openrustbooks_core::ai_markdown::lint::LintResult> {
    let service = state.ai_service()
        .ok_or_else(|| CoreError::ai_pipeline("AI service not initialized"))?;

    service.pipeline.linter.lint_and_fix(&content).await
}