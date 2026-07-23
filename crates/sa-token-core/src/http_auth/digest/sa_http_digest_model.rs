//! `SaHttpDigestModel` —— 1:1 对应 Java `cn.dev33.satoken.httpauth.digest.SaHttpDigestModel`
//!
//! HTTP Digest 认证请求/响应模型。

use crate::util::sa_token_consts::DEFAULT_HTTP_AUTH_REALM;
use serde::{Deserialize, Serialize};

/// Digest 认证参数模型（对应 Java `SaHttpDigestModel`）。
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SaHttpDigestModel {
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 领域
    pub realm: String,
    /// 随机数
    pub nonce: String,
    /// 请求 uri
    pub uri: String,
    /// 请求方法
    pub method: String,
    /// 保护质量
    pub qop: String,
    /// nonce 计数器
    pub nc: String,
    /// 客户端随机数
    pub cnonce: String,
    /// opaque
    pub opaque: String,
    /// 请求摘要
    pub response: String,
}

impl SaHttpDigestModel {
    /// 默认 Realm（对应 Java `DEFAULT_REALM`）。
    pub const DEFAULT_REALM: &'static str = DEFAULT_HTTP_AUTH_REALM;
    /// 默认 qop（对应 Java `DEFAULT_QOP`）。
    pub const DEFAULT_QOP: &'static str = "auth";

    /// 创建模型（对应 Java `SaHttpDigestModel(username, password)`）。
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
            realm: Self::DEFAULT_REALM.to_string(),
            ..Default::default()
        }
    }

    /// 创建带 realm 的模型（对应 Java 三参构造）。
    pub fn with_realm(
        username: impl Into<String>,
        password: impl Into<String>,
        realm: impl Into<String>,
    ) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
            realm: realm.into(),
            ..Default::default()
        }
    }
}
