//! AI Provider abstraction and implementations.

use crate::{
    CoreError, CoreResult,
    config::{AiConfig, OllamaConfig, OpenAiConfig},
};
use async_openai::types::chat::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    ChatCompletionResponseStream, CreateChatCompletionRequest,
};
use async_openai::{Client, config::OpenAIConfig};
use async_trait::async_trait;
use futures::StreamExt;
use std::pin::Pin;
use std::sync::Arc;
use tiktoken_rs::{CoreBPE, bpe_for_model, cl100k_base_singleton};
use url::Url;

/// Trait for AI clients
#[async_trait]
pub trait AiClient: Send + Sync {
    /// Complete a prompt and return the response
    async fn complete(
        &self,
        prompt: &str,
        max_tokens: usize,
        streaming: bool,
    ) -> CoreResult<AiResponse>;

    /// Stream completion tokens
    async fn stream_complete(
        &self,
        prompt: &str,
        max_tokens: usize,
    ) -> CoreResult<Box<dyn TokenStream + Send>>;

    /// Count tokens for a prompt
    fn count_tokens(&self, text: &str) -> CoreResult<usize>;

    /// Get model name
    fn model_name(&self) -> &str;
}

/// Automatically implement AiClient for Arc<dyn AiClient>
#[async_trait]
impl AiClient for Arc<dyn AiClient + Send + Sync> {
    async fn complete(
        &self,
        prompt: &str,
        max_tokens: usize,
        streaming: bool,
    ) -> CoreResult<AiResponse> {
        self.as_ref().complete(prompt, max_tokens, streaming).await
    }

    async fn stream_complete(
        &self,
        prompt: &str,
        max_tokens: usize,
    ) -> CoreResult<Box<dyn TokenStream + Send>> {
        self.as_ref().stream_complete(prompt, max_tokens).await
    }

    fn count_tokens(&self, text: &str) -> CoreResult<usize> {
        self.as_ref().count_tokens(text)
    }

    fn model_name(&self) -> &str {
        self.as_ref().model_name()
    }
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
pub trait TokenStream: Send + Unpin + 'static {
    async fn next_token(&mut self) -> CoreResult<Option<String>>;
    fn is_done(&self) -> bool;
}

/// Provider whose base URL is being validated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    /// OpenAI (official hosted endpoint).
    OpenAi,
    /// Ollama (local loopback endpoint).
    Ollama,
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAi => write!(f, "openai"),
            Self::Ollama => write!(f, "ollama"),
        }
    }
}

/// ハードコードされた既定ホストのみ許可する。
///
/// - OpenAI: scheme `https` + host `api.openai.com` + port なし/443
/// - Ollama: scheme `http` + loopback host（`localhost` / `127.0.0.1` / `[::1]`）+ port 11434
///
/// path は互換性のため厳密に検査しない。userinfo は常に拒否する。
/// カスタムエンドポイント（Azure OpenAI 等）の opt-in は将来課題（ADR）。
/// エラーメッセージには host と理由のみを含め、userinfo を漏らさない。
pub fn validate_base_url(url: &str, provider: ProviderKind) -> CoreResult<()> {
    let parsed = Url::parse(url)
        .map_err(|_| CoreError::validation(format!("Invalid {provider} base URL")))?;

    // userinfo 付き URL は見せかけのホストになり得るため拒否する。
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(CoreError::validation(format!(
            "Base URL for {provider} must not contain userinfo"
        )));
    }

    let host = parsed.host_str().unwrap_or_default();
    let port = parsed.port();
    let allowed = match provider {
        ProviderKind::OpenAi => {
            parsed.scheme() == "https"
                && host.eq_ignore_ascii_case("api.openai.com")
                && matches!(port, None | Some(443))
        }
        ProviderKind::Ollama => {
            parsed.scheme() == "http"
                && port == Some(11434)
                && (host.eq_ignore_ascii_case("localhost")
                    || host.eq_ignore_ascii_case("127.0.0.1")
                    || host.eq_ignore_ascii_case("[::1]"))
        }
    };

    if allowed {
        Ok(())
    } else {
        Err(CoreError::validation(format!(
            "Base URL host '{host}' is not allowed for provider {provider}"
        )))
    }
}

