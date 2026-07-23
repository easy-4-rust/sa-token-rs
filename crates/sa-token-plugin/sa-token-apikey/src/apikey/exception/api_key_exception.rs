//! General API Key exception.

/// Error carrying the offending key and stable detailed code.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("API Key error[{code}]: {message}")]
pub struct ApiKeyException {
    /// Stable code from `SaApiKeyErrorCode`.
    pub code: i32,
    /// Offending key, when available.
    pub api_key: Option<String>,
    /// Human-readable description.
    pub message: String,
}

impl ApiKeyException {
    /// Creates an API Key error.
    pub fn new(code: i32, api_key: Option<&str>, message: impl Into<String>) -> Self {
        Self {
            code,
            api_key: api_key.map(str::to_string),
            message: message.into(),
        }
    }
}
