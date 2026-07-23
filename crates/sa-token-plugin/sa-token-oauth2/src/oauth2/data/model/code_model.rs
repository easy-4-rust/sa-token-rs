use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::now_millis;

/// Persisted OAuth2 authorization-code record.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeModel {
    pub code: Option<String>,
    pub client_id: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub login_id: Option<Value>,
    pub redirect_uri: Option<String>,
    pub nonce: Option<String>,
    pub create_time: i64,
}

impl Default for CodeModel {
    fn default() -> Self {
        Self {
            code: None,
            client_id: None,
            scopes: None,
            login_id: None,
            redirect_uri: None,
            nonce: None,
            create_time: now_millis(),
        }
    }
}

impl fmt::Debug for CodeModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CodeModel")
            .field("code", &self.code.as_ref().map(|_| "***"))
            .field("client_id", &self.client_id)
            .field("scopes", &self.scopes)
            .field("login_id", &self.login_id)
            .field("redirect_uri", &self.redirect_uri)
            .field("nonce", &self.nonce.as_ref().map(|_| "***"))
            .field("create_time", &self.create_time)
            .finish()
    }
}
