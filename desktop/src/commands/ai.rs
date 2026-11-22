//! AI Markdown pipeline commands.

use crate::commands::ipc_error;
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
#[cfg(not(test))]
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tokio::sync::oneshot;

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

/// 確認ダイアログの応答を解釈し、キャンセル／無応答をエラーにする純粋関数。
fn interpret_confirmation(
    operation: &str,
    result: Result<bool, oneshot::error::RecvError>,
) -> Result<(), String> {
    match result {
        Ok(true) => Ok(()),
        Ok(false) => Err(format!("{operation} がキャンセルされました")),
        Err(_) => Err(format!("{operation} の確認ダイアログが応答しませんでした")),
    }
}

/// 送信前にネイティブ確認ダイアログを出し、キャンセルなら Err を返す。
#[cfg(not(test))]
async fn confirm_send<R: tauri::Runtime>(
    app: &AppHandle<R>,
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

    interpret_confirmation(operation, rx.await)
}

/// テスト用の確認待ちフック。本番のダイアログの代わりに 1 回だけ呼ばれる。
///
/// 確認待ち中に `AppState::disable_ai_service()` を呼んで、
/// confirm 後に service を取り直す経路を外部 AI API なしで検証する。
#[cfg(test)]
static TEST_CONFIRM_HOOK: std::sync::Mutex<Option<Box<dyn FnOnce() + Send>>> =
    std::sync::Mutex::new(None);

/// テスト用 confirm_send。ダイアログを出さず、フックを実行して承認扱いにする。
#[cfg(test)]
async fn confirm_send<R: tauri::Runtime>(
    _app: &AppHandle<R>,
    operation: &str,
    fields: &[(&str, String)],
) -> Result<(), String> {
    // 本番と同じ要約を組み立てる（引数を実際に使う確認ダイアログの契約を保つ）。
    let _ = build_summary(operation, fields);
    if let Some(hook) = TEST_CONFIRM_HOOK.lock().unwrap().take() {
        hook();
    }
    Ok(())
}

/// command 本体。test からも同じ経路で呼べるよう generic の `AppHandle<R>` を使う。
async fn generate_markdown_impl<R: tauri::Runtime>(
    state: &AppState,
    app: &AppHandle<R>,
    prompt: String,
    context: Option<String>,
    template: Option<String>,
) -> Result<GenerateResult, String> {
    // confirm 待ち中に設定が変わり得るため、service は confirmation の後に取得する。
    confirm_send(
        app,
        "Markdown 生成",
        &[
            ("prompt", prompt.clone()),
            ("context", context.clone().unwrap_or_default()),
            ("template", template.clone().unwrap_or_default()),
        ],
    )
    .await?;

    let service = state
        .ai_service()
        .ok_or_else(|| ipc_error(CoreError::ai_pipeline("AI service not initialized")))?;

    service
        .pipeline()
        .generate(GenerateRequest {
            prompt,
            context,
            template,
        })
        .await
        .map_err(ipc_error)
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
    generate_markdown_impl(state.inner().as_ref(), &app, prompt, context, template).await
}

/// command 本体。confirm 後に service を取り直す。
async fn refine_markdown_impl<R: tauri::Runtime>(
    state: &AppState,
    app: &AppHandle<R>,
    content: String,
    instruction: String,
    context: Option<String>,
) -> Result<RefineResult, String> {
    confirm_send(
        app,
        "Markdown 改善",
        &[
            ("content", content.clone()),
            ("instruction", instruction.clone()),
            ("context", context.clone().unwrap_or_default()),
        ],
    )
    .await?;

    let service = state
        .ai_service()
        .ok_or_else(|| ipc_error(CoreError::ai_pipeline("AI service not initialized")))?;

    service
        .pipeline()
        .refine(RefineRequest {
            content,
            instruction,
            context,
        })
        .await
        .map_err(ipc_error)
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
    refine_markdown_impl(state.inner().as_ref(), &app, content, instruction, context).await
}