/// Create an AI client from configuration
pub async fn create_client(config: &AiConfig) -> CoreResult<Arc<dyn AiClient + Send + Sync>> {
    match config.default_provider.as_str() {
        "openai" => Ok(Arc::new(OpenAiClient::new(&config.openai).await?)),
        "ollama" => Ok(Arc::new(OllamaClient::new(&config.ollama).await?)),
        _ => Err(CoreError::ai_provider(format!(
            "Unknown provider: {}",
            config.default_provider
        ))),
    }
}

/// OpenAI-compatible client
pub struct OpenAiClient {
    client: Client<OpenAIConfig>,
    model: String,
    tokenizer: &'static CoreBPE,
}

impl OpenAiClient {
    pub async fn new(config: &OpenAiConfig) -> CoreResult<Self> {
        validate_base_url(&config.base_url, ProviderKind::OpenAi)?;

        let openai_config = OpenAIConfig::new()
            .with_api_base(&config.base_url)
            .with_api_key(config.api_key.as_deref().unwrap_or(""))
            .with_org_id(config.organization.as_deref().unwrap_or(""));

        let client = Client::with_config(openai_config);
        let tokenizer = bpe_for_model(&config.model)?;

        Ok(Self {
            client,
            model: config.model.clone(),
            tokenizer,
        })
    }
}

#[async_trait]
impl AiClient for OpenAiClient {
    async fn complete(
        &self,
        prompt: &str,
        max_tokens: usize,
        _streaming: bool,
    ) -> CoreResult<AiResponse> {
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
            max_completion_tokens: Some(max_tokens as u32),
            temperature: Some(0.3),
            top_p: Some(0.9),
            stream: Some(false),
            ..Default::default()
        };

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| CoreError::ai_provider(format!("OpenAI API error: {}", e)))?;

        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.as_ref())
            .cloned()
            .unwrap_or_default();

        let tokens_used = response
            .usage
            .map(|u| u.total_tokens as usize)
            .unwrap_or_else(|| self.count_tokens(&content).unwrap_or(0));

        Ok(AiResponse {
            content,
            tokens_used,
            model: self.model.clone(),
        })
    }

    async fn stream_complete(
        &self,
        prompt: &str,
        max_tokens: usize,
    ) -> CoreResult<Box<dyn TokenStream + Send>> {
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
            max_completion_tokens: Some(max_tokens as u32),
            temperature: Some(0.3),
            top_p: Some(0.9),
            stream: Some(true),
            ..Default::default()
        };

        let stream = self
            .client
            .chat()
            .create_stream(request)
            .await
            .map_err(|e| CoreError::ai_provider(format!("OpenAI stream error: {}", e)))?;

        Ok(Box::new(OpenAiTokenStream {
            stream,
            buffer: String::new(),
        }))
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
    stream: ChatCompletionResponseStream,
    buffer: String,
}

impl std::fmt::Debug for OpenAiTokenStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAiTokenStream").finish()
    }
}

