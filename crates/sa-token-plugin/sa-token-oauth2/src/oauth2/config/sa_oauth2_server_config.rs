use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::oauth2::consts::SaOAuth2Consts;
use crate::oauth2::data::model::loader::SaClientModel;
use crate::oauth2::exception::SaOAuth2ClientModelException;

use super::SaOAuth2OidcConfig;

/// OAuth2 server-wide grant, timeout, scope, and client settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaOAuth2ServerConfig {
    pub enable_authorization_code: bool,
    pub enable_implicit: bool,
    pub enable_password: bool,
    pub enable_client_credentials: bool,
    pub code_timeout: i64,
    pub access_token_timeout: i64,
    pub refresh_token_timeout: i64,
    pub client_token_timeout: i64,
    pub max_access_token_count: i32,
    pub max_refresh_token_count: i32,
    pub max_client_token_count: i32,
    pub is_new_refresh: bool,
    pub openid_digest_prefix: String,
    pub unionid_digest_prefix: String,
    pub higher_scope: Option<String>,
    pub lower_scope: Option<String>,
    pub mode4_return_access_token: bool,
    pub hide_status_field: bool,
    pub oidc: SaOAuth2OidcConfig,
    pub clients: BTreeMap<String, SaClientModel>,
}

impl SaOAuth2ServerConfig {
    /// Registers a client by its declared ID.
    ///
    /// # Errors
    ///
    /// Returns code 30101 when the client ID is absent or empty.
    pub fn add_client(
        &mut self,
        client: SaClientModel,
    ) -> Result<Option<SaClientModel>, SaOAuth2ClientModelException> {
        let client_id = client
            .client_id
            .as_deref()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| SaOAuth2ClientModelException::new("client_id 不可为空", 30101))?
            .to_owned();
        Ok(self.clients.insert(client_id, client))
    }
}

impl Default for SaOAuth2ServerConfig {
    fn default() -> Self {
        Self {
            enable_authorization_code: true,
            enable_implicit: true,
            enable_password: true,
            enable_client_credentials: true,
            code_timeout: 300,
            access_token_timeout: 7_200,
            refresh_token_timeout: 2_592_000,
            client_token_timeout: 7_200,
            max_access_token_count: 12,
            max_refresh_token_count: 12,
            max_client_token_count: 12,
            is_new_refresh: false,
            openid_digest_prefix: SaOAuth2Consts::OPENID_DEFAULT_DIGEST_PREFIX.into(),
            unionid_digest_prefix: SaOAuth2Consts::UNIONID_DEFAULT_DIGEST_PREFIX.into(),
            higher_scope: None,
            lower_scope: None,
            mode4_return_access_token: false,
            hide_status_field: false,
            oidc: SaOAuth2OidcConfig::default(),
            clients: BTreeMap::new(),
        }
    }
}
