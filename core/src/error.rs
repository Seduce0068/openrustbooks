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
}
