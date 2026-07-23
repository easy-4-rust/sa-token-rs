//! `FirewallCheckException` —— 1:1 对应 Java `cn.dev33.satoken.exception.FirewallCheckException`

use std::fmt;

/// 防火墙拦截异常
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirewallCheckException {
    /// 拦截信息
    pub message: String,
}

impl FirewallCheckException {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }

    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::FirewallCheck {
            message: self.message,
        }
    }
}

impl fmt::Display for FirewallCheckException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[FirewallCheckException] {}", self.message)
    }
}

impl std::error::Error for FirewallCheckException {}

impl From<FirewallCheckException> for super::SaTokenException {
    fn from(e: FirewallCheckException) -> Self {
        e.into_sa_token_exception()
    }
}
