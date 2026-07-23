//! Configuration corresponding to Java `SaApiKeyConfig`.

use serde::{Deserialize, Serialize};

/// API Key generation, lifetime and index settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaApiKeyConfig {
    /// Prefix prepended to generated keys.
    pub prefix: String,
    /// Lifetime in seconds; `-1` means permanent.
    pub timeout: i64,
    /// Whether account-to-key indexes are persisted.
    pub is_record_index: bool,
}

impl Default for SaApiKeyConfig {
    fn default() -> Self {
        Self {
            prefix: "AK-".to_string(),
            timeout: 2_592_000,
            is_record_index: true,
        }
    }
}
