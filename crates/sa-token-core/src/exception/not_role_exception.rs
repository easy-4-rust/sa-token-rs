//! `NotRoleException` —— 1:1 对应 Java `cn.dev33.satoken.exception.NotRoleException`

use std::fmt;

/// 缺少角色异常
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotRoleException {
    /// 缺少的角色
    pub role: String,
    /// 账号类型
    pub login_type: String,
}

impl NotRoleException {
    pub fn new(role: impl Into<String>, login_type: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            login_type: login_type.into(),
        }
    }

    pub fn get_role(&self) -> &str {
        &self.role
    }

    pub fn get_login_type(&self) -> &str {
        &self.login_type
    }

    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::NotRole {
            role: self.role,
            login_type: self.login_type,
        }
    }
}

impl fmt::Display for NotRoleException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[NotRoleException] {} 缺少角色: {}",
            self.login_type, self.role
        )
    }
}

impl std::error::Error for NotRoleException {}

impl From<NotRoleException> for super::SaTokenException {
    fn from(e: NotRoleException) -> Self {
        e.into_sa_token_exception()
    }
}
