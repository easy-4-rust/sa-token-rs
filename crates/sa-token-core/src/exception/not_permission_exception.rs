//! `NotPermissionException` —— 1:1 对应 Java `cn.dev33.satoken.exception.NotPermissionException`

use std::fmt;

/// 缺少权限异常
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotPermissionException {
    /// 缺少的权限码
    pub permission: String,
    /// 账号类型
    pub login_type: String,
}

impl NotPermissionException {
    pub fn new(permission: impl Into<String>, login_type: impl Into<String>) -> Self {
        Self {
            permission: permission.into(),
            login_type: login_type.into(),
        }
    }

    pub fn get_permission(&self) -> &str {
        &self.permission
    }

    pub fn get_login_type(&self) -> &str {
        &self.login_type
    }

    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::NotPermission {
            permission: self.permission,
            login_type: self.login_type,
        }
    }
}

impl fmt::Display for NotPermissionException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[NotPermissionException] {} 缺少权限: {}",
            self.login_type, self.permission
        )
    }
}

impl std::error::Error for NotPermissionException {}

impl From<NotPermissionException> for super::SaTokenException {
    fn from(e: NotPermissionException) -> Self {
        e.into_sa_token_exception()
    }
}
