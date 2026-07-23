//! `SaCheckHttpBasic` —— 1:1 对应 Java `cn.dev33.satoken.annotation.SaCheckHttpBasic`
//!
//! Http Basic 认证校验：通过 Basic 认证后才能进入方法。

use crate::util::sa_token_consts::DEFAULT_HTTP_AUTH_REALM;

/// 注解运行时元数据（对应 Java `@SaCheckHttpBasic`）
///
/// Java 端 `account` 为 `user:pass` 合并串；Rust 拆为 `account` + `password` 便于校验。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaCheckHttpBasicMeta {
    /// 领域（对应 Java `realm()`，默认 `"Sa-Token"`）
    pub realm: &'static str,
    /// 账号（对应 Java `account` 中冒号前半段，或完整 `user:pass` 的 user 部分）
    pub account: &'static str,
    /// 密码（Rust 扩展字段；Java 合在 `account` 的 `user:pass` 中）
    pub password: &'static str,
}

impl SaCheckHttpBasicMeta {
    /// 使用 Java 默认值创建（`realm="Sa-Token"`, `account=""`, `password=""`）。
    pub const fn new() -> Self {
        Self {
            realm: DEFAULT_HTTP_AUTH_REALM,
            account: "",
            password: "",
        }
    }

    /// 指定 realm / account / password。
    pub const fn with_credentials(
        realm: &'static str,
        account: &'static str,
        password: &'static str,
    ) -> Self {
        Self {
            realm,
            account,
            password,
        }
    }
}

impl Default for SaCheckHttpBasicMeta {
    fn default() -> Self {
        Self::new()
    }
}
