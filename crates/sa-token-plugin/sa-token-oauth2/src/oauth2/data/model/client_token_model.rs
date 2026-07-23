use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{expires_in_at, now_millis};

/// Persisted OAuth2 client-token record.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientTokenModel {
    pub client_token: Option<String>,
    pub expires_time: i64,
    pub client_id: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub token_type: Option<String>,
    pub grant_type: Option<String>,
    pub extra_data: Option<BTreeMap<String, Value>>,
    pub create_time: i64,
}

impl ClientTokenModel {
    pub fn expires_in(&self) -> i64 {
        self.expires_in_at(now_millis())
    }

    pub fn expires_in_at(&self, now: i64) -> i64 {
        expires_in_at(self.expires_time, now)
    }
}

impl Default for ClientTokenModel {
    fn default() -> Self {
        Self {
            client_token: None,
            expires_time: 0,
            client_id: None,
            scopes: None,
            token_type: None,
            grant_type: None,
            extra_data: None,
            create_time: now_millis(),
        }
    }
}

impl fmt::Debug for ClientTokenModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientTokenModel")
            .field("client_token", &self.client_token.as_ref().map(|_| "***"))
            .field("expires_time", &self.expires_time)
            .field("client_id", &self.client_id)
            .field("scopes", &self.scopes)
            .field("token_type", &self.token_type)
            .field("grant_type", &self.grant_type)
            .field("extra_data", &self.extra_data)
            .field("create_time", &self.create_time)
            .finish()
    }
}
