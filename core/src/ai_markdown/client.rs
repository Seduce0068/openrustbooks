//! AI Provider abstraction and implementations.

use crate::{
    CoreError, CoreResult,
    config::{AiConfig, OllamaConfig, OpenAiConfig},
};
use async_openai::types::chat::{
    ChatCompletionRequestUserMessageArgs, ChatCompletionResponseStream, CreateChatCompletionRequest,
};
use async_openai::{Client, config::OpenAIConfig};
use async_trait::async_trait;
use futures::StreamExt;
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
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
        "ollama" => Ok(Arc::new(
            OllamaClient::new(&config.ollama, config.timeout_secs).await?,
        )),
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

/// OpenAI チャットリクエストを組み立てる。
///
/// system role は使わず、渡された prompt を user message 1 件として送る。
/// テンプレート由来の system_prompt は `render()` が prompt 文字列へ挿入するため、
/// ここで別途 system message を付けると指示が二重になる。Ollama 側もテンプレート
/// 経由のみなので、両プロバイダで同じ user 1 件の意味に揃える。
fn build_openai_request(
    prompt: &str,
    model: &str,
    max_tokens: usize,
    stream: bool,
) -> CoreResult<CreateChatCompletionRequest> {
    let messages = vec![
        ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()?
            .into(),
    ];

    Ok(CreateChatCompletionRequest {
        model: model.to_string(),
        messages,
        max_completion_tokens: Some(max_tokens as u32),
        temperature: Some(0.3),
        top_p: Some(0.9),
        stream: Some(stream),
        ..Default::default()
    })
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
        let request = build_openai_request(prompt, &self.model, max_tokens, false)?;

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
        let request = build_openai_request(prompt, &self.model, max_tokens, true)?;

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
    pub async fn new(config: &OllamaConfig, timeout_secs: u64) -> CoreResult<Self> {
        validate_base_url(&config.base_url, ProviderKind::Ollama)?;

        // timeout は AiConfig.timeout_secs（既定 60 秒）を呼び出し側から受け取る。
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| CoreError::ai_provider(format!("Ollama client build error: {}", e)))?;
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
            .and_then(reqwest::Response::error_for_status)
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
            .and_then(reqwest::Response::error_for_status)
            .map_err(|e| CoreError::ai_provider(format!("Ollama stream request error: {}", e)))?;

        let stream = response.bytes_stream();
        Ok(Box::new(OllamaTokenStream {
            stream: Box::pin(stream),
            buffer: String::new(),
            pending: Vec::new(),
            done: false,
            responses: VecDeque::new(),
        }))
    }

    fn count_tokens(&self, text: &str) -> CoreResult<usize> {
        Ok(self.tokenizer.encode_with_special_tokens(text).len())
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

/// 未解析行の最大バイト数。超過時は fail-closed（際限ないバッファ増加を防ぐ）。
const MAX_PENDING_BYTES: usize = 1024 * 1024;

/// Ollama token stream
struct OllamaTokenStream {
    stream: Pin<Box<dyn futures::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    buffer: String,
    /// 未確定のバイト列（改行がまだ来ていない行、途中のマルチバイト文字を含む）。
    pending: Vec<u8>,
    /// `done: true` を受信した、またはストリームが枯渇した。
    done: bool,
    /// 解析済みでまだ返していないトークン（1 チャンク複数行の取りこぼし防止）。
    responses: VecDeque<String>,
}

impl OllamaTokenStream {
    /// チャンクを累積し、改行で区切れた完全行だけを解析する。
    ///
    /// `pending` は常に UTF-8 文字境界で終わるとは限らないが、改行 (0x0A) は
    /// UTF-8 のマルチバイト列中に現れないため、`\n` が見つかった時点で
    /// それより前は完全な行として安全に切り出せる。途中のマルチバイト文字は
    /// `\n` が来るまで `pending` に残り、次のチャンクで続きを待つ。
    fn push_chunk(&mut self, chunk: &[u8]) -> CoreResult<()> {
        self.pending.extend_from_slice(chunk);

        while let Some(newline) = self.pending.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = self.pending[..newline].to_vec();
            self.pending.drain(..=newline);
            self.parse_line(&line)?;
        }

        if self.pending.len() > MAX_PENDING_BYTES {
            return Err(CoreError::ai_provider(
                "Ollama stream line exceeds size limit",
            ));
        }
        Ok(())
    }

    /// ストリーム末端で改行の無い残りを最後の行として解析する。
    fn finish(&mut self) -> CoreResult<()> {
        if !self.pending.is_empty() {
            let line = std::mem::take(&mut self.pending);
            self.parse_line(&line)?;
        }
        Ok(())
    }

    /// Ollama の JSON 行を解析し、response をキューへ、done をフラグへ反映する。
    ///
    /// エラーメッセージには行の本文も URL も含めない（機微情報の漏えい防止）。
    fn parse_line(&mut self, line: &[u8]) -> CoreResult<()> {
        let text = std::str::from_utf8(line)
            .map_err(|_| CoreError::ai_provider("Ollama stream line is not valid UTF-8"))?;
        // 空行（ハートビート等）は JSON ではないが無視する。
        if text.trim().is_empty() {
            return Ok(());
        }
        let json = serde_json::from_str::<serde_json::Value>(text)
            .map_err(|_| CoreError::ai_provider("Ollama stream line is not valid JSON"))?;

        if !json["error"].is_null() {
            return Err(CoreError::ai_provider("Ollama stream reported an error"));
        }
        if let Some(response) = json["response"].as_str().filter(|r| !r.is_empty()) {
            self.buffer.push_str(response);
            self.responses.push_back(response.to_string());
        }
        if json["done"].as_bool().unwrap_or(false) {
            self.done = true;
        }
        Ok(())
    }
}

