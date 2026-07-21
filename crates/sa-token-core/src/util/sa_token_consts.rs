//! Sa-Token 常量定义（对应 Java `cn.dev33.satoken.util.SaTokenConsts`）。

/// 默认 Token 名称
pub const DEFAULT_TOKEN_NAME: &str = "satoken";

/// 默认 Token 超时（30天，单位：秒）
pub const DEFAULT_TIMEOUT: i64 = 60 * 60 * 24 * 30;

/// 默认活跃超时（-1 表示不限制）
pub const DEFAULT_ACTIVE_TIMEOUT: i64 = -1;

/// 默认是否允许并发登录
pub const DEFAULT_IS_CONCURRENT: bool = true;

/// 默认是否共享 Token
pub const DEFAULT_IS_SHARE: bool = true;

/// 默认最大登录数（-1 表示不限制）
pub const DEFAULT_MAX_LOGIN_COUNT: i32 = -1;

/// 默认 Token 风格：UUID
pub const DEFAULT_TOKEN_STYLE: &str = "uuid";

/// Session 类型：登录会话
pub const SESSION_TYPE_LOGIN: &str = "login";

/// Session 类型：Token 会话
pub const SESSION_TYPE_TOKEN: &str = "token";

/// Session 常量：用户数据 key
pub const SESSION_KEY_USER: &str = "USER";

/// Session 常量：角色列表 key
pub const SESSION_KEY_ROLE_LIST: &str = "ROLE_LIST";

/// Session 常量：权限列表 key
pub const SESSION_KEY_PERMISSION_LIST: &str = "PERMISSION_LIST";

/// Redis Key 前缀
pub const REDIS_KEY_PREFIX: &str = "satoken:";

/// 永不过期
pub const NEVER_EXPIRE: i64 = -1;

/// 值不存在时的过期时间标记
pub const NOT_VALUE_EXPIRE: i64 = -2;
