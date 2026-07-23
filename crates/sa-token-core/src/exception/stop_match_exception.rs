//! `StopMatchException` —— 1:1 对应 Java `cn.dev33.satoken.exception.StopMatchException`

use std::fmt;

/// 停止匹配异常（无附加数据，跳出匹配链）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopMatchException;

impl StopMatchException {
    pub fn new() -> Self {
        Self
    }

    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::StopMatch
    }
}

impl Default for StopMatchException {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for StopMatchException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[StopMatchException]")
    }
}

impl std::error::Error for StopMatchException {}

impl From<StopMatchException> for super::SaTokenException {
    fn from(e: StopMatchException) -> Self {
        e.into_sa_token_exception()
    }
}