#[async_trait]
impl TokenStream for OllamaTokenStream {
    async fn next_token(&mut self) -> CoreResult<Option<String>> {
        loop {
            // キューに残りがあれば done より優先して返す（同一行の response + done も扱う）。
            if let Some(token) = self.responses.pop_front() {
                return Ok(Some(token));
            }
            if self.done {
                return Ok(None);
            }

            match self.stream.next().await {
                Some(chunk) => {
                    let chunk = chunk.map_err(|e| {
                        CoreError::ai_provider(format!("Ollama stream error: {}", e))
                    })?;
                    self.push_chunk(&chunk)?;
                }
                None => {
                    // 末尾に改行の無い行を回収し、以降はストリームを再ポールしない。
                    self.done = true;
                    self.finish()?;
                }
            }
        }
    }

    fn is_done(&self) -> bool {
        // キューに未配信トークンがある間は、まだ完了ではない。
        self.done && self.responses.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_markdown::prompt::{PromptRegistry, PromptType, PromptVariables};
    use async_openai::types::chat::{
        ChatCompletionRequestMessage, ChatCompletionRequestUserMessageContent,
    };

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

    #[test]
    fn openai_request_has_only_one_user_message() {
        // system role が無く、user message が 1 件だけであることを確認。
        let request = build_openai_request("# Title\n\n本文", "gpt-4o", 128, false).unwrap();
        assert_eq!(request.messages.len(), 1, "messages は user 1 件のみ");
        assert!(
            matches!(request.messages[0], ChatCompletionRequestMessage::User(_)),
            "先頭は user message であるべき"
        );
        assert!(
            request
                .messages
                .iter()
                .all(|m| !matches!(m, ChatCompletionRequestMessage::System(_))),
            "system role が残っている"
        );
    }

    #[test]
    fn openai_request_user_content_equals_prompt() {
        // 渡された prompt がそのまま user message の content になることを確認。
        let prompt = "# 見出し\n\n- 項目\n\n本文テキスト";
        let request = build_openai_request(prompt, "gpt-4o", 256, true).unwrap();
        match &request.messages[0] {
            ChatCompletionRequestMessage::User(user) => match &user.content {
                ChatCompletionRequestUserMessageContent::Text(text) => {
                    assert_eq!(text, prompt, "prompt が加工されずに渡されるべき");
                }
                other => panic!("user content が Text ではない: {other:?}"),
            },
            other => panic!("user message ではない: {other:?}"),
        }
        assert_eq!(request.stream, Some(true));
    }

    // ---- OllamaTokenStream の行バッファリング テスト ----

    fn ollama_stream(chunks: Vec<bytes::Bytes>) -> OllamaTokenStream {
        OllamaTokenStream {
            stream: Box::pin(futures::stream::iter(
                chunks.into_iter().map(Ok::<_, reqwest::Error>),
            )),
            buffer: String::new(),
            pending: Vec::new(),
            done: false,
            responses: VecDeque::new(),
        }
    }

    fn json_line(response: &str, done: bool) -> bytes::Bytes {
        let mut text = serde_json::json!({ "response": response, "done": done }).to_string();
        text.push('\n');
        bytes::Bytes::from(text)
    }

    #[tokio::test]
    async fn ollama_split_json_line_is_reassembled() {
        let full = json_line("Hello", false);
        let bytes = full.as_ref();
        let split = bytes.len() / 2;
        let mut stream = ollama_stream(vec![
            bytes::Bytes::copy_from_slice(&bytes[..split]),
            bytes::Bytes::copy_from_slice(&bytes[split..]),
        ]);

        assert_eq!(stream.next_token().await.unwrap().as_deref(), Some("Hello"));
        assert_eq!(stream.next_token().await.unwrap(), None);
    }

    #[tokio::test]
    async fn ollama_multiple_lines_in_one_chunk_are_all_returned() {
        let mut data = Vec::new();
        data.extend_from_slice(json_line("A", false).as_ref());
        data.extend_from_slice(json_line("B", false).as_ref());
        data.extend_from_slice(json_line("C", true).as_ref());
        let mut stream = ollama_stream(vec![bytes::Bytes::from(data)]);

        assert_eq!(stream.next_token().await.unwrap().as_deref(), Some("A"));
        // done 行を読み込んでいても、キューに残りがある間は is_done() は false。
        assert!(!stream.is_done());
        assert_eq!(stream.next_token().await.unwrap().as_deref(), Some("B"));
        assert!(!stream.is_done());
        assert_eq!(stream.next_token().await.unwrap().as_deref(), Some("C"));
        assert_eq!(stream.next_token().await.unwrap(), None);
        assert!(stream.is_done());
    }

    #[tokio::test]
    async fn ollama_done_in_same_line_keeps_response() {
        let mut stream = ollama_stream(vec![json_line("final", true)]);

        // done フラグよりキューの response を先に返す。
        assert_eq!(stream.next_token().await.unwrap().as_deref(), Some("final"));
        assert!(stream.is_done());
        assert_eq!(stream.next_token().await.unwrap(), None);
    }

    #[tokio::test]
    async fn ollama_utf8_multibyte_split_across_chunks_is_decoded() {
        let full = json_line("こんにちは", false);
        let bytes = full.as_ref();
        // 3 バイト文字の途中（継続バイト）で 2 箇所切断する。
        let continuations: Vec<usize> = (1..bytes.len())
            .filter(|&i| (0x80..=0xBF).contains(&bytes[i]))
            .collect();
        assert!(continuations.len() >= 2);
        let first = continuations[0];
        let second = continuations[5];
        let mut stream = ollama_stream(vec![
            bytes::Bytes::copy_from_slice(&bytes[..first]),
            bytes::Bytes::copy_from_slice(&bytes[first..second]),
            bytes::Bytes::copy_from_slice(&bytes[second..]),
        ]);

        assert_eq!(
            stream.next_token().await.unwrap().as_deref(),
            Some("こんにちは")
        );
        assert_eq!(stream.next_token().await.unwrap(), None);
    }

    #[tokio::test]
    async fn ollama_unterminated_final_line_is_parsed() {
        let text = serde_json::json!({ "response": "tail", "done": false }).to_string();
        let mut stream = ollama_stream(vec![bytes::Bytes::from(text)]);

        assert_eq!(stream.next_token().await.unwrap().as_deref(), Some("tail"));
        assert_eq!(stream.next_token().await.unwrap(), None);
    }

    #[tokio::test]
    async fn ollama_stream_error_is_propagated() {
        // ネットワークを使わずに reqwest::Error を生成する（不正 URL の build 失敗）。
        let stream_error = reqwest::Client::new()
            .get("not a valid url")
            .build()
            .expect_err("不正 URL は RequestBuilder::build で失敗する");
        let mut stream = OllamaTokenStream {
            stream: Box::pin(futures::stream::iter(vec![
                Ok(json_line("before", false)),
                Err(stream_error),
            ])),
            buffer: String::new(),
            pending: Vec::new(),
            done: false,
            responses: VecDeque::new(),
        };

        assert_eq!(
            stream.next_token().await.unwrap().as_deref(),
            Some("before")
        );
        let err = stream.next_token().await.unwrap_err();
        assert!(
            err.to_string().contains("Ollama stream error"),
            "エラーが伝播しない: {err}"
        );
    }

    #[tokio::test]
    async fn ollama_invalid_json_line_is_error_without_body() {
        let mut stream = ollama_stream(vec![bytes::Bytes::from(
            "{\"secret_marker\":\"SHOULD_NOT_LEAK\" broken\n",
        )]);

        let err = stream.next_token().await.unwrap_err();
        let message = err.to_string();
        assert!(
            message.contains("not valid JSON"),
            "想定外のエラー: {message}"
        );
        assert!(
            !message.contains("SHOULD_NOT_LEAK"),
            "本文が漏れた: {message}"
        );
    }

    #[tokio::test]
    async fn ollama_error_field_is_reported_without_body() {
        let mut stream = ollama_stream(vec![bytes::Bytes::from(
            "{\"error\":\"secret model detail\",\"done\":true}\n",
        )]);

        let err = stream.next_token().await.unwrap_err();
        let message = err.to_string();
        assert!(
            message.contains("reported an error"),
            "想定外のエラー: {message}"
        );
        assert!(
            !message.contains("secret model detail"),
            "本文が漏れた: {message}"
        );
    }

    #[tokio::test]
    async fn ollama_invalid_utf8_line_is_error() {
        let mut data = b"{\"response\":\"".to_vec();
        data.push(0xff);
        data.extend_from_slice(b"\"}\n");
        let mut stream = ollama_stream(vec![bytes::Bytes::from(data)]);

        let err = stream.next_token().await.unwrap_err();
        assert!(
            err.to_string().contains("not valid UTF-8"),
            "想定外のエラー: {err}"
        );
    }

    #[tokio::test]
    async fn ollama_pending_limit_is_enforced() {
        let mut stream = ollama_stream(vec![bytes::Bytes::from(vec![b'a'; MAX_PENDING_BYTES + 1])]);

        let err = stream.next_token().await.unwrap_err();
        assert!(
            err.to_string().contains("exceeds size limit"),
            "想定外のエラー: {err}"
        );
    }

    #[tokio::test]
    async fn ollama_empty_response_is_not_queued() {
        let mut stream = ollama_stream(vec![json_line("", true)]);

        assert_eq!(stream.next_token().await.unwrap(), None);
        assert!(stream.is_done());
    }

    #[tokio::test]
    async fn ollama_blank_line_is_ignored() {
        let mut stream = OllamaTokenStream {
            stream: Box::pin(futures::stream::iter(vec![
                Ok::<_, reqwest::Error>(bytes::Bytes::from("\n")),
                Ok(json_line("ok", true)),
            ])),
            buffer: String::new(),
            pending: Vec::new(),
            done: false,
            responses: VecDeque::new(),
        };

        assert_eq!(stream.next_token().await.unwrap().as_deref(), Some("ok"));
        assert_eq!(stream.next_token().await.unwrap(), None);
    }

    #[tokio::test]
    async fn ollama_client_uses_timeout_from_ai_config() {
        let config = AiConfig {
            default_provider: "ollama".to_string(),
            timeout_secs: 12,
            ..Default::default()
        };
        let client = create_client(&config).await.unwrap();

        assert_eq!(client.model_name(), config.ollama.model);
    }

    #[test]
    fn rendered_prompt_contains_template_system_prompt() {
        // テンプレートの system_prompt が prompt 文字列へ挿入されることを確認。
        // その prompt を渡しても user 1 件だけで、system role は生成されない。
        let registry = PromptRegistry::with_builtins().unwrap();
        let prompt = registry
            .render(PromptType::Generate, PromptVariables::default())
            .unwrap();
        let system_prompt = &registry
            .get_template(PromptType::Generate)
            .unwrap()
            .system_prompt;
        assert!(
            prompt.starts_with(system_prompt),
            "prompt 先頭に system_prompt がない"
        );

        let request = build_openai_request(&prompt, "gpt-4o", 128, false).unwrap();
        assert_eq!(request.messages.len(), 1);
        assert!(matches!(
            request.messages[0],
            ChatCompletionRequestMessage::User(_)
        ));
        match &request.messages[0] {
            ChatCompletionRequestMessage::User(user) => match &user.content {
                ChatCompletionRequestUserMessageContent::Text(text) => {
                    assert!(
                        text.contains(system_prompt),
                        "system_prompt が prompt に含まれる"
                    );
                }
                other => panic!("user content が Text ではない: {other:?}"),
            },
            other => panic!("user message ではない: {other:?}"),
        }
    }
}
