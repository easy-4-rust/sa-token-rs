//! `DisableServiceException` —— 1:1 对应 Java `cn.dev33.satoken.exception.DisableServiceException`

use std::fmt;

/// 账号被封禁异常
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisableServiceException {
    /// 被封禁账号 ID
    pub login_id: String,
    /// 业务标识
    pub service: String,
    /// 剩余封禁时间（秒）
    pub disable_time: i64,
}

impl DisableServiceException {
    pub fn new(login_id: impl Into<String>, service: impl Into<String>, disable_time: i64) -> Self {
        Self {
            login_id: login_id.into(),
            service: service.into(),
            disable_time,
        }
    }

    pub fn get_login_id(&self) -> &str {
        &self.login_id
    }

    pub fn get_service(&self) -> &str {
        &self.service
    }

    pub fn get_disable_time(&self) -> i64 {
        self.disable_time
    }

    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::DisableService {
            login_id: self.login_id,
            service: self.service,
            disable_time: self.disable_time,
        }
    }
}

impl fmt::Display for DisableServiceException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[DisableServiceException] login_id={} service={} 剩余={}s",
            self.login_id, self.service, self.disable_time
        )
    }
}

impl std::error::Error for DisableServiceException {}

impl From<DisableServiceException> for super::SaTokenException {
    fn from(e: DisableServiceException) -> Self {
        e.into_sa_token_exception()
    }
}
