//! High-level pipeline orchestration for AI Markdown operations.

use crate::ai_markdown::{
    AiMarkdownPipeline, FrontMatterRequest, GenerateRequest, PipelineConfig, RefineRequest,
    TableRequest,
};
use crate::{CoreError, CoreResult, config::AppConfig};
use std::sync::Arc;

/// リトライ回数を 0..=10 にクランプする（呼び出し増と +1 のオーバーフローを防ぐ）。
pub(crate) fn clamp_max_retries(value: usize) -> usize {
    value.min(10)
}

/// High-level service for AI Markdown operations
pub struct AiMarkdownService {
    pipeline: Arc<AiMarkdownPipeline>,
    config: AppConfig,
}

impl AiMarkdownService {
    /// Get reference to the pipeline for direct access
    pub fn pipeline(&self) -> &AiMarkdownPipeline {
        &self.pipeline
    }
}

impl AiMarkdownService {
    /// Create a new service from app configuration
    pub async fn new(config: &AppConfig) -> CoreResult<Self> {
        let pipeline_config = PipelineConfig {
            max_retries: clamp_max_retries(config.ai.max_retries),
            auto_fix: config.lint.auto_fix,
            show_diff: true,
            token_limit: config.ai.max_tokens_per_request,
            streaming: config.ai.streaming,
        };

        let pipeline = AiMarkdownPipeline::new(&config.ai, &config.lint, pipeline_config).await?;
        Ok(Self {
            pipeline: Arc::new(pipeline),
            config: config.clone(),
        })
    }

    /// Generate Markdown from natural language
    pub async fn generate(
        &self,
        prompt: &str,
        context: Option<&str>,
    ) -> CoreResult<GenerateResult> {
        self.pipeline
            .generate(GenerateRequest {
                prompt: prompt.to_string(),
                context: context.map(|s| s.to_string()),
                template: None,
            })
            .await
    }

    /// Refine existing Markdown
    pub async fn refine(
        &self,
        content: &str,
        instruction: &str,
        context: Option<&str>,
    ) -> CoreResult<RefineResult> {
        self.pipeline
            .refine(RefineRequest {
                content: content.to_string(),
                instruction: instruction.to_string(),
                context: context.map(|s| s.to_string()),
            })
            .await
    }

    /// Convert data to Markdown table
    pub async fn to_table(
        &self,
        data: &str,
        format: TableFormat,
        headers: Option<Vec<String>>,
    ) -> CoreResult<TableResult> {
        self.pipeline
            .convert_to_table(TableRequest {
                data: data.to_string(),
                format,
                headers,
                alignment: None,
            })
            .await
    }

    /// Generate front matter for document
    pub async fn front_matter(
        &self,
        content: &str,
        schema: Option<serde_json::Value>,
    ) -> CoreResult<FrontMatterResult> {
        self.pipeline
            .generate_front_matter(FrontMatterRequest {
                content: content.to_string(),
                schema,
            })
            .await
    }

    /// 疑似ストリーム: generate() の完了本文を 1 回だけ on_token へ渡す。
    /// トークン単位の配信ではない（ADR-0216）。
    pub async fn stream_generate<F>(
        &self,
        prompt: &str,
        context: Option<&str>,
        mut on_token: F,
    ) -> CoreResult<GenerateResult>
    where
        F: FnMut(&str) + Send + 'static,
    {
        // 本番の stream_complete は未接続（ADR-0216）。完了本文を 1 回だけ通知する従来の挙動を維持する。
        let result = self.generate(prompt, context).await?;
        if !result.content.is_empty() {
            on_token(&result.content);
        }
        Ok(result)
    }

    /// Check if AI features are enabled
    pub fn is_enabled(&self) -> bool {
        self.config.ai.enabled
    }

    /// Get current AI model name
    pub fn model_name(&self) -> &str {
        &self.config.ai.openai.model
    }

    /// Estimate token count for a prompt
    pub fn estimate_tokens(&self, text: &str) -> CoreResult<usize> {
        // This would use the client's tokenizer
        // Rough approximation: 1 token ≈ 4 chars for English, ~2 for Japanese
        Ok(text.chars().count() / 3)
    }
}

/// Result types re-exported for convenience
pub use crate::ai_markdown::{
    FrontMatterResult, GenerateResult, RefineResult, TableAlignment, TableFormat, TableResult,
};

/// Streaming callback type
pub type TokenCallback = Box<dyn FnMut(&str) + Send>;

