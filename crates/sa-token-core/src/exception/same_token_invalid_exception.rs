//! `SameTokenInvalidException` —— 1:1 对应 Java `cn.dev33.satoken.exception.SameTokenInvalidException`

use std::fmt;

/// Same-Token 无效异常
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SameTokenInvalidException {
    /// 异常描述
    pub message: String,
}

impl SameTokenInvalidException {
    /// 构造 Same-Token 无效异常
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }

    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::SameTokenInvalid {
            message: self.message,
        }
    }
}

impl fmt::Display for SameTokenInvalidException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[SameTokenInvalidException] {}", self.message)
    }
}

impl std::error::Error for SameTokenInvalidException {}

impl From<SameTokenInvalidException> for super::SaTokenException {
    fn from(e: SameTokenInvalidException) -> Self {
        e.into_sa_token_exception()
    }
}
