//! 异常模块（对应 Java `cn.dev33.satoken.exception`）。
//!
//! 将 Java 的 20+ 异常类折叠为单一 Rust enum。

/// Sa-Token 统一结果类型
pub type SaResult<T> = std::result::Result<T, SaTokenException>;

/// Sa-Token 统一异常枚举
///
/// 将 Java 的 20+ RuntimeException 子类折叠为单一 enum，每个 variant 对应一个 Java 异常类。
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum SaTokenException {
    /// 未登录（对应 `NotLoginException`）
    #[error("未登录: {message}, login_type={login_type}")]
    NotLogin {
        /// 错误信息
        message: String,
        /// 账号类型
        login_type: String,
    },

    /// 缺少权限（对应 `NotPermissionException`）
    #[error("缺少权限: {permission}, login_type={login_type}")]
    NotPermission {
        /// 缺少的权限码
        permission: String,
        /// 账号类型
        login_type: String,
    },

    /// 缺少角色（对应 `NotRoleException`）
    #[error("缺少角色: {role}, login_type={login_type}")]
    NotRole {
        /// 缺少的角色
        role: String,
        /// 账号类型
        login_type: String,
    },

    /// 未通过二级认证（对应 `NotSafeException`）
    #[error("未通过二级认证: service={service}, login_type={login_type}")]
    NotSafe {
        /// 业务标识
        service: String,
        /// 账号类型
        login_type: String,
    },

    /// 账号已被封禁（对应 `DisableServiceException`）
    #[error("账号已被封禁: login_id={login_id}, service={service}, 剩余={disable_time}s")]
    DisableService {
        /// 被封禁的账号 ID
        login_id: String,
        /// 业务标识
        service: String,
        /// 剩余封禁时间（秒）
        disable_time: i64,
    },

    /// Same-Token 无效（对应 `SameTokenInvalidException`）
    #[error("Same-Token 无效")]
    SameTokenInvalid,

    /// 上下文无效（对应 `InvalidContextException`）
    #[error("上下文无效")]
    InvalidContext,

    /// 非 Web 上下文（对应 `NotWebContextException`）
    #[error("非 Web 上下文")]
    NotWebContext,

    /// 防火墙拦截（对应 `FirewallCheckException`）
    #[error("防火墙拦截: {message}")]
    FirewallCheck {
        /// 拦截信息
        message: String,
    },

    /// 请求路径无效（对应 `RequestPathInvalidException`）
    #[error("请求路径无效: {path}")]
    RequestPathInvalid {
        /// 无效路径
        path: String,
    },

    /// 插件错误（对应 `SaTokenPluginException`）
    #[error("插件错误: {message}")]
    Plugin {
        /// 错误信息
        message: String,
    },

    /// API 被禁用（对应 `ApiDisabledException`）
    #[error("API 被禁用")]
    ApiDisabled,

    /// HTTP Basic 认证失败（对应 `NotHttpBasicAuthException`）
    #[error("HTTP Basic 认证失败")]
    NotHttpBasicAuth,

    /// HTTP Digest 认证失败（对应 `NotHttpDigestAuthException`）
    #[error("HTTP Digest 认证失败")]
    NotHttpDigestAuth,

    /// JSON 转换失败（对应 `SaJsonConvertException`）
    #[error("JSON 转换失败: {message}")]
    JsonConvert {
        /// 错误信息
        message: String,
    },

    /// 停止匹配（对应 `StopMatchException`）
    #[error("停止匹配")]
    StopMatch,

    /// TOTP 认证失败（对应 `TotpAuthException`）
    #[error("TOTP 认证失败")]
    TotpAuth,

    /// IO 错误
    #[error("IO 错误: {message}")]
    Io {
        /// 错误信息
        message: String,
    },

    /// 其他错误
    #[error("其他错误: {message}")]
    Other {
        /// 错误信息
        message: String,
    },
}

impl SaTokenException {
    /// 创建未登录异常
    pub fn not_login(message: impl Into<String>, login_type: impl Into<String>) -> Self {
        Self::NotLogin {
            message: message.into(),
            login_type: login_type.into(),
        }
    }

    /// 创建缺少权限异常
    pub fn not_permission(permission: impl Into<String>, login_type: impl Into<String>) -> Self {
        Self::NotPermission {
            permission: permission.into(),
            login_type: login_type.into(),
        }
    }

    /// 创建缺少角色异常
    pub fn not_role(role: impl Into<String>, login_type: impl Into<String>) -> Self {
        Self::NotRole {
            role: role.into(),
            login_type: login_type.into(),
        }
    }

    /// 创建未通过二级认证异常
    pub fn not_safe(service: impl Into<String>, login_type: impl Into<String>) -> Self {
        Self::NotSafe {
            service: service.into(),
            login_type: login_type.into(),
        }
    }

    /// 创建账号封禁异常
    pub fn disable_service(
        login_id: impl Into<String>,
        service: impl Into<String>,
        disable_time: i64,
    ) -> Self {
        Self::DisableService {
            login_id: login_id.into(),
            service: service.into(),
            disable_time,
        }
    }

    /// 创建防火墙拦截异常
    pub fn firewall_check(message: impl Into<String>) -> Self {
        Self::FirewallCheck {
            message: message.into(),
        }
    }

    /// 创建其他错误
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other {
            message: message.into(),
        }
    }
}
