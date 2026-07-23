//! JWT-backed temporary token template (Java `SaTempTemplateForJwt`).

use sa_token_core::exception::{SaResult, SaTokenException};
use sa_token_core::sa_manager::SaManager;
use sa_token_core::temp::sa_temp_template::{SaTempTemplate, DEFAULT_NAMESPACE};
use sa_token_core::util::sa_fox_util;
use serde_json::Value;

use super::error::sa_temp_jwt_error_code::SaTempJwtErrorCode;
use super::sa_jwt_util::SaJwtUtil;

/// Temporary token template using JWT as storage backend.
#[derive(Debug, Clone)]
pub struct SaTempTemplateForJwt {
    namespace: String,
}

impl Default for SaTempTemplateForJwt {
    fn default() -> Self {
        Self::new(DEFAULT_NAMESPACE)
    }
}

impl SaTempTemplateForJwt {
    /// Creates a template bound to the given namespace.
    pub fn new(namespace: impl Into<String>) -> Self {
        let namespace = namespace.into();
        if namespace.is_empty() {
            panic!("namespace 不能为空");
        }
        Self { namespace }
    }

    /// Returns configured JWT secret key or a detailed framework error.
    pub fn resolve_jwt_secret_key(&self) -> SaResult<String> {
        let config = SaManager::config();
        let key = config.jwt_secret_key().to_string();
        if sa_fox_util::is_empty(&key) {
            return Err(SaTokenException::with_code(
                SaTempJwtErrorCode::CODE_30301,
                "请配置：jwtSecretKey",
            ));
        }
        Ok(key)
    }

    /// Lists temp tokens for a value — disabled in JWT mode.
    pub fn get_temp_token_list(&self, _value: &Value) -> SaResult<Vec<String>> {
        Err(SaTokenException::with_code(
            SaTempJwtErrorCode::CODE_30304,
            "jwt cannot get token list",
        ))
    }
}

impl SaTempTemplate for SaTempTemplateForJwt {
    fn create_token(&self, value: &Value, timeout: i64) -> SaResult<String> {
        let secret = self.resolve_jwt_secret_key()?;
        SaJwtUtil::create_token(value, timeout, &secret)
    }

    fn save_token(&self, token: &str, value: &Value, timeout: i64) -> SaResult<()> {
        let _ = (token, value, timeout);
        Err(SaTokenException::with_code(
            SaTempJwtErrorCode::CODE_30302,
            "jwt cannot save token mapping",
        ))
    }

    fn parse_token(&self, token: &str) -> SaResult<Option<Value>> {
        let secret = self.resolve_jwt_secret_key()?;
        SaJwtUtil::get_value(token, &secret).map(Some)
    }

    fn get_timeout(&self, token: &str) -> SaResult<i64> {
        let secret = self.resolve_jwt_secret_key()?;
        SaJwtUtil::get_timeout(token, &secret)
    }

    fn delete_token(&self, _token: &str) -> SaResult<()> {
        Err(SaTokenException::with_code(
            SaTempJwtErrorCode::CODE_30302,
            "jwt cannot delete token",
        ))
    }

    fn splicing_temp_token_save_key(&self, token: &str) -> String {
        format!(
            "{}:{}:{}",
            SaManager::config().get_token_name(),
            self.namespace,
            token
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sa_token_core::config::sa_token_config::SaTokenConfig;
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn delete_token_is_disabled_with_code() {
        SaManager::set_config(Arc::new(SaTokenConfig {
            jwt_secret_key: "secret".into(),
            ..Default::default()
        }));
        let tpl = SaTempTemplateForJwt::default();
        let err = tpl.delete_token("abc").expect_err("delete disabled");
        assert_eq!(err.code(), SaTempJwtErrorCode::CODE_30302);
    }

    #[test]
    fn create_and_parse_via_trait() {
        SaManager::set_config(Arc::new(SaTokenConfig {
            jwt_secret_key: "secret".into(),
            ..Default::default()
        }));
        let tpl = SaTempTemplateForJwt::default();
        let value = json!(10001);
        let token = tpl.create_token(&value, 60).expect("create");
        let parsed = tpl.parse_token(&token).expect("parse");
        assert_eq!(parsed, Some(value));
    }
}
