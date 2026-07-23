//! `InvalidContextException` —— 1:1 对应 Java `cn.dev33.satoken.exception.InvalidContextException`

use std::fmt;

/// 上下文无效异常（Java 中已标记 `@Deprecated`，由 `SaTokenContextException` 取代）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidContextException {
    /// 错误信息
    pub message: String,
}

impl InvalidContextException {
    /// 构造上下文无效异常
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// 获取错误信息
    pub fn get_message(&self) -> &str {
        &self.message
    }

    /// 转化为统一异常枚举
    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::InvalidContext {
            message: self.message,
        }
    }
}

impl fmt::Display for InvalidContextException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[InvalidContextException] {}", self.message)
    }
}

impl std::error::Error for InvalidContextException {}

impl From<InvalidContextException> for super::SaTokenException {
    fn from(e: InvalidContextException) -> Self {
        e.into_sa_token_exception()
    }
}
