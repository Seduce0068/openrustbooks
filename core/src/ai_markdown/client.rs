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

/// `extra_headers` の検証失敗時に返す固定文言。
///
/// ヘッダ名・値を一切含めないことで、機微値がエラー文言や Debug へ漏れることを防ぐ（ADR-0102）。
const INVALID_EXTRA_HEADER: &str = "OpenAI extra header is invalid";

/// 予約ヘッダを拒否した際の固定文言（キー・値を含めない）。
const RESERVED_EXTRA_HEADER: &str = "OpenAI extra header is reserved";

/// async-openai の custom headers は既存ヘッダを上書きする
/// （`async-openai-0.42.2/src/config.rs` の `with_header` 参照）。
/// `authorization` / `openai-organization` / `openai-project` / `openai-beta` を
/// config から差し込めると `with_api_key` / `with_org_id` の認証情報を上書きできるため、
/// 大文字小文字を問わず拒否する（ADR-0221）。
fn is_reserved_extra_header(name: &reqwest::header::HeaderName) -> bool {
    let name = name.as_str();
    name.eq_ignore_ascii_case("authorization")
        || name.eq_ignore_ascii_case("openai-organization")
        || name.eq_ignore_ascii_case("openai-project")
        || name.eq_ignore_ascii_case("openai-beta")
}

impl OpenAiClient {
    pub async fn new(config: &OpenAiConfig) -> CoreResult<Self> {
        validate_base_url(&config.base_url, ProviderKind::OpenAi)?;

        let openai_config = OpenAIConfig::new()
            .with_api_base(&config.base_url)
            .with_api_key(config.api_key.as_deref().unwrap_or(""))
            .with_org_id(config.organization.as_deref().unwrap_or(""));

        // config.toml の extra_headers を全リクエストへ付与する（ADR-0102）。
        // HashMap の反復順に依存しないようキーでソートし、順序を決定的にする。
        // キーは `HeaderName` にパースしてから渡す（`String` は `IntoHeaderName` ではない）。
        // 認証情報を上書きし得る予約ヘッダは拒否し、それ以外の検証失敗も含め
        // 固定文言で fail-closed として値を漏らさない（ADR-0221）。
        let mut headers: Vec<_> = config.extra_headers.iter().collect();
        headers.sort_by(|a, b| a.0.cmp(b.0));
        let mut openai_config = openai_config;
        for (name, value) in headers {
            let header_name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|_| CoreError::ai_provider(INVALID_EXTRA_HEADER))?;
            if is_reserved_extra_header(&header_name) {
                return Err(CoreError::ai_provider(RESERVED_EXTRA_HEADER));
            }
            openai_config = openai_config
                .with_header(header_name, value)
                .map_err(|_| CoreError::ai_provider(INVALID_EXTRA_HEADER))?;
        }

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
            .map_err(|e| CoreError::ai_provider(e.to_string()))?;

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
            .map_err(|e| CoreError::ai_provider(e.to_string()))?;

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
            let chunk = chunk.map_err(|e| CoreError::ai_provider(e.to_string()))?;
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
}

/// Ollama の `/api/generate` リクエストを組み立てる。
///
/// `options` は固定値（num_predict / temperature / top_p）をベースに、
/// config.toml 由来の `config_options` を上書きマージする。ただし
/// `num_predict` / `model` / `stream` は予約キーとし、config 側からは上書きできない
/// （num_predict は呼び出し側の max_tokens、model / stream はトップレベル項目を優先）。
fn build_ollama_request(
    prompt: &str,
    model: &str,
    max_tokens: usize,
    stream: bool,
    config_options: &serde_json::Map<String, serde_json::Value>,
) -> serde_json::Value {
    let mut options = serde_json::Map::new();
    options.insert("num_predict".to_string(), serde_json::json!(max_tokens));
    options.insert("temperature".to_string(), serde_json::json!(0.3));
    options.insert("top_p".to_string(), serde_json::json!(0.9));

    for (key, value) in config_options {
        if matches!(key.as_str(), "num_predict" | "model" | "stream") {
            continue;
        }
        options.insert(key.clone(), value.clone());
    }

    serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": stream,
        "options": options,
    })
}

