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

/// Session 类型：匿名 Token 会话（对应 Java `SESSION_TYPE__ANON`）
pub const SESSION_TYPE_ANON: &str = "Anon-Token-Session";

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

/// 默认禁用服务标识（对应 Java `SaTokenConsts.DEFAULT_DISABLE_SERVICE`）
pub const DEFAULT_DISABLE_SERVICE: &str = "login";

/// 默认封禁等级（对应 Java `SaTokenConsts.DEFAULT_DISABLE_LEVEL`）
pub const DEFAULT_DISABLE_LEVEL: i32 = 1;

/// 阶梯封禁最低等级（对应 Java `SaTokenConsts.MIN_DISABLE_LEVEL`）
pub const MIN_DISABLE_LEVEL: i32 = 1;

/// 未封禁等级标记（对应 Java `SaTokenConsts.NOT_DISABLE_LEVEL`）
pub const NOT_DISABLE_LEVEL: i32 = -2;

/// 默认二级认证服务标识（对应 Java `SaTokenConsts.DEFAULT_SAFE_AUTH_SERVICE`）
pub const DEFAULT_SAFE_AUTH_SERVICE: &str = "important";

/// Http Basic/Digest 默认 realm（对应 Java `SaHttpBasicTemplate.DEFAULT_REALM`）
pub const DEFAULT_HTTP_AUTH_REALM: &str = "Sa-Token";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_consts_match_java_defaults() {
        assert_eq!(DEFAULT_TOKEN_NAME, "satoken");
        assert_eq!(DEFAULT_TIMEOUT, 60 * 60 * 24 * 30);
        assert_eq!(NEVER_EXPIRE, -1);
        assert_eq!(NOT_VALUE_EXPIRE, -2);
        assert_eq!(DEFAULT_DISABLE_SERVICE, "login");
    }
}
