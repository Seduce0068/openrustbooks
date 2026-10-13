//! AI Provider abstraction and implementations.

use async_trait::async_trait;
use crate::{CoreResult, CoreError, config::{AiConfig, OpenAiConfig, OllamaConfig}};
use async_openai::{Client, config::OpenAIConfig};
use async_openai::types::{
    ChatCompletionRequestMessage, CreateChatCompletionRequest, Role,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
};
use futures::StreamExt;
use tiktoken_rs::{get_bpe_from_model, CoreBPE};

/// Trait for AI clients
#[async_trait]
pub trait AiClient: Send + Sync {
    /// Complete a prompt and return the response
    async fn complete(&self, prompt: &str, max_tokens: usize, streaming: bool) -> CoreResult<AiResponse>;

    /// Stream completion tokens
    async fn stream_complete(&self, prompt: &str, max_tokens: usize) -> CoreResult<Box<dyn TokenStream>>;

    /// Count tokens for a prompt
    fn count_tokens(&self, text: &str) -> CoreResult<usize>;

    /// Get model name
    fn model_name(&self) -> &str;
}

/// AI response
#[derive(Debug, Clone)]
pub struct AiResponse {
    pub content: String,
    pub tokens_used: usize,
    pub model: String,
}

/// Token stream for streaming responses
#[async_trait]
pub trait TokenStream: Send + Sync {
    async fn next_token(&mut self) -> CoreResult<Option<String>>;
    fn is_done(&self) -> bool;
}

/// Create an AI client from configuration
pub async fn create_client(config: &AiConfig) -> CoreResult<Box<dyn AiClient>> {
    match config.default_provider.as_str() {
        "openai" => Ok(Box::new(OpenAiClient::new(&config.openai).await?)),
        "ollama" => Ok(Box::new(OllamaClient::new(&config.ollama).await?)),
        _ => Err(CoreError::ai_provider(format!("Unknown provider: {}", config.default_provider))),
    }
}

/// OpenAI-compatible client
pub struct OpenAiClient {
    client: Client<OpenAIConfig>,
    model: String,
    tokenizer: CoreBPE,
}

impl OpenAiClient {
    pub async fn new(config: &OpenAiConfig) -> CoreResult<Self> {
        let openai_config = OpenAIConfig::new()
            .with_api_base(&config.base_url)
            .with_api_key(config.api_key.as_deref().unwrap_or(""))
            .with_org_id(config.organization.as_deref().unwrap_or(""));

        let client = Client::with_config(openai_config);
        let tokenizer = get_bpe_from_model(&config.model)?;

        Ok(Self {
            client,
            model: config.model.clone(),
            tokenizer,
        })
    }
}

#[async_trait]
impl AiClient for OpenAiClient {
    async fn complete(&self, prompt: &str, max_tokens: usize, _streaming: bool) -> CoreResult<AiResponse> {
        let messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content("You are a Markdown expert. Generate clean, well-structured Markdown that passes markdownlint validation.")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()?
                .into(),
        ];

        let request = CreateChatCompletionRequest {
            model: self.model.clone(),
            messages,
            max_tokens: Some(max_tokens as u32),
            temperature: Some(0.3),
            top_p: Some(0.9),
            stream: Some(false),
            ..Default::default()
        };

        let response = self.client.chat().create(request).await
            .map_err(|e| CoreError::ai_provider(format!("OpenAI API error: {}", e)))?;

        let content = response.choices.first()
            .and_then(|c| c.message.content.as_ref())
            .cloned()
            .unwrap_or_default();

        let tokens_used = response.usage.map(|u| u.total_tokens as usize).unwrap_or_else(|| {
            self.count_tokens(&content).unwrap_or(0)
        });