/// Ollama local LLM client
pub struct OllamaClient {
    client: reqwest::Client,
    base_url: String,
    model: String,
    tokenizer: &'static CoreBPE,
    /// config.toml の `ollama.options`。リクエストの options へマージされる。
    options: serde_json::Map<String, serde_json::Value>,
}

impl OllamaClient {
    pub async fn new(config: &OllamaConfig, timeout_secs: u64) -> CoreResult<Self> {
        validate_base_url(&config.base_url, ProviderKind::Ollama)?;

        // timeout は AiConfig.timeout_secs（既定 60 秒）を呼び出し側から受け取る。
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| CoreError::ai_provider(e.to_string()))?;
        let tokenizer = cl100k_base_singleton(); // Approximation
        let options = config
            .options
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();

        Ok(Self {
            client,
            base_url: config.base_url.clone(),
            model: config.model.clone(),
            tokenizer,
            options,
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
        let request = build_ollama_request(prompt, &self.model, max_tokens, false, &self.options);

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|e| CoreError::ai_provider(e.to_string()))?;

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| CoreError::ai_provider(e.to_string()))?;

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
        let request = build_ollama_request(prompt, &self.model, max_tokens, true, &self.options);

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|e| CoreError::ai_provider(e.to_string()))?;

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
                    let chunk = chunk.map_err(|e| CoreError::ai_provider(e.to_string()))?;
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
        // done 行を読み込んでいても、キューに残りがある間は next_token() は
        // None を返さず、A → B → C の順に配信される（ADR-0189 の回帰検知）。
        assert_eq!(stream.next_token().await.unwrap().as_deref(), Some("B"));
        assert_eq!(stream.next_token().await.unwrap().as_deref(), Some("C"));
        // キューが空になった後に初めて完了（None）を返す。
        assert_eq!(stream.next_token().await.unwrap(), None);
        assert_eq!(stream.next_token().await.unwrap(), None);
    }

    #[tokio::test]
    async fn ollama_done_in_same_line_keeps_response() {
        let mut stream = ollama_stream(vec![json_line("final", true)]);

        // done フラグよりキューの response を先に返し、その後 None で完了する。
        assert_eq!(stream.next_token().await.unwrap().as_deref(), Some("final"));
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
        let message = err.to_string();
        let prefix = "AI provider error: ";
        assert!(
            message.starts_with(prefix) && message.len() > prefix.len(),
            "元のエラーが伝播しない: {message}"
        );
        assert!(
            !message.contains("not a valid url"),
            "試行 URL が漏れた: {message}"
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

        // 空 response はキューされないため、トークンを返さずに完了する。
        assert_eq!(stream.next_token().await.unwrap(), None);
        assert_eq!(stream.next_token().await.unwrap(), None);
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

    // ---- OpenAiClient.extra_headers の配線テスト ----

    #[tokio::test]
    async fn openai_valid_extra_headers_are_accepted() {
        let config = OpenAiConfig {
            extra_headers: std::collections::HashMap::from([
                ("X-Custom".to_string(), "custom-value".to_string()),
                ("another-header".to_string(), "test".to_string()),
            ]),
            ..Default::default()
        };
        OpenAiClient::new(&config)
            .await
            .expect("妥当な extra_headers は受理されるべき");
    }

    #[tokio::test]
    async fn openai_invalid_extra_header_value_is_rejected_without_leaking() {
        let secret = "SHOULD_NOT_LEAK_VALUE";
        let config = OpenAiConfig {
            extra_headers: std::collections::HashMap::from([(
                "X-Custom".to_string(),
                format!("{secret}\n"), // 改行は HeaderValue として不正
            )]),
            ..Default::default()
        };
        let err = OpenAiClient::new(&config)
            .await
            .err()
            .expect("不正なヘッダ値は拒否されるべき");
        let message = err.to_string();
        assert!(
            message.contains("OpenAI extra header is invalid"),
            "想定外のエラー: {message}"
        );
        assert!(!message.contains(secret), "値が漏れた: {message}");
    }

    #[tokio::test]
    async fn openai_invalid_extra_header_name_is_rejected_without_leaking() {
        let name = "Bad Header Name"; // 空白は HeaderName として不正
        let config = OpenAiConfig {
            extra_headers: std::collections::HashMap::from([(
                name.to_string(),
                "value".to_string(),
            )]),
            ..Default::default()
        };
        let err = OpenAiClient::new(&config)
            .await
            .err()
            .expect("不正なヘッダ名は拒否されるべき");
        let message = err.to_string();
        assert!(
            message.contains("OpenAI extra header is invalid"),
            "想定外のエラー: {message}"
        );
        assert!(!message.contains(name), "キーが漏れた: {message}");
    }

    #[tokio::test]
    async fn openai_reserved_extra_headers_are_rejected_case_insensitively() {
        for name in [
            "authorization",
            "AUTHORIZATION",
            "OpenAI-Organization",
            "openai-project",
            "OpenAI-BETA",
        ] {
            let value = format!("SECRET_FOR_{name}");
            let config = OpenAiConfig {
                extra_headers: std::collections::HashMap::from([(name.to_string(), value.clone())]),
                ..Default::default()
            };
            let err = OpenAiClient::new(&config)
                .await
                .err()
                .expect("予約ヘッダは拒否されるべき");
            let message = err.to_string();
            assert!(
                message.contains("OpenAI extra header is reserved"),
                "想定外のエラー: {message}"
            );
            assert!(!message.contains(&value), "値が漏れた: {message}");
            assert!(!message.contains(name), "キーが漏れた: {message}");
        }
    }

    // ---- Ollama options マージのテスト ----

    #[test]
    fn ollama_options_merge_keeps_fixed_values_and_custom_values() {
        let mut config_options = serde_json::Map::new();
        config_options.insert("num_ctx".to_string(), serde_json::json!(4096));
        config_options.insert("temperature".to_string(), serde_json::json!(0.7));
        // 予約キーは config 側から上書きできない。
        config_options.insert("num_predict".to_string(), serde_json::json!(999));
        config_options.insert("model".to_string(), serde_json::json!("evil-model"));
        config_options.insert("stream".to_string(), serde_json::json!(true));

        let request = build_ollama_request("hi", "llama3.1:8b", 128, false, &config_options);
        let options = &request["options"];

        assert_eq!(
            options["num_predict"],
            serde_json::json!(128),
            "max_tokens 優先"
        );
        assert_eq!(options["top_p"], serde_json::json!(0.9), "固定値");
        assert_eq!(
            options["temperature"],
            serde_json::json!(0.7),
            "custom 採用"
        );
        assert_eq!(options["num_ctx"], serde_json::json!(4096), "custom 採用");
        assert!(
            options.get("model").is_none(),
            "model は options に混入しない"
        );
        assert!(
            options.get("stream").is_none(),
            "stream は options に混入しない"
        );
        assert_eq!(request["model"], serde_json::json!("llama3.1:8b"));
        assert_eq!(request["stream"], serde_json::json!(false));
        assert_eq!(request["prompt"], serde_json::json!("hi"));
    }

    #[tokio::test]
    async fn ollama_client_keeps_config_options() {
        let options =
            std::collections::HashMap::from([("num_ctx".to_string(), serde_json::json!(2048))]);
        let config = OllamaConfig {
            options,
            ..Default::default()
        };
        let client = OllamaClient::new(&config, 60).await.unwrap();
        assert_eq!(
            client.options.get("num_ctx"),
            Some(&serde_json::json!(2048))
        );
    }
}
