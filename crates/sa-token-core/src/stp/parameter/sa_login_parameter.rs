//! 登录参数（对应 Java `cn.dev33.satoken.stp.parameter.SaLoginParameter`）。
use serde::{Deserialize, Serialize};

use crate::config::sa_token_config::SaTokenConfig;

/// 登录参数
///
/// 对应 Java `SaLoginParameter`，封装登录时的所有可选参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaLoginParameter {
    /// 设备类型
    pub device_type: String,
    /// 设备 ID
    pub device_id: String,
    /// Token 有效期（秒），-1 代表永久有效
    pub timeout: i64,
    /// 活跃超时时间（秒），-1 代表不限制
    pub active_timeout: Option<i64>,
    /// 是否允许并发登录
    pub is_concurrent: Option<bool>,
    /// 是否共享 Token
    pub is_share: Option<bool>,
    /// 最大登录数
    pub max_login_count: Option<i32>,
    /// 是否持久化 Cookie
    pub is_lasting_cookie: Option<bool>,
    /// 是否写入响应头
    pub is_write_header: Option<bool>,
    /// 扩展数据（JWT Claims 等）
    pub extra_data: Option<serde_json::Value>,
    /// 终端扩展数据
    pub terminal_extra_data: Option<serde_json::Value>,
    /// 预定 Token 值
    pub token: Option<String>,
}

impl Default for SaLoginParameter {
    fn default() -> Self {
        Self {
            device_type: String::new(),
            device_id: String::new(),
            timeout: -1,
            active_timeout: None,
            is_concurrent: None,
            is_share: None,
            max_login_count: None,
            is_lasting_cookie: None,
            is_write_header: None,
            extra_data: None,
            terminal_extra_data: None,
            token: None,
        }
    }
}

impl SaLoginParameter {
    /// 创建登录参数
    pub fn create() -> Self {
        Self::default()
    }

    /// 从全局配置初始化默认值
    pub fn with_default_values(config: &SaTokenConfig) -> Self {
        Self {
            timeout: config.timeout,
            ..Self::default()
        }
    }

    /// 设置设备类型
    pub fn set_device_type(mut self, device_type: impl Into<String>) -> Self {
        self.device_type = device_type.into();
        self
    }

    /// 设置设备 ID
    pub fn set_device_id(mut self, device_id: impl Into<String>) -> Self {
        self.device_id = device_id.into();
        self
    }

    /// 设置超时时间
    pub fn set_timeout(mut self, timeout: i64) -> Self {
        self.timeout = timeout;
        self
    }

    /// 设置活跃超时
    pub fn set_active_timeout(mut self, active_timeout: i64) -> Self {
        self.active_timeout = Some(active_timeout);
        self
    }

    /// 设置是否并发登录
    pub fn set_is_concurrent(mut self, is_concurrent: bool) -> Self {
        self.is_concurrent = Some(is_concurrent);
        self
    }

    /// 设置是否共享 Token
    pub fn set_is_share(mut self, is_share: bool) -> Self {
        self.is_share = Some(is_share);
        self
    }

    /// 设置最大登录数
    pub fn set_max_login_count(mut self, max_login_count: i32) -> Self {
        self.max_login_count = Some(max_login_count);
        self
    }

    /// 设置是否持久化 Cookie
    pub fn set_is_lasting_cookie(mut self, is_lasting_cookie: bool) -> Self {
        self.is_lasting_cookie = Some(is_lasting_cookie);
        self
    }

    /// 设置是否写入响应头
    pub fn set_is_write_header(mut self, is_write_header: bool) -> Self {
        self.is_write_header = Some(is_write_header);
        self
    }

    /// 设置扩展数据
    pub fn set_extra_data(mut self, extra_data: serde_json::Value) -> Self {
        self.extra_data = Some(extra_data);
        self
    }

    /// 设置终端扩展数据
    pub fn set_terminal_extra_data(mut self, terminal_extra_data: serde_json::Value) -> Self {
        self.terminal_extra_data = Some(terminal_extra_data);
        self
    }

    /// 设置预定 Token 值
    pub fn set_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// 获取超时时间（考虑全局配置）
    pub fn get_timeout(&self, config: &SaTokenConfig) -> i64 {
        if self.timeout >= 0 {
            self.timeout
        } else {
            config.timeout
        }
    }

    /// 获取活跃超时（考虑全局配置）
    pub fn get_active_timeout(&self, config: &SaTokenConfig) -> i64 {
        self.active_timeout.unwrap_or(config.active_timeout)
    }

    /// 是否并发登录（考虑全局配置）
    pub fn get_is_concurrent(&self, config: &SaTokenConfig) -> bool {
        self.is_concurrent.unwrap_or(config.is_concurrent)
    }

    /// 是否共享 Token（考虑全局配置）
    pub fn get_is_share(&self, config: &SaTokenConfig) -> bool {
        self.is_share.unwrap_or(config.is_share)
    }

    /// 获取最大登录数（考虑全局配置）
    pub fn get_max_login_count(&self, config: &SaTokenConfig) -> i32 {
        self.max_login_count.unwrap_or(config.max_login_count)
    }

    /// 是否持久化 Cookie（考虑全局配置）
    pub fn get_is_lasting_cookie(&self, config: &SaTokenConfig) -> bool {
        self.is_lasting_cookie.unwrap_or(config.is_lasting_cookie)
    }

    /// 是否写入响应头（考虑全局配置）
    pub fn get_is_write_header(&self, config: &SaTokenConfig) -> bool {
        self.is_write_header.unwrap_or(config.is_write_header)
    }
}
