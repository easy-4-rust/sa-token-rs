use serde::{Deserialize, Serialize};

/// OpenID Connect settings nested under the OAuth2 server configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaOAuth2OidcConfig {
    pub iss: Option<String>,
    pub id_token_timeout: i64,
}

impl Default for SaOAuth2OidcConfig {
    fn default() -> Self {
        Self {
            iss: None,
            id_token_timeout: 600,
        }
    }
}
