//! Persisted API Key model corresponding to Java `ApiKeyModel`.

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::apikey::error::SaApiKeyErrorCode;
use crate::apikey::exception::ApiKeyException;

/// API Key metadata and authorization scopes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiKeyModel {
    /// Display title.
    pub title: Option<String>,
    /// Description.
    pub intro: Option<String>,
    /// API Key value.
    pub api_key: String,
    /// Bound account id.
    pub login_id: String,
    /// Creation time in milliseconds.
    pub create_time: i64,
    /// Expiration time in milliseconds; `-1` means permanent.
    pub expires_time: i64,
    /// Whether the key is enabled.
    pub is_valid: bool,
    /// Granted scopes.
    pub scopes: Vec<String>,
    /// Extension data.
    pub extra_data: BTreeMap<String, Value>,
}

impl Default for ApiKeyModel {
    fn default() -> Self {
        Self {
            title: None,
            intro: None,
            api_key: String::new(),
            login_id: String::new(),
            create_time: now_millis(),
            expires_time: 0,
            is_valid: true,
            scopes: Vec::new(),
            extra_data: BTreeMap::new(),
        }
    }
}

impl ApiKeyModel {
    /// Adds scopes, preserving Java's insertion order and duplicate behavior.
    pub fn add_scopes(mut self, scopes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.scopes.extend(scopes.into_iter().map(Into::into));
        self
    }

    /// Validates fields before persistence.
    ///
    /// # Errors
    /// Returns code `12304` when a required field is invalid.
    pub fn validate_for_save(&self) -> Result<(), ApiKeyException> {
        if self.api_key.is_empty()
            || self.login_id.is_empty()
            || self.create_time == 0
            || self.expires_time == 0
        {
            return Err(ApiKeyException::new(
                SaApiKeyErrorCode::CODE_12304,
                (!self.api_key.is_empty()).then_some(self.api_key.as_str()),
                "API Key 字段自检未通过",
            ));
        }
        Ok(())
    }

    /// Remaining lifetime in seconds (`-1` permanent, `-2` expired).
    pub fn expires_in(&self) -> i64 {
        if self.expires_time == -1 {
            return -1;
        }
        let remaining = (self.expires_time - now_millis()) / 1_000;
        if remaining < 1 { -2 } else { remaining }
    }

    /// Returns whether the key has expired.
    pub fn time_expired(&self) -> bool {
        self.expires_time != -1 && now_millis() > self.expires_time
    }
}

pub(crate) fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as i64)
}
