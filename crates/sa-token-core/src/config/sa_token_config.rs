//! Sa-Token 配置（对应 Java `cn.dev33.satoken.config.SaTokenConfig`）。
use serde::{Deserialize, Serialize};

use super::sa_cookie_config::SaCookieConfig;

/// Token 风格
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SaTokenStyle {
    /// UUID 风格（带连字符）
    Uuid,
    /// 简单 UUID（无连字符）
    SimpleUuid,
    /// 随机 32 位
    Random32,
    /// 随机 64 位
    Random64,
    /// 随机 128 位
    Random128,
    /// Base64 编码
    Base64,
    /// JWT
    Jwt,
}

impl Default for SaTokenStyle {
    fn default() -> Self {
        Self::Uuid
    }
}

impl std::fmt::Display for SaTokenStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uuid => write!(f, "uuid"),
            Self::SimpleUuid => write!(f, "simple-uuid"),
            Self::Random32 => write!(f, "random-32"),
            Self::Random64 => write!(f, "random-64"),
            Self::Random128 => write!(f, "random-128"),
            Self::Base64 => write!(f, "base64"),
            Self::Jwt => write!(f, "jwt"),
        }
    }
}

/// Sa-Token 全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaTokenConfig {
    /// Token 名称
    pub token_name: String,
    /// Token 有效期（秒），-1 代表永久有效
    pub timeout: i64,
    /// Token 最低活跃频率（秒），-1 代表不限制
    pub active_timeout: i64,
    /// 是否允许同一账号并发登录
    pub is_concurrent: bool,
    /// 多人登录同一账号时，是否共用一个 Token
    pub is_share: bool,
    /// 同一账号最大登录数量，-1 代表不限制
    pub max_login_count: i32,
    /// 是否尝试从 Body 里读取 Token
    pub is_read_body: bool,
    /// 是否尝试从 Header 里读取 Token
    pub is_read_header: bool,
    /// 是否尝试从 Cookie 里读取 Token
    pub is_read_cookie: bool,
    /// 是否输出操作日志
    pub is_log: bool,
    /// Token 风格
    pub token_style: SaTokenStyle,
    /// Token 前缀
    pub token_prefix: String,
    /// 是否在登录后写入 Token 到响应头
    pub is_write_header: bool,
    /// JWT 密钥
    pub jwt_secret_key: String,
    /// Cookie 配置
    pub cookie: SaCookieConfig,
    /// 是否持久化 Cookie
    pub is_lasting_cookie: bool,
    /// Cookie 自动填充前缀
    pub cookie_auto_fill_prefix: bool,
    /// 是否检查 Same-Token
    pub check_same_token: bool,
    /// Same-Token 超时时间（秒）
    pub same_token_timeout: i64,
    /// 是否立即创建 Token-Session
    pub right_now_create_token_session: bool,
}

impl Default for SaTokenConfig {
    fn default() -> Self {
        Self {
            token_name: "satoken".to_string(),
            timeout: 60 * 60 * 24 * 30,
            active_timeout: -1,
            is_concurrent: true,
            is_share: true,
            max_login_count: -1,
            is_read_body: true,
            is_read_header: true,
            is_read_cookie: true,
            is_log: true,
            token_style: SaTokenStyle::Uuid,
            token_prefix: String::new(),
            is_write_header: true,
            jwt_secret_key: String::new(),
            cookie: SaCookieConfig::default(),
            is_lasting_cookie: true,
            cookie_auto_fill_prefix: false,
            check_same_token: false,
            same_token_timeout: 60 * 60 * 24,
            right_now_create_token_session: false,
        }
    }
}

impl SaTokenConfig {
    /// 获取 Token 名称
    pub fn token_name(&self) -> &str {
        &self.token_name
    }

    /// 设置 Token 名称
    pub fn set_token_name(&mut self, name: impl Into<String>) {
        self.token_name = name.into();
    }

    /// 获取 Token 超时时间
    pub fn timeout(&self) -> i64 {
        self.timeout
    }

    /// 设置 Token 超时时间
    pub fn set_timeout(&mut self, timeout: i64) {
        self.timeout = timeout;
    }

    /// 获取活跃超时时间
    pub fn active_timeout(&self) -> i64 {
        self.active_timeout
    }

    /// 设置活跃超时时间
    pub fn set_active_timeout(&mut self, timeout: i64) {
        self.active_timeout = timeout;
    }

    /// 是否允许并发登录
    pub fn is_concurrent(&self) -> bool {
        self.is_concurrent
    }

    /// 设置是否允许并发登录
    pub fn set_is_concurrent(&mut self, concurrent: bool) {
        self.is_concurrent = concurrent;
    }

    /// 是否共享 Token
    pub fn is_share(&self) -> bool {
        self.is_share
    }

    /// 设置是否共享 Token
    pub fn set_is_share(&mut self, share: bool) {
        self.is_share = share;
    }

    /// 获取最大登录数
    pub fn max_login_count(&self) -> i32 {
        self.max_login_count
    }

    /// 设置最大登录数
    pub fn set_max_login_count(&mut self, count: i32) {
        self.max_login_count = count;
    }

    /// 是否从 Body 读取 Token
    pub fn is_read_body(&self) -> bool {
        self.is_read_body
    }

    /// 是否从 Header 读取 Token
    pub fn is_read_header(&self) -> bool {
        self.is_read_header
    }

    /// 是否从 Cookie 读取 Token
    pub fn is_read_cookie(&self) -> bool {
        self.is_read_cookie
    }

    /// 是否输出日志
    pub fn is_log(&self) -> bool {
        self.is_log
    }

    /// 设置是否输出日志
    pub fn set_is_log(&mut self, log: bool) {
        self.is_log = log;
    }

    /// 获取 Token 风格
    pub fn token_style(&self) -> &SaTokenStyle {
        &self.token_style
    }

    /// 设置 Token 风格
    pub fn set_token_style(&mut self, style: SaTokenStyle) {
        self.token_style = style;
    }

    /// 获取 Token 前缀
    pub fn token_prefix(&self) -> &str {
        &self.token_prefix
    }

    /// 设置 Token 前缀
    pub fn set_token_prefix(&mut self, prefix: impl Into<String>) {
        self.token_prefix = prefix.into();
    }

    /// 是否在登录后写入响应头
    pub fn is_write_header(&self) -> bool {
        self.is_write_header
    }

    /// 设置是否在登录后写入响应头
    pub fn set_is_write_header(&mut self, write: bool) {
        self.is_write_header = write;
    }

    /// 获取 JWT 密钥
    pub fn jwt_secret_key(&self) -> &str {
        &self.jwt_secret_key
    }

    /// 设置 JWT 密钥
    pub fn set_jwt_secret_key(&mut self, key: impl Into<String>) {
        self.jwt_secret_key = key.into();
    }

    /// 获取 Cookie 配置
    pub fn cookie(&self) -> &SaCookieConfig {
        &self.cookie
    }

    /// 设置 Cookie 配置
    pub fn set_cookie(&mut self, cookie: SaCookieConfig) {
        self.cookie = cookie;
    }

    /// 是否持久化 Cookie
    pub fn is_lasting_cookie(&self) -> bool {
        self.is_lasting_cookie
    }

    /// 设置是否持久化 Cookie
    pub fn set_is_lasting_cookie(&mut self, lasting: bool) {
        self.is_lasting_cookie = lasting;
    }

    /// 获取 Cookie 超时时间
    pub fn cookie_timeout(&self) -> i64 {
        if self.is_lasting_cookie {
            60 * 60 * 24 * 365
        } else {
            -1
        }
    }
}
