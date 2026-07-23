//! `NotLoginException` —— 1:1 对应 Java `cn.dev33.satoken.exception.NotLoginException`
//!
//! 表示会话未通过登录认证校验。`scene` 字段告知前端触发该异常的具体原因。

use std::fmt;

use crate::error::SaErrorCode;

/// 未能读取到有效 token（对应 Java `NOT_TOKEN`）
pub const NOT_TOKEN: &str = "-1";
/// token 无效（对应 Java `INVALID_TOKEN`）
pub const INVALID_TOKEN: &str = "-2";
/// token 已过期（对应 Java `TOKEN_TIMEOUT`）
pub const TOKEN_TIMEOUT: &str = "-3";
/// token 已被顶下线（对应 Java `BE_REPLACED`）
pub const BE_REPLACED: &str = "-4";
/// token 已被踢下线（对应 Java `KICK_OUT`）
pub const KICK_OUT: &str = "-5";
/// token 已被冻结（对应 Java `TOKEN_FREEZE`）
pub const TOKEN_FREEZE: &str = "-6";
/// 未按指定前缀提交 token（对应 Java `NO_PREFIX`）
pub const NO_PREFIX: &str = "-7";

/// 默认提示语（对应 Java `DEFAULT_MESSAGE`）
pub const DEFAULT_MESSAGE: &str = "当前会话未登录";

/// 异常消息常量
pub const NOT_TOKEN_MESSAGE: &str = "未能读取到有效 token";
pub const INVALID_TOKEN_MESSAGE: &str = "token 无效";
pub const TOKEN_TIMEOUT_MESSAGE: &str = "token 已过期";
pub const BE_REPLACED_MESSAGE: &str = "token 已被顶下线";
pub const KICK_OUT_MESSAGE: &str = "token 已被踢下线";
pub const TOKEN_FREEZE_MESSAGE: &str = "token 已被冻结";
pub const NO_PREFIX_MESSAGE: &str = "未按照指定前缀提交 token";

/// 异常 token 标志集合（对应 Java `ABNORMAL_LIST`）
pub const ABNORMAL_LIST: [&str; 7] = [
    NOT_TOKEN,
    INVALID_TOKEN,
    TOKEN_TIMEOUT,
    BE_REPLACED,
    KICK_OUT,
    TOKEN_FREEZE,
    NO_PREFIX,
];

/// 兼容旧导出名
pub use NOT_TOKEN as NOT_LOGIN;
pub use NOT_TOKEN as TOKEN_NOT_PROVIDED;
pub use INVALID_TOKEN as TOKEN_INVALID;
pub use BE_REPLACED as TOKEN_BE_REPLACED;
pub use KICK_OUT as TOKEN_KICK_OUT;
pub use NO_PREFIX as TOKEN_NO_PREFIX;

/// `NotLoginException` —— 1:1 对应 Java 同名类
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotLoginException {
    /// 异常消息
    pub message: String,
    /// 账号类型（对应 Java `loginType`）
    pub login_type: String,
    /// 未登录场景值（对应 Java `type`）
    pub scene: String,
    /// 细分状态码（对应 Java `getCode()`）
    pub code: i32,
}

impl NotLoginException {
    /// 构造未登录异常
    pub fn new(message: impl Into<String>, login_type: impl Into<String>, scene: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            login_type: login_type.into(),
            scene: scene.into(),
            code: SaErrorCode::CODE_UNDEFINED,
        }
    }

    /// 静态构造（对应 Java `newInstance`）
    pub fn new_instance(
        login_type: impl Into<String>,
        scene: impl Into<String>,
        message: impl Into<String>,
        token: Option<&str>,
    ) -> Self {
        let message = match token {
            Some(token) if !token.is_empty() => format!("{}：{}", message.into(), token),
            _ => message.into(),
        };
        Self::new(message, login_type, scene)
    }

    /// 写入细分状态码（对应 Java `setCode`）
    pub fn with_code(mut self, code: i32) -> Self {
        self.code = code;
        self
    }

    /// 获取场景值（对应 Java `getType`）
    pub fn get_scene(&self) -> &str {
        &self.scene
    }

    /// 获取账号类型（对应 Java `getLoginType`）
    pub fn get_login_type(&self) -> &str {
        &self.login_type
    }

    /// 获取异常信息
    pub fn get_message(&self) -> &str {
        &self.message
    }

    /// 获取细分状态码
    pub fn get_code(&self) -> i32 {
        self.code
    }

    /// 转化为统一异常枚举
    pub fn into_sa_token_exception(self) -> super::SaTokenException {
        super::SaTokenException::NotLogin {
            message: self.message,
            login_type: self.login_type,
            scene: self.scene,
            code: self.code,
        }
    }
}

impl fmt::Display for NotLoginException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[NotLoginException scene={}] {}: {}",
            self.scene, self.login_type, self.message
        )
    }
}

impl std::error::Error for NotLoginException {}

impl From<NotLoginException> for super::SaTokenException {
    fn from(e: NotLoginException) -> Self {
        e.into_sa_token_exception()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_instance_appends_token() {
        let err = NotLoginException::new_instance("login", INVALID_TOKEN, INVALID_TOKEN_MESSAGE, Some("tok"));
        assert!(err.message.contains("tok"));
        assert_eq!(err.scene, INVALID_TOKEN);
    }

    #[test]
    fn abnormal_list_matches_java() {
        assert_eq!(ABNORMAL_LIST.len(), 7);
        assert!(ABNORMAL_LIST.contains(&NOT_TOKEN));
    }
}
