//! `SaHttpDigestUtil` —— 1:1 对应 Java `cn.dev33.satoken.httpauth.digest.SaHttpDigestUtil`
//!
//! HTTP Digest 认证工具。

use crate::exception::SaTokenException;

use super::sa_http_digest_model::SaHttpDigestModel;
use super::sa_http_digest_template::SaHttpDigestTemplate;

/// HTTP Digest 工具（对应 Java `SaHttpDigestUtil`）。
pub struct SaHttpDigestUtil;

impl SaHttpDigestUtil {
    /// 获取 Authorization 值（对应 Java `getAuthorizationValue`）。
    pub fn get_authorization_value() -> Option<String> {
        SaHttpDigestTemplate::get_authorization_value()
    }

    /// 获取 Digest 模型（对应 Java `getAuthorizationValueToModel`）。
    pub fn get_authorization_value_to_model() -> Option<SaHttpDigestModel> {
        SaHttpDigestTemplate::get_authorization_value_to_model()
    }

    /// 根据 hope 模型校验（对应 Java `check(SaHttpDigestModel hopeModel)`）。
    pub fn check_with_model(hope: &SaHttpDigestModel) -> Result<(), SaTokenException> {
        SaHttpDigestTemplate::check_with_model(hope)
    }

    /// 根据用户名密码校验（对应 Java `check(String username, String password)`）。
    pub fn check_with_account(username: &str, password: &str) -> Result<(), SaTokenException> {
        SaHttpDigestTemplate::check_with_account(username, password)
    }

    /// 根据用户名密码与 realm 校验（对应 Java 三参 `check`）。
    pub fn check_with_account_and_realm(
        username: &str,
        password: &str,
        realm: &str,
    ) -> Result<(), SaTokenException> {
        SaHttpDigestTemplate::check_with_account_and_realm(username, password, realm)
    }

    /// 使用全局配置校验（对应 Java `check()`）。
    pub fn check() -> Result<(), SaTokenException> {
        SaHttpDigestTemplate::check()
    }
}
