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
use tauri::{AppHandle, State, command};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tokio::sync::oneshot;

/// Convert CoreError to String for Tauri commands
fn map_err(e: CoreError) -> String {
    e.to_string()
}

/// ダイアログに出す要約のプレビュー文字数。
const PREVIEW_CHARS: usize = 80;

/// 操作名と各フィールドの文字数・先頭だけを並べた要約を作る。
fn build_summary(operation: &str, fields: &[(&str, String)]) -> String {
    let mut summary = format!("「{operation}」で以下の内容を AI に送信します。\n");
    for (name, value) in fields {
        let count = value.chars().count();
        let preview: String = value.chars().take(PREVIEW_CHARS).collect();
        summary.push_str(&format!("\n- {name}: {count} chars\n{preview}\n"));
    }
    summary
}

/// 送信前にネイティブ確認ダイアログを出し、キャンセルなら Err を返す。
async fn confirm_send(
    app: &AppHandle,
    operation: &str,
    fields: &[(&str, String)],
) -> Result<(), String> {
    let (tx, rx) = oneshot::channel::<bool>();
    app.dialog()
        .message(build_summary(operation, fields))
        .title("AI への送信確認")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "送信する".into(),
            "キャンセル".into(),
        ))
        .show(move |accepted| {
            let _ = tx.send(accepted);
        });

    match rx.await {
        Ok(true) => Ok(()),
        Ok(false) => Err(format!("{operation} がキャンセルされました")),
        Err(_) => Err(format!("{operation} の確認ダイアログが応答しませんでした")),
    }
}

/// Generate Markdown from natural language prompt
#[command]
pub async fn generate_markdown(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    prompt: String,
    context: Option<String>,
    template: Option<String>,
) -> Result<GenerateResult, String> {
    let service = state
        .ai_service()
        .ok_or_else(|| CoreError::ai_pipeline("AI service not initialized").to_string())?;

    confirm_send(
        &app,
        "Markdown 生成",
        &[
            ("prompt", prompt.clone()),
            ("context", context.clone().unwrap_or_default()),
            ("template", template.clone().unwrap_or_default()),
        ],
    )
    .await?;

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
    app: AppHandle,
    content: String,
    instruction: String,
    context: Option<String>,
) -> Result<RefineResult, String> {
    let service = state
        .ai_service()
        .ok_or_else(|| CoreError::ai_pipeline("AI service not initialized").to_string())?;

    confirm_send(
        &app,
        "Markdown 改善",
        &[
            ("content", content.clone()),
            ("instruction", instruction.clone()),
            ("context", context.clone().unwrap_or_default()),
        ],
    )
    .await?;

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
    app: AppHandle,
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

    confirm_send(
        &app,
        "テーブル変換",
        &[
            ("data", data.clone()),
            ("format", format.clone()),
            (
                "headers",
                headers.as_ref().map(|v| v.join(", ")).unwrap_or_default(),
            ),
            (
                "alignment",
                alignment.as_ref().map(|v| v.join(", ")).unwrap_or_default(),
            ),
        ],
    )
    .await?;

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
    app: AppHandle,
    content: String,
    schema: Option<serde_json::Value>,
) -> Result<FrontMatterResult, String> {
    let service = state
        .ai_service()
        .ok_or_else(|| CoreError::ai_pipeline("AI service not initialized").to_string())?;

    confirm_send(
        &app,
        "フロントマター生成",
        &[
            ("content", content.clone()),
            (
                "schema",
                schema.as_ref().map(|v| v.to_string()).unwrap_or_default(),
            ),
        ],
    )
    .await?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_contains_operation_field_name_count_and_preview() {
        let summary = build_summary(
            "Markdown 生成",
            &[
                ("prompt", "猫の話を書いて".to_string()),
                ("context", "資料".to_string()),
            ],
        );

        assert!(summary.contains("Markdown 生成"));
        assert!(summary.contains("prompt"));
        assert!(summary.contains("7 chars")); // 「猫の話を書いて」は 7 文字
        assert!(summary.contains("猫の話を書いて"));
        assert!(summary.contains("context"));
        assert!(summary.contains("2 chars"));
    }

    #[test]
    fn summary_truncates_long_multibyte_value_to_80_chars() {
        let long = "あ".repeat(100);
        let summary = build_summary("テスト", &[("content", long.clone())]);
        let preview = "あ".repeat(PREVIEW_CHARS);
        let over = "あ".repeat(PREVIEW_CHARS + 1);

        // 文字数は全体、プレビューは先頭 80 文字だけ。
        assert!(summary.contains("100 chars"));
        assert!(summary.contains(&preview));
        assert!(!summary.contains(&over));
        assert!(!summary.contains(&long));
    }

    #[test]
    fn summary_shows_zero_chars_for_empty_field() {
        let summary = build_summary("テスト", &[("context", String::new())]);

        assert!(summary.contains("context: 0 chars"));
    }
}
