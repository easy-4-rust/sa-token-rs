//! Simple JWT mode: JWT carries extra data while login state remains DAO-backed.

use std::collections::HashMap;

use serde_json::Value;

use super::{SaJwtResult, SaJwtTemplate};

/// Simple-mode JWT behavior isolated from the global facade.
#[derive(Debug, Clone)]
pub struct StpLogicJwtForSimple {
    login_type: String,
    secret: String,
}

impl StpLogicJwtForSimple {
    pub fn new(login_type: impl Into<String>, secret: impl Into<String>) -> Self {
        Self {
            login_type: login_type.into(),
            secret: secret.into(),
        }
    }

    pub fn create_token_value(
        &self,
        login_id: Value,
        extra_data: HashMap<String, Value>,
    ) -> SaJwtResult<String> {
        SaJwtTemplate.create_token(&self.login_type, login_id, extra_data, &self.secret)
    }

    pub fn get_extra(&self, token: &str, key: &str) -> SaJwtResult<Option<Value>> {
        Ok(SaJwtTemplate
            .get_payloads_not_check(token, &self.login_type, &self.secret)?
            .remove(key))
    }

    pub fn is_support_share_token(&self) -> bool {
        false
    }

    pub fn is_support_extra(&self) -> bool {
        true
    }
}
