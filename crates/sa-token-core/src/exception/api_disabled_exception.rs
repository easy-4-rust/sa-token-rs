//! `ApiDisabledException` —— 1:1 对应 Java `cn.dev33.satoken.exception.ApiDisabledException`

use std::fmt;

/// API 被禁用异常
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiDisabledException;

impl ApiDisabledException {
    pub fn new() -> Self {
        Self
    }

    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::ApiDisabled
    }
}

impl Default for ApiDisabledException {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ApiDisabledException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[ApiDisabledException] API 已被禁用")
    }
}

impl std::error::Error for ApiDisabledException {}

impl From<ApiDisabledException> for super::SaTokenException {
    fn from(e: ApiDisabledException) -> Self {
        e.into_sa_token_exception()
    }
}
