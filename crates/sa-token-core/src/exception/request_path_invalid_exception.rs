//! `RequestPathInvalidException` —— 1:1 对应 Java `cn.dev33.satoken.exception.RequestPathInvalidException`

use std::fmt;

/// 请求路径无效异常
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestPathInvalidException {
    /// 无效路径
    pub path: String,
}

impl RequestPathInvalidException {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::RequestPathInvalid { path: self.path }
    }
}

impl fmt::Display for RequestPathInvalidException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[RequestPathInvalidException] path={}", self.path)
    }
}

impl std::error::Error for RequestPathInvalidException {}

impl From<RequestPathInvalidException> for super::SaTokenException {
    fn from(e: RequestPathInvalidException) -> Self {
        e.into_sa_token_exception()
    }
}
