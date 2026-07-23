//! `SaTokenPluginException` —— 1:1 对应 Java `cn.dev33.satoken.exception.SaTokenPluginException`

use std::fmt;

/// Sa-Token 插件异常
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaTokenPluginException {
    /// 错误信息
    pub message: String,
}

impl SaTokenPluginException {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }

    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::Plugin {
            message: self.message,
        }
    }
}

impl fmt::Display for SaTokenPluginException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[SaTokenPluginException] {}", self.message)
    }
}

impl std::error::Error for SaTokenPluginException {}

impl From<SaTokenPluginException> for super::SaTokenException {
    fn from(e: SaTokenPluginException) -> Self {
        e.into_sa_token_exception()
    }
}
