//! Markdown lint の Wasm バインディングと JS 公開用 DTO。
//!
//! 判定の権威は [`crate::ai_markdown::lint::MarkdownLinter`] と
//! [`crate::config::LintConfig`] にあり、ここでは JS 境界向けの変換と
//! fail-closed な設定解釈だけを担う。呼び出し元の [`super`] が `wasm32`
//! でのみこのモジュールをコンパイルするため、各項目に個別の cfg は付与しない。

use crate::{
    ai_markdown::lint::{LintIssue, LintSeverity, MarkdownLinter},
    config::LintConfig,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Wasm-compatible Markdown linter
///
/// featureless の rumdl を直接配線した実 lint。ADR-0156 の fail-closed スタブを
/// 解消し、ブラウザでも `MarkdownLinter` の判定結果をそのまま返す。
/// `converged` / `conflicting_rules` は `WasmLintFixResult` にまだ公開しない
/// （ADR-0156 の未決定を維持し、構造体変更を最小限に留める）。
#[wasm_bindgen]
pub struct WasmMarkdownLinter {
    inner: MarkdownLinter,
    /// `LintConfig.auto_fix` を保持し、修正経路の分岐に使う。
    auto_fix: bool,
}

#[wasm_bindgen]
impl WasmMarkdownLinter {
    /// JS 側の設定を解釈して lint 本体を構築する。
    ///
    /// `null` / `undefined`（引数省略）は `LintConfig::default()` を使う。
    /// それ以外は `serde_wasm_bindgen::from_value` で完全な `LintConfig` を要求し、
    /// パース失敗（型不一致・必須フィールド不足）は既定へ反転させずエラーにする
    /// （fail-closed）。未知のキーは serde が無視する。パース成功後も
    /// `MarkdownLinter::new` がルール設定を検証し、不正な rule_config はエラーにする。
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<Self, JsValue> {
        // null / undefined（引数省略）は既定設定。それ以外はパース失敗を
        // 黙って既定に落とさずエラーにする（部分オブジェクトの
        // `auto_fix:false` が既定 true へ反転する fail-open を避ける）。
        let lint_config = if config.is_null() || config.is_undefined() {
            LintConfig::default()
        } else {
            match serde_wasm_bindgen::from_value::<LintConfig>(config) {
                Ok(value) => value,
                Err(error) => {
                    return Err(JsValue::from_str(&format!("Invalid lint config: {error}")));
                }
            }
        };
        let auto_fix = lint_config.auto_fix;
        let inner = MarkdownLinter::new(&lint_config).map_err(super::js_error)?;
        Ok(Self { inner, auto_fix })
    }

    /// Markdown を lint し、issues を JS 配列として返す。
    #[wasm_bindgen]
    pub fn lint(&self, content: String) -> Result<JsValue, JsValue> {
        console_log!("Linting markdown (content: {} chars)", content.len());
        let issues = self.inner.lint(&content).map_err(super::js_error)?;
        let wasm_issues: Vec<WasmLintIssue> = issues.iter().map(to_wasm_issue).collect();
        serde_wasm_bindgen::to_value(&wasm_issues)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    /// Lint し、`auto_fix` が有効なら自動修正して結果を返す。
    ///
    /// 分岐そのものは `validate_with_auto_fix` に集約し、ここでは
    /// `LintResultWrapper` を JS 公開用の `WasmLintFixResult` へ変換するだけ。
    #[wasm_bindgen]
    pub async fn lint_and_fix(&self, content: String) -> Result<WasmLintFixResult, JsValue> {
        let result =
            crate::ai_markdown::lint::validate_with_auto_fix(&self.inner, &content, self.auto_fix)
                .await
                .map_err(super::js_error)?;

        Ok(WasmLintFixResult {
            passed: result.passed,
            issues: result.issues.iter().map(to_wasm_issue).collect(),
            fixed_content: result.fixed_content,
            fixes_applied: result.fixes_applied,
        })
    }
}

/// Wasm-compatible lint issue
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct WasmLintIssue {
    // `wasm/tests.rs` が rule ID を検証するため `pub(super)` で親へ見せる。
    pub(super) rule: String,
    pub(super) rule_name: String,
    pub(super) line: usize,
    pub(super) column: usize,
    // GOAL.md の必須事項に従い、native の LintIssue と同じ範囲情報を公開する。
    pub(super) end_line: Option<usize>,
    pub(super) end_column: Option<usize>,
    pub(super) message: String,
    pub(super) severity: String,
    pub(super) fixable: bool,
}

/// `LintIssue` を JS 公開用の `WasmLintIssue` へ変換する。
///
/// severity は `LintSeverity` の serde 表現（lowercase）と揃えた文字列にする。
fn to_wasm_issue(issue: &LintIssue) -> WasmLintIssue {
    let severity = match issue.severity {
        LintSeverity::Error => "error",
        LintSeverity::Warning => "warning",
        LintSeverity::Info => "info",
    };
    WasmLintIssue {
        rule: issue.rule.clone(),
        rule_name: issue.rule_name.clone(),
        line: issue.line,
        column: issue.column,
        end_line: issue.end_line,
        end_column: issue.end_column,
        message: issue.message.clone(),
        severity: severity.to_string(),
        fixable: issue.fixable,
    }
}

/// Wasm-compatible lint fix result
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct WasmLintFixResult {
    passed: bool,
    issues: Vec<WasmLintIssue>,
    fixed_content: String,
    fixes_applied: usize,
}

#[wasm_bindgen]
impl WasmLintFixResult {
    #[wasm_bindgen(getter)]
    pub fn passed(&self) -> bool {
        self.passed
    }

    #[wasm_bindgen(getter)]
    pub fn issues(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.issues)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    #[wasm_bindgen(getter)]
    pub fn fixed_content(&self) -> String {
        self.fixed_content.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn fixes_applied(&self) -> usize {
        self.fixes_applied
    }
}
