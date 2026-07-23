//! `SaCheckHttpDigest` —— 1:1 对应 Java `cn.dev33.satoken.annotation.SaCheckHttpDigest`
//!
//! Http Digest 认证校验：通过 Digest 认证后才能进入方法。

use crate::util::sa_token_consts::DEFAULT_HTTP_AUTH_REALM;

/// 注解运行时元数据（对应 Java `@SaCheckHttpDigest`）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaCheckHttpDigestMeta {
    /// 领域（对应 Java `realm()`，默认 `"Sa-Token"`）
    pub realm: &'static str,
    /// 用户名（对应 Java `username()` / `value` 中的 user）
    pub account: &'static str,
    /// 密码（对应 Java `password()`）
    pub password: &'static str,
}

impl SaCheckHttpDigestMeta {
    /// 使用 Java 默认值创建（`realm="Sa-Token"`, 账号密码为空）。
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

impl Default for SaCheckHttpDigestMeta {
    fn default() -> Self {
        Self::new()
    }
}
