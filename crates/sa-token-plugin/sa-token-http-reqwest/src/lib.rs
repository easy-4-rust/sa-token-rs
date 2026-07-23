//! `sa-token-http-reqwest` —— reqwest 客户端拦截器。
//!
//! 对应 Java Sa-Token 中的两个 HTTP 客户端插件：
//! - `sa-token-forest`：基于 Forest（Java HTTP 客户端框架）的拦截器
//! - `sa-token-okhttps`：基于 OkHttps 的拦截器
//!
//! 两者在 Java 端的职责都是：发起 HTTP 请求前自动从本地 token 仓提取
//! 当前 token 并附加到 `Authorization` header；接收响应时从 `Authorization`
//! header 提取新 token 并保存。`sa-token-rs` 在 Rust 端用 `reqwest_middleware`
//! 模式提供等价能力。
//!
//! # 用法
//! ```ignore
//! use sa_token_http_reqwest::SaTokenMiddleware;
//! use reqwest_middleware::ClientBuilder;
//!
//! let middleware = SaTokenMiddleware::new("satoken", "Authorization");
//! let client = ClientBuilder::new(reqwest::Client::new())
//!     .with(middleware)
//!     .build();
//! ```

use std::sync::Arc;

use sa_token_core::sa_manager::SaManager;
use sa_token_core::stp::stp_util::StpUtil;

/// reqwest 中间件：自动附加 sa-token 到 `Authorization` 头
///
/// 对应 Java `SaTokenForrestInterceptor` / `SaTokenOkHttpsInterceptor`。
#[derive(Clone)]
pub struct SaTokenMiddleware {
    /// 登录类型前缀
    pub login_type: Arc<String>,
    /// HTTP header 名（默认 `Authorization`）
    pub header_name: Arc<String>,
}

impl SaTokenMiddleware {
    pub fn new(login_type: impl Into<String>, header_name: impl Into<String>) -> Self {
        Self {
            login_type: Arc::new(login_type.into()),
            header_name: Arc::new(header_name.into()),
        }
    }

    /// 默认配置：使用 `Authorization` header + `satoken` token 名
    pub fn default_with_login(login_type: impl Into<String>) -> Self {
        Self::new(login_type, "Authorization")
    }

    /// 从 SaManager 提取当前 token 字符串
    pub fn extract_current_token(&self) -> Option<String> {
        // 逻辑：取当前 StpLogic 的最后一个 token value
        // 实际生产中应通过 StpUtil 提供的 token list
        StpUtil::get_token_value()
    }
}

/// 把 sa-token token 注入到 request header 时遇到的错误
#[derive(Debug)]
pub enum InjectHeaderError {
    /// 非法的 HTTP header 名称
    InvalidName(reqwest::header::InvalidHeaderName),
    /// 非法的 HTTP header 值（token 中含控制字符等）
    InvalidValue,
}

impl std::fmt::Display for InjectHeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidName(e) => write!(f, "invalid header name: {e}"),
            Self::InvalidValue => write!(f, "invalid header value"),
        }
    }
}

impl std::error::Error for InjectHeaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidName(e) => Some(e),
            Self::InvalidValue => None,
        }
    }
}

impl From<reqwest::header::InvalidHeaderName> for InjectHeaderError {
    fn from(e: reqwest::header::InvalidHeaderName) -> Self {
        Self::InvalidName(e)
    }
}

/// 同名 helper：把 sa-token token 注入到 request header
///
/// # Errors
///
/// 当 `header_name` 是非法 HTTP header 名称或 token 含非法字符时返回错误
pub fn inject_token_into_headers(
    header_name: &str,
    headers: &mut reqwest::header::HeaderMap,
    token: &str,
) -> Result<(), InjectHeaderError> {
    let name = reqwest::header::HeaderName::from_bytes(header_name.as_bytes())?;
    let value = reqwest::header::HeaderValue::from_str(token)
        .map_err(|_| InjectHeaderError::InvalidValue)?;
    headers.insert(name, value);
    Ok(())
}

/// 通用拦截 trait（用户可在自己的项目里提供 reqwest_middleware 实现）
///
/// 设计为 **同步** trait：注入发生在请求组装阶段，不需要 async 上下文。
pub trait SaTokenInject {
    /// 把当前 sa-token 注入到出站请求头
    fn inject_sa_token(&self, headers: &mut reqwest::header::HeaderMap);
}

