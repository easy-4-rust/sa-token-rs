//! `SaJsonConvertException` —— 1:1 对应 Java `cn.dev33.satoken.exception.SaJsonConvertException`

use std::fmt;

/// JSON 转换异常
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaJsonConvertException {
    /// 错误信息
    pub message: String,
}

impl SaJsonConvertException {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }

    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::JsonConvert {
            message: self.message,
        }
    }
}

impl fmt::Display for SaJsonConvertException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[SaJsonConvertException] {}", self.message)
    }
}

impl std::error::Error for SaJsonConvertException {}

impl From<SaJsonConvertException> for super::SaTokenException {
    fn from(e: SaJsonConvertException) -> Self {
        e.into_sa_token_exception()
    }
}

impl From<serde_json::Error> for SaJsonConvertException {
    fn from(e: serde_json::Error) -> Self {
        Self::new(e.to_string())
    }
}
