//! `SaTokenContextException` —— 1:1 对应 Java `cn.dev33.satoken.exception.SaTokenContextException`

use std::fmt;

use crate::error::SaErrorCode;

/// Sa-Token 上下文异常（继承语义上对应 Java `InvalidContextException` 子类）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaTokenContextException {
    /// 错误信息
    pub message: String,
    /// 细分状态码
    pub code: i32,
}

impl SaTokenContextException {
    /// 构造上下文异常
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: SaErrorCode::CODE_UNDEFINED,
        }
    }

    /// 构造带状态码的上下文异常（对应 Java `setCode` 链式写法）
    pub fn with_code(message: impl Into<String>, code: i32) -> Self {
        Self {
            message: message.into(),
            code,
        }
    }

    /// 写入细分状态码
    pub fn set_code(mut self, code: i32) -> Self {
        self.code = code;
        self
    }

    /// 获取错误信息
    pub fn get_message(&self) -> &str {
        &self.message
    }

    /// 获取细分状态码
    pub fn get_code(&self) -> i32 {
        self.code
    }

    /// 转化为统一异常枚举
    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::with_code(self.code, self.message)
    }
}

impl fmt::Display for SaTokenContextException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[SaTokenContextException] {}", self.message)
    }
}

impl std::error::Error for SaTokenContextException {}

impl From<SaTokenContextException> for super::SaTokenException {
    fn from(e: SaTokenContextException) -> Self {
        e.into_sa_token_exception()
    }
}
