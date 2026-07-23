//! `SaTtlMethods` —— 1:1 对应 Java `cn.dev33.satoken.util.SaTtlMethods`
//!
//! Java 端是时间过期策略枚举常量枚举。
//! Rust 端以类型化 i64 提供。

/// 不限时长（-1 表示「永不过期」）
pub const NEVER_EXPIRE: i64 = -1;
/// 默认 token 超时（秒）
pub const DEFAULT_TIMEOUT: i64 = 60 * 60 * 24 * 7; // 7 天

/// TTL 工具
pub struct SaTtlMethods;

impl SaTtlMethods {
    pub const NEVER_EXPIRE: i64 = NEVER_EXPIRE;
    pub const DEFAULT_TIMEOUT: i64 = DEFAULT_TIMEOUT;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ttl_constants() {
        assert_eq!(SaTtlMethods::NEVER_EXPIRE, -1);
        assert_eq!(SaTtlMethods::DEFAULT_TIMEOUT, 60 * 60 * 24 * 7);
    }
}
