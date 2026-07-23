//! 异常模块（对应 Java `cn.dev33.satoken.exception`）。
//!
//! 将 Java 的 20+ 异常类折叠为单一 Rust enum。

/// Sa-Token 统一结果类型
// ---------- 子模块声明 ----------
pub mod api_disabled_exception;
pub mod back_result_exception;
pub mod disable_service_exception;
pub mod firewall_check_exception;
pub mod invalid_context_exception;
pub mod not_http_basic_auth_exception;
pub mod not_http_digest_auth_exception;
pub mod not_impl_exception;
pub mod not_login_exception;
pub mod not_permission_exception;
pub mod not_role_exception;
pub mod not_safe_exception;
pub mod not_web_context_exception;
pub mod request_path_invalid_exception;
pub mod sa_json_convert_exception;
pub mod sa_token_context_exception;
pub mod sa_token_exception;
pub mod sa_token_plugin_exception;
pub mod same_token_invalid_exception;
pub mod stop_match_exception;
pub mod totp_auth_exception;

// ---------- re-exports ----------
pub use not_login_exception::ABNORMAL_LIST;
pub use not_login_exception::BE_REPLACED;
pub use not_login_exception::BE_REPLACED_MESSAGE;
pub use not_login_exception::DEFAULT_MESSAGE;
pub use not_login_exception::INVALID_TOKEN;
pub use not_login_exception::INVALID_TOKEN_MESSAGE;
pub use not_login_exception::KICK_OUT;
pub use not_login_exception::KICK_OUT_MESSAGE;
pub use not_login_exception::NOT_TOKEN;
pub use not_login_exception::NOT_TOKEN_MESSAGE;
pub use not_login_exception::NO_PREFIX;
pub use not_login_exception::NO_PREFIX_MESSAGE;
pub use not_login_exception::TOKEN_FREEZE;
pub use not_login_exception::TOKEN_FREEZE_MESSAGE;
pub use not_login_exception::TOKEN_TIMEOUT;
pub use not_login_exception::TOKEN_TIMEOUT_MESSAGE;

pub use firewall_check_exception::FirewallCheckException;
pub use not_http_basic_auth_exception::NotHttpBasicAuthException;
pub use not_http_digest_auth_exception::NotHttpDigestAuthException;
pub use request_path_invalid_exception::RequestPathInvalidException;
pub use sa_token_context_exception::SaTokenContextException;
pub use stop_match_exception::StopMatchException;
pub use api_disabled_exception::ApiDisabledException;
pub use back_result_exception::BackResultException;
pub use disable_service_exception::DisableServiceException;
pub use invalid_context_exception::InvalidContextException;
pub use not_impl_exception::NotImplException;
pub use not_login_exception::NotLoginException;
pub use not_permission_exception::NotPermissionException;
pub use not_role_exception::NotRoleException;
pub use not_safe_exception::NotSafeException;
pub use not_web_context_exception::NotWebContextException;
pub use sa_json_convert_exception::SaJsonConvertException;
pub use sa_token_plugin_exception::SaTokenPluginException;
pub use same_token_invalid_exception::SameTokenInvalidException;
pub use totp_auth_exception::TotpAuthException;

pub type SaResult<T> = std::result::Result<T, SaTokenException>;

/// Sa-Token 统一异常枚举
///
/// 将 Java 的 20+ RuntimeException 子类折叠为单一 enum，每个 variant 对应一个 Java 异常类。
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum SaTokenException {
    /// Framework error carrying Java-compatible detailed code metadata.
    #[error("Sa-Token 错误[{code}]: {message}")]
    Framework {
        /// Detailed code from [`crate::error::SaErrorCode`].
        code: i32,
        /// Human-readable error message.
        message: String,
    },

    /// 未登录（对应 `NotLoginException`）
    #[error("未登录: {message}, login_type={login_type}, scene={scene}")]
    NotLogin {
        /// 错误信息
        message: String,
        /// 账号类型
        login_type: String,
        /// 未登录场景值（Java `type`）
        scene: String,
        /// 细分状态码
        code: i32,
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
        /// 未通过校验的 Token 值
        token_value: String,
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
    #[error("Same-Token 无效: {message}")]
    SameTokenInvalid {
        /// 异常描述
        message: String,
    },

    /// 上下文无效（对应 `InvalidContextException`）
    #[error("上下文无效: {message}")]
    InvalidContext {
        /// 错误信息
        message: String,
    },

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

    /// 自定义返回结果（对应 `BackResultException`）
    #[error("自定义返回: {result}")]
    BackResult {
        /// 自定义返回内容
        result: String,
    },

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
    /// Creates a framework error with a detailed Java-compatible code.
    pub fn with_code(code: i32, message: impl Into<String>) -> Self {
        Self::Framework {
            code,
            message: message.into(),
        }
    }

    /// Returns the detailed code aligned with Java `SaTokenException.getCode()`.
    pub fn code(&self) -> i32 {
        use crate::error::SaErrorCode;
        match self {
            Self::Framework { code, .. } => *code,
            Self::NotLogin { code, .. } if *code != SaErrorCode::CODE_UNDEFINED => *code,
            Self::NotPermission { .. } => SaErrorCode::CODE_11051,
            Self::NotRole { .. } => SaErrorCode::CODE_11041,
            Self::NotSafe { .. } => SaErrorCode::CODE_11071,
            Self::DisableService { .. } => SaErrorCode::CODE_11061,
            Self::SameTokenInvalid { .. } => SaErrorCode::CODE_10301,
            Self::InvalidContext { .. } => SaErrorCode::CODE_10002,
            Self::NotHttpBasicAuth => SaErrorCode::CODE_10311,
            Self::NotHttpDigestAuth => SaErrorCode::CODE_10312,
            Self::ApiDisabled => SaErrorCode::CODE_11031,
            _ => SaErrorCode::CODE_UNDEFINED,
        }
    }

    /// Attaches a detailed code while preserving the rendered error message.
    pub fn set_code(self, code: i32) -> Self {
        Self::with_code(code, self.to_string())
    }

    /// Creates a JSON conversion error.
    pub fn json_convert(message: impl Into<String>) -> Self {
        Self::JsonConvert {
            message: message.into(),
        }
    }

    /// 创建未登录异常
    pub fn not_login(message: impl Into<String>, login_type: impl Into<String>) -> Self {
        Self::NotLogin {
            message: message.into(),
            login_type: login_type.into(),
            scene: not_login_exception::NOT_TOKEN.to_string(),
            code: crate::error::SaErrorCode::CODE_UNDEFINED,
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
            token_value: String::new(),
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

    /// 创建阶梯封禁异常（对应 Java `DisableServiceException` + `CODE_11061`）
    pub fn disable_service_level(
        login_id: impl Into<String>,
        service: impl Into<String>,
        level: i32,
        limit_level: i32,
        disable_time: i64,
    ) -> Self {
        let service = service.into();
        let login_id = login_id.into();
        Self::with_code(
            crate::error::SaErrorCode::CODE_11061,
            format!(
                "此账号已被禁止访问服务：{service}（login_id={login_id}，封禁等级={level}，校验等级={limit_level}，剩余={disable_time}s）"
            ),
        )
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
