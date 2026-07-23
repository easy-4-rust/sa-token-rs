//! Explicit JWT failures with Java-compatible detailed codes.

/// JWT result type.
pub type SaJwtResult<T> = Result<T, SaJwtException>;

/// JWT operation failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("JWT error[{code}]: {message}")]
pub struct SaJwtException {
    code: i32,
    message: String,
}

impl SaJwtException {
    /// Creates an error with a detailed code.
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Returns the detailed Java-compatible error code.
    pub fn code(&self) -> i32 {
        self.code
    }

    /// Returns the non-sensitive diagnostic message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns an error when a required string is absent or blank.
    pub fn require_non_empty(
        value: Option<&str>,
        message: impl Into<String>,
        code: i32,
    ) -> SaJwtResult<&str> {
        value
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| Self::new(code, message))
    }
}
