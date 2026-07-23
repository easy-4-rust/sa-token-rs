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

    // ----------------------------------------------------------------
    // M1.1: Java `SaTokenConfig` getter / setter 1:1 parity.
    // Names are snake_cased versions of the Java camelCase names so that
    // e.g. Java `getIsConcurrent()` -> `get_is_concurrent()`. Each method
    // is documented with the original Chinese javadoc, which doubles as
    // the canonical description of the field it touches.
    // ----------------------------------------------------------------

    /// 是否启用动态 activeTimeout 功能，如不需要请设置为 false，节省缓存请求次数
    pub fn get_dynamic_active_timeout(&self) -> bool {
        self.dynamic_active_timeout
    }

    /// 设置 dynamicActiveTimeout
    pub fn set_dynamic_active_timeout(&mut self, value: bool) -> &mut Self {
        self.dynamic_active_timeout = value;
        self
    }

    /// 在多人登录同一账号时，是否共用一个 token（对应 Java `getIsShare`）
    pub fn get_is_share(&self) -> bool {
        self.is_share
    }

    /// 设置 isShare（返回 `&mut Self` 以镜像 Java builder 语义）
    pub fn set_is_share_via_builder(&mut self, value: bool) -> &mut Self {
        self.is_share = value;
        self
    }

    /// 在 isConcurrent=false 时，决定新旧设备谁将放弃会话（OLD_DEVICE / NEW_DEVICE）
    pub fn get_replaced_login_exit_mode(&self) -> SaReplacedLoginExitMode {
        self.replaced_login_exit_mode
    }

    /// 设置 replacedLoginExitMode
    pub fn set_replaced_login_exit_mode(&mut self, value: SaReplacedLoginExitMode) -> &mut Self {
        self.replaced_login_exit_mode = value;
        self
    }

    /// 在 isConcurrent=false 时，顶人下线的范围（CURR_DEVICE_TYPE / ALL_DEVICE_TYPE）
    pub fn get_replaced_range(&self) -> SaReplacedRange {
        self.replaced_range
    }

    /// 设置 replacedRange
    pub fn set_replaced_range(&mut self, value: SaReplacedRange) -> &mut Self {
        self.replaced_range = value;
        self
    }

    /// 同一账号最大登录数量，-1 代表不限（只有在 isConcurrent=true, isShare=false 时此配置项才有意义）
    pub fn get_max_login_count(&self) -> i32 {
        self.max_login_count
    }

    /// 溢出 maxLoginCount 的客户端，将以何种方式注销下线（LOGOUT / KICKOUT / REPLACED）
    pub fn get_overflow_logout_mode(&self) -> SaLogoutMode {
        self.overflow_logout_mode
    }

    /// 设置 overflowLogoutMode
    pub fn set_overflow_logout_mode(&mut self, value: SaLogoutMode) -> &mut Self {
        self.overflow_logout_mode = value;
        self
    }

    /// 在每次创建 token 时的最高循环次数（-1=不循环尝试，直接使用）
    pub fn get_max_try_times(&self) -> i32 {
        self.max_try_times
    }

    /// 设置 maxTryTimes
    pub fn set_max_try_times(&mut self, value: i32) -> &mut Self {
        self.max_try_times = value;
        self
    }

    /// 是否尝试从请求体里读取 token
    pub fn get_is_read_body(&self) -> bool {
        self.is_read_body
    }

    /// 设置 isReadBody
    pub fn set_is_read_body(&mut self, value: bool) -> &mut Self {
        self.is_read_body = value;
        self
    }

    /// 是否尝试从 header 里读取 token
    pub fn get_is_read_header(&self) -> bool {
        self.is_read_header
    }

    /// 设置 isReadHeader
    pub fn set_is_read_header(&mut self, value: bool) -> &mut Self {
        self.is_read_header = value;
        self
    }

    /// 是否尝试从 cookie 里读取 token（Java `getIsReadCookie`）
    pub fn get_is_read_cookie(&self) -> bool {
        self.is_read_cookie
    }

    /// 设置 isReadCookie
    pub fn set_is_read_cookie(&mut self, value: bool) -> &mut Self {
        self.is_read_cookie = value;
        self
    }

    /// 是否为持久 Cookie（Java `getIsLastingCookie`）
    pub fn get_is_lasting_cookie(&self) -> bool {
        self.is_lasting_cookie
    }

    /// 设置 isLastingCookie
    pub fn set_is_lasting_cookie_via_builder(&mut self, value: bool) -> &mut Self {
        self.is_lasting_cookie = value;
        self
    }

    /// 是否在登录后将 token 写入到响应头（Java `getIsWriteHeader`）
    pub fn get_is_write_header(&self) -> bool {
        self.is_write_header
    }

    /// 注销范围（Java `getLogoutRange`）
    pub fn get_logout_range(&self) -> SaLogoutRange {
        self.logout_range
    }

    /// 设置 logoutRange
    pub fn set_logout_range(&mut self, value: SaLogoutRange) -> &mut Self {
        self.logout_range = value;
        self
    }

    /// 如果 token 已被冻结，是否保留其操作权（Java `getIsLogoutKeepFreezeOps`）
    pub fn get_is_logout_keep_freeze_ops(&self) -> bool {
        self.is_logout_keep_freeze_ops
    }

    /// 设置 isLogoutKeepFreezeOps
    pub fn set_is_logout_keep_freeze_ops(&mut self, value: bool) -> &mut Self {
        self.is_logout_keep_freeze_ops = value;
        self
    }

    /// 在注销 token 后，是否保留其对应的 Token-Session（Java `getIsLogoutKeepTokenSession`）
    pub fn get_is_logout_keep_token_session(&self) -> bool {
        self.is_logout_keep_token_session
    }

    /// 设置 isLogoutKeepTokenSession
    pub fn set_is_logout_keep_token_session(&mut self, value: bool) -> &mut Self {
        self.is_logout_keep_token_session = value;
        self
    }

    /// 在登录时，是否立即创建对应的 Token-Session（Java `getRightNowCreateTokenSession`）
    pub fn get_right_now_create_token_session(&self) -> bool {
        self.right_now_create_token_session
    }

    /// 设置 rightNowCreateTokenSession
    pub fn set_right_now_create_token_session(&mut self, value: bool) -> &mut Self {
        self.right_now_create_token_session = value;
        self
    }

    /// token 风格（Java `getTokenStyle`）
    pub fn get_token_style(&self) -> &SaTokenStyle {
        &self.token_style
    }

    /// 默认 SaTokenDao 实现类中，每次清理过期数据间隔的时间（Java `getDataRefreshPeriod`）
    pub fn get_data_refresh_period(&self) -> i32 {
        self.data_refresh_period
    }

    /// 设置 dataRefreshPeriod
    pub fn set_data_refresh_period(&mut self, value: i32) -> &mut Self {
        self.data_refresh_period = value;
        self
    }

    /// 获取 Token-Session 时是否必须登录（Java `getTokenSessionCheckLogin`）
    pub fn get_token_session_check_login(&self) -> bool {
        self.token_session_check_login
    }

    /// 设置 tokenSessionCheckLogin
    pub fn set_token_session_check_login(&mut self, value: bool) -> &mut Self {
        self.token_session_check_login = value;
        self
    }

    /// 是否打开自动续签 activeTimeout（Java `getAutoRenew`）
    pub fn get_auto_renew(&self) -> bool {
        self.auto_renew
    }

    /// 设置 autoRenew
    pub fn set_auto_renew(&mut self, value: bool) -> &mut Self {
        self.auto_renew = value;
        self
    }

    /// token 前缀（Java `getTokenPrefix`）
    pub fn get_token_prefix(&self) -> &str {
        &self.token_prefix
    }

    /// cookie 模式是否自动填充 token 前缀（Java `getCookieAutoFillPrefix`）
    pub fn get_cookie_auto_fill_prefix(&self) -> bool {
        self.cookie_auto_fill_prefix
    }

    /// 设置 cookieAutoFillPrefix
    pub fn set_cookie_auto_fill_prefix(&mut self, value: bool) -> &mut Self {
        self.cookie_auto_fill_prefix = value;
        self
    }

    /// 是否在初始化配置时在控制台打印版本字符画（Java `getIsPrint`）
    pub fn get_is_print(&self) -> bool {
        self.is_print
    }

    /// 设置 isPrint
    pub fn set_is_print(&mut self, value: bool) -> &mut Self {
        self.is_print = value;
        self
    }

    /// 是否打印操作日志（Java `getIsLog`）
    pub fn get_is_log(&self) -> bool {
        self.is_log
    }

    /// 设置 isLog
    pub fn set_is_log_via_builder(&mut self, value: bool) -> &mut Self {
        self.is_log = value;
        self
    }

    /// 日志等级（Java `getLogLevel`）
    pub fn get_log_level(&self) -> &str {
        &self.log_level
    }

    /// 日志等级 int 值（Java `getLogLevelInt`）
    pub fn get_log_level_int(&self) -> i32 {
        self.log_level_int
    }

    /// 设置 logLevel；同时按 Java `SaFoxUtil.translateLogLevelToInt` 联动 logLevelInt
    pub fn set_log_level(&mut self, level: impl Into<String>) -> &mut Self {
        let level = level.into();
        self.log_level = level.clone();
        self.log_level_int = crate::config::log_level_coupling::translate_log_level_to_int(&level);
        self
    }

    /// 设置 logLevelInt；同时按 Java `SaFoxUtil.translateLogLevelToString` 联动 logLevel
    pub fn set_log_level_int(&mut self, level_int: i32) -> &mut Self {
        self.log_level_int = level_int;
        self.log_level =
            crate::config::log_level_coupling::translate_log_level_to_string(level_int).to_owned();
        self
    }

    /// 是否打印彩色日志（Java `getIsColorLog`，可为 null）
    pub fn get_is_color_log(&self) -> Option<bool> {
        self.is_color_log
    }

    /// 设置 isColorLog（接受 `Option<bool>` 以镜像 Java `Boolean` 可空性）
    pub fn set_is_color_log(&mut self, value: Option<bool>) -> &mut Self {
        self.is_color_log = value;
        self
    }

    /// jwt 秘钥（Java `getJwtSecretKey`）
    pub fn get_jwt_secret_key(&self) -> &str {
        &self.jwt_secret_key
    }

    /// Http Basic 认证的默认账号和密码（Java `getHttpBasic`）
    pub fn get_http_basic(&self) -> &str {
        &self.http_basic
    }

    /// 设置 httpBasic
    pub fn set_http_basic(&mut self, value: impl Into<String>) -> &mut Self {
        self.http_basic = value.into();
        self
    }

    /// Http Digest 认证的默认账号和密码（Java `getHttpDigest`）
    pub fn get_http_digest(&self) -> &str {
        &self.http_digest
    }

    /// 设置 httpDigest
    pub fn set_http_digest(&mut self, value: impl Into<String>) -> &mut Self {
        self.http_digest = value.into();
        self
    }

    /// 配置当前项目的网络访问地址（Java `getCurrDomain`，可为 null）
    pub fn get_curr_domain(&self) -> Option<&str> {
        self.curr_domain.as_deref()
    }

    /// 设置 currDomain（接受 `Option<String>` 以镜像 Java 字段可空性）
    pub fn set_curr_domain(&mut self, value: Option<String>) -> &mut Self {
        self.curr_domain = value;
        self
    }

    /// Same-Token 的有效期（Java `getSameTokenTimeout`，单位：秒）
    pub fn get_same_token_timeout(&self) -> i64 {
        self.same_token_timeout
    }

    /// 是否校验 Same-Token（Java `getCheckSameToken`）
    pub fn get_check_same_token(&self) -> bool {
        self.check_same_token
    }

    /// 设置 checkSameToken
    pub fn set_check_same_token(&mut self, value: bool) -> &mut Self {
        self.check_same_token = value;
        self
    }

    /// SaCookieConfig 引用（Java `getCookie`）
    pub fn get_cookie(&self) -> &SaCookieConfig {
        &self.cookie
    }

    /// 设置 cookie（返回 `&mut Self` 以镜像 Java builder）
    pub fn set_cookie_via_builder(&mut self, cookie: SaCookieConfig) -> &mut Self {
        self.cookie = cookie;
        self
    }
}