        Ok(AiResponse {
            content,
            tokens_used,
            model: self.model.clone(),
        })
    }

    async fn stream_complete(&self, prompt: &str, max_tokens: usize) -> CoreResult<Box<dyn TokenStream>> {
        let messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content("You are a Markdown expert. Generate clean, well-structured Markdown that passes markdownlint validation.")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()?
                .into(),
        ];

        let request = CreateChatCompletionRequest {
            model: self.model.clone(),
            messages,
            max_tokens: Some(max_tokens as u32),
            temperature: Some(0.3),
            top_p: Some(0.9),
            stream: Some(true),
            ..Default::default()
        };

        let stream = self.client.chat().create_stream(request).await
            .map_err(|e| CoreError::ai_provider(format!("OpenAI stream error: {}", e)))?;

        Ok(Box::new(OpenAiTokenStream { stream, buffer: String::new() }))
    }

    fn count_tokens(&self, text: &str) -> CoreResult<usize> {
        Ok(self.tokenizer.encode_with_special_tokens(text).len())
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

/// OpenAI token stream
struct OpenAiTokenStream {
    stream: async_openai::types::ChatCompletionStream,
    buffer: String,
}

#[async_trait]
impl TokenStream for OpenAiTokenStream {
    async fn next_token(&mut self) -> CoreResult<Option<String>> {
        while let Some(chunk) = self.stream.next().await {
            let chunk = chunk.map_err(|e| CoreError::ai_provider(format!("Stream error: {}", e)))?;
            if let Some(choice) = chunk.choices.first() {
                if let Some(delta) = &choice.delta.content {
                    self.buffer.push_str(delta);
                    return Ok(Some(delta.clone()));
                }
                if choice.finish_reason.is_some() {
                    return Ok(None);
                }
            }
        }
        Ok(None)
    }

    fn is_done(&self) -> bool {
        false // Simplified
    }
}

/// Ollama local LLM client
pub struct OllamaClient {
    client: reqwest::Client,
    base_url: String,
    model: String,
    tokenizer: CoreBPE,
}

impl OllamaClient {
    pub async fn new(config: &OllamaConfig) -> CoreResult<Self> {
        let client = reqwest::Client::new();
        let tokenizer = get_bpe_from_model("cl100k_base")?; // Approximation

        Ok(Self {
            client,
            base_url: config.base_url.clone(),
            model: config.model.clone(),
            tokenizer,
        })
    }
}

#[async_trait]
impl AiClient for OllamaClient {
    async fn complete(&self, prompt: &str, max_tokens: usize, _streaming: bool) -> CoreResult<AiResponse> {
        let request = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "num_predict": max_tokens,
                "temperature": 0.3,
                "top_p": 0.9,
            }
        });

        let response = self.client
            .post(&format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| CoreError::ai_provider(format!("Ollama request error: {}", e)))?;

        let result: serde_json::Value = response.json().await
            .map_err(|e| CoreError::ai_provider(format!("Ollama response error: {}", e)))?;

        let content = result["response"].as_str().unwrap_or("").to_string();
        let tokens_used = self.count_tokens(&content)?;

        Ok(AiResponse {
            content,
            tokens_used,
            model: self.model.clone(),
        })
    }

    async fn stream_complete(&self, prompt: &str, max_tokens: usize) -> CoreResult<Box<dyn TokenStream>> {
        let request = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": true,
            "options": {
                "num_predict": max_tokens,
                "temperature": 0.3,
                "top_p": 0.9,
            }
        });

        let response = self.client
            .post(&format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| CoreError::ai_provider(format!("Ollama stream request error: {}", e)))?;

        let stream = response.bytes_stream();
        Ok(Box::new(OllamaTokenStream { stream, buffer: String::new() }))
    }

    fn count_tokens(&self, text: &str) -> CoreResult<usize> {
        Ok(self.tokenizer.encode_with_special_tokens(text).len())
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

/// Ollama token stream
struct OllamaTokenStream {
    stream: futures::stream::BoxStream<'static, Result<bytes::Bytes, reqwest::Error>>,
    buffer: String,
}

#[async_trait]
impl TokenStream for OllamaTokenStream {
    async fn next_token(&mut self) -> CoreResult<Option<String>> {
        use futures::StreamExt;

        while let Some(chunk) = self.stream.next().await {
            let chunk = chunk.map_err(|e| CoreError::ai_provider(format!("Ollama stream error: {}", e)))?;
            let text = String::from_utf8_lossy(&chunk);

            // Ollama streams JSON lines
            for line in text.lines() {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(response) = json["response"].as_str() {
                        self.buffer.push_str(response);
                        return Ok(Some(response.to_string()));
                    }
                    if json["done"].as_bool().unwrap_or(false) {
                        return Ok(None);
                    }
                }
            }
        }
        Ok(None)
    }

    fn is_done(&self) -> bool {
        false
    }
}