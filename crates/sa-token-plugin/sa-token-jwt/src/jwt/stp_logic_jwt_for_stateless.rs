//! Stateless JWT mode: token state is entirely self-contained.

use std::collections::HashMap;

use serde_json::Value;

use super::{SaJwtResult, SaJwtTemplate};

/// Stateless-mode JWT behavior with no DAO or token-session dependency.
#[derive(Debug, Clone)]
pub struct StpLogicJwtForStateless {
    login_type: String,
    secret: String,
}

impl StpLogicJwtForStateless {
    pub fn new(login_type: impl Into<String>, secret: impl Into<String>) -> Self {
        Self {
            login_type: login_type.into(),
            secret: secret.into(),
        }
    }

    pub fn create_login_session(
        &self,
        login_id: Value,
        device_type: &str,
        timeout: i64,
        extra_data: HashMap<String, Value>,
    ) -> SaJwtResult<String> {
        SaJwtTemplate.create_token_full(
            &self.login_type,
            login_id,
            device_type,
            timeout,
            extra_data,
            &self.secret,
        )
    }

    pub fn get_login_id(&self, token: &str) -> SaJwtResult<Value> {
        SaJwtTemplate.get_login_id(token, &self.login_type, &self.secret)
    }

    pub fn get_extra(&self, token: &str, key: &str) -> SaJwtResult<Option<Value>> {
        Ok(SaJwtTemplate
            .get_payloads(token, &self.login_type, &self.secret)?
            .remove(key))
    }

    pub fn get_token_timeout(&self, token: &str) -> i64 {
        SaJwtTemplate.get_timeout(token, &self.login_type, &self.secret)
    }

    pub fn get_login_device_type(&self, token: &str) -> SaJwtResult<Option<String>> {
        Ok(SaJwtTemplate
            .get_payloads_not_check(token, &self.login_type, &self.secret)?
            .get(SaJwtTemplate::DEVICE_TYPE)
            .and_then(Value::as_str)
            .map(str::to_owned))
    }

    pub fn supports_persistent_dao(&self) -> bool {
        false
    }

    pub fn is_support_extra(&self) -> bool {
        true
    }
}
