//! `NotSafeException` —— 1:1 对应 Java `cn.dev33.satoken.exception.NotSafeException`

use std::fmt;

/// 异常提示语（对应 Java `BE_MESSAGE`）
pub const BE_MESSAGE: &str = "二级认证校验失败";

/// 未通过二级认证异常
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotSafeException {
    /// 账号类型
    pub login_type: String,
    /// 未通过校验的 Token 值
    pub token_value: String,
    /// 业务标识
    pub service: String,
}

impl NotSafeException {
    /// 构造二级认证失败异常（对应 Java 三参构造）
    pub fn new(
        login_type: impl Into<String>,
        token_value: impl Into<String>,
        service: impl Into<String>,
    ) -> Self {
        Self {
            login_type: login_type.into(),
            token_value: token_value.into(),
            service: service.into(),
        }
    }

    /// 兼容旧两参构造
    pub fn from_service(service: impl Into<String>, login_type: impl Into<String>) -> Self {
        Self::new(login_type, String::new(), service)
    }

    pub fn get_login_type(&self) -> &str {
        &self.login_type
    }

    pub fn get_token_value(&self) -> &str {
        &self.token_value
    }

    pub fn get_service(&self) -> &str {
        &self.service
    }

    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::NotSafe {
            service: self.service,
            login_type: self.login_type,
            token_value: self.token_value,
        }
    }
}

impl fmt::Display for NotSafeException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[NotSafeException] {} service={} token={}",
            self.login_type, self.service, self.token_value
        )
    }
}

impl std::error::Error for NotSafeException {}

impl From<NotSafeException> for super::SaTokenException {
    fn from(e: NotSafeException) -> Self {
        e.into_sa_token_exception()
    }
}
