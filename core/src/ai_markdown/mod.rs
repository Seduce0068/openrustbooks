//! AI Markdown Pipeline - Core module for AI-assisted Markdown generation and refinement.
//!
//! This module implements the pipeline described in ADR-0002:
//! 1. User input (natural language / existing MD + instruction)
//! 2. AI API generates/modifies Markdown (streaming support)
//! 3. `rumdl` validates and auto-fixes (`.markdownlint.yaml` compliant)
//! 4. Retry up to N times (default 3) until lint passes
//! 5. Diff display → User confirmation → Save/Preview

/// HTTP-backed AI client. Native only: `reqwest`/`async-openai` do not build
/// for `wasm32-unknown-unknown`. Browser builds will get a `fetch`-based client.
#[cfg(not(target_arch = "wasm32"))]
pub mod client;
pub mod diff;
pub mod lint;
/// Retry/lint orchestration. Native only, because it drives [`client`].
#[cfg(not(target_arch = "wasm32"))]
pub mod pipeline;
pub mod prompt;

#[cfg(not(target_arch = "wasm32"))]
pub use pipeline::AiMarkdownService;

#[cfg(not(target_arch = "wasm32"))]
use crate::{
    CoreError, CoreResult,
    config::{AiConfig, LintConfig},
};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

/// Main entry point for the AI Markdown pipeline
#[cfg(not(target_arch = "wasm32"))]
pub struct AiMarkdownPipeline {
    client: Arc<dyn client::AiClient + Send + Sync>,
    linter: lint::MarkdownLinter,
    prompt_registry: prompt::PromptRegistry,
    config: PipelineConfig,
}

/// Pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Maximum retry attempts for lint validation
    pub max_retries: usize,
    /// Whether to auto-apply lint fixes
    pub auto_fix: bool,
    /// Whether to show diff before applying
    pub show_diff: bool,
    /// Token limit per request
    pub token_limit: usize,
    /// Enable streaming responses
    pub streaming: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            auto_fix: true,
            show_diff: true,
            token_limit: 8192,
            streaming: true,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl AiMarkdownPipeline {
    /// Get reference to the linter
    pub fn linter(&self) -> &lint::MarkdownLinter {
        &self.linter
    }

    /// Create a new pipeline with the given configuration
    pub async fn new(
        ai_config: &AiConfig,
        lint_config: &LintConfig,
        pipeline_config: PipelineConfig,
    ) -> CoreResult<Self> {
        let client = client::create_client(ai_config).await?;
        let linter = lint::MarkdownLinter::new(lint_config)?;
        let prompt_registry = prompt::PromptRegistry::load()?;

        Ok(Self {
            client,
            linter,
            prompt_registry,
            config: pipeline_config,
        })
    }

    /// Generate Markdown from natural language prompt
    pub async fn generate(&self, request: GenerateRequest) -> CoreResult<GenerateResult> {
        self.execute_pipeline(request.into()).await
    }

    /// Refine existing Markdown with instructions
    pub async fn refine(&self, request: RefineRequest) -> CoreResult<RefineResult> {
        self.execute_pipeline(request.into()).await
    }

    /// Convert structured data (CSV/JSON) to Markdown table
    pub async fn convert_to_table(&self, request: TableRequest) -> CoreResult<TableResult> {
        self.execute_pipeline(request.into()).await
    }

    /// Generate front matter for a document
    pub async fn generate_front_matter(
        &self,
        request: FrontMatterRequest,
    ) -> CoreResult<FrontMatterResult> {
        self.execute_pipeline(request.into()).await
    }

    /// Core pipeline execution with retry logic
    async fn execute_pipeline<Req, Res>(&self, mut request: PipelineRequest<Req>) -> CoreResult<Res>
    where
        Req: PipelineInput + Send + Sync,
        Res: PipelineOutput + Send + Sync,
    {
        let mut current_content = request.initial_content().unwrap_or_default();
        // `PipelineConfig` を直接生成しても呼び出し数が 11 を超えないよう、
        // pipeline.rs の clamp_max_retries と同じ上限をここでも担保する。
        let max = self.config.max_retries.min(10);

        for attempt in 0..=max {
            tracing::debug!(
                "Pipeline attempt {}/{}",
                attempt.saturating_add(1),
                max.saturating_add(1)
            );

            // 1. Generate/modify content via AI
            let ai_result = self.call_ai(&request, &current_content, attempt).await?;

            // 2. Apply lint validation（auto_fix が有効なときだけ自動修正）
            let lint_result =
                validate_content(&self.linter, &ai_result.content, self.config.auto_fix).await?;

            // 3. Check if lint passes
            // lint feedback を prompt に反映できない要求（Table / FrontMatter）は
            // リトライしても同じ prompt / content になるため、初回不合格時点で打ち切る。
            if lint_result.passed || attempt == max || !request.supports_lint_feedback() {
                let original = request.initial_content().unwrap_or_default();
                let fixed = lint_result.fixed_content.clone();
                return Ok(Res::from_pipeline_result(PipelineExecutionResult {
                    original_content: original.clone(),
                    generated_content: ai_result.content,
                    final_content: fixed.clone(),
                    lint_passed: lint_result.passed,
                    lint_issues: lint_result.issues,
                    diff: if self.config.show_diff {
                        Some(crate::ai_markdown::diff::compute(&original, &fixed))
                    } else {
                        None
                    },
                    attempts: attempt.saturating_add(1),
                    tokens_used: ai_result.tokens_used,
                }));
            }

            // 4. Prepare for retry with lint feedback
            current_content = lint_result.fixed_content;

            // Add lint errors to next prompt as context
            request.add_lint_feedback(&lint_result.issues);
        }

        // The `for` loop always returns on the final attempt
        // (`attempt == max`), so this is unreachable in practice.
        // Keep a safe error instead of panicking to satisfy the type checker.
        Err(CoreError::ai_pipeline(
            "Pipeline failed after max retries".to_string(),
        ))
    }

    /// Call AI with appropriate prompt
    async fn call_ai<Req: PipelineInput>(
        &self,
        request: &Req,
        current_content: &str,
        attempt: usize,
    ) -> CoreResult<client::AiResponse> {
        let prompt = self.prompt_registry.render(
            request.prompt_type(),
            request.prompt_variables(current_content, attempt),
        )?;
        self.client
            .complete(&prompt, self.config.token_limit, self.config.streaming)
            .await
    }
}

