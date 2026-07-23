//! `SaHttpBasicTemplate` —— 1:1 对应 Java `cn.dev33.satoken.httpauth.basic.SaHttpBasicTemplate`
//!
//! HTTP Basic 认证模板。

use crate::context::sa_holder::SaHolder;
use crate::error::SaErrorCode;
use crate::exception::SaTokenException;
use crate::sa_manager::SaManager;
use crate::secure::sa_base64_util::SaBase64Util;
use crate::util::sa_fox_util::SaFoxUtil;
use crate::util::sa_token_consts::DEFAULT_HTTP_AUTH_REALM;

use super::sa_http_basic_account::SaHttpBasicAccount;

/// HTTP Basic 模板（对应 Java `SaHttpBasicTemplate`）。
pub struct SaHttpBasicTemplate;

impl SaHttpBasicTemplate {
    /// 默认 Realm（对应 Java `DEFAULT_REALM`）。
    pub const DEFAULT_REALM: &'static str = DEFAULT_HTTP_AUTH_REALM;

    /// 校验失败时设置 401 响应头并抛出异常（对应 Java `throwNotBasicAuthException`）。
    pub fn throw_not_basic_auth_exception(realm: &str) -> SaTokenException {
        let response = SaHolder::get_response();
        response.set_status(401);
        response.set_header("WWW-Authenticate", &format!("Basic Realm={realm}"));
        SaTokenException::with_code(SaErrorCode::CODE_10311, "HTTP Basic 认证失败")
    }

    /// 从 Authorization 头解析 username/password（静态解析，不依赖上下文）。
    pub fn parse_authorization(auth_header: &str) -> Option<(String, String)> {
        let stripped = auth_header.strip_prefix("Basic ")?;
        let decoded = SaBase64Util::decode(stripped.trim()).ok()?;
        let s = String::from_utf8(decoded).ok()?;
        let (u, p) = s.split_once(':')?;
        Some((u.to_string(), p.to_string()))
    }

    /// 构造 Authorization 头（对应 Java 客户端提交格式）。
    pub fn build_authorization(username: &str, password: &str) -> String {
        let raw = format!("{}:{}", username, password);
        format!("Basic {}", SaBase64Util::encode(raw.as_bytes()))
    }

    /// 获取浏览器提交的 Basic 凭证（裁剪前缀并解码，对应 Java `getAuthorizationValue`）。
    pub fn get_authorization_value() -> Option<String> {
        let authorization = SaHolder::get_request().get_header("Authorization")?;
        if !authorization.starts_with("Basic ") {
            return None;
        }
        SaBase64Util::decode(&authorization[6..])
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
    }

    /// 获取 Basic 账号对象（对应 Java `getHttpBasicAccount`）。
    pub fn get_http_basic_account() -> Option<SaHttpBasicAccount> {
        let value = Self::get_authorization_value()?;
        SaHttpBasicAccount::from_username_and_password(&value).ok()
    }

    /// 使用全局配置账号校验（对应 Java `check()`）。
    pub fn check() -> Result<(), SaTokenException> {
        let account = SaManager::config().http_basic.clone();
        Self::check_with_realm_and_account(Self::DEFAULT_REALM, &account)
    }

    /// 使用指定账号校验（对应 Java `check(String account)`）。
    pub fn check_with_account(account: &str) -> Result<(), SaTokenException> {
        Self::check_with_realm_and_account(Self::DEFAULT_REALM, account)
    }

    /// 使用指定 Realm 与账号校验（对应 Java `check(String realm, String account)`）。
    pub fn check_with_realm_and_account(
        realm: &str,
        account: &str,
    ) -> Result<(), SaTokenException> {
        let mut expected = account.to_string();
        if SaFoxUtil::is_empty(&expected) {
            expected = SaManager::config().http_basic.clone();
        }
        let authorization = Self::get_authorization_value().unwrap_or_default();
        if SaFoxUtil::is_empty(&authorization) || authorization != expected {
            return Err(Self::throw_not_basic_auth_exception(realm));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::mock::sa_request_for_mock::SaRequestForMock;
    use crate::context::mock::sa_response_for_mock::SaResponseForMock;
    use crate::context::mock::sa_storage_for_mock::SaStorageForMock;
    use crate::context::model::sa_request::SaRequest;
    use crate::context::model::sa_response::SaResponse;
    use crate::context::model::sa_storage::SaStorage;
    use crate::context::sa_token_context::SaTokenContext;
    use crate::context::sa_token_context_for_thread_local::SaTokenContextForThreadLocal;
    use crate::sa_manager::SaManager;
    use std::sync::Arc;

    fn set_context(auth: Option<&str>) -> Arc<SaResponseForMock> {
        SaManager::set_sa_token_context(Arc::new(SaTokenContextForThreadLocal));
        let mut req = SaRequestForMock::new().with_url("/api").with_method("GET");
        if let Some(value) = auth {
            req = req.with_header("Authorization", value);
        }
        let res = Arc::new(SaResponseForMock::new());
        let req: Arc<dyn SaRequest> = Arc::new(req);
        let res_dyn: Arc<dyn SaResponse> = res.clone();
        let stg: Arc<dyn SaStorage> = Arc::new(SaStorageForMock::new());
        SaTokenContextForThreadLocal.set_context(req, res_dyn, stg);
        res
    }

    #[test]
    fn check_passes_with_matching_authorization() {
        let auth = SaHttpBasicTemplate::build_authorization("sa", "123456");
        set_context(Some(&auth));
        assert!(SaHttpBasicTemplate::check_with_account("sa:123456").is_ok());
    }

    #[test]
    fn check_fails_without_authorization() {
        let res = set_context(None);
        assert!(SaHttpBasicTemplate::check_with_account("sa:123456").is_err());
        assert_eq!(res.status(), 401);
    }
}
