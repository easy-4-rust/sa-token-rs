use std::fmt;

use serde::{Deserialize, Serialize};

use crate::oauth2::config::SaOAuth2ServerConfig;

/// OAuth2 client registration and per-client token policy.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaClientModel {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub contract_scopes: Vec<String>,
    pub allow_redirect_uris: Vec<String>,
    pub allow_grant_types: Vec<String>,
    pub subject_id: Option<String>,
    pub access_token_timeout: i64,
    pub refresh_token_timeout: i64,
    pub client_token_timeout: i64,
    pub max_access_token_count: i32,
    pub max_refresh_token_count: i32,
    pub max_client_token_count: i32,
    pub is_new_refresh: bool,
    pub is_auto_confirm: bool,
}

impl SaClientModel {
    pub fn from_server_config(config: &SaOAuth2ServerConfig) -> Self {
        Self {
            client_id: None,
            client_secret: None,
            contract_scopes: Vec::new(),
            allow_redirect_uris: Vec::new(),
            allow_grant_types: Vec::new(),
            subject_id: None,
            access_token_timeout: config.access_token_timeout,
            refresh_token_timeout: config.refresh_token_timeout,
            client_token_timeout: config.client_token_timeout,
            max_access_token_count: config.max_access_token_count,
            max_refresh_token_count: config.max_refresh_token_count,
            max_client_token_count: config.max_client_token_count,
            is_new_refresh: config.is_new_refresh,
            is_auto_confirm: false,
        }
    }

    pub fn add_contract_scopes(&mut self, scopes: impl IntoIterator<Item = String>) -> &mut Self {
        self.contract_scopes.extend(scopes);
        self
    }

    pub fn add_allow_redirect_uris(
        &mut self,
        redirect_uris: impl IntoIterator<Item = String>,
    ) -> &mut Self {
        self.allow_redirect_uris.extend(redirect_uris);
        self
    }

    pub fn add_allow_grant_types(
        &mut self,
        grant_types: impl IntoIterator<Item = String>,
    ) -> &mut Self {
        self.allow_grant_types.extend(grant_types);
        self
    }
}

impl Default for SaClientModel {
    fn default() -> Self {
        Self::from_server_config(&SaOAuth2ServerConfig::default())
    }
}

impl fmt::Debug for SaClientModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SaClientModel")
            .field("client_id", &self.client_id)
            .field("client_secret", &self.client_secret.as_ref().map(|_| "***"))
            .field("contract_scopes", &self.contract_scopes)
            .field("allow_redirect_uris", &self.allow_redirect_uris)
            .field("allow_grant_types", &self.allow_grant_types)
            .field("subject_id", &self.subject_id)
            .field("access_token_timeout", &self.access_token_timeout)
            .field("refresh_token_timeout", &self.refresh_token_timeout)
            .field("client_token_timeout", &self.client_token_timeout)
            .field("max_access_token_count", &self.max_access_token_count)
            .field("max_refresh_token_count", &self.max_refresh_token_count)
            .field("max_client_token_count", &self.max_client_token_count)
            .field("is_new_refresh", &self.is_new_refresh)
            .field("is_auto_confirm", &self.is_auto_confirm)
            .finish()
    }
}