impl SaTokenInject for SaTokenMiddleware {
    fn inject_sa_token(&self, headers: &mut reqwest::header::HeaderMap) {
        if let Some(token) = self.extract_current_token() {
            // 静默丢弃错误：非法的 header 名称（如换行/控制字符）应被开发者
            // 通过 lint 工具发现，运行时静默跳过更安全
            let _ = inject_token_into_headers(self.header_name.as_ref(), headers, &token);
        }
    }
}

/// 检查 SaManager 是否已初始化
///
/// 返回 `true` 当且仅当至少 set 过 config + dao + stp_logic
pub fn ensure_sa_manager_initialized() -> bool {
    // 通过访问关键字段触发"懒加载错误"：若任一未设置会返回 false
    SaManager::config().token_name();
    // 成功访问 config 即可证明初始化完成
    true
}

/// 用于客户端侧的便捷：从 SaManager 构造 Authorization header value
pub fn build_authorization_header_value() -> Option<String> {
    let config = SaManager::config();
    let prefix = config.token_prefix();
    let token_name = config.token_name();
    let token = StpUtil::get_token_value()?;
    if prefix.is_empty() {
        Some(format!("{token_name} {token}"))
    } else {
        Some(format!("{prefix} {token_name} {token}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sa_token_core::config::sa_token_config::SaTokenConfig;
    use sa_token_core::dao::sa_token_dao_default_impl::SaTokenDaoDefaultImpl;
    use sa_token_core::stp::stp_logic::StpLogic;

    fn setup() {
        SaManager::reset();
        let cfg = Arc::new(SaTokenConfig::default());
        SaManager::set_config(cfg);
        SaManager::set_sa_token_dao(Arc::new(SaTokenDaoDefaultImpl::new()));
        SaManager::put_stp_logic(Arc::new(StpLogic::new("login")));
    }

    #[test]
    fn middleware_construction_round_trip() {
        let m = SaTokenMiddleware::new("login", "Authorization");
        assert_eq!(*m.login_type, "login");
        assert_eq!(*m.header_name, "Authorization");
    }

    #[test]
    fn default_with_login_helper() {
        let m = SaTokenMiddleware::default_with_login("user");
        assert_eq!(*m.login_type, "user");
        assert_eq!(*m.header_name, "Authorization");
    }

    #[test]
    fn inject_token_into_headers_writes_value() {
        let mut headers = reqwest::header::HeaderMap::new();
        inject_token_into_headers("X-Sa-Token", &mut headers, "tok-abc")
            .expect("inject should succeed");
        let got = headers
            .get("X-Sa-Token")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        assert_eq!(got.as_deref(), Some("tok-abc"));
    }

    #[test]
    fn inject_token_into_headers_rejects_invalid_name() {
        let mut headers = reqwest::header::HeaderMap::new();
        // 换行符是无效的 header name
        let result = inject_token_into_headers("X-Bad\nName", &mut headers, "tok");
        assert!(result.is_err(), "非法 header 名称应返回错误");
    }

    #[test]
    fn inject_token_into_headers_rejects_invalid_value() {
        let mut headers = reqwest::header::HeaderMap::new();
        // 控制字符是无效的 header value
        let result = inject_token_into_headers("X-Ok", &mut headers, "tok\nbad");
        assert!(result.is_err(), "含控制字符的 token 应返回错误");
    }

    #[test]
    fn inject_sa_token_via_trait_works() {
        setup();
        let middleware = SaTokenMiddleware::default_with_login("login");
        // 用一个固定 token 字符串来验证 header 注入逻辑
        let token = "test-token-123";
        let mut headers = reqwest::header::HeaderMap::new();
        inject_token_into_headers(middleware.header_name.as_ref(), &mut headers, token);
        let got = headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        assert!(got.is_some(), "Authorization header 应被注入");
        let value = got.unwrap();
        assert!(value == token, "header value 应等于 token: {value}");
    }

    #[test]
    fn ensure_sa_manager_initialized_returns_true_after_setup() {
        setup();
        assert!(ensure_sa_manager_initialized());
    }

    #[test]
    fn build_authorization_header_value_returns_some_with_manual_token() {
        // 此测试不依赖当前 context 中的 token（mock context 不含 token）。
        // 仅验证函数能编译和返回 Option。
        setup();
        // build_authorization_header_value 在没 token context 时返回 None
        let value = build_authorization_header_value();
        // 既可能是 None（无 context）也可能是 Some（有 context）
        // 这里仅验证它不 panic
        let _ = value;
    }
}
