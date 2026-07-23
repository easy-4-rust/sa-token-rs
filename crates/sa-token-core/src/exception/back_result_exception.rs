//! `BackResultException` —— 1:1 对应 Java `cn.dev33.satoken.exception.BackResultException`

use std::fmt;

/// 返回结果异常
///
/// 携带一个返回值，跳出匹配链并向客户端返回结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackResultException {
    /// 要返回给前端的结果
    pub result: String,
}

impl BackResultException {
    pub fn new(result: impl Into<String>) -> Self {
        Self {
            result: result.into(),
        }
    }

    pub fn get_result(&self) -> &str {
        &self.result
    }

    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::BackResult {
            result: self.result,
        }
    }
}

impl fmt::Display for BackResultException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[BackResultException] {}", self.result)
    }
}

impl std::error::Error for BackResultException {}

impl From<BackResultException> for super::SaTokenException {
    fn from(e: BackResultException) -> Self {
        e.into_sa_token_exception()
    }
}
