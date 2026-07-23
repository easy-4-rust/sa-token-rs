use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{expires_in_at, now_millis};

/// Persisted OAuth2 access-token record.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct AccessTokenModel {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_time: i64,
    pub refresh_expires_time: i64,
    pub client_id: Option<String>,
    pub login_id: Option<Value>,
    pub scopes: Option<Vec<String>>,
    pub token_type: Option<String>,
    pub grant_type: Option<String>,
    pub extra_data: Option<BTreeMap<String, Value>>,
    pub create_time: i64,
}

impl AccessTokenModel {
    pub fn expires_in(&self) -> i64 {
        self.expires_in_at(now_millis())
    }

    pub fn expires_in_at(&self, now: i64) -> i64 {
        expires_in_at(self.expires_time, now)
    }

    pub fn refresh_expires_in_at(&self, now: i64) -> i64 {
        expires_in_at(self.refresh_expires_time, now)
    }

    pub fn refresh_expires_in(&self) -> i64 {
        self.refresh_expires_in_at(now_millis())
    }
}

impl Default for AccessTokenModel {
    fn default() -> Self {
        Self {
            access_token: None,
            refresh_token: None,
            expires_time: 0,
            refresh_expires_time: 0,
            client_id: None,
            login_id: None,
            scopes: None,
            token_type: None,
            grant_type: None,
            extra_data: None,
            create_time: now_millis(),
        }
    }
}

impl fmt::Debug for AccessTokenModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AccessTokenModel")
            .field("access_token", &self.access_token.as_ref().map(|_| "***"))
            .field("refresh_token", &self.refresh_token.as_ref().map(|_| "***"))
            .field("expires_time", &self.expires_time)
            .field("refresh_expires_time", &self.refresh_expires_time)
            .field("client_id", &self.client_id)
            .field("login_id", &self.login_id)
            .field("scopes", &self.scopes)
            .field("token_type", &self.token_type)
            .field("grant_type", &self.grant_type)
            .field("extra_data", &self.extra_data)
            .field("create_time", &self.create_time)
            .finish()
    }
}
