//! `SaLoginModel` —— 1:1 对应 Java `cn.dev33.satoken.stp.SaLoginModel`
//!
//! 单次登录请求的所有参数封装。

/// 登录模型
#[derive(Debug, Clone)]
pub struct SaLoginModel {
    /// 账号
    pub login_id: String,
    /// 设备
    pub device: String,
    /// 可选登录配置
    pub config: Option<SaLoginConfig>,
    /// 是否优先从 cookie 读取 token 名
    pub cookie_auto_call: bool,
    /// 自定义 token 名（默认为空走 config.login_type）
    pub token_name: String,
}

impl SaLoginModel {
    pub fn new(login_id: impl Into<String>) -> Self {
        Self {
            login_id: login_id.into(),
            device: String::new(),
            config: None,
            cookie_auto_call: true,
            token_name: String::new(),
        }
    }

    pub fn with_device(mut self, s: impl Into<String>) -> Self {
        self.device = s.into();
        self
    }

    pub fn with_config(mut self, c: SaLoginConfig) -> Self {
        self.config = Some(c);
        self
    }
}

use super::sa_login_config::SaLoginConfig;
