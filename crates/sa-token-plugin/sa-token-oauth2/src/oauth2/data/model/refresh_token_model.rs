use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{expires_in_at, now_millis};

/// Persisted OAuth2 refresh-token record.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct RefreshTokenModel {
    pub refresh_token: Option<String>,
    pub expires_time: i64,
    pub client_id: Option<String>,
    pub login_id: Option<Value>,
    pub scopes: Option<Vec<String>>,
    pub extra_data: Option<BTreeMap<String, Value>>,
    pub create_time: i64,
}

impl RefreshTokenModel {
    pub fn expires_in(&self) -> i64 {
        self.expires_in_at(now_millis())
    }

    pub fn expires_in_at(&self, now: i64) -> i64 {
        expires_in_at(self.expires_time, now)
    }
}

impl Default for RefreshTokenModel {
    fn default() -> Self {
        Self {
            refresh_token: None,
            expires_time: 0,
            client_id: None,
            login_id: None,
            scopes: None,
            extra_data: None,
            create_time: now_millis(),
        }
    }
}

impl fmt::Debug for RefreshTokenModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RefreshTokenModel")
            .field("refresh_token", &self.refresh_token.as_ref().map(|_| "***"))
            .field("expires_time", &self.expires_time)
            .field("client_id", &self.client_id)
            .field("login_id", &self.login_id)
            .field("scopes", &self.scopes)
            .field("extra_data", &self.extra_data)
            .field("create_time", &self.create_time)
            .finish()
    }
}
