//! Mixin JWT mode: identity and expiry are in JWT while sessions remain available.

use std::collections::HashMap;

use serde_json::Value;

use super::{SaJwtResult, SaJwtTemplate};

/// Mixin-mode JWT behavior isolated from the global facade.
#[derive(Debug, Clone)]
pub struct StpLogicJwtForMixin {
    login_type: String,
    secret: String,
}

impl StpLogicJwtForMixin {
    pub fn new(login_type: impl Into<String>, secret: impl Into<String>) -> Self {
        Self {
            login_type: login_type.into(),
            secret: secret.into(),
        }
    }

    pub fn create_token_value(
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

    pub fn supports_token_session(&self) -> bool {
        true
    }

    pub fn is_support_share_token(&self) -> bool {
        false
    }
}