/// auto_fix に応じて lint のみ／自動修正付き lint を実行する
#[cfg(not(target_arch = "wasm32"))]
async fn validate_content(
    linter: &lint::MarkdownLinter,
    content: &str,
    auto_fix: bool,
) -> CoreResult<lint::LintResultWrapper> {
    lint::validate_with_auto_fix(linter, content, auto_fix).await
}

/// Pipeline request trait
pub trait PipelineInput {
    fn prompt_type(&self) -> prompt::PromptType;
    fn prompt_variables(&self, current_content: &str, attempt: usize) -> prompt::PromptVariables;
    fn initial_content(&self) -> Option<String>;
    fn add_lint_feedback(&mut self, issues: &[lint::LintIssue]);

    /// `add_lint_feedback` が実際に prompt へ反映されるかどうか。
    ///
    /// `false` の要求は lint 不合格後にリトライしても prompt が変わらないため、
    /// pipeline は初回失敗時点で結果を返し、余分な AI 呼び出しを避ける。
    /// 既定は `true`（feedback を反映する既存要求を壊さないため）。
    fn supports_lint_feedback(&self) -> bool {
        true
    }
}

/// Pipeline result trait
pub trait PipelineOutput {
    fn from_pipeline_result(result: PipelineExecutionResult) -> Self;
}

/// Unified pipeline execution result
#[derive(Debug, Clone)]
pub struct PipelineExecutionResult {
    pub original_content: String,
    pub generated_content: String,
    pub final_content: String,
    pub lint_passed: bool,
    pub lint_issues: Vec<lint::LintIssue>,
    pub diff: Option<diff::DiffResult>,
    pub attempts: usize,
    pub tokens_used: usize,
}

// Re-export client::AiResponse for use in this module
#[cfg(not(target_arch = "wasm32"))]
pub use client::AiResponse;

// --- Request/Response Types ---

