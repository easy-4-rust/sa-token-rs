//! `SaHttpDigestTemplate` —— 1:1 对应 Java `cn.dev33.satoken.httpauth.digest.SaHttpDigestTemplate`
//!
//! HTTP Digest 认证模板。

use std::collections::HashMap;

use crate::context::sa_holder::SaHolder;
use crate::error::SaErrorCode;
use crate::exception::SaTokenException;
use crate::sa_manager::SaManager;
use crate::secure::sa_secure_util::SaSecureUtil;
use crate::util::sa_fox_util::{SaFoxUtil, random_string};

use super::sa_http_digest_model::SaHttpDigestModel;

/// HTTP Digest 模板（对应 Java `SaHttpDigestTemplate`）。
pub struct SaHttpDigestTemplate;

impl SaHttpDigestTemplate {
    /// 构建认证失败响应头（对应 Java `buildResponseHeaderValue`）。
    pub fn build_response_header_value(model: &SaHttpDigestModel) -> String {
        format!(
            "Digest realm=\"{}\", qop=\"{}\", nonce=\"{}\", nc={}, opaque=\"{}\"",
            model.realm, model.qop, model.nonce, model.nc, model.opaque
        )
    }

    /// 校验失败时设置 401 并抛异常（对应 Java `throwNotHttpDigestAuthException`）。
    pub fn throw_not_http_digest_auth_exception(
        mut model: SaHttpDigestModel,
    ) -> SaTokenException {
        if model.realm.is_empty() {
            model.realm = SaHttpDigestModel::DEFAULT_REALM.to_string();
        }
        if model.qop.is_empty() {
            model.qop = SaHttpDigestModel::DEFAULT_QOP.to_string();
        }
        if model.nonce.is_empty() {
            model.nonce = random_string(32);
        }
        if model.opaque.is_empty() {
            model.opaque = random_string(32);
        }
        if model.nc.is_empty() {
            model.nc = "00000001".to_string();
        }

        let response = SaHolder::get_response();
        response.set_status(401);
        response.set_header(
            "WWW-Authenticate",
            &Self::build_response_header_value(&model),
        );
        SaTokenException::with_code(SaErrorCode::CODE_10312, "HTTP Digest 认证失败")
    }

    /// 获取 Authorization 值（对应 Java `getAuthorizationValue`）。
    pub fn get_authorization_value() -> Option<String> {
        let authorization = SaHolder::get_request().get_header("Authorization")?;
        authorization.strip_prefix("Digest ").map(str::to_string)
    }

    /// 解析 Authorization 为模型（对应 Java `getAuthorizationValueToModel`）。
    pub fn get_authorization_value_to_model() -> Option<SaHttpDigestModel> {
        let authorization = Self::get_authorization_value()?;
        let mut map = HashMap::new();
        for part in authorization.split(',') {
            let part = part.trim();
            if let Some((key, value)) = part.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().trim_matches('"').to_string();
                map.insert(key, value);
            } else if let Some(eq) = part.find('=') {
                let key = part[..eq].trim().to_string();
                let value = part[eq + 1..].trim().trim_matches('"').to_string();
                map.insert(key, value);
            }
        }

