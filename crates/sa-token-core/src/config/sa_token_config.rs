//! Sa-Token 配置（对应 Java `cn.dev33.satoken.config.SaTokenConfig`）。
use serde::{Deserialize, Serialize};

use super::sa_cookie_config::SaCookieConfig;
use crate::stp::parameter::enums::sa_logout_mode::SaLogoutMode;
use crate::stp::parameter::enums::sa_logout_range::SaLogoutRange;
use crate::stp::parameter::enums::sa_replaced_login_exit_mode::SaReplacedLoginExitMode;
use crate::stp::parameter::enums::sa_replaced_range::SaReplacedRange;

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
    /// Tik compact token.
    Tik,
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
            Self::Tik => write!(f, "tik"),
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
    /// Whether per-login dynamic active timeout is enabled.
    pub dynamic_active_timeout: bool,
    /// 是否允许同一账号并发登录
    pub is_concurrent: bool,
    /// 多人登录同一账号时，是否共用一个 Token
    pub is_share: bool,
    /// Which client gives up its session when concurrent login is disabled.
    pub replaced_login_exit_mode: SaReplacedLoginExitMode,
    /// Device range affected by replacement.
    pub replaced_range: SaReplacedRange,
    /// 同一账号最大登录数量，-1 代表不限制
    pub max_login_count: i32,
    /// Logout mode used when the login count overflows.
    pub overflow_logout_mode: SaLogoutMode,
    /// Maximum attempts used to generate a unique token.
    pub max_try_times: i32,
    /// 是否尝试从 Body 里读取 Token
    pub is_read_body: bool,
    /// 是否尝试从 Header 里读取 Token
    pub is_read_header: bool,
    /// 是否尝试从 Cookie 里读取 Token
    pub is_read_cookie: bool,
    /// Whether cookies persist after the browser closes.
    pub is_lasting_cookie: bool,
    /// 是否输出操作日志
    pub is_log: bool,
    /// Whether startup version art is printed.
    pub is_print: bool,
    /// Configured log level name.
    pub log_level: String,
    /// Numeric log level.
    pub log_level_int: i32,
    /// Optional colored-log override.
    pub is_color_log: Option<bool>,
    /// Token 风格
    pub token_style: SaTokenStyle,
    /// Token 前缀
    pub token_prefix: String,
    /// 是否在登录后写入 Token 到响应头
    pub is_write_header: bool,
    /// Default logout scope.
    pub logout_range: SaLogoutRange,
    /// Whether a frozen token retains logout operations.
    pub is_logout_keep_freeze_ops: bool,
    /// Whether logout keeps the token session.
    pub is_logout_keep_token_session: bool,
    /// JWT 密钥
    pub jwt_secret_key: String,
    /// Cookie 配置
    pub cookie: SaCookieConfig,
    /// Cookie 自动填充前缀
    pub cookie_auto_fill_prefix: bool,
    /// 是否检查 Same-Token
    pub check_same_token: bool,
    /// Same-Token 超时时间（秒）
    pub same_token_timeout: i64,
    /// 是否立即创建 Token-Session
    pub right_now_create_token_session: bool,
    /// Default DAO expired-data cleanup interval.
    pub data_refresh_period: i32,
    /// Whether token-session reads require login.
    pub token_session_check_login: bool,
    /// Whether active timeout is renewed automatically.
    pub auto_renew: bool,
    /// Default HTTP Basic credentials.
    pub http_basic: String,
    /// Default HTTP Digest credentials.
    pub http_digest: String,
    /// Current public application domain.
    pub curr_domain: Option<String>,
}

impl Default for SaTokenConfig {
    fn default() -> Self {
        Self {
            token_name: "satoken".to_string(),
            timeout: 60 * 60 * 24 * 30,
            active_timeout: -1,
            dynamic_active_timeout: false,
            is_concurrent: true,
            is_share: false,
            replaced_login_exit_mode: SaReplacedLoginExitMode::default(),
            replaced_range: SaReplacedRange::default(),
            max_login_count: 12,
            overflow_logout_mode: SaLogoutMode::default(),
            max_try_times: 12,
            is_read_body: true,
            is_read_header: true,
            is_read_cookie: true,
            is_lasting_cookie: true,
            is_log: false,
            is_print: true,
            log_level: "trace".to_owned(),
            log_level_int: 1,
            is_color_log: None,
            token_style: SaTokenStyle::Uuid,
            token_prefix: String::new(),
            is_write_header: false,
            logout_range: SaLogoutRange::default(),
            is_logout_keep_freeze_ops: false,
            is_logout_keep_token_session: false,
            jwt_secret_key: String::new(),
            cookie: SaCookieConfig::default(),
            cookie_auto_fill_prefix: false,
            check_same_token: false,
            same_token_timeout: 60 * 60 * 24,
            right_now_create_token_session: false,
            data_refresh_period: 30,
            token_session_check_login: true,
            auto_renew: true,
            http_basic: String::new(),
            http_digest: String::new(),
            curr_domain: None,
        }
    }
}

impl SaTokenConfig {
    /// 获取 Token 名称
    pub fn token_name(&self) -> &str {
        &self.token_name
    }

    /// 获取 Token 名称的克隆版本
    pub fn get_token_name(&self) -> String {
        self.token_name.clone()
    }

    /// 设置 Token 名称
    pub fn set_token_name(&mut self, name: impl Into<String>) {
        self.token_name = name.into();
    }

    /// 获取 Same-Token 超时时间
    pub fn get_same_token_timeout(&self) -> i64 {
        self.same_token_timeout
    }

    /// 设置 Same-Token 超时时间
    pub fn set_same_token_timeout(&mut self, timeout: i64) {
        self.same_token_timeout = timeout;
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
