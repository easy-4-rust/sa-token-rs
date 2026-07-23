//! `TotpAuthException` —— 1:1 对应 Java `cn.dev33.satoken.exception.TotpAuthException`

use std::fmt;

/// TOTP 认证失败异常
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TotpAuthException {
    /// 错误信息
    pub message: String,
}

impl TotpAuthException {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }

    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::TotpAuth
    }
}

impl fmt::Display for TotpAuthException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[TotpAuthException] {}", self.message)
    }
}

impl std::error::Error for TotpAuthException {}

impl From<TotpAuthException> for super::SaTokenException {
    fn from(e: TotpAuthException) -> Self {
        e.into_sa_token_exception()
    }
}