/// format 文字列を TableFormat に解釈する（大小文字無視）。
fn parse_table_format(format: &str) -> Result<TableFormat, String> {
    match format.to_lowercase().as_str() {
        "csv" => Ok(TableFormat::Csv),
        "json" => Ok(TableFormat::Json),
        "tsv" => Ok(TableFormat::Tsv),
        _ => Err(ipc_error(CoreError::validation(
            "Invalid format. Use csv, json, or tsv",
        ))),
    }
}

/// alignment 文字列群を TableAlignment に解釈する（大小文字無視、未知値は Left）。
fn parse_table_alignment(alignment: Option<Vec<String>>) -> Option<Vec<TableAlignment>> {
    alignment.map(|a| {
        a.into_iter()
            .map(|s| match s.to_lowercase().as_str() {
                "left" => TableAlignment::Left,
                "center" => TableAlignment::Center,
                "right" => TableAlignment::Right,
                _ => TableAlignment::Left,
            })
            .collect()
    })
}

/// command 本体。入力エラーは確認ダイアログより前に返す既存契約を維持する。
async fn convert_to_table_impl<R: tauri::Runtime>(
    state: &AppState,
    app: &AppHandle<R>,
    data: String,
    format: String,
    headers: Option<Vec<String>>,
    alignment: Option<Vec<String>>,
) -> Result<TableResult, String> {
    // parse_table_format は confirm 前に置き、不正入力ならダイアログを出さない。
    let table_format = parse_table_format(&format)?;

    confirm_send(
        app,
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

    let service = state
        .ai_service()
        .ok_or_else(|| ipc_error(CoreError::ai_pipeline("AI service not initialized")))?;

    let table_alignment = parse_table_alignment(alignment);

    service
        .pipeline()
        .convert_to_table(TableRequest {
            data,
            format: table_format,
            headers,
            alignment: table_alignment,
        })
        .await
        .map_err(ipc_error)
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
    convert_to_table_impl(
        state.inner().as_ref(),
        &app,
        data,
        format,
        headers,
        alignment,
    )
    .await
}

/// command 本体。confirm 後に service を取り直す。
async fn generate_front_matter_impl<R: tauri::Runtime>(
    state: &AppState,
    app: &AppHandle<R>,
    content: String,
    schema: Option<serde_json::Value>,
) -> Result<FrontMatterResult, String> {
    confirm_send(
        app,
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

    let service = state
        .ai_service()
        .ok_or_else(|| ipc_error(CoreError::ai_pipeline("AI service not initialized")))?;

    service
        .pipeline()
        .generate_front_matter(FrontMatterRequest { content, schema })
        .await
        .map_err(ipc_error)
}

/// Generate front matter for a document
#[command]
pub async fn generate_front_matter(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    content: String,
    schema: Option<serde_json::Value>,
) -> Result<FrontMatterResult, String> {
    generate_front_matter_impl(state.inner().as_ref(), &app, content, schema).await
}

/// Lint Markdown content
#[command]
pub async fn lint_markdown(
    state: State<'_, Arc<AppState>>,
    content: String,
) -> Result<Vec<openrustbooks_core::ai_markdown::lint::LintIssue>, String> {
    let service = state
        .ai_service()
        .ok_or_else(|| ipc_error(CoreError::ai_pipeline("AI service not initialized")))?;

    service
        .pipeline()
        .linter()
        .lint(&content)
        .map_err(ipc_error)
}

/// Lint and auto-fix Markdown content
#[command]
pub async fn lint_and_fix_markdown(
    state: State<'_, Arc<AppState>>,
    content: String,
) -> Result<openrustbooks_core::ai_markdown::lint::LintResultWrapper, String> {
    let service = state
        .ai_service()
        .ok_or_else(|| ipc_error(CoreError::ai_pipeline("AI service not initialized")))?;

    service
        .pipeline()
        .linter()
        .lint_and_fix(&content)
        .await
        .map_err(ipc_error)
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

    // --- interpret_confirmation: 3 分岐 ---

    #[test]
    fn confirmation_ok_true_returns_ok() {
        assert_eq!(interpret_confirmation("Markdown 生成", Ok(true)), Ok(()));
    }

    #[test]
    fn confirmation_ok_false_returns_canceled_error() {
        assert_eq!(
            interpret_confirmation("Markdown 生成", Ok(false)),
            Err("Markdown 生成 がキャンセルされました".to_string())
        );
    }

    #[tokio::test]
    async fn confirmation_err_returns_no_response_error() {
        // 送信側をドロップすると rx.await が RecvError を返す。
        let (tx, rx) = oneshot::channel::<bool>();
        drop(tx);
        assert_eq!(
            interpret_confirmation("Markdown 生成", rx.await),
            Err("Markdown 生成 の確認ダイアログが応答しませんでした".to_string())
        );
    }

    // --- parse_table_format: csv / json / TSV(大小混在) / 不正 ---

    #[test]
    fn table_format_csv() {
        assert!(matches!(parse_table_format("csv"), Ok(TableFormat::Csv)));
    }

    #[test]
    fn table_format_json() {
        assert!(matches!(parse_table_format("json"), Ok(TableFormat::Json)));
    }

    #[test]
    fn table_format_mixed_case_tsv() {
        assert!(matches!(parse_table_format("TsV"), Ok(TableFormat::Tsv)));
    }

    /// AI provider の生レスポンス（secret / host / body）は IPC へ出さず、
    /// code 由来の固定文言だけを返す。内部ログにも生値は書かない。
    #[test]
    fn ai_provider_error_is_redacted_for_ipc() {
        let secret = "sk-ai-leak-do-not-expose";
        let host = "https://api.secret.internal";
        let body = "response-body-with-token";
        let raw = CoreError::ai_provider(format!("key={secret} host={host} body={body}"));
        let rendered = ipc_error(raw);

        assert_eq!(rendered, "AI provider request failed");
        for needle in [secret, host, body] {
            assert!(
                !rendered.contains(needle),
                "IPC 結果へ {needle} が漏れています: {rendered}"
            );
        }
    }

    #[test]
    fn table_format_invalid_returns_error() {
        assert_eq!(
            parse_table_format("xml").unwrap_err(),
            ipc_error(CoreError::validation(
                "Invalid format. Use csv, json, or tsv"
            ))
        );
    }

    // --- parse_table_alignment: left / center / right / 未知 / None ---

    #[test]
    fn table_alignment_left_center_right() {
        let parsed = parse_table_alignment(Some(vec![
            "left".to_string(),
            "center".to_string(),
            "right".to_string(),
        ]))
        .expect("Some を渡したので Some が返る");

        assert_eq!(parsed.len(), 3);
        assert!(matches!(parsed[0], TableAlignment::Left));
        assert!(matches!(parsed[1], TableAlignment::Center));
        assert!(matches!(parsed[2], TableAlignment::Right));
    }

    #[test]
    fn table_alignment_case_insensitive_and_unknown_falls_back_to_left() {
        let parsed = parse_table_alignment(Some(vec![
            "LEFT".to_string(),
            "CeNtEr".to_string(),
            "RIGHT".to_string(),
            "bogus".to_string(),
        ]))
        .expect("Some を渡したので Some が返る");

        assert_eq!(parsed.len(), 4);
        assert!(matches!(parsed[0], TableAlignment::Left));
        assert!(matches!(parsed[1], TableAlignment::Center));
        assert!(matches!(parsed[2], TableAlignment::Right));
        assert!(matches!(parsed[3], TableAlignment::Left)); // 未知値は Left にフォールバック
    }

    #[test]
    fn table_alignment_none_returns_none() {
        assert!(parse_table_alignment(None).is_none());
    }

    /// confirm 待ち中に service を無効化するフックを仕込み、confirm 後の
    /// 再取得で拒否されること（= 古い service で pipeline が呼ばれないこと）を検証する。
    async fn assert_rejected_when_disabled_during_confirm<T, F, Fut>(
        state: &Arc<AppState>,
        handle: &tauri::AppHandle<tauri::test::MockRuntime>,
        run: F,
    ) where
        F: FnOnce(Arc<AppState>, tauri::AppHandle<tauri::test::MockRuntime>) -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        // ollama は初期化時に HTTP 要求を出さない。毎回作り直して確認前は Some にする。
        let mut config = state.config();
        config.ai.enabled = true;
        config.ai.default_provider = "ollama".to_string();
        state.update_config(config).unwrap();
        state.refresh_ai_service().await.unwrap();
        assert!(
            state.ai_service().is_some(),
            "confirm 前は service が存在すること"
        );

        let disabling = state.clone();
        *TEST_CONFIRM_HOOK.lock().unwrap() = Some(Box::new(move || disabling.disable_ai_service()));

        let result = run(state.clone(), handle.clone()).await;

        assert!(
            TEST_CONFIRM_HOOK.lock().unwrap().is_none(),
            "confirm 待ちフックは 1 回だけ消費されること"
        );
        assert!(
            state.ai_service().is_none(),
            "confirm 中に service が無効化されていること"
        );
        match result {
            Err(message) => assert_eq!(
                message,
                ipc_error(CoreError::ai_pipeline("AI service not initialized"))
            ),
            Ok(_) => panic!("service が無効なのに pipeline が呼ばれた"),
        }
    }

    /// 4 コマンドすべてで、confirm 待ち中の無効化後に古い service で送信しないこと。
    /// 入力エラーが confirm より前に返る既存契約も同時に検証する。
    /// 外部 AI API への送信は行わない（ollama 設定 + service None で拒否）。
    #[tokio::test]
    async fn ai_commands_recheck_service_after_confirm() {
        use openrustbooks_core::AppConfig;
        use tauri::test::mock_app;

        let app = mock_app();
        let handle = app.handle().clone();
        let state = Arc::new(AppState::new(AppConfig::default()));

        // parse_table_format の入力エラーは確認ダイアログより前に返る。
        *TEST_CONFIRM_HOOK.lock().unwrap() = Some(Box::new(|| {
            panic!("入力エラーなのに確認ダイアログが呼ばれた");
        }));
        let invalid = convert_to_table_impl(
            state.as_ref(),
            &handle,
            "a,b\n1,2".to_string(),
            "xml".to_string(),
            None,
            None,
        )
        .await;
        assert!(
            TEST_CONFIRM_HOOK.lock().unwrap().is_some(),
            "不正 format では confirm フックが消費されないこと"
        );
        TEST_CONFIRM_HOOK.lock().unwrap().take();
        match invalid {
            Err(message) => assert_eq!(
                message,
                ipc_error(CoreError::validation(
                    "Invalid format. Use csv, json, or tsv"
                ))
            ),
            Ok(_) => panic!("不正 format が confirm 前に拒否されなかった"),
        }

        // 静的フックの取り違えを避けるため、4 コマンドは直列に検証する。
        assert_rejected_when_disabled_during_confirm(&state, &handle, |state, handle| async move {
            generate_markdown_impl(state.as_ref(), &handle, "猫の話".to_string(), None, None).await
        })
        .await;

        assert_rejected_when_disabled_during_confirm(&state, &handle, |state, handle| async move {
            refine_markdown_impl(
                state.as_ref(),
                &handle,
                "# 見出し".to_string(),
                "短く".to_string(),
                None,
            )
            .await
        })
        .await;

        assert_rejected_when_disabled_during_confirm(&state, &handle, |state, handle| async move {
            convert_to_table_impl(
                state.as_ref(),
                &handle,
                "a,b\n1,2".to_string(),
                "csv".to_string(),
                None,
                None,
            )
            .await
        })
        .await;

        assert_rejected_when_disabled_during_confirm(&state, &handle, |state, handle| async move {
            generate_front_matter_impl(state.as_ref(), &handle, "# 見出し".to_string(), None).await
        })
        .await;
    }
}
