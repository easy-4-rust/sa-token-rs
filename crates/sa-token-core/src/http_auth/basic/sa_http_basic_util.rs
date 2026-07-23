//! `SaHttpBasicUtil` —— 1:1 对应 Java `cn.dev33.satoken.httpauth.basic.SaHttpBasicUtil`
//!
//! HTTP Basic 认证工具类。

use crate::exception::SaTokenException;

use super::sa_http_basic_account::SaHttpBasicAccount;
use super::sa_http_basic_template::SaHttpBasicTemplate;

/// HTTP Basic 工具（对应 Java `SaHttpBasicUtil`）。
pub struct SaHttpBasicUtil;

impl SaHttpBasicUtil {
    /// 获取 Authorization 解码值（对应 Java `getAuthorizationValue`）。
    pub fn get_authorization_value() -> Option<String> {
        SaHttpBasicTemplate::get_authorization_value()
    }

    /// 获取 Basic 账号对象（对应 Java `getHttpBasicAccount`）。
    pub fn get_http_basic_account() -> Option<SaHttpBasicAccount> {
        SaHttpBasicTemplate::get_http_basic_account()
    }

    /// 使用全局配置校验（对应 Java `check()`）。
    pub fn check() -> Result<(), SaTokenException> {
        SaHttpBasicTemplate::check()
    }

    /// 使用指定账号校验（对应 Java `check(String account)`）。
    pub fn check_with_account(account: &str) -> Result<(), SaTokenException> {
        SaHttpBasicTemplate::check_with_account(account)
    }

    /// 使用指定 Realm 与账号校验（对应 Java `check(String realm, String account)`）。
    pub fn check_with_realm_and_account(
        realm: &str,
        account: &str,
    ) -> Result<(), SaTokenException> {
        SaHttpBasicTemplate::check_with_realm_and_account(realm, account)
    }
}