#[async_trait]
impl TokenStream for OpenAiTokenStream {
    async fn next_token(&mut self) -> CoreResult<Option<String>> {
        while let Some(chunk) = self.stream.next().await {
            let chunk =
                chunk.map_err(|e| CoreError::ai_provider(format!("Stream error: {}", e)))?;
            if let Some(choice) = chunk.choices.first() {
                if let Some(delta) = &choice.delta.content {
                    self.buffer.push_str(delta);
                    return Ok(Some(delta.to_string()));
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
    tokenizer: &'static CoreBPE,
}

impl OllamaClient {
    pub async fn new(config: &OllamaConfig) -> CoreResult<Self> {
        validate_base_url(&config.base_url, ProviderKind::Ollama)?;

        let client = reqwest::Client::new();
        let tokenizer = cl100k_base_singleton(); // Approximation

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
    async fn complete(
        &self,
        prompt: &str,
        max_tokens: usize,
        _streaming: bool,
    ) -> CoreResult<AiResponse> {
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

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| CoreError::ai_provider(format!("Ollama request error: {}", e)))?;

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| CoreError::ai_provider(format!("Ollama response error: {}", e)))?;

        let content = result["response"].as_str().unwrap_or("").to_string();
        let tokens_used = self.count_tokens(&content)?;

        Ok(AiResponse {
            content,
            tokens_used,
            model: self.model.clone(),
        })
    }

    async fn stream_complete(
        &self,
        prompt: &str,
        max_tokens: usize,
    ) -> CoreResult<Box<dyn TokenStream + Send>> {
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

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| CoreError::ai_provider(format!("Ollama stream request error: {}", e)))?;

        let stream = response.bytes_stream();
        Ok(Box::new(OllamaTokenStream {
            stream: Box::pin(stream),
            buffer: String::new(),
        }))
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
    stream: Pin<Box<dyn futures::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    buffer: String,
}

#[async_trait]
impl TokenStream for OllamaTokenStream {
    async fn next_token(&mut self) -> CoreResult<Option<String>> {
        use futures::StreamExt;

        while let Some(chunk) = self.stream.next().await {
            let chunk =
                chunk.map_err(|e| CoreError::ai_provider(format!("Ollama stream error: {}", e)))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_openai_url_is_allowed() {
        validate_base_url("https://api.openai.com/v1", ProviderKind::OpenAi).unwrap();
        // 大小文字・明示 port 443 も同一ホストとして許可する。
        validate_base_url("https://API.OPENAI.COM/v1", ProviderKind::OpenAi).unwrap();
        validate_base_url("https://api.openai.com:443/v1", ProviderKind::OpenAi).unwrap();
    }

    #[test]
    fn http_attacker_host_is_rejected() {
        assert!(validate_base_url("http://attacker.example/v1", ProviderKind::OpenAi).is_err());
        assert!(validate_base_url("http://attacker.example:11434", ProviderKind::Ollama).is_err());
    }

    #[test]
    fn https_attacker_host_is_rejected() {
        assert!(validate_base_url("https://attacker.example/v1", ProviderKind::OpenAi).is_err());
        assert!(validate_base_url("https://attacker.example:11434", ProviderKind::Ollama).is_err());
    }

    #[test]
    fn loopback_ollama_urls_are_allowed() {
        validate_base_url("http://localhost:11434", ProviderKind::Ollama).unwrap();
        validate_base_url("http://127.0.0.1:11434", ProviderKind::Ollama).unwrap();
        validate_base_url("http://[::1]:11434", ProviderKind::Ollama).unwrap();
        validate_base_url("http://LOCALHOST:11434", ProviderKind::Ollama).unwrap();
    }

    #[test]
    fn non_http_schemes_are_rejected() {
        assert!(validate_base_url("ftp://api.openai.com/v1", ProviderKind::OpenAi).is_err());
        assert!(validate_base_url("ftp://[::1]:11434", ProviderKind::Ollama).is_err());
        assert!(validate_base_url("file:///etc/passwd", ProviderKind::OpenAi).is_err());
    }

    #[test]
    fn userinfo_is_rejected() {
        assert!(
            validate_base_url(
                "https://user:secret@api.openai.com/v1",
                ProviderKind::OpenAi
            )
            .is_err()
        );
        assert!(
            validate_base_url(
                "https://api.openai.com@attacker.example/v1",
                ProviderKind::OpenAi
            )
            .is_err()
        );
        assert!(
            validate_base_url("http://user:secret@localhost:11434", ProviderKind::Ollama).is_err()
        );
    }

    #[test]
    fn error_message_does_not_echo_url() {
        let err = validate_base_url(
            "https://user:secret@attacker.example/v1",
            ProviderKind::OpenAi,
        )
        .expect_err("userinfo 付き URL は拒否されるべき");
        let message = err.to_string();
        assert!(!message.contains("secret"), "userinfo が漏洩: {message}");
        assert!(
            !message.contains("attacker.example/v1"),
            "URL 全体が漏洩: {message}"
        );
    }

    #[test]
    fn non_default_ports_are_rejected() {
        assert!(validate_base_url("https://api.openai.com:8443/v1", ProviderKind::OpenAi).is_err());
        assert!(validate_base_url("http://localhost:8080", ProviderKind::Ollama).is_err());
        assert!(validate_base_url("http://localhost", ProviderKind::Ollama).is_err());
    }

    #[test]
    fn urls_for_the_wrong_provider_are_rejected() {
        assert!(validate_base_url("https://api.openai.com/v1", ProviderKind::Ollama).is_err());
        assert!(validate_base_url("http://localhost:11434", ProviderKind::OpenAi).is_err());
    }

    #[test]
    fn malformed_url_is_rejected() {
        assert!(validate_base_url("not-a-url", ProviderKind::OpenAi).is_err());
        assert!(validate_base_url("https://", ProviderKind::Ollama).is_err());
    }
}
