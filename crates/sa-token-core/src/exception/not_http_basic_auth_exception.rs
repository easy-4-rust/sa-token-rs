//! `NotHttpBasicAuthException` —— 1:1 对应 Java `cn.dev33.satoken.exception.NotHttpBasicAuthException`

use std::fmt;

/// HTTP Basic 认证失败异常
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotHttpBasicAuthException;

impl NotHttpBasicAuthException {
    pub fn new() -> Self {
        Self
    }

    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::NotHttpBasicAuth
    }
}

impl Default for NotHttpBasicAuthException {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NotHttpBasicAuthException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[NotHttpBasicAuthException] HTTP Basic 认证失败")
    }
}

impl std::error::Error for NotHttpBasicAuthException {}

impl From<NotHttpBasicAuthException> for super::SaTokenException {
    fn from(e: NotHttpBasicAuthException) -> Self {
        e.into_sa_token_exception()
    }
}
