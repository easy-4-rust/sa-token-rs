//! `SaLoginConfig` —— 1:1 对应 Java `cn.dev33.satoken.stp.SaLoginConfig`
//!
//! 登录配置项：timeout、是否新建 token、是否多端登录踢下线等。

/// 登录配置
#[derive(Debug, Clone)]
pub struct SaLoginConfig {
    /// token 超时（秒）
    pub timeout: i64,
    /// 是否共用同一 token（false = 同一账号多端共用 token）
    pub is_share: bool,
    /// 是否新建 token（强制踢下线旧的）
    pub is_create_session: bool,
    /// 被踢下线时是否响应错误信息
    pub is_replaced_login_failed: bool,
    /// 设备类型
    pub device_type: String,
    /// 多账号体系下的 login_type
    pub login_type: String,
}

impl Default for SaLoginConfig {
    fn default() -> Self {
        Self {
            timeout: -1,
            is_share: false,
            is_create_session: true,
            is_replaced_login_failed: false,
            device_type: String::new(),
            login_type: "login".to_string(),
        }
    }
}

impl SaLoginConfig {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_timeout(mut self, timeout: i64) -> Self {
        self.timeout = timeout;
        self
    }
    pub fn with_share(mut self, is_share: bool) -> Self {
        self.is_share = is_share;
        self
    }
    pub fn with_create_session(mut self, b: bool) -> Self {
        self.is_create_session = b;
        self
    }
    pub fn with_device_type(mut self, s: impl Into<String>) -> Self {
        self.device_type = s.into();
        self
    }
    pub fn with_login_type(mut self, s: impl Into<String>) -> Self {
        self.login_type = s.into();
        self
    }
}