/// Callback invoked once a streaming operation completes successfully
pub type CompleteCallback = Box<dyn FnOnce(&GenerateResult) + Send>;

/// Callback invoked once a streaming operation fails
pub type ErrorCallback = Box<dyn FnOnce(&CoreError) + Send>;

/// Streaming generate options
#[derive(Default)]
pub struct StreamOptions {
    pub on_token: Option<TokenCallback>,
    pub on_complete: Option<CompleteCallback>,
    pub on_error: Option<ErrorCallback>,
}

/// Builder for AI operations
pub struct AiOperationBuilder {
    service: Arc<AiMarkdownService>,
    prompt: String,
    context: Option<String>,
    instruction: Option<String>,
    content: Option<String>,
    format: Option<TableFormat>,
    headers: Option<Vec<String>>,
    schema: Option<serde_json::Value>,
}

impl AiOperationBuilder {
    pub fn new(service: Arc<AiMarkdownService>) -> Self {
        Self {
            service,
            prompt: String::new(),
            context: None,
            instruction: None,
            content: None,
            format: None,
            headers: None,
            schema: None,
        }
    }

    pub fn prompt(mut self, prompt: &str) -> Self {
        self.prompt = prompt.to_string();
        self
    }

    pub fn context(mut self, context: &str) -> Self {
        self.context = Some(context.to_string());
        self
    }

    pub fn instruction(mut self, instruction: &str) -> Self {
        self.instruction = Some(instruction.to_string());
        self
    }

    pub fn content(mut self, content: &str) -> Self {
        self.content = Some(content.to_string());
        self
    }

    pub fn format(mut self, format: TableFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn headers(mut self, headers: Vec<String>) -> Self {
        self.headers = Some(headers);
        self
    }

    pub fn schema(mut self, schema: serde_json::Value) -> Self {
        self.schema = Some(schema);
        self
    }

    /// Execute as generation
    pub async fn generate(self) -> CoreResult<GenerateResult> {
        if self.prompt.is_empty() {
            return Err(CoreError::ai_pipeline("Prompt is required for generation"));
        }
        self.service
            .generate(&self.prompt, self.context.as_deref())
            .await
    }

    /// Execute as refinement
    pub async fn refine(self) -> CoreResult<RefineResult> {
        let content = self
            .content
            .ok_or_else(|| CoreError::ai_pipeline("Content is required for refinement"))?;
        let instruction = self
            .instruction
            .ok_or_else(|| CoreError::ai_pipeline("Instruction is required for refinement"))?;
        self.service
            .refine(&content, &instruction, self.context.as_deref())
            .await
    }

    /// Execute as table conversion
    pub async fn table(self) -> CoreResult<TableResult> {
        let data = self
            .content
            .ok_or_else(|| CoreError::ai_pipeline("Data is required for table conversion"))?;
        let format = self.format.unwrap_or(TableFormat::Csv);
        self.service.to_table(&data, format, self.headers).await
    }

    /// Execute as front matter generation
    pub async fn front_matter(self) -> CoreResult<FrontMatterResult> {
        let content = self.content.ok_or_else(|| {
            CoreError::ai_pipeline("Content is required for front matter generation")
        })?;
        self.service.front_matter(&content, self.schema).await
    }
}

/// Extension trait for easier AI operations
pub trait AiMarkdownExt {
    fn ai(&self) -> AiOperationBuilder;
}

impl AiMarkdownExt for Arc<AiMarkdownService> {
    fn ai(&self) -> AiOperationBuilder {
        AiOperationBuilder::new(self.clone())
    }
}

impl AiMarkdownExt for &Arc<AiMarkdownService> {
    fn ai(&self) -> AiOperationBuilder {
        AiOperationBuilder::new((*self).clone())
    }
}

#[cfg(test)]
mod tests {
    use super::clamp_max_retries;

    #[test]
    fn clamp_zero_stays_zero() {
        assert_eq!(clamp_max_retries(0), 0);
    }

    #[test]
    fn clamp_three_stays_three() {
        assert_eq!(clamp_max_retries(3), 3);
    }

    #[test]
    fn clamp_five_stays_five() {
        assert_eq!(clamp_max_retries(5), 5);
    }

    #[test]
    fn clamp_eleven_becomes_ten() {
        assert_eq!(clamp_max_retries(11), 10);
    }

    #[test]
    fn clamp_usize_max_becomes_ten() {
        assert_eq!(clamp_max_retries(usize::MAX), 10);
    }
}
