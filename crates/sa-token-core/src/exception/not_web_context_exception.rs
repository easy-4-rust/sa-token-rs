//! `NotWebContextException` —— 1:1 对应 Java `cn.dev33.satoken.exception.NotWebContextException`

use std::fmt;

/// 非 Web 上下文异常
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotWebContextException;

impl NotWebContextException {
    pub fn new() -> Self {
        Self
    }

    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::NotWebContext
    }
}

impl Default for NotWebContextException {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NotWebContextException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[NotWebContextException] 非 Web 上下文")
    }
}

impl std::error::Error for NotWebContextException {}

impl From<NotWebContextException> for super::SaTokenException {
    fn from(e: NotWebContextException) -> Self {
        e.into_sa_token_exception()
    }
}
