//! Error types for openrustbooks-core.

use thiserror::Error;

/// Core error type
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error("Configuration error: {0}")]
    Config(#[source] Box<figment::Error>),

    #[error("TOML deserialization error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Archive error: {0}")]
    Archive(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("AI pipeline error: {0}")]
    AiPipeline(String),

    #[error("AI provider error: {0}")]
    AiProvider(String),

    #[cfg(not(target_arch = "wasm32"))]
    #[error("AI provider error: {0}")]
    OpenAiError(#[source] Box<async_openai::error::OpenAIError>),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error("Tokio task join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Lint error: {0}")]
    Lint(String),

    #[error("Token limit exceeded: {0}")]
    TokenLimitExceeded(String),

    #[error("Validation error: {0}")]
    Validation(String),

    /// 設定が読み取られてから変更された場合の CAS 衝突（native / wasm 共通）。
    #[error("Configuration conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

// Boxed variants need manual `From` impls: deriving `#[from]` would store the
// large source error inline and inflate `CoreError` past clippy's threshold.
#[cfg(not(target_arch = "wasm32"))]
impl From<figment::Error> for CoreError {
    fn from(err: figment::Error) -> Self {
        Self::Config(Box::new(err))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<async_openai::error::OpenAIError> for CoreError {
    fn from(err: async_openai::error::OpenAIError) -> Self {
        Self::OpenAiError(Box::new(err))
    }
}

/// Result type alias for core operations
pub type CoreResult<T> = Result<T, CoreError>;

impl CoreError {
    /// IPC 契約で使う固定エラーコードを返す（設定値・パスは含まない）。
    ///
    /// `Config` variant は native のみ存在するため、そのアームだけ cfg で分ける。
    /// 新しい variant を追加した場合は既定で `CONFIG_INTERNAL` に fall back する。
    pub fn code(&self) -> &'static str {
        match self {
            Self::Conflict(_) => "CONFIG_CONFLICT",
            Self::Validation(_) | Self::UnsupportedFormat(_) | Self::TomlDeserialize(_) => {
                "CONFIG_VALIDATION"
            }
            Self::Io(_) | Self::Storage(_) | Self::Serde(_) | Self::TomlSerialize(_) => {
                "CONFIG_STORAGE"
            }
            #[cfg(not(target_arch = "wasm32"))]
            Self::Config(_) => "CONFIG_STORAGE",
            _ => "CONFIG_INTERNAL",
        }
    }

    /// Create a storage error
    pub fn storage(msg: impl Into<String>) -> Self {
        Self::Storage(msg.into())
    }

    /// Create an AI pipeline error
    pub fn ai_pipeline(msg: impl Into<String>) -> Self {
        Self::AiPipeline(msg.into())
    }

    /// Create an AI provider error
    pub fn ai_provider(msg: impl Into<String>) -> Self {
        Self::AiProvider(msg.into())
    }

    /// Create an archive error
    pub fn archive(msg: impl Into<String>) -> Self {
        Self::Archive(msg.into())
    }

    /// Create a lint error
    pub fn lint(msg: impl Into<String>) -> Self {
        Self::Lint(msg.into())
    }

    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = CoreError::storage("test message");
        assert!(err.to_string().contains("test message"));
    }

    #[test]
    fn code_returns_fixed_ipc_codes() {
        assert_eq!(
            CoreError::Conflict("stale".into()).code(),
            "CONFIG_CONFLICT"
        );
        assert_eq!(CoreError::validation("bad").code(), "CONFIG_VALIDATION");
        assert_eq!(
            CoreError::UnsupportedFormat("x".into()).code(),
            "CONFIG_VALIDATION"
        );
        assert_eq!(CoreError::storage("disk").code(), "CONFIG_STORAGE");
        assert_eq!(CoreError::internal("bug").code(), "CONFIG_INTERNAL");
        assert_eq!(
            CoreError::DocumentNotFound("x".into()).code(),
            "CONFIG_INTERNAL"
        );
    }

    #[test]
    fn code_maps_conversion_errors_without_changing_display() {
        let toml_de = toml::from_str::<toml::Value>("x = ").unwrap_err();
        let err = CoreError::from(toml_de);
        assert_eq!(err.code(), "CONFIG_VALIDATION");
        assert!(err.to_string().starts_with("TOML deserialization error:"));

        let io = CoreError::from(std::io::Error::other("io"));
        assert_eq!(io.code(), "CONFIG_STORAGE");
        let serde = CoreError::from(serde_json::from_str::<serde_json::Value>("{").unwrap_err());
        assert_eq!(serde.code(), "CONFIG_STORAGE");
        let toml_ser = CoreError::from(toml::to_string(&Option::<u8>::None).unwrap_err());
        assert_eq!(toml_ser.code(), "CONFIG_STORAGE");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn code_maps_native_config_error_to_storage() {
        let err = CoreError::from(figment::Error::from("bad config"));
        assert_eq!(err.code(), "CONFIG_STORAGE");
    }
}