        Some(SaHttpDigestModel {
            username: map.remove("username").unwrap_or_default(),
            realm: map.remove("realm").unwrap_or_default(),
            nonce: map.remove("nonce").unwrap_or_default(),
            uri: map.remove("uri").unwrap_or_default(),
            method: SaHolder::get_request().get_method(),
            qop: map.remove("qop").unwrap_or_default(),
            nc: map.remove("nc").unwrap_or_default(),
            cnonce: map.remove("cnonce").unwrap_or_default(),
            opaque: map.remove("opaque").unwrap_or_default(),
            response: map.remove("response").unwrap_or_default(),
            password: String::new(),
        })
    }

    /// 计算 response（对应 Java `calcResponse`）。
    pub fn calc_response(model: &SaHttpDigestModel) -> String {
        let frag1 = SaSecureUtil::md5(&format!(
            "{}:{}:{}",
            model.username, model.realm, model.password
        ));
        let frag2 = format!(
            "{}:{}:{}:{}",
            model.nonce, model.nc, model.cnonce, model.qop
        );
        let frag3 = SaSecureUtil::md5(&format!("{}:{}", model.method, model.uri));
        SaSecureUtil::md5(&format!("{frag1}:{frag2}:{frag3}"))
    }

    /// 将 hope 中非空字段覆盖到 req（对应 Java `copyHopeToReq`）。
    pub fn copy_hope_to_req(hope: &SaHttpDigestModel, req: &mut SaHttpDigestModel) {
        req.username.clone_from(&hope.username);
        req.password.clone_from(&hope.password);
        if !hope.realm.is_empty() {
            req.realm.clone_from(&hope.realm);
        }
        if !hope.nonce.is_empty() {
            req.nonce.clone_from(&hope.nonce);
        }
        if !hope.uri.is_empty() {
            req.uri.clone_from(&hope.uri);
        }
        if !hope.method.is_empty() {
            req.method.clone_from(&hope.method);
        }
        if !hope.qop.is_empty() {
            req.qop.clone_from(&hope.qop);
        }
        if !hope.nc.is_empty() {
            req.nc.clone_from(&hope.nc);
        }
        if !hope.opaque.is_empty() {
            req.opaque.clone_from(&hope.opaque);
        }
    }

    /// 根据 hope 模型校验（对应 Java `check(SaHttpDigestModel hopeModel)`）。
    pub fn check_with_model(hope: &SaHttpDigestModel) -> Result<(), SaTokenException> {
        if hope.username.is_empty() {
            return Err(SaTokenException::other("必须提供希望的 username 参数"));
        }
        if hope.password.is_empty() {
            return Err(SaTokenException::other("必须提供希望的 password 参数"));
        }

        let Some(mut req_model) = Self::get_authorization_value_to_model() else {
            return Err(Self::throw_not_http_digest_auth_exception(hope.clone()));
        };
        Self::copy_hope_to_req(hope, &mut req_model);
        let calculated = Self::calc_response(&req_model);
        if calculated != req_model.response {
            return Err(Self::throw_not_http_digest_auth_exception(hope.clone()));
        }
        Ok(())
    }

    /// 根据用户名密码校验（对应 Java `check(String username, String password)`）。
    pub fn check_with_account(username: &str, password: &str) -> Result<(), SaTokenException> {
        Self::check_with_model(&SaHttpDigestModel::new(username, password))
    }

    /// 根据用户名密码与 realm 校验（对应 Java 三参 `check`）。
    pub fn check_with_account_and_realm(
        username: &str,
        password: &str,
        realm: &str,
    ) -> Result<(), SaTokenException> {
        Self::check_with_model(&SaHttpDigestModel::with_realm(username, password, realm))
    }

    /// 使用全局配置校验（对应 Java `check()`）。
    pub fn check() -> Result<(), SaTokenException> {
        let http_digest = SaManager::config().http_digest.clone();
        if SaFoxUtil::is_empty(&http_digest) {
            return Err(SaTokenException::other("未配置全局 Http Digest 认证参数"));
        }
        let Some((username, password)) = http_digest.split_once(':') else {
            return Err(SaTokenException::other(
                "全局 Http Digest 认证参数配置错误，格式应如：username:password",
            ));
        };
        Self::check_with_account(username, password)
    }

    /// 构造 Digest Authorization 头（测试/客户端辅助）。
    pub fn build_authorization(model: &SaHttpDigestModel) -> String {
        format!(
            "Digest username=\"{}\", realm=\"{}\", nonce=\"{}\", uri=\"{}\", response=\"{}\", qop={}, nc={}, cnonce=\"{}\", opaque=\"{}\"",
            model.username,
            model.realm,
            model.nonce,
            model.uri,
            model.response,
            model.qop,
            model.nc,
            model.cnonce,
            model.opaque
        )
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

    fn build_valid_request(username: &str, password: &str) -> String {
        let mut hope = SaHttpDigestModel::new(username, password);
        hope.nonce = "test-nonce".to_string();
        hope.uri = "/api".to_string();
        hope.method = "GET".to_string();
        hope.qop = SaHttpDigestModel::DEFAULT_QOP.to_string();
        hope.nc = "00000001".to_string();
        hope.cnonce = "client-nonce".to_string();
        hope.opaque = "opaque".to_string();
        hope.response = SaHttpDigestTemplate::calc_response(&hope);
        SaHttpDigestTemplate::build_authorization(&hope)
    }

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
    fn digest_check_passes_with_valid_authorization() {
        let auth = build_valid_request("sa", "123456");
        set_context(Some(&auth));
        assert!(SaHttpDigestTemplate::check_with_account("sa", "123456").is_ok());
    }

    #[test]
    fn digest_check_fails_without_authorization() {
        let res = set_context(None);
        assert!(SaHttpDigestTemplate::check_with_account("sa", "123456").is_err());
        assert_eq!(res.status(), 401);
    }
}