#[derive(Debug, Clone)]
pub struct GenerateRequest {
    pub prompt: String,
    pub context: Option<String>,
    pub template: Option<String>,
}

impl PipelineInput for GenerateRequest {
    fn prompt_type(&self) -> prompt::PromptType {
        prompt::PromptType::Generate
    }
    fn prompt_variables(&self, current_content: &str, attempt: usize) -> prompt::PromptVariables {
        prompt::PromptVariables {
            user_prompt: self.prompt.clone(),
            context: self.context.clone(),
            current_content: current_content.to_string(),
            attempt,
            template: self.template.clone(),
            ..Default::default()
        }
    }
    fn initial_content(&self) -> Option<String> {
        None
    }
    fn add_lint_feedback(&mut self, issues: &[lint::LintIssue]) {
        let feedback = issues
            .iter()
            .map(|i| format!("- {}: {}", i.rule, i.message))
            .collect::<Vec<_>>()
            .join("\n");
        self.context = Some(format!(
            "{}\n\nPrevious lint issues to fix:\n{}",
            self.context.as_deref().unwrap_or(""),
            feedback
        ));
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GenerateResult {
    pub content: String,
    pub diff: Option<diff::DiffResult>,
    pub lint_passed: bool,
    pub attempts: usize,
    pub tokens_used: usize,
}

impl PipelineOutput for GenerateResult {
    fn from_pipeline_result(r: PipelineExecutionResult) -> Self {
        Self {
            content: r.final_content,
            diff: r.diff,
            lint_passed: r.lint_passed,
            attempts: r.attempts,
            tokens_used: r.tokens_used,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RefineRequest {
    pub content: String,
    pub instruction: String,
    pub context: Option<String>,
}

impl PipelineInput for RefineRequest {
    fn prompt_type(&self) -> prompt::PromptType {
        prompt::PromptType::Refine
    }
    fn prompt_variables(&self, _current_content: &str, attempt: usize) -> prompt::PromptVariables {
        prompt::PromptVariables {
            user_prompt: self.instruction.clone(),
            context: self.context.clone(),
            current_content: self.content.clone(),
            attempt,
            ..Default::default()
        }
    }
    fn initial_content(&self) -> Option<String> {
        Some(self.content.clone())
    }
    fn add_lint_feedback(&mut self, issues: &[lint::LintIssue]) {
        let feedback = issues
            .iter()
            .map(|i| format!("- {}: {}", i.rule, i.message))
            .collect::<Vec<_>>()
            .join("\n");
        self.instruction = format!(
            "{}\n\nPrevious lint issues to fix:\n{}",
            self.instruction, feedback
        );
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RefineResult {
    pub content: String,
    pub diff: Option<diff::DiffResult>,
    pub lint_passed: bool,
    pub attempts: usize,
    pub tokens_used: usize,
}

impl PipelineOutput for RefineResult {
    fn from_pipeline_result(r: PipelineExecutionResult) -> Self {
        Self {
            content: r.final_content,
            diff: r.diff,
            lint_passed: r.lint_passed,
            attempts: r.attempts,
            tokens_used: r.tokens_used,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum TableFormat {
    Csv,
    Json,
    Tsv,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum TableAlignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub struct TableRequest {
    pub data: String, // CSV or JSON
    pub format: TableFormat,
    pub headers: Option<Vec<String>>,
    pub alignment: Option<Vec<TableAlignment>>,
}

impl PipelineInput for TableRequest {
    fn prompt_type(&self) -> prompt::PromptType {
        prompt::PromptType::Table
    }
    fn prompt_variables(&self, current_content: &str, attempt: usize) -> prompt::PromptVariables {
        prompt::PromptVariables {
            user_prompt: self.data.clone(),
            current_content: current_content.to_string(),
            attempt,
            extra: {
                let mut m = std::collections::HashMap::new();
                m.insert(
                    "format".to_string(),
                    serde_json::json!(format!("{:?}", self.format)),
                );
                if let Some(h) = &self.headers {
                    m.insert("headers".to_string(), serde_json::json!(h));
                }
                if let Some(a) = &self.alignment {
                    m.insert("alignment".to_string(), serde_json::json!(a));
                }
                m
            },
            ..Default::default()
        }
    }
    fn initial_content(&self) -> Option<String> {
        None
    }
    fn add_lint_feedback(&mut self, _issues: &[lint::LintIssue]) {}
    fn supports_lint_feedback(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TableResult {
    pub content: String,
    pub diff: Option<diff::DiffResult>,
    pub lint_passed: bool,
    pub attempts: usize,
    pub tokens_used: usize,
}

impl PipelineOutput for TableResult {
    fn from_pipeline_result(r: PipelineExecutionResult) -> Self {
        Self {
            content: r.final_content,
            diff: r.diff,
            lint_passed: r.lint_passed,
            attempts: r.attempts,
            tokens_used: r.tokens_used,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrontMatterRequest {
    pub content: String,
    pub schema: Option<serde_json::Value>,
}

impl PipelineInput for FrontMatterRequest {
    fn prompt_type(&self) -> prompt::PromptType {
        prompt::PromptType::FrontMatter
    }
    fn prompt_variables(&self, _current_content: &str, attempt: usize) -> prompt::PromptVariables {
        prompt::PromptVariables {
            user_prompt: "Generate appropriate YAML front matter for this document".to_string(),
            current_content: self.content.clone(),
            attempt,
            extra: {
                let mut m = std::collections::HashMap::new();
                if let Some(s) = &self.schema {
                    m.insert("schema".to_string(), s.clone());
                }
                m
            },
            ..Default::default()
        }
    }
    fn initial_content(&self) -> Option<String> {
        Some(self.content.clone())
    }
    fn add_lint_feedback(&mut self, _issues: &[lint::LintIssue]) {}
    fn supports_lint_feedback(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FrontMatterResult {
    pub content: String, // Full document with front matter
    pub front_matter: serde_json::Map<String, serde_json::Value>,
    pub diff: Option<diff::DiffResult>,
    pub lint_passed: bool,
    pub attempts: usize,
    pub tokens_used: usize,
}

impl PipelineOutput for FrontMatterResult {
    fn from_pipeline_result(r: PipelineExecutionResult) -> Self {
        // Extract front matter from final content
        let (fm, _) = crate::document::extract_front_matter(&r.final_content);
        Self {
            content: r.final_content,
            front_matter: fm.unwrap_or_default(),
            diff: r.diff,
            lint_passed: r.lint_passed,
            attempts: r.attempts,
            tokens_used: r.tokens_used,
        }
    }
}

// Pipeline request wrapper
#[cfg(not(target_arch = "wasm32"))]
struct PipelineRequest<Req> {
    inner: Req,
}

#[cfg(not(target_arch = "wasm32"))]
impl<Req: PipelineInput> From<Req> for PipelineRequest<Req> {
    fn from(inner: Req) -> Self {
        Self { inner }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<Req: PipelineInput> PipelineInput for PipelineRequest<Req> {
    fn prompt_type(&self) -> prompt::PromptType {
        self.inner.prompt_type()
    }
    fn prompt_variables(&self, current_content: &str, attempt: usize) -> prompt::PromptVariables {
        self.inner.prompt_variables(current_content, attempt)
    }
    fn initial_content(&self) -> Option<String> {
        self.inner.initial_content()
    }
    fn add_lint_feedback(&mut self, issues: &[lint::LintIssue]) {
        self.inner.add_lint_feedback(issues)
    }
    fn supports_lint_feedback(&self) -> bool {
        self.inner.supports_lint_feedback()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const DIRTY: &str = "# Title\n\nTrailing spaces here   \n";

    fn linter() -> lint::MarkdownLinter {
        lint::MarkdownLinter::new(&LintConfig::default()).expect("linter should build")
    }

    #[tokio::test]
    async fn auto_fix_false_keeps_content_unchanged() {
        let result = validate_content(&linter(), DIRTY, false)
            .await
            .expect("validate should succeed");

        assert_eq!(result.fixed_content, DIRTY);
        assert_eq!(result.fixes_applied, 0);
        assert!(result.converged, "lint のみの経路は収束扱いになること");
        assert!(
            result.conflicting_rules.is_empty(),
            "lint のみの経路には競合ルールがないこと"
        );
        assert!(
            !result.issues.is_empty(),
            "自動修正しないので問題は残ること"
        );
    }

    fn strict_linter() -> lint::MarkdownLinter {
        let config = LintConfig {
            warnings_as_errors: true,
            ..Default::default()
        };
        lint::MarkdownLinter::new(&config).expect("linter should build")
    }

    #[tokio::test]
    async fn warnings_as_errors_fails_lint_only_path() {
        // MD033（インライン HTML）は修正不能な Warning を出す
        let content = "# Title\n\n<div>html</div>\n";

        let lenient = validate_content(&linter(), content, false)
            .await
            .expect("validate should succeed");
        assert!(
            lenient
                .issues
                .iter()
                .any(|i| i.severity == lint::LintSeverity::Warning),
            "比較対象に Warning があること"
        );
        assert!(
            lenient.passed,
            "既定 false の lint のみ経路では Warning を無視すること"
        );

        let strict = validate_content(&strict_linter(), content, false)
            .await
            .expect("validate should succeed");
        assert!(
            strict
                .issues
                .iter()
                .any(|i| i.severity == lint::LintSeverity::Warning),
            "severity は Warning のままであること"
        );
        assert!(
            !strict.passed,
            "true の lint のみ経路では Warning で不合格とすること"
        );
        assert_eq!(
            strict.fixed_content, content,
            "lint のみ経路は本文を変えないこと"
        );
    }

    #[tokio::test]
    async fn auto_fix_true_applies_fixes() {
        let result = validate_content(&linter(), DIRTY, true)
            .await
            .expect("validate should succeed");

        assert_ne!(result.fixed_content, DIRTY);
        assert!(result.fixes_applied > 0);
        assert!(result.passed);
        assert!(result.converged);
        assert!(result.conflicting_rules.is_empty());
    }

    /// complete の呼び出し回数だけを記録するモック（外部 API は呼ばない）
    struct CountingAiClient {
        content: &'static str,
        calls: AtomicUsize,
    }

    #[async_trait]
    impl client::AiClient for CountingAiClient {
        async fn complete(
            &self,
            _prompt: &str,
            _max_tokens: usize,
            _streaming: bool,
        ) -> CoreResult<client::AiResponse> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(client::AiResponse {
                content: self.content.to_string(),
                tokens_used: 11,
                model: "mock".to_string(),
            })
        }

        async fn stream_complete(
            &self,
            _prompt: &str,
            _max_tokens: usize,
        ) -> CoreResult<Box<dyn client::TokenStream + Send>> {
            Err(CoreError::ai_pipeline(
                "stream is unused in tests".to_string(),
            ))
        }

        fn count_tokens(&self, text: &str) -> CoreResult<usize> {
            Ok(text.len())
        }

        fn model_name(&self) -> &str {
            "mock"
        }
    }

    /// モックを直接差し込んで pipeline を組み立てる（本番コードは変更しない）
    fn pipeline_with_mock(
        mock: Arc<CountingAiClient>,
        linter: lint::MarkdownLinter,
        config: PipelineConfig,
    ) -> AiMarkdownPipeline {
        AiMarkdownPipeline {
            client: mock,
            linter,
            prompt_registry: prompt::PromptRegistry::with_builtins()
                .expect("built-in templates should load"),
            config,
        }
    }

    fn zero_retry_config() -> PipelineConfig {
        PipelineConfig {
            max_retries: 0,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn zero_max_retries_runs_once_and_reports_lint_pass() {
        // lint を通る生成物なら 1 回で完了し lint_passed=true
        let clean = "# Title\n\nTrailing spaces here\n";
        let mock = Arc::new(CountingAiClient {
            content: clean,
            calls: AtomicUsize::new(0),
        });
        let pipeline = pipeline_with_mock(mock.clone(), linter(), zero_retry_config());

        let result = pipeline
            .generate(GenerateRequest {
                prompt: "タイトルを生成".to_string(),
                context: None,
                template: None,
            })
            .await
            .expect("pipeline should succeed");

        assert_eq!(result.attempts, 1, "max_retries=0 は初期 1 回のみ");
        assert_eq!(
            mock.calls.load(Ordering::SeqCst),
            1,
            "complete は 1 回だけ呼ばれること"
        );
        assert!(
            result.lint_passed,
            "クリーンな生成物は lint 合格と報告すること"
        );
        assert_eq!(result.content, clean, "生成物がそのまま返ること");
    }

    #[tokio::test]
    async fn zero_max_retries_keeps_ok_when_lint_fails() {
        // 最終試行の lint 不合格でも Ok ＋ lint_passed=false（ADR-0116）
        let html = "# Title\n\n<div>html</div>\n";
        let mock = Arc::new(CountingAiClient {
            content: html,
            calls: AtomicUsize::new(0),
        });
        let config = PipelineConfig {
            max_retries: 0,
            auto_fix: false,
            ..Default::default()
        };
        let pipeline = pipeline_with_mock(mock.clone(), strict_linter(), config);

        let result = pipeline
            .generate(GenerateRequest {
                prompt: "HTML を含む生成".to_string(),
                context: None,
                template: None,
            })
            .await;
        assert!(
            result.is_ok(),
            "最終 attempt の lint 不合格でも Ok を返すこと"
        );
        let result = result.expect("lint 不合格でも Ok を返すこと");

        assert_eq!(result.attempts, 1, "lint 不合格でもリトライしないこと");
        assert_eq!(
            mock.calls.load(Ordering::SeqCst),
            1,
            "complete は 1 回だけ呼ばれること"
        );
        assert!(!result.lint_passed, "不合格をそのまま報告すること");
        assert_eq!(result.content, html, "lint のみ経路は本文を変えないこと");
        assert!(
            result.diff.is_some(),
            "show_diff 既定で差分が保持されること"
        );
    }

    /// lint 不合格を確実に生じさせるモック生成物（修正不能な HTML）。
    const UNFIXABLE_HTML: &str = "# Title\n\n<div>html</div>\n";

    /// リトライ余裕を残し、feedback の有無で呼び出し回数が変わることを検証する設定。
    fn retry_config() -> PipelineConfig {
        PipelineConfig {
            max_retries: 2,
            auto_fix: false,
            ..Default::default()
        }
    }

    #[test]
    fn lint_feedback_support_by_request_type() {
        let generate = GenerateRequest {
            prompt: "p".to_string(),
            context: None,
            template: None,
        };
        let refine = RefineRequest {
            content: "# T\n".to_string(),
            instruction: "i".to_string(),
            context: None,
        };
        let table = TableRequest {
            data: "a,b\n1,2\n".to_string(),
            format: TableFormat::Csv,
            headers: None,
            alignment: None,
        };
        let front_matter = FrontMatterRequest {
            content: "# T\n".to_string(),
            schema: None,
        };

        assert!(generate.supports_lint_feedback());
        assert!(refine.supports_lint_feedback());
        assert!(!table.supports_lint_feedback());
        assert!(!front_matter.supports_lint_feedback());
    }

    /// no-op 要求（Table）は初回 lint 不合格で打ち切り、AI は 1 回のみ呼ばれる。
    #[tokio::test]
    async fn table_request_without_lint_feedback_stops_after_first_attempt() {
        let mock = Arc::new(CountingAiClient {
            content: UNFIXABLE_HTML,
            calls: AtomicUsize::new(0),
        });
        let pipeline = pipeline_with_mock(mock.clone(), strict_linter(), retry_config());

        let result = pipeline
            .convert_to_table(TableRequest {
                data: "a,b\n1,2\n".to_string(),
                format: TableFormat::Csv,
                headers: None,
                alignment: None,
            })
            .await
            .expect("pipeline should succeed");

        assert_eq!(result.attempts, 1, "no-op 要求は初回のみで打ち切ること");
        assert_eq!(
            mock.calls.load(Ordering::SeqCst),
            1,
            "complete は 1 回だけ呼ばれること"
        );
        assert!(!result.lint_passed, "lint 不合格をそのまま報告すること");
        assert_eq!(
            result.content, UNFIXABLE_HTML,
            "lint のみ経路は本文を変えないこと"
        );
    }

    /// no-op 要求（FrontMatter）も同様に 1 回のみ呼ばれる。
    #[tokio::test]
    async fn front_matter_request_without_lint_feedback_stops_after_first_attempt() {
        let mock = Arc::new(CountingAiClient {
            content: UNFIXABLE_HTML,
            calls: AtomicUsize::new(0),
        });
        let pipeline = pipeline_with_mock(mock.clone(), strict_linter(), retry_config());

        let result = pipeline
            .generate_front_matter(FrontMatterRequest {
                content: "# Title\n".to_string(),
                schema: None,
            })
            .await
            .expect("pipeline should succeed");

        assert_eq!(result.attempts, 1, "no-op 要求は初回のみで打ち切ること");
        assert_eq!(
            mock.calls.load(Ordering::SeqCst),
            1,
            "complete は 1 回だけ呼ばれること"
        );
        assert!(!result.lint_passed, "lint 不合格をそのまま報告すること");
        assert_eq!(result.content, UNFIXABLE_HTML, "生成物がそのまま返ること");
    }

    /// feedback 有効要求（Generate）は従来どおり max_retries + 1 回リトライする。
    #[tokio::test]
    async fn generate_request_with_lint_feedback_retries_until_max() {
        let mock = Arc::new(CountingAiClient {
            content: UNFIXABLE_HTML,
            calls: AtomicUsize::new(0),
        });
        let pipeline = pipeline_with_mock(mock.clone(), strict_linter(), retry_config());

        let result = pipeline
            .generate(GenerateRequest {
                prompt: "HTML を含む生成".to_string(),
                context: None,
                template: None,
            })
            .await
            .expect("pipeline should succeed");

        assert_eq!(
            result.attempts, 3,
            "feedback 有効要求は max_retries + 1 回試行すること"
        );
        assert_eq!(
            mock.calls.load(Ordering::SeqCst),
            3,
            "feedback 有効要求は従来どおり retry すること"
        );
        assert!(!result.lint_passed, "不合格をそのまま報告すること");
    }

    /// feedback 有効要求（Refine）も従来どおりリトライする。
    #[tokio::test]
    async fn refine_request_with_lint_feedback_retries_until_max() {
        let mock = Arc::new(CountingAiClient {
            content: UNFIXABLE_HTML,
            calls: AtomicUsize::new(0),
        });
        let pipeline = pipeline_with_mock(mock.clone(), strict_linter(), retry_config());

        let result = pipeline
            .refine(RefineRequest {
                content: "# Title\n".to_string(),
                instruction: "HTML を追加".to_string(),
                context: None,
            })
            .await
            .expect("pipeline should succeed");

        assert_eq!(
            result.attempts, 3,
            "feedback 有効要求は max_retries + 1 回試行すること"
        );
        assert_eq!(
            mock.calls.load(Ordering::SeqCst),
            3,
            "feedback 有効要求は従来どおり retry すること"
        );
        assert!(!result.lint_passed, "不合格をそのまま報告すること");
    }

    /// PipelineConfig を直接生成して max_retries=usize::MAX でも 11 回で打ち切り、
    /// 実質的な無限ループ / オーバーフローが起きないこと（R4 指摘の回帰テスト）。
    #[tokio::test]
    async fn oversized_max_retries_is_clamped_inside_pipeline() {
        let mock = Arc::new(CountingAiClient {
            content: UNFIXABLE_HTML,
            calls: AtomicUsize::new(0),
        });
        let config = PipelineConfig {
            max_retries: usize::MAX,
            auto_fix: false,
            ..Default::default()
        };
        let pipeline = pipeline_with_mock(mock.clone(), strict_linter(), config);

        let result = pipeline
            .generate(GenerateRequest {
                prompt: "HTML を含む生成".to_string(),
                context: None,
                template: None,
            })
            .await
            .expect("pipeline should succeed");

        assert_eq!(
            result.attempts, 11,
            "max_retries は 10 にクランプされること"
        );
        assert_eq!(
            mock.calls.load(Ordering::SeqCst),
            11,
            "complete は最大 11 回に留まること"
        );
        assert!(!result.lint_passed, "不合格をそのまま報告すること");
    }
}
