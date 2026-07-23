//! `NotHttpDigestAuthException` —— 1:1 对应 Java `cn.dev33.satoken.exception.NotHttpDigestAuthException`

use std::fmt;

/// HTTP Digest 认证失败异常
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotHttpDigestAuthException;

impl NotHttpDigestAuthException {
    pub fn new() -> Self {
        Self
    }

    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::NotHttpDigestAuth
    }
}

impl Default for NotHttpDigestAuthException {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NotHttpDigestAuthException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[NotHttpDigestAuthException] HTTP Digest 认证失败")
    }
}

impl std::error::Error for NotHttpDigestAuthException {}

impl From<NotHttpDigestAuthException> for super::SaTokenException {
    fn from(e: NotHttpDigestAuthException) -> Self {
        e.into_sa_token_exception()
    }
}
