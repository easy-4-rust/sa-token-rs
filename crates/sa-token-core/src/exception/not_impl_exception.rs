//! `NotImplException` —— 1:1 对应 Java `cn.dev33.satoken.exception.NotImplException`

use std::fmt;

/// 未实现异常（用于标记未实现的功能点）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotImplException {
    /// 错误信息
    pub message: String,
}

impl NotImplException {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }

    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::Other {
            message: format!("NotImpl: {}", self.message),
        }
    }
}

impl fmt::Display for NotImplException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[NotImplException] {}", self.message)
    }
}

impl std::error::Error for NotImplException {}

impl From<NotImplException> for super::SaTokenException {
    fn from(e: NotImplException) -> Self {
        e.into_sa_token_exception()
    }
}
